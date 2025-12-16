#![recursion_limit = "512"]
use assert_matches::*;
use candid::CandidType;
use candid::{encode_args, Principal};
use canhttp::http::json::{ConstantSizeId, Id};
use ic_canister_runtime::CyclesWalletRuntime;
use ic_cdk::call::RejectCode;
use ic_pocket_canister_runtime::{
    CanisterHttpReject, CanisterHttpReply, JsonRpcRequestMatcher, JsonRpcResponse,
    MockHttpOutcalls, MockHttpOutcallsBuilder, PocketIcRuntime,
};
use pocket_ic::{common::rest::CanisterHttpResponse, ErrorCode, RejectResponse};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use sol_rpc_client::{RequestBuilder, SolRpcEndpoint};
use sol_rpc_client::{SolRpcClient, SolRpcConfig};
use sol_rpc_int_tests::{Setup, DEFAULT_CALLER_TEST_ID};
use sol_rpc_types::{
    CommitmentLevel, ConfirmedTransactionStatusWithSignature, ConsensusStrategy,
    GetSignaturesForAddressLimit, GetSlotParams, GetTransactionEncoding, HttpOutcallError,
    InstallArgs, InstructionError, LegacyRejectionCode, Mode, MultiRpcResult, PrioritizationFee,
    ProviderError, RpcAccess, RpcAuth, RpcError, RpcResult, RpcSource, RpcSources, Slot,
    SolanaCluster, SupportedRpcProvider, SupportedRpcProviderId, TransactionDetails,
    TransactionError,
};
use solana_account_decoder_client_types::{
    token::UiTokenAmount, UiAccount, UiAccountData, UiAccountEncoding,
};
use solana_pubkey::pubkey;
use solana_signer::Signer;
use solana_transaction_status_client_types::{
    option_serializer::OptionSerializer, EncodedConfirmedTransactionWithStatusMeta,
    EncodedTransaction, EncodedTransactionWithStatusMeta, TransactionBinaryEncoding,
    TransactionConfirmationStatus, TransactionStatus, UiLoadedAddresses, UiTransactionStatusMeta,
};
use std::{fmt::Debug, iter::zip, num::NonZeroU8, str::FromStr};
use strum::IntoEnumIterator;

const USDC_PUBLIC_KEY: solana_pubkey::Pubkey =
    pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
// See: https://internetcomputer.org/docs/references/cycles-cost-formulas#https-outcalls
const HTTP_OUTCALL_BASE_FEE: u128 = (3_000_000 + 60_000 * 34) * 34;

const SLOT: Slot = 386_766_418;
const SLOTS: [Slot; 3] = [SLOT, 386_862_552, 386_976_279];

mod get_provider_tests {
    use super::*;

    #[tokio::test]
    async fn should_get_providers() {
        let setup = Setup::new().await;
        let client = setup.client(MockHttpOutcalls::never()).build();
        let providers = client.get_providers().await;

        assert_eq!(providers.len(), 11);

        assert_eq!(
            providers[0],
            (
                SupportedRpcProviderId::AlchemyMainnet,
                SupportedRpcProvider {
                    cluster: SolanaCluster::Mainnet,
                    access: RpcAccess::Authenticated {
                        auth: RpcAuth::BearerToken {
                            url: "https://solana-mainnet.g.alchemy.com/v2".to_string(),
                        },
                        public_url: Some(
                            "https://solana-mainnet.g.alchemy.com/v2/demo".to_string()
                        ),
                    }
                },
            )
        );

        setup.drop().await;
    }
}

mod get_account_info_tests {
    use super::*;

    #[tokio::test]
    async fn should_get_account_info() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, offset) in zip(rpc_sources(), (0..).step_by(3)) {
            let mocks = mock_for_ids(
                get_account_info_request,
                get_account_info_response,
                offset..=offset + 2,
            );
            let client = setup.client(mocks).with_rpc_sources(sources).build();

            let results = client
                .get_account_info(USDC_PUBLIC_KEY)
                .send()
                .await
                .expect_consistent();

            assert_eq!(
                results,
                Ok(UiAccount {
                    lamports: 88849814690250,
                    data: UiAccountData::Binary("1234".to_string(), UiAccountEncoding::Base58),
                    owner: "11111111111111111111111111111111".to_string(),
                    executable: false,
                    rent_epoch: 18446744073709551615,
                    space: Some(0),
                }
                .into())
            );
        }

        setup.drop().await;
    }

    #[tokio::test]
    async fn should_not_get_account_info() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, offset) in zip(rpc_sources(), (0..).step_by(3)) {
            let mocks = mock_for_ids(
                get_account_info_request,
                not_found_response,
                offset..=offset + 2,
            );
            let client = setup.client(mocks).with_rpc_sources(sources).build();

            let results = client
                .get_account_info(USDC_PUBLIC_KEY)
                .send()
                .await
                .expect_consistent();

            assert_eq!(results, Ok(None));
        }

        setup.drop().await;
    }
}

mod get_block_tests {
    use super::*;

    #[tokio::test]
    async fn should_get_block() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, offset) in zip(rpc_sources(), (0..).step_by(3)) {
            let mocks = mock_for_ids(get_block_request, get_block_response, offset..=offset + 2);
            let client = setup.client(mocks).with_rpc_sources(sources).build();

            let results = client.get_block(577996).send().await.expect_consistent();

            assert_eq!(
                results,
                Ok(Some(
                    solana_transaction_status_client_types::UiConfirmedBlock {
                        previous_blockhash: "4Pcj2yJkCYyhnWe8Ze3uK2D2EtesBxhAevweDoTcxXf3"
                            .to_string(),
                        blockhash: "8QeCusqSTKeC23NwjTKRBDcPuEfVLtszkxbpL6mXQEp4".to_string(),
                        parent_slot: 372877611,
                        block_time: Some(1744122369),
                        block_height: Some(360854634),
                        transactions: None,
                        signatures: None,
                        rewards: None,
                        num_reward_partitions: None,
                    }
                ))
            );
        }

        setup.drop().await;
    }

    #[tokio::test]
    async fn should_not_get_block() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, offset) in zip(rpc_sources(), (0..).step_by(3)) {
            let mocks = mock_for_ids(get_block_request, not_found_response, offset..=offset + 2);
            let client = setup.client(mocks).with_rpc_sources(sources).build();

            let results = client.get_block(577996).send().await.expect_consistent();

            assert_eq!(results, Ok(None));
        }

        setup.drop().await;
    }
}

mod get_slot_tests {
    use super::*;

    #[tokio::test]
    async fn should_get_slot_with_full_params() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        let params = json!([{"commitment": "processed", "minContextSlot": 100}]);
        let mocks = MockHttpOutcallsBuilder::new()
            .given(get_slot_request().with_params(params.clone()).with_id(0))
            .respond_with(get_slot_response(1230).with_id(0))
            .given(get_slot_request().with_params(params.clone()).with_id(1))
            .respond_with(get_slot_response(1230).with_id(1))
            .given(get_slot_request().with_params(params).with_id(2))
            .respond_with(get_slot_response(1230).with_id(2));

        let client = setup.client(mocks);

        let slot = client
            .build()
            .get_slot()
            .with_params(GetSlotParams {
                commitment: Some(CommitmentLevel::Processed),
                min_context_slot: Some(100),
            })
            .with_rounding_error(10)
            .send()
            .await
            .expect_consistent();

        assert_eq!(slot, Ok(1230));

        setup.drop().await;
    }

    #[tokio::test]
    async fn should_get_slot_without_rounding() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, offset) in zip(rpc_sources(), (0..).step_by(3)) {
            let mocks = mock_with_response_slots_for_ids(
                get_slot_request,
                get_slot_response,
                [1234; 3],
                offset..=offset + 2,
            );
            let client = setup.client(mocks).with_rpc_sources(sources).build();

            let results = client
                .get_slot()
                .with_rounding_error(0)
                .send()
                .await
                .expect_consistent();

            assert_eq!(results, Ok(1234));
        }

        setup.drop().await;
    }

    #[tokio::test]
    async fn should_get_consistent_result_with_rounding() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, offset) in zip(rpc_sources(), (0..).step_by(3)) {
            let mocks = mock_with_response_slots_for_ids(
                get_slot_request,
                get_slot_response,
                [1234, 1229, 1237],
                offset..=offset + 2,
            );
            let client = setup.client(mocks).with_rpc_sources(sources).build();

            let results = client.get_slot().send().await.expect_consistent();

            assert_eq!(results, Ok(1220));
        }

        setup.drop().await;
    }

    #[tokio::test]
    async fn should_get_inconsistent_result_without_rounding() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, offset) in zip(rpc_sources(), (0..).step_by(3)) {
            let mocks = mock_with_response_slots_for_ids(
                get_slot_request,
                get_slot_response,
                [1234, 1229, 1237],
                offset..=offset + 2,
            );
            let client = setup.client(mocks).with_rpc_sources(sources).build();

            let results: Vec<RpcResult<_>> = client
                .get_slot()
                .with_rounding_error(0)
                .send()
                .await
                .expect_inconsistent()
                .into_iter()
                .map(|(_source, result)| result)
                .collect();

            assert_eq!(results, vec![Ok(1234), Ok(1229), Ok(1237)]);
        }

        setup.drop().await;
    }
}

mod get_recent_prioritization_fees_tests {
    use super::*;

    #[tokio::test]
    async fn should_get_fees_with_rounding() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        let mocks = mock_for_ids(
            get_recent_prioritization_fees_request,
            get_recent_prioritization_fees_response,
            0..=2,
        );
        let client = setup.client(mocks).build();

        let fees = client
            .get_recent_prioritization_fees(&[USDC_PUBLIC_KEY])
            .unwrap()
            .with_max_slot_rounding_error(10)
            .with_max_length(NonZeroU8::new(5).unwrap())
            .send()
            .await
            .expect_consistent();

        assert_eq!(
            fees,
            Ok(vec![
                PrioritizationFee {
                    prioritization_fee: 535775,
                    slot: 338225906
                },
                PrioritizationFee {
                    prioritization_fee: 65038,
                    slot: 338225907
                },
                PrioritizationFee {
                    prioritization_fee: 0,
                    slot: 338225908
                },
                PrioritizationFee {
                    prioritization_fee: 0,
                    slot: 338225909
                },
                PrioritizationFee {
                    prioritization_fee: 0,
                    slot: 338225910
                },
            ])
        );

        setup.drop().await;
    }
}

mod send_transaction_tests {
    use super::*;

    #[tokio::test]
    async fn should_send_transaction() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, offset) in zip(rpc_sources(), (0..).step_by(3)) {
            let transaction = some_transaction();

            let mocks = mock_for_ids(
                || send_transaction_request(&transaction),
                send_transaction_response,
                offset..=offset + 2,
            );
            let client = setup.client(mocks).with_rpc_sources(sources).build();

            let results = client
                .send_transaction(transaction)
                .send()
                .await
                .expect_consistent();

            assert_eq!(results, Ok(some_signature()));
        }

        setup.drop().await;
    }
}

mod get_transaction_tests {
    use super::*;

    #[tokio::test]
    async fn should_get_transaction() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, offset) in zip(rpc_sources(), (0..).step_by(3)) {
            let mocks = mock_for_ids(
                get_transaction_request,
                get_transaction_response,
                offset..=offset + 2,
            );
            let client = setup.client(mocks).with_rpc_sources(sources).build();

            let results = client
                .get_transaction(some_signature())
                .with_encoding(GetTransactionEncoding::Base64)
                .send()
                .await
                .expect_consistent();

            assert_eq!(
                results,
                Ok(Some(EncodedConfirmedTransactionWithStatusMeta {
                    slot: 369_139_986,
                    transaction: EncodedTransactionWithStatusMeta {
                        transaction: EncodedTransaction::Binary("ARAJPXmph5xbnfO74gv8tBIwTA0yw0BuRZvqrr113O9BTj0T4kXejUz3jh1RCasjsZkr2do/ZjMIOg56TTvRlQgBAAMGDEiA3o3u6XvTb57cHKZkhrHuNhISrOgMMafRPe48Q4QgJhAewgMolkoyq6sTbFQFuR86447k9ky2veh5uGg40kK5Pth9DxkikievxiovoyrY6lRfLhWKUZINPu2s+AlMAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADBkZv5SEXMv/srbpyw5vnvIzlu8X3EmssQ5s6QAAAAMGkhusDr3enQhfGliLPnjUOYbtCSz9fET+Twnd+37hJkr+3Zt+dBsrfJ0eCM1bDr9NITRuvFbzpE4a9q1ZEXggDBAAFAqQBAAAFAgACqAELVaozzA/wZnC9ckuJIt1EqfSq6QAzzGYyZzOAmQEAAHF0Ee4i3YhEjwv/FswzZpkBBxEiM0RVZneImaq7zN3u/wCqVTPMZpkSNFZ4mrze8BI0VniavN7wEjRWeJq83vASNFZ4mrze8BI0VniavN7wEjRWeJq83vASNFZ4mrze8BI0VniavN7wEjRWeJq83vASNFZ4mrze8AxIgN6N7ul702+e3BymZIYDAgABDAIAAADoAwAAAAAAAA==".to_string(), TransactionBinaryEncoding::Base64),
                        meta: Some(UiTransactionStatusMeta {
                            err: None,
                            status: Ok(()),
                            fee: 5000_u64,
                            pre_balances: vec![
                                463360320850,
                                6608068,
                                2060160,
                                1,
                                1,
                                1141440
                            ],
                            post_balances: vec![
                                463360314850,
                                6609068,
                                2060160,
                                1,
                                1,
                                1141440
                            ],
                            inner_instructions: Some(vec![]).into(),
                            log_messages: Some(vec![
                                "Program ComputeBudget111111111111111111111111111111 invoke [1]".to_string(),
                                "Program ComputeBudget111111111111111111111111111111 success".to_string(),
                                "Program E2uCGJ4TtYyKPGaK57UMfbs9sgaumwDEZF1aAY6fF3mS invoke [1]".to_string(),
                                "Program E2uCGJ4TtYyKPGaK57UMfbs9sgaumwDEZF1aAY6fF3mS consumed 110 of 270 compute units".to_string(),
                                "Program E2uCGJ4TtYyKPGaK57UMfbs9sgaumwDEZF1aAY6fF3mS success".to_string(),
                                "Program 11111111111111111111111111111111 invoke [1]".to_string(),
                                "Program 11111111111111111111111111111111 success".to_string()
                            ]).into(),
                            pre_token_balances: Some(vec![]).into(),
                            post_token_balances: Some(vec![]).into(),
                            rewards: Some(vec![]).into(),
                            loaded_addresses: Some(UiLoadedAddresses::default()).into(),
                            return_data: OptionSerializer::Skip,
                            compute_units_consumed: Some(410_u64).into(),
                            cost_units: Some(2084_u64).into(),
                        }),
                        version: None,
                    },
                    block_time: Some(1_758_792_475),
                }))
            );
        }

        setup.drop().await;
    }
}

mod generic_request_tests {
    use super::*;

    #[tokio::test]
    async fn should_require_base_http_outcall_fee() {
        async fn check<Config, Params, CandidOutput, Output>(
            request: RequestBuilder<
                CyclesWalletRuntime<PocketIcRuntime<'_>>,
                Config,
                Params,
                CandidOutput,
                Output,
            >,
        ) where
            Config: CandidType + Clone + Send,
            Params: CandidType + Clone + Send,
            CandidOutput: Into<Output> + CandidType + DeserializeOwned,
        {
            let result = request
                .with_cycles(HTTP_OUTCALL_BASE_FEE - 1)
                .try_send()
                .await;
            assert!(result.is_err_and(|err| err.to_string().contains("Not enough cycles")));
        }

        let setup = Setup::new().await.with_mock_api_keys().await;
        let client = setup.client(MockHttpOutcalls::never()).build();

        for endpoint in SolRpcEndpoint::iter() {
            match endpoint {
                SolRpcEndpoint::GetSlot => {
                    check(client.get_slot()).await;
                }
                SolRpcEndpoint::GetAccountInfo => {
                    check(client.get_account_info(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetBalance => {
                    check(client.get_balance(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetBlock => {
                    check(client.get_block(577996)).await;
                }
                SolRpcEndpoint::GetRecentPrioritizationFees => {
                    check(client.get_recent_prioritization_fees(&[]).unwrap()).await;
                }
                SolRpcEndpoint::GetSignaturesForAddress => {
                    check(client.get_signatures_for_address(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetSignatureStatuses => {
                    check(client.get_signature_statuses(&[some_signature()]).unwrap()).await;
                }
                SolRpcEndpoint::GetTokenAccountBalance => {
                    check(client.get_token_account_balance(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetTransaction => {
                    check(client.get_transaction(some_signature())).await;
                }
                SolRpcEndpoint::JsonRequest => {
                    check(client.json_request(get_version_request_body())).await;
                }
                SolRpcEndpoint::SendTransaction => {
                    check(client.send_transaction(some_transaction())).await;
                }
            }
        }

        setup.drop().await;
    }

    #[tokio::test]
    async fn should_not_require_cycles_in_demo_mode() {
        async fn check<Config, Params, CandidOutput, Output>(
            request: RequestBuilder<
                CyclesWalletRuntime<PocketIcRuntime<'_>>,
                Config,
                Params,
                CandidOutput,
                Output,
            >,
        ) where
            Config: CandidType + Clone + Send,
            Params: CandidType + Clone + Send,
            CandidOutput: Into<Output> + CandidType + DeserializeOwned,
        {
            let result = request.with_cycles(0).try_send().await;
            assert!(result.is_ok());
        }

        let setup = Setup::new().await.with_mock_api_keys().await;
        setup
            .upgrade_canister(InstallArgs {
                mode: Some(Mode::Demo),
                ..Default::default()
            })
            .await;
        // We always return a dummy response so that individual responses
        // do not need to be mocked.
        let client = setup
            .client(mock_all_endpoints(
                |request| request,
                CanisterHttpReply::with_status(403),
            ))
            .build();

        for endpoint in SolRpcEndpoint::iter() {
            match endpoint {
                SolRpcEndpoint::GetSlot => {
                    check(client.get_slot()).await;
                }
                SolRpcEndpoint::GetAccountInfo => {
                    check(client.get_account_info(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetBalance => {
                    check(client.get_balance(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetBlock => {
                    check(client.get_block(577996)).await;
                }
                SolRpcEndpoint::GetRecentPrioritizationFees => {
                    check(client.get_recent_prioritization_fees(&[]).unwrap()).await;
                }
                SolRpcEndpoint::GetSignaturesForAddress => {
                    check(client.get_signatures_for_address(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetSignatureStatuses => {
                    check(client.get_signature_statuses(&[some_signature()]).unwrap()).await;
                }
                SolRpcEndpoint::GetTokenAccountBalance => {
                    check(client.get_token_account_balance(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetTransaction => {
                    check(client.get_transaction(some_signature())).await;
                }
                SolRpcEndpoint::JsonRequest => {
                    check(client.json_request(get_version_request_body())).await;
                }
                SolRpcEndpoint::SendTransaction => {
                    check(client.send_transaction(some_transaction())).await;
                }
            }
        }

        setup.drop().await;
    }

    #[tokio::test]
    async fn request_should_succeed_in_demo_mode() {
        let setup = Setup::with_args(InstallArgs {
            mode: Some(Mode::Demo),
            ..Default::default()
        })
        .await
        .with_mock_api_keys()
        .await;
        let mocks = mock_for_ids(get_version_request, get_version_response, 0..=2);
        let client = setup.client(mocks).build();

        let result = client
            .json_request(get_version_request_body())
            .with_cycles(0)
            .send()
            .await
            .expect_consistent();

        assert_matches!(result, Ok(msg) if msg == r#"{"feature-set":3640012085,"solana-core":"2.3.6"}"#);

        setup.drop().await;
    }
}

mod retrieve_logs_tests {
    use super::*;

    #[tokio::test]
    async fn should_retrieve_logs() {
        let setup = Setup::with_args(InstallArgs {
            manage_api_keys: Some(vec![DEFAULT_CALLER_TEST_ID]),
            ..Default::default()
        })
        .await;
        assert_eq!(setup.retrieve_logs("DEBUG").await, vec![]);
        assert_eq!(setup.retrieve_logs("INFO").await, vec![]);

        // Generate some log
        setup
            .client(MockHttpOutcalls::never())
            .build()
            .update_api_keys(&[(
                SupportedRpcProviderId::AlchemyMainnet,
                Some("unauthorized-api-key".to_string()),
            )])
            .await;

        assert_eq!(setup.retrieve_logs("DEBUG").await, vec![]);
        assert!(setup.retrieve_logs("INFO").await[0]
            .message
            .contains("Updating API keys"));
    }
}

mod update_api_key_tests {
    use super::*;

    #[tokio::test]
    async fn should_update_api_key() {
        let setup = Setup::with_args(InstallArgs {
            manage_api_keys: Some(vec![DEFAULT_CALLER_TEST_ID]),
            ..Default::default()
        })
        .await;

        let provider = SupportedRpcProviderId::AlchemyMainnet;
        let api_key = "test-api-key";
        let client = setup.client(MockHttpOutcalls::never()).build();
        client
            .update_api_keys(&[(provider, Some(api_key.to_string()))])
            .await;
        setup
            .verify_api_key((provider, Some(api_key.to_string())))
            .await;

        client.update_api_keys(&[(provider, None)]).await;
        setup.verify_api_key((provider, None)).await;
    }

    #[tokio::test]
    #[should_panic(expected = "You are not authorized")]
    async fn should_prevent_unauthorized_update_api_keys() {
        let setup = Setup::new().await;
        setup
            .client(MockHttpOutcalls::never())
            .build()
            .update_api_keys(&[(
                SupportedRpcProviderId::AlchemyMainnet,
                Some("unauthorized-api-key".to_string()),
            )])
            .await;
    }

    #[tokio::test]
    #[should_panic(expected = "Trying to set API key for unauthenticated provider")]
    async fn should_prevent_unauthenticated_update_api_keys() {
        let setup = Setup::with_args(InstallArgs {
            manage_api_keys: Some(vec![DEFAULT_CALLER_TEST_ID]),
            ..Default::default()
        })
        .await;
        setup
            .client(MockHttpOutcalls::never())
            .build()
            .update_api_keys(&[(
                SupportedRpcProviderId::PublicNodeMainnet,
                Some("invalid-api-key".to_string()),
            )])
            .await;
    }

    #[tokio::test]
    async fn should_prevent_unauthorized_call_to_verify_api_key() {
        let setup = Setup::new().await.with_mock_api_keys().await;
        let args = (SupportedRpcProviderId::AlchemyMainnet, Some("test-key"));

        for unauthorized_principal in [Principal::anonymous(), DEFAULT_CALLER_TEST_ID] {
            let result = setup
                .as_ref()
                .query_call(
                    setup.sol_rpc_canister_id(),
                    unauthorized_principal,
                    "verifyApiKey",
                    encode_args(args).unwrap(),
                )
                .await;

            assert_eq!(
                result,
                Err(RejectResponse {
                    reject_code: pocket_ic::RejectCode::CanisterReject,
                    reject_message: "You are not authorized".to_string(),
                    error_code: ErrorCode::CanisterRejectedMessage,
                    certified: false,
                })
            );
        }

        setup.drop().await;
    }
}

mod canister_upgrade_tests {
    use super::*;

    #[tokio::test]
    async fn upgrade_should_keep_api_keys() {
        let setup = Setup::with_args(InstallArgs {
            manage_api_keys: Some(vec![DEFAULT_CALLER_TEST_ID]),
            ..Default::default()
        })
        .await;
        let provider = SupportedRpcProviderId::AlchemyMainnet;
        let api_key = "test-api-key";
        let client = setup.client(MockHttpOutcalls::never()).build();
        client
            .update_api_keys(&[(provider, Some(api_key.to_string()))])
            .await;
        setup
            .verify_api_key((provider, Some(api_key.to_string())))
            .await;

        setup.upgrade_canister(InstallArgs::default()).await;

        setup
            .verify_api_key((provider, Some(api_key.to_string())))
            .await;
    }

    #[tokio::test]
    async fn upgrade_should_keep_manage_api_key_principals() {
        let setup = Setup::with_args(InstallArgs {
            manage_api_keys: Some(vec![DEFAULT_CALLER_TEST_ID]),
            ..Default::default()
        })
        .await;
        setup
            .upgrade_canister(InstallArgs {
                manage_api_keys: None,
                ..Default::default()
            })
            .await;
        setup
            .client(MockHttpOutcalls::never())
            .build()
            .update_api_keys(&[(
                SupportedRpcProviderId::AlchemyMainnet,
                Some("authorized-api-key".to_string()),
            )])
            .await;
    }

    #[tokio::test]
    #[should_panic(expected = "You are not authorized")]
    async fn upgrade_should_change_manage_api_key_principals() {
        let setup = Setup::with_args(InstallArgs {
            manage_api_keys: Some(vec![DEFAULT_CALLER_TEST_ID]),
            ..Default::default()
        })
        .await;
        setup
            .upgrade_canister(InstallArgs {
                manage_api_keys: Some(vec![]),
                ..Default::default()
            })
            .await;
        setup
            .client(MockHttpOutcalls::never())
            .build()
            .update_api_keys(&[(
                SupportedRpcProviderId::AlchemyMainnet,
                Some("unauthorized-api-key".to_string()),
            )])
            .await;
    }
}

fn rpc_sources() -> Vec<RpcSources> {
    vec![
        RpcSources::Default(SolanaCluster::Devnet),
        RpcSources::Default(SolanaCluster::Mainnet),
        RpcSources::Custom(vec![
            RpcSource::Supported(SupportedRpcProviderId::AlchemyMainnet),
            RpcSource::Supported(SupportedRpcProviderId::DrpcMainnet),
            RpcSource::Supported(SupportedRpcProviderId::PublicNodeMainnet),
        ]),
    ]
}

mod cycles_cost_tests {
    use super::*;
    use std::ops::RangeFrom;

    #[tokio::test]
    async fn should_be_idempotent() {
        async fn check<Config, Params, CandidOutput, Output>(
            request: RequestBuilder<
                CyclesWalletRuntime<PocketIcRuntime<'_>>,
                Config,
                Params,
                CandidOutput,
                Output,
            >,
        ) where
            Config: CandidType + Clone + Send,
            Params: CandidType + Clone + Send,
        {
            let cycles_cost_1 = request.clone().request_cost().send().await.unwrap();
            let cycles_cost_2 = request.request_cost().send().await.unwrap();
            assert_eq!(cycles_cost_1, cycles_cost_2);
            assert!(cycles_cost_1 > 0);
        }

        let setup = Setup::new().await.with_mock_api_keys().await;
        let client = setup.client(MockHttpOutcalls::never()).build();

        for endpoint in SolRpcEndpoint::iter() {
            match endpoint {
                SolRpcEndpoint::GetSlot => {
                    check(client.get_slot()).await;
                }
                SolRpcEndpoint::JsonRequest => {
                    check(client.json_request(get_version_request_body())).await;
                }
                SolRpcEndpoint::GetAccountInfo => {
                    check(client.get_account_info(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetBalance => {
                    check(client.get_balance(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetBlock => {
                    check(client.get_block(577996)).await;
                }
                SolRpcEndpoint::GetRecentPrioritizationFees => {
                    check(client.get_recent_prioritization_fees(&[]).unwrap()).await
                }
                SolRpcEndpoint::GetSignaturesForAddress => {
                    check(client.get_signatures_for_address(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetSignatureStatuses => {
                    check(client.get_signature_statuses(&[some_signature()]).unwrap()).await;
                }
                SolRpcEndpoint::GetTransaction => {
                    check(client.get_transaction(some_signature())).await;
                }
                SolRpcEndpoint::GetTokenAccountBalance => {
                    check(client.get_token_account_balance(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::SendTransaction => {
                    check(client.send_transaction(some_transaction())).await;
                }
            }
        }

        setup.drop().await;
    }

    #[tokio::test]
    async fn should_be_zero_when_in_demo_mode() {
        async fn check<Config, Params, CandidOutput, Output>(
            request: RequestBuilder<
                CyclesWalletRuntime<PocketIcRuntime<'_>>,
                Config,
                Params,
                CandidOutput,
                Output,
            >,
        ) where
            Config: CandidType + Clone + Send,
            Params: CandidType + Clone + Send,
        {
            let cycles_cost = request.request_cost().send().await;
            assert_eq!(cycles_cost, Ok(0));
        }

        let setup = Setup::new().await.with_mock_api_keys().await;
        setup
            .upgrade_canister(InstallArgs {
                mode: Some(Mode::Demo),
                ..Default::default()
            })
            .await;
        let client = setup.client(MockHttpOutcalls::never()).build();

        for endpoint in SolRpcEndpoint::iter() {
            match endpoint {
                SolRpcEndpoint::GetSlot => {
                    check(client.get_slot()).await;
                }
                SolRpcEndpoint::GetAccountInfo => {
                    check(client.get_account_info(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetBalance => {
                    check(client.get_balance(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetBlock => {
                    check(client.get_block(577996)).await;
                }
                SolRpcEndpoint::GetRecentPrioritizationFees => {
                    check(client.get_recent_prioritization_fees(&[]).unwrap()).await;
                }
                SolRpcEndpoint::GetSignaturesForAddress => {
                    check(client.get_signatures_for_address(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetSignatureStatuses => {
                    check(client.get_signature_statuses(&[some_signature()]).unwrap()).await;
                }
                SolRpcEndpoint::GetTokenAccountBalance => {
                    check(client.get_token_account_balance(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetTransaction => {
                    check(client.get_transaction(some_signature())).await;
                }
                SolRpcEndpoint::JsonRequest => {
                    check(client.json_request(get_version_request_body())).await;
                }
                SolRpcEndpoint::SendTransaction => {
                    check(client.send_transaction(some_transaction())).await;
                }
            }
        }

        setup.drop().await;
    }

    #[tokio::test]
    async fn should_get_exact_cycles_cost() {
        async fn check<Config, Params, CandidOutput, Output>(
            setup: &Setup,
            request: RequestBuilder<
                CyclesWalletRuntime<PocketIcRuntime<'_>>,
                Config,
                Params,
                MultiRpcResult<CandidOutput>,
                MultiRpcResult<Output>,
            >,
            expected_cycles_cost: u128,
        ) where
            Config: CandidType + Clone + Send,
            Params: CandidType + Clone + Send,
            CandidOutput: CandidType + DeserializeOwned,
            Output: Debug,
            MultiRpcResult<CandidOutput>: Into<MultiRpcResult<Output>>,
        {
            let five_percents = 5_u8;

            let cycles_cost = request.clone().request_cost().send().await.unwrap();
            assert_within(cycles_cost, expected_cycles_cost, five_percents);

            let cycles_before = setup.sol_rpc_canister_cycles_balance().await;
            // Request with exact cycles amount should succeed
            let result = request
                .clone()
                .with_cycles(cycles_cost)
                .send()
                .await
                .expect_consistent();
            if let Err(RpcError::ProviderError(ProviderError::TooFewCycles { .. })) = result {
                panic!("BUG: estimated cycles cost was insufficient!: {result:?}");
            }
            let cycles_after = setup.sol_rpc_canister_cycles_balance().await;
            let cycles_consumed = cycles_before + cycles_cost - cycles_after;

            assert!(
                cycles_after > cycles_before,
                "BUG: not enough cycles requested. Requested {cycles_cost} cycles, but consumed {cycles_consumed} cycles"
            );

            // Same request with fewer cycles should fail.
            let results = request
                .with_cycles(cycles_cost - 1)
                .send()
                .await
                .expect_inconsistent();

            assert!(
                results.iter().any(|(_provider, result)| matches!(
                    result,
                    &Err(RpcError::ProviderError(ProviderError::TooFewCycles {
                        expected: _,
                        received: _
                    }))
                )),
                "BUG: Expected at least one TooFewCycles error, but got {results:?}"
            );
        }

        let setup = Setup::new().await.with_mock_api_keys().await;

        // The cycles cost of an HTTPS outcall is independent of the response, so we always
        // return a dummy response (403 Forbidden). This avoids needing to mock specific
        // responses for each endpoint.
        fn add_mocks_for(
            rpc_method: &str,
            mut mocks: MockHttpOutcallsBuilder,
            request_ids: &mut RangeFrom<u64>,
        ) -> MockHttpOutcallsBuilder {
            // Mock 5 HTTPS outcalls with dummy responses:
            // - first canister call: exact number of cycles, calls to all 3 providers succeed
            // - second canister call: insufficient cycles, calls to only 2 providers succeed
            for id in request_ids.by_ref().take(5) {
                mocks = mocks
                    .given(JsonRpcRequestMatcher::with_method(rpc_method).with_id(id))
                    .respond_with(CanisterHttpReply::with_status(403));
            }
            // Advance ID by 1 but do not mock an HTTPS outcall since the call to the third
            // provider fails due to insufficient cycles.
            for _ in request_ids.by_ref().take(1) {}
            mocks
        }

        let mut mocks = MockHttpOutcallsBuilder::new();
        let mut ids = 0_u64..;
        for endpoint in SolRpcEndpoint::iter() {
            match endpoint {
                SolRpcEndpoint::JsonRequest => mocks = add_mocks_for("getVersion", mocks, &mut ids),
                // Mock once for each value of `TransactionDetails`
                SolRpcEndpoint::GetBlock => {
                    for _ in 0..3 {
                        mocks = add_mocks_for(endpoint.rpc_method(), mocks, &mut ids)
                    }
                }
                _ => mocks = add_mocks_for(endpoint.rpc_method(), mocks, &mut ids),
            };
        }
        let client = setup.client(mocks).build();

        for endpoint in SolRpcEndpoint::iter() {
            // To find out the expected_cycles_cost for a new endpoint, set the amount to 0
            // and run the test. It should fail and report the amount of cycles needed.
            match endpoint {
                SolRpcEndpoint::GetAccountInfo => {
                    check(
                        &setup,
                        client.get_account_info(USDC_PUBLIC_KEY),
                        1_752_822_400,
                    )
                    .await;
                }
                SolRpcEndpoint::GetBalance => {
                    check(&setup, client.get_balance(USDC_PUBLIC_KEY), 1_731_769_600).await;
                }
                SolRpcEndpoint::GetBlock => {
                    for transaction_details in TransactionDetails::iter() {
                        let expected_cycles_cost = match transaction_details {
                            TransactionDetails::Accounts => 164_743_232_800,
                            TransactionDetails::None => 1_772_855_200,
                            TransactionDetails::Signatures => 23_122_271_200,
                        };
                        check(
                            &setup,
                            client
                                .get_block(577996)
                                .with_transaction_details(transaction_details),
                            expected_cycles_cost,
                        )
                        .await
                    }
                }
                SolRpcEndpoint::GetRecentPrioritizationFees => {
                    check(
                        &setup,
                        client.get_recent_prioritization_fees(&[]).unwrap(),
                        2_378_204_800,
                    )
                    .await;
                }
                SolRpcEndpoint::GetSignaturesForAddress => {
                    check(
                        &setup,
                        client.get_signatures_for_address(USDC_PUBLIC_KEY),
                        22_601_010_400,
                    )
                    .await;
                }
                SolRpcEndpoint::GetSignatureStatuses => {
                    check(
                        &setup,
                        client.get_signature_statuses(&[some_signature()]).unwrap(),
                        1_744_458_400,
                    )
                    .await;
                }
                SolRpcEndpoint::GetSlot => {
                    check(&setup, client.get_slot(), 1_714_103_200).await;
                }
                SolRpcEndpoint::GetTokenAccountBalance => {
                    check(
                        &setup,
                        client.get_token_account_balance(USDC_PUBLIC_KEY),
                        1_732_259_200,
                    )
                    .await;
                }
                SolRpcEndpoint::GetTransaction => {
                    check(
                        &setup,
                        client.get_transaction(some_signature()),
                        2_381_264_800,
                    )
                    .await;
                }
                SolRpcEndpoint::JsonRequest => {
                    check(
                        &setup,
                        client.json_request(get_version_request_body()),
                        1_791_582_400,
                    )
                    .await;
                }
                SolRpcEndpoint::SendTransaction => {
                    check(
                        &setup,
                        client.send_transaction(some_transaction()),
                        1_799_416_000,
                    )
                    .await
                }
            }
        }

        setup.drop().await;
    }
}

mod rpc_config_tests {
    use super::*;

    #[tokio::test]
    async fn should_respect_response_size_estimate() {
        async fn check<Config, Params, CandidOutput, Output>(
            request: RequestBuilder<
                CyclesWalletRuntime<PocketIcRuntime<'_>>,
                Config,
                Params,
                MultiRpcResult<CandidOutput>,
                MultiRpcResult<Output>,
            >,
        ) where
            Config: CandidType + Clone + Send + SolRpcConfig + Default,
            Params: CandidType + Clone + Send,
            CandidOutput: CandidType + DeserializeOwned,
            Output: Debug + PartialEq,
            MultiRpcResult<CandidOutput>: Into<MultiRpcResult<Output>>,
        {
            let result = request
                .with_response_size_estimate(1_999_999)
                .with_cycles(1_000_000_000_000)
                .try_send()
                .await;
            // We do not care about the actual result here, only that the request matches the mock
            // with the correct value for the response size estimate.
            assert!(result.is_ok());
        }

        let setup = Setup::new().await.with_mock_api_keys().await;

        let client = setup
            .client(mock_all_endpoints(
                |request| request.with_max_response_bytes(1_999_999_u64),
                CanisterHttpReply::with_status(403),
            ))
            .build();

        for endpoint in SolRpcEndpoint::iter() {
            match endpoint {
                SolRpcEndpoint::GetAccountInfo => {
                    check(client.get_account_info(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetBalance => {
                    check(client.get_balance(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetBlock => check(client.get_block(577996)).await,
                SolRpcEndpoint::GetRecentPrioritizationFees => {
                    check(client.get_recent_prioritization_fees(&[]).unwrap()).await;
                }
                SolRpcEndpoint::GetSignaturesForAddress => {
                    check(client.get_signatures_for_address(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetSignatureStatuses => {
                    check(client.get_signature_statuses(&[some_signature()]).unwrap()).await;
                }
                SolRpcEndpoint::GetSlot => {
                    check(client.get_slot()).await;
                }
                SolRpcEndpoint::GetTokenAccountBalance => {
                    check(client.get_token_account_balance(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetTransaction => {
                    check(client.get_transaction(some_signature())).await;
                }
                SolRpcEndpoint::JsonRequest => {
                    check(client.json_request(get_version_request_body())).await;
                }
                SolRpcEndpoint::SendTransaction => {
                    check(client.send_transaction(some_transaction())).await
                }
            }
        }

        setup.drop().await;
    }

    #[tokio::test]
    async fn should_respect_response_strategy() {
        async fn check<F, Config, Params, CandidOutput, Output>(
            setup: &Setup,
            request: F,
            offset: &mut u64,
            json_rpc_request: JsonRpcRequestMatcher,
            json_rpc_response: JsonRpcResponse,
        ) where
            F: Fn(
                SolRpcClient<CyclesWalletRuntime<PocketIcRuntime<'_>>>,
            ) -> RequestBuilder<
                CyclesWalletRuntime<PocketIcRuntime<'_>>,
                Config,
                Params,
                MultiRpcResult<CandidOutput>,
                MultiRpcResult<Output>,
            >,
            Config: CandidType + Clone + Send + SolRpcConfig + Default,
            Params: CandidType + Clone + Send,
            CandidOutput: CandidType + DeserializeOwned,
            Output: Debug + PartialEq,
            MultiRpcResult<CandidOutput>: Into<MultiRpcResult<Output>>,
        {
            let mocks = MockHttpOutcallsBuilder::new()
                .given(json_rpc_request.clone().with_id(*offset))
                .respond_with(json_rpc_response.clone().with_id(*offset))
                .given(json_rpc_request.clone().with_id(*offset + 1))
                .respond_with(json_rpc_response.clone().with_id(*offset + 1))
                .given(json_rpc_request.clone().with_id(*offset + 2))
                .respond_with(CanisterHttpReject::with_reject_code(RejectCode::SysFatal));
            let client = setup.client(mocks).build();
            *offset += 3;

            let result = request(client)
                .with_response_consensus(ConsensusStrategy::Equality)
                .send()
                .await;
            assert_matches!(result, MultiRpcResult::Inconsistent(_));

            let mocks = MockHttpOutcallsBuilder::new()
                .given(json_rpc_request.clone().with_id(*offset))
                .respond_with(json_rpc_response.clone().with_id(*offset))
                .given(json_rpc_request.clone().with_id(*offset + 1))
                .respond_with(json_rpc_response.clone().with_id(*offset + 1))
                .given(json_rpc_request.clone().with_id(*offset + 2))
                .respond_with(CanisterHttpReject::with_reject_code(RejectCode::SysFatal));
            let client = setup.client(mocks).build();
            *offset += 3;

            let result = request(client)
                .with_response_consensus(ConsensusStrategy::Threshold {
                    total: Some(3),
                    min: 2,
                })
                .send()
                .await;
            assert_matches!(result, MultiRpcResult::Consistent(_));
        }

        let setup = Setup::new().await.with_mock_api_keys().await;
        let mut offset = 0;
        for endpoint in SolRpcEndpoint::iter() {
            match endpoint {
                SolRpcEndpoint::GetAccountInfo => {
                    check(
                        &setup,
                        |client| client.get_account_info(USDC_PUBLIC_KEY),
                        &mut offset,
                        get_account_info_request(),
                        get_account_info_response(),
                    )
                    .await;
                }
                SolRpcEndpoint::GetBalance => {
                    check(
                        &setup,
                        |client| {
                            client
                                .get_balance(USDC_PUBLIC_KEY)
                                .with_min_context_slot(100)
                                .with_commitment(CommitmentLevel::Confirmed)
                        },
                        &mut offset,
                        get_balance_request(),
                        get_balance_response(SLOT),
                    )
                    .await;
                }
                SolRpcEndpoint::GetBlock => {
                    check(
                        &setup,
                        |client| client.get_block(577996),
                        &mut offset,
                        get_block_request(),
                        get_block_response(),
                    )
                    .await
                }
                SolRpcEndpoint::GetRecentPrioritizationFees => {
                    check(
                        &setup,
                        |client| {
                            client
                                .get_recent_prioritization_fees(&[USDC_PUBLIC_KEY])
                                .unwrap()
                                .with_max_slot_rounding_error(10)
                                .with_max_length(NonZeroU8::new(5).unwrap())
                        },
                        &mut offset,
                        get_recent_prioritization_fees_request(),
                        get_recent_prioritization_fees_response(),
                    )
                    .await;
                }
                SolRpcEndpoint::GetSignaturesForAddress => {
                    check(
                        &setup,
                        |client| {
                            client
                                .get_signatures_for_address(USDC_PUBLIC_KEY)
                                .with_limit(GetSignaturesForAddressLimit::try_from(5).unwrap())
                        },
                        &mut offset,
                        get_signatures_for_address_request(),
                        get_signatures_for_address_response(),
                    )
                    .await;
                }
                SolRpcEndpoint::GetSignatureStatuses => {
                    check(
                        &setup,
                        |client| {
                            client
                                .get_signature_statuses(&[some_signature(), another_signature()])
                                .unwrap()
                                .with_search_transaction_history(true)
                        },
                        &mut offset,
                        get_signature_statuses_request(),
                        get_signature_statuses_response(SLOT),
                    )
                    .await;
                }
                SolRpcEndpoint::GetSlot => {
                    check(
                        &setup,
                        |client| client.get_slot(),
                        &mut offset,
                        get_slot_request(),
                        get_slot_response(1234),
                    )
                    .await;
                }
                SolRpcEndpoint::GetTokenAccountBalance => {
                    check(
                        &setup,
                        |client| {
                            client
                                .get_token_account_balance(USDC_PUBLIC_KEY)
                                .with_commitment(CommitmentLevel::Confirmed)
                        },
                        &mut offset,
                        get_token_account_balance_request(),
                        get_token_account_balance_response(SLOT),
                    )
                    .await;
                }
                SolRpcEndpoint::GetTransaction => {
                    check(
                        &setup,
                        |client| {
                            client
                                .get_transaction(some_signature())
                                .with_encoding(GetTransactionEncoding::Base64)
                        },
                        &mut offset,
                        get_transaction_request(),
                        get_transaction_response(),
                    )
                    .await;
                }
                SolRpcEndpoint::JsonRequest => {
                    check(
                        &setup,
                        |client| client.json_request(get_version_request_body()),
                        &mut offset,
                        get_version_request(),
                        get_version_response(),
                    )
                    .await;
                }
                SolRpcEndpoint::SendTransaction => {
                    let transaction = some_transaction();
                    check(
                        &setup,
                        |client| client.send_transaction(transaction.clone()),
                        &mut offset,
                        send_transaction_request(&transaction),
                        send_transaction_response(),
                    )
                    .await
                }
            }
        }

        setup.drop().await;
    }
}

mod get_balance_tests {
    use super::*;

    #[tokio::test]
    async fn should_get_balance() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, offset) in zip(rpc_sources(), (0..).step_by(3)) {
            let mocks = mock_with_response_slots_for_ids(
                get_balance_request,
                get_balance_response,
                SLOTS,
                offset..=offset + 2,
            );
            let client = setup.client(mocks).with_rpc_sources(sources).build();

            let results = client
                .get_balance(USDC_PUBLIC_KEY)
                .with_min_context_slot(100)
                .with_commitment(CommitmentLevel::Confirmed)
                .send()
                .await
                .expect_consistent();

            assert_eq!(results, Ok(389_086_612_571_u64));
        }

        setup.drop().await;
    }
}

mod get_token_account_balance_tests {
    use super::*;

    #[tokio::test]
    async fn should_get_token_account_balance() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, offset) in zip(rpc_sources(), (0..).step_by(3)) {
            let mocks = mock_with_response_slots_for_ids(
                get_token_account_balance_request,
                get_token_account_balance_response,
                SLOTS,
                offset..=offset + 2,
            );
            let client = setup.client(mocks).with_rpc_sources(sources).build();

            let results = client
                .get_token_account_balance(USDC_PUBLIC_KEY)
                .with_commitment(CommitmentLevel::Confirmed)
                .send()
                .await
                .expect_consistent();

            assert_eq!(
                results,
                Ok(UiTokenAmount {
                    amount: "9864".to_string(),
                    decimals: 2,
                    ui_amount: Some(98.64),
                    ui_amount_string: "98.64".to_string(),
                })
            );
        }

        setup.drop().await;
    }
}

mod get_signature_statuses_tests {
    use super::*;

    #[tokio::test]
    async fn should_get_signature_statuses() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, offset) in zip(rpc_sources(), (0..).step_by(3)) {
            let mocks = mock_with_response_slots_for_ids(
                get_signature_statuses_request,
                get_signature_statuses_response,
                SLOTS,
                offset..=offset + 2,
            );
            let client = setup.client(mocks).with_rpc_sources(sources).build();

            let results = client
                .get_signature_statuses(&[some_signature(), another_signature()])
                .unwrap()
                .with_search_transaction_history(true)
                .send()
                .await
                .expect_consistent();

            assert_eq!(
                results,
                Ok(vec![
                    Some(TransactionStatus {
                        slot: 48,
                        confirmations: None,
                        status: Ok(()),
                        err: None,
                        confirmation_status: Some(TransactionConfirmationStatus::Finalized),
                    }),
                    None,
                ])
            );
        }

        setup.drop().await;
    }
}

mod get_signatures_for_address_tests {
    use super::*;

    #[tokio::test]
    async fn should_get_signatures_for_address() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, offset) in zip(rpc_sources(), (0..).step_by(3)) {
            let mocks = mock_for_ids(
                get_signatures_for_address_request,
                get_signatures_for_address_response,
                offset..=offset + 2,
            );
            let client = setup.client(mocks).with_rpc_sources(sources).build();

            let results = client
                .get_signatures_for_address(USDC_PUBLIC_KEY)
                .with_limit(GetSignaturesForAddressLimit::try_from(5).unwrap())
                .send()
                .await
                .expect_consistent();

            assert_eq!(
                results,
                Ok(vec![
                    ConfirmedTransactionStatusWithSignature {
                        signature: sol_rpc_types::Signature::from_str("3jPA8CnZb9sfs4zVAypa9KB7VAGwrTdXB6mg9H1H9XpATN6Y8iek4Y21Nb9LjbrpYACbF9USV8RBWvXFFhVoQUAs").unwrap(),
                        confirmation_status: Some(TransactionConfirmationStatus::Finalized.into()),
                        memo: None,
                        slot: 340_372_399,
                        err: None,
                        block_time: Some(1_747_389_084)
                    },
                    ConfirmedTransactionStatusWithSignature {
                        signature: sol_rpc_types::Signature::from_str("3WM42nYDQAHgBWFd6SbJ3pj1AGgiTJfxXJ2d5dHu49GgqSUui5qdh64S5yLCN1cMKcLMFVKKo776GrtVhfatLqP6").unwrap(),
                        confirmation_status: Some(TransactionConfirmationStatus::Finalized.into()),
                        memo: None,
                        slot: 340_372_399,
                        err: None,
                        block_time: Some(1_747_389_084)
                    },
                    ConfirmedTransactionStatusWithSignature {
                        signature: sol_rpc_types::Signature::from_str("5iByUT1gTNXDY24hRx25YmQeebvUMD6jsNpGcu2jh1yjKmYwdo5GtRrYozyhdtdcn8SurwHq6EMp4YTpHgdansjc").unwrap(),
                        confirmation_status: Some(TransactionConfirmationStatus::Finalized.into()),
                        memo: None,
                        slot: 340_372_399,
                        err: None,
                        block_time: Some(1_747_389_084)
                    },
                    ConfirmedTransactionStatusWithSignature {
                        signature: sol_rpc_types::Signature::from_str("2Zuhxr6qMGwBrpV611Ema7pZAy1WGSkQyurTcbfyoXwFMNuziUJbM6FCyoL8WxTRG6G3fEik2wSFeN76miUeUnmJ").unwrap(),
                        confirmation_status: Some(TransactionConfirmationStatus::Finalized.into()),
                        memo: None,
                        slot: 340_372_399,
                        err: None,
                        block_time: Some(1_747_389_084)
                    },
                    ConfirmedTransactionStatusWithSignature {
                        signature: sol_rpc_types::Signature::from_str("4V1j8jZvXjcUdRoWQBRzxFVigfr61bJdHGsCFAkTm5h4z28FkrDczuTpcvwTRamiwiGm7E77EB5DKRBwG1mUEC8f").unwrap(),
                        confirmation_status: Some(TransactionConfirmationStatus::Finalized.into()),
                        memo: None,
                        slot: 340_372_399,
                        err: Some(TransactionError::InstructionError(3, InstructionError::Custom(6_001))),
                        block_time: Some(1_747_389_084)
                    }])
            );
        }

        setup.drop().await;
    }
}

mod metrics_tests {
    use super::*;
    use ic_pocket_canister_runtime::CanisterHttpReject;

    #[tokio::test]
    async fn should_retrieve_metrics() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        let mocks = MockHttpOutcallsBuilder::new()
            .given(get_slot_request().with_id(0))
            .respond_with(get_slot_response(1_450_305).with_id(0))
            .given(get_slot_request().with_id(1))
            .respond_with(get_slot_response(1_450_305).with_id(1))
            .given(get_slot_request().with_id(2))
            .respond_with(JsonRpcResponse::from(json!({
              "jsonrpc": "2.0",
              "error": {
                  "code": -32603,
                  "message": "Internal error: failed to get slot: Node is behind",
                  "data": null
              },
              "id": Id::from(ConstantSizeId::from(2_u8)),
            })))
            .given(get_slot_request().with_id(3))
            .respond_with(CanisterHttpReply::with_status(429))
            .given(get_slot_request().with_id(4))
            .respond_with(CanisterHttpReply::with_status(500))
            .given(get_slot_request().with_id(5))
            .respond_with(
                CanisterHttpReject::with_reject_code(RejectCode::SysFatal)
                    .with_message("Fatal error!"),
            );
        let client = setup
            .client(mocks)
            .with_consensus_strategy(ConsensusStrategy::Threshold {
                total: Some(6),
                min: 2,
            })
            .with_rpc_sources(RpcSources::Custom(vec![
                RpcSource::Supported(SupportedRpcProviderId::AlchemyMainnet),
                RpcSource::Supported(SupportedRpcProviderId::AnkrMainnet),
                RpcSource::Supported(SupportedRpcProviderId::ChainstackMainnet),
                RpcSource::Supported(SupportedRpcProviderId::DrpcMainnet),
                RpcSource::Supported(SupportedRpcProviderId::HeliusMainnet),
                RpcSource::Supported(SupportedRpcProviderId::PublicNodeMainnet),
            ]))
            .build();
        let result = client.get_slot().send().await;
        assert_eq!(result, MultiRpcResult::Consistent(Ok(1_450_300)));

        let mocks = MockHttpOutcallsBuilder::new()
            .given(get_account_info_request().with_id(6))
            .respond_with(
                CanisterHttpReject::with_reject_code(RejectCode::SysFatal)
                    .with_message("Http body exceeds size limit of 2000000 bytes."),
            );
        let client = setup
            .client(mocks)
            .with_rpc_sources(RpcSources::Custom(vec![RpcSource::Supported(
                SupportedRpcProviderId::AlchemyMainnet,
            )]))
            .with_response_size_estimate(2_000_000)
            .build();
        let result = client
            .get_account_info(USDC_PUBLIC_KEY)
            // To avoid retries, we set a high response size estimate,
            // which incurs a large cycles cost.
            .with_cycles(1_000_000_000_000)
            .send()
            .await;
        assert_eq!(
            result,
            MultiRpcResult::Consistent(Err(RpcError::HttpOutcallError(
                HttpOutcallError::IcError {
                    code: LegacyRejectionCode::SysFatal,
                    message: "Http body exceeds size limit of 2000000 bytes.".to_string()
                }
            )))
        );

        setup
            .check_metrics()
            .await
            // `solrpc_requests` counters
            .assert_contains_metric_matching(r#"solrpc_requests\{method="getSlot",host="solana-mainnet.g.alchemy.com"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_requests\{method="getSlot",host="rpc.ankr.com"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_requests\{method="getSlot",host="solana-mainnet.core.chainstack.com"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_requests\{method="getSlot",host="lb.drpc.org"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_requests\{method="getSlot",host="mainnet.helius-rpc.com"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_requests\{method="getSlot",host="solana-rpc.publicnode.com"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_requests\{method="getAccountInfo",host="solana-mainnet.g.alchemy.com"\} 1 \d+"#)
            // `solrpc_responses` counters: success
            .assert_contains_metric_matching(r#"solrpc_responses\{method="getSlot",host="solana-mainnet.g.alchemy.com"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_responses\{method="getSlot",host="rpc.ankr.com"\} 1 \d+"#)
            // `solrpc_responses` counters: JSON-RPC error
            .assert_contains_metric_matching(r#"solrpc_responses\{method="getSlot",host="solana-mainnet.core.chainstack.com",error="json-rpc"\} 1 \d+"#)
            // `solrpc_responses` counters: HTTP error
            .assert_contains_metric_matching(r#"solrpc_responses\{method="getSlot",host="lb.drpc.org",error="http",status="429"\} .*"#)
            .assert_contains_metric_matching(r#"solrpc_responses\{method="getSlot",host="mainnet.helius-rpc.com",error="http",status="500"\} .*"#)
            // `solrpc_responses` counters: IC error
            .assert_contains_metric_matching(r#"solrpc_responses\{method="getSlot",host="solana-rpc.publicnode.com",error="ic",code="SYS_FATAL"\} .*"#)
            // `solrpc_responses` counters: insufficient cycles
            .assert_contains_metric_matching(r#"solrpc_responses\{method="getAccountInfo",host="solana-mainnet.g.alchemy.com",error="max-response-size-exceeded"\} .*"#)
            // `solrpc_latencies` latency histograms
            .assert_contains_metric_matching(r#"solrpc_latencies_bucket\{method="getSlot",host="solana-mainnet.g.alchemy.com",le="\d+"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_latencies_bucket\{method="getSlot",host="rpc.ankr.com",le="\d+"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_latencies_bucket\{method="getSlot",host="solana-mainnet.core.chainstack.com",le="\d+"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_latencies_bucket\{method="getSlot",host="lb.drpc.org",le="\d+"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_latencies_bucket\{method="getSlot",host="mainnet.helius-rpc.com",le="\d+"\} 1 \d+"#)
            .assert_does_not_contain_metric_matching(r#"solrpc_latencies\{method="getSlot",host="solana-rpc.publicnode.com",le="\d+"\} 1 \d+"#)
            .assert_does_not_contain_metric_matching(r#"solrpc_latencies_bucket\{method="getAccountInfo",host="solana-mainnet.g.alchemy.com",le="\d+"\} 1 \d+"#)
            // `solrpc_inconsistent_responses` counters: inconsistent results
            .assert_contains_metric_matching(r#"solrpc_inconsistent_responses\{method="getSlot",host="solana-mainnet.g.alchemy.com"} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_inconsistent_responses\{method="getSlot",host="rpc.ankr.com"} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_inconsistent_responses\{method="getSlot",host="solana-mainnet.core.chainstack.com"} 1 \d+"#)
            .assert_does_not_contain_metric_matching(r#"solrpc_inconsistent_responses\{method="getSlot",host="lb.drpc.org"} 1 \d+"#)
            .assert_does_not_contain_metric_matching(r#"solrpc_inconsistent_responses\{method="getSlot",host="mainnet.helius-rpc.com"} 1 \d+"#)
            .assert_does_not_contain_metric_matching(r#"solrpc_inconsistent_responses\{method="getSlot",host="solana-rpc.publicnode.com"} 1 \d+"#)
            .assert_does_not_contain_metric_matching(r#"solrpc_inconsistent_responses\{method="getAccountInfo",host="solana-mainnet.g.alchemy.com"} 1 \d+"#);
    }

    #[tokio::test]
    async fn should_not_record_metrics_when_not_enough_cycles() {
        let setup = Setup::new().await.with_mock_api_keys().await;
        let client = setup.client(MockHttpOutcalls::never()).build();

        // Send a small enough amount that all outcalls fail due to insufficient cycles, but enough
        // so that all requests have at least the base HTTP outcall fee
        let result = client
            .get_slot()
            .with_cycles(550_000_000)
            .send()
            .await
            .expect_inconsistent();
        assert!(result.iter().all(|(_source, e)| matches!(
            e,
            Err(RpcError::ProviderError(ProviderError::TooFewCycles { .. }))
        )));

        setup
            .check_metrics()
            .await
            .assert_does_not_contain_metric_matching(r#"solrpc_requests.*"#)
            .assert_does_not_contain_metric_matching(r#"solrpc_responses.*"#)
            .assert_does_not_contain_metric_matching(r#"solrpc_latencies_bucket.*"#)
            .assert_does_not_contain_metric_matching(r#"solrpc_inconsistent_responses.*"#);
    }
}

#[tokio::test]
async fn should_not_drain_canister_balance_when_insufficient_cycles_attached() {
    let setup = Setup::new().await.with_mock_api_keys().await;

    let client = setup
        .client(MockHttpOutcalls::never())
        .with_rpc_sources(RpcSources::Custom(vec![RpcSource::Supported(
            SupportedRpcProviderId::AnkrMainnet,
        )]))
        .build();

    let required_cycles = client
        .get_block(0)
        .with_transaction_details(TransactionDetails::Signatures)
        .request_cost()
        .send()
        .await
        .unwrap();

    for cycles in [0_u128, required_cycles - 1_000] {
        let balance_before = setup.get_canister_cycle_balance().await;
        let results = client
            .get_block(0)
            .with_transaction_details(TransactionDetails::Signatures)
            .with_cycles(cycles)
            .try_send()
            .await;

        assert!(
            results.is_err()
                || matches!(
                    results,
                    Ok(MultiRpcResult::Consistent(Err(RpcError::ProviderError(
                        ProviderError::TooFewCycles { .. }
                    ))))
                )
        );

        let balance_after = setup.get_canister_cycle_balance().await;

        // Rejecting requests with insufficient cycles attached still costs a small amount in execution costs
        assert!(
            balance_after >= balance_before - 30_000_000,
            "Canister cycle balance decrease: {:?}",
            balance_before - balance_after
        );
    }
}

#[tokio::test]
async fn should_log_request_and_response() {
    let setup = Setup::new().await.with_mock_api_keys().await;

    let mocks = MockHttpOutcallsBuilder::new()
        .given(get_slot_request())
        .respond_with(get_slot_response(1234));
    let client = setup
        .client(mocks)
        .with_rpc_sources(RpcSources::Custom(vec![RpcSource::Supported(
            SupportedRpcProviderId::AlchemyMainnet,
        )]))
        .build();

    let results = client
        .get_slot()
        .with_rounding_error(0)
        .send()
        .await
        .expect_consistent();
    assert_eq!(results, Ok(1234));

    let logs = setup.retrieve_logs("TRACE_HTTP").await;
    assert_eq!(logs.len(), 2, "Unexpected amount of logs: {logs:?}");

    assert_eq!(logs[0].message, "JSON-RPC request with id `00000000000000000000` to solana-mainnet.g.alchemy.com: JsonRpcRequest { jsonrpc: V2, method: \"getSlot\", id: String(\"00000000000000000000\"), params: Some(GetSlotParams { config: None }) }");
    assert_eq!(logs[1].message, "Got response for request with id `00000000000000000000`. Response with status 200 OK: JsonRpcResponse { jsonrpc: V2, id: String(\"00000000000000000000\"), result: Ok(1234) }");

    setup.drop().await;
}

#[tokio::test]
async fn should_change_default_providers_when_one_keeps_failing() {
    let setup = Setup::new().await.with_mock_api_keys().await;

    let mocks = MockHttpOutcallsBuilder::new()
        .given(
            get_slot_request()
                .with_host("solana-mainnet.g.alchemy.com")
                .with_id(0),
        )
        .respond_with(get_slot_response(1200).with_id(0))
        .given(get_slot_request().with_host("lb.drpc.org").with_id(1))
        .respond_with(CanisterHttpReply::with_status(500))
        .given(
            get_slot_request()
                .with_host("mainnet.helius-rpc.com")
                .with_id(2),
        )
        .respond_with(get_slot_response(1200).with_id(2));
    let client = setup
        .client(mocks)
        .with_consensus_strategy(ConsensusStrategy::Threshold {
            min: 2,
            total: Some(3),
        })
        .build();

    let slot = client.get_slot().send().await.expect_consistent();
    assert_eq!(slot, Ok(1200));

    let mocks = MockHttpOutcallsBuilder::new()
        .given(get_slot_request().with_host("rpc.ankr.com").with_id(3))
        .respond_with(get_slot_response(1200).with_id(3));
    let client = setup
        .client(mocks)
        .with_consensus_strategy(ConsensusStrategy::Equality)
        .with_rpc_sources(RpcSources::Custom(vec![RpcSource::Supported(
            SupportedRpcProviderId::AnkrMainnet,
        )]))
        .build();

    let slot = client.get_slot().send().await.expect_consistent();
    assert_eq!(slot, Ok(1200));

    let mocks = MockHttpOutcallsBuilder::new()
        .given(
            get_slot_request()
                .with_host("solana-mainnet.g.alchemy.com")
                .with_id(4),
        )
        .respond_with(get_slot_response(1200).with_id(4))
        .given(get_slot_request().with_host("rpc.ankr.com").with_id(5))
        .respond_with(get_slot_response(1200).with_id(5))
        .given(
            get_slot_request()
                .with_host("mainnet.helius-rpc.com")
                .with_id(6),
        )
        .respond_with(get_slot_response(1200).with_id(6));
    let client = setup
        .client(mocks)
        .with_consensus_strategy(ConsensusStrategy::Threshold {
            min: 3,
            total: Some(3),
        })
        .build();

    let slot = client.get_slot().send().await.expect_consistent();
    assert_eq!(slot, Ok(1200));

    setup.drop().await;
}

fn assert_within(actual: u128, expected: u128, percentage_error: u8) {
    assert!(percentage_error <= 100);
    let error_margin = expected.saturating_mul(percentage_error as u128) / 100;
    let lower_bound = expected.saturating_sub(error_margin);
    let upper_bound = expected.saturating_add(error_margin);
    assert!(
        lower_bound <= actual && actual <= upper_bound,
        "Expected {} <= {} <= {}",
        lower_bound,
        actual,
        upper_bound
    );
}

fn some_transaction() -> solana_transaction::Transaction {
    let keypair = solana_keypair::Keypair::new();
    solana_transaction::Transaction::new_signed_with_payer(
        &[],
        Some(&keypair.pubkey()),
        &[keypair],
        solana_hash::Hash::from_str("4Pcj2yJkCYyhnWe8Ze3uK2D2EtesBxhAevweDoTcxXf3").unwrap(),
    )
}

fn some_signature() -> solana_signature::Signature {
    solana_signature::Signature::from_str(
        "KbYRTmvx4uz3xuRRGNdKyt1jBngz2TjLp9nPebT4h3LQzAG7BfYrd5pSU2xDT7dVg3EXXbZugH8XbKwiGU7Jqzw",
    )
    .unwrap()
}

fn another_signature() -> solana_signature::Signature {
    solana_signature::Signature::from_str(
        "4XLJdFbdYYzzBMqvji9bq6ZgzRx5G9edjkJQGprMoAarJSbNbbHt1DTCZqcA7mYk4bJPgC6w7tFjYEtw1jJJSdyw",
    )
    .unwrap()
}

fn mock_all_endpoints(
    request: impl Fn(JsonRpcRequestMatcher) -> JsonRpcRequestMatcher,
    response: impl Into<CanisterHttpResponse>,
) -> MockHttpOutcallsBuilder {
    let mut mocks = MockHttpOutcallsBuilder::new();
    let mut ids = 0_u64..;
    let response = response.into();
    for endpoint in SolRpcEndpoint::iter() {
        let rpc_method = if endpoint == SolRpcEndpoint::JsonRequest {
            "getVersion"
        } else {
            endpoint.rpc_method()
        };
        for id in ids.by_ref().take(3) {
            mocks = mocks
                .given(request(
                    JsonRpcRequestMatcher::with_method(rpc_method).with_id(id),
                ))
                .respond_with(response.clone());
        }
    }
    mocks
}

fn get_account_info_request() -> JsonRpcRequestMatcher {
    JsonRpcRequestMatcher::with_method("getAccountInfo")
        .with_params(json!([USDC_PUBLIC_KEY.to_string(), null]))
        .with_id(0)
}

fn get_balance_request() -> JsonRpcRequestMatcher {
    JsonRpcRequestMatcher::with_method("getBalance")
        .with_params(json!([USDC_PUBLIC_KEY.to_string(), {"commitment": "confirmed", "minContextSlot": 100}]))
        .with_id(0)
}

fn get_block_request() -> JsonRpcRequestMatcher {
    JsonRpcRequestMatcher::with_method("getBlock")
        .with_params(json!([577996, {"transactionDetails": "none"}]))
        .with_id(0)
}

fn get_recent_prioritization_fees_request() -> JsonRpcRequestMatcher {
    JsonRpcRequestMatcher::with_method("getRecentPrioritizationFees")
        .with_params(json!([[USDC_PUBLIC_KEY.to_string()]]))
        .with_id(0)
}

fn get_signatures_for_address_request() -> JsonRpcRequestMatcher {
    JsonRpcRequestMatcher::with_method("getSignaturesForAddress")
        .with_params(json!([USDC_PUBLIC_KEY.to_string(), {"limit": 5}]))
        .with_id(0)
}

fn get_signature_statuses_request() -> JsonRpcRequestMatcher {
    JsonRpcRequestMatcher::with_method("getSignatureStatuses")
        .with_params(json!([
            [some_signature().to_string(), another_signature().to_string()],
            { "searchTransactionHistory": true }
        ]))
        .with_id(0)
}

fn get_slot_request() -> JsonRpcRequestMatcher {
    JsonRpcRequestMatcher::with_method("getSlot")
        .with_params(json!([null]))
        .with_id(0)
}

fn get_token_account_balance_request() -> JsonRpcRequestMatcher {
    JsonRpcRequestMatcher::with_method("getTokenAccountBalance")
        .with_params(json!([USDC_PUBLIC_KEY.to_string(), {"commitment": "confirmed"}]))
        .with_id(0)
}

fn get_transaction_request() -> JsonRpcRequestMatcher {
    JsonRpcRequestMatcher::with_method("getTransaction")
        .with_params(json!([some_signature().to_string(), {"encoding": "base64"}]))
        .with_id(0)
}

fn get_version_request_body() -> Value {
    json!({"jsonrpc": "2.0", "id": Id::from(ConstantSizeId::ZERO), "method": "getVersion"})
}

fn get_version_request() -> JsonRpcRequestMatcher {
    JsonRpcRequestMatcher::with_method("getVersion").with_id(0)
}

fn send_transaction_request(
    transaction: &solana_transaction::Transaction,
) -> JsonRpcRequestMatcher {
    fn serialize_transaction(transaction: &solana_transaction::Transaction) -> String {
        use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
        let serialized = bincode::serialize(transaction).expect("Failed to serialize transaction");
        BASE64_STANDARD.encode(serialized)
    }

    JsonRpcRequestMatcher::with_method("sendTransaction")
        .with_params(json!([serialize_transaction(transaction), {"encoding": "base64"}]))
        .with_id(0)
}

fn get_account_info_response() -> JsonRpcResponse {
    JsonRpcResponse::from(json!({
        "id": Id::from(ConstantSizeId::ZERO),
        "jsonrpc": "2.0",
        "result": {
            "context": { "apiVersion": "2.0.15", "slot": 341197053 },
            "value": {
                "data": ["1234", "base58"],
                "executable": false,
                "lamports": 88849814690250u64,
                "owner": "11111111111111111111111111111111",
                "rentEpoch": 18446744073709551615u64,
                "space": 0
            }
        },
    }))
}

fn get_balance_response(slot: Slot) -> JsonRpcResponse {
    JsonRpcResponse::from(json!({
        "id": Id::from(ConstantSizeId::ZERO),
        "jsonrpc": "2.0",
        "result": {
            // context should be filtered out by transform
            "context": { "slot": slot, "apiVersion": "2.1.9" },
            "value": 389086612571_u64
        },
    }))
}

fn get_block_response() -> JsonRpcResponse {
    JsonRpcResponse::from(json!({
        "id": Id::from(ConstantSizeId::ZERO),
        "jsonrpc": "2.0",
        "result":{
            "blockHeight": 360854634,
            "blockTime": 1744122369,
            "parentSlot": 372877611,
            "blockhash": "8QeCusqSTKeC23NwjTKRBDcPuEfVLtszkxbpL6mXQEp4",
            "previousBlockhash": "4Pcj2yJkCYyhnWe8Ze3uK2D2EtesBxhAevweDoTcxXf3"}
    }))
}

fn get_recent_prioritization_fees_response() -> JsonRpcResponse {
    JsonRpcResponse::from(json!({
        "jsonrpc": "2.0",
        "result": [
            {
                "prioritizationFee": 0,
                "slot": 338225766
            },
            {
                "prioritizationFee": 203228,
                "slot": 338225767
            },
            {
                "prioritizationFee": 110788,
                "slot": 338225768
            },
            {
                "prioritizationFee": 395962,
                "slot": 338225769
            },
            {
                "prioritizationFee": 0,
                "slot": 338225770
            },
            {
                "prioritizationFee": 395477,
                "slot": 338225771
            },
            {
                "prioritizationFee": 202136,
                "slot": 338225772
            },
            {
                "prioritizationFee": 0,
                "slot": 338225773
            },
            {
                "prioritizationFee": 0,
                "slot": 338225774
            },
            {
                "prioritizationFee": 0,
                "slot": 338225775
            },
            {
                "prioritizationFee": 2894338,
                "slot": 338225776
            },
            {
                "prioritizationFee": 0,
                "slot": 338225777
            },
            {
                "prioritizationFee": 162918,
                "slot": 338225778
            },
            {
                "prioritizationFee": 238785,
                "slot": 338225779
            },
            {
                "prioritizationFee": 10714,
                "slot": 338225780
            },
            {
                "prioritizationFee": 81000,
                "slot": 338225781
            },
            {
                "prioritizationFee": 0,
                "slot": 338225782
            },
            {
                "prioritizationFee": 0,
                "slot": 338225783
            },
            {
                "prioritizationFee": 202136,
                "slot": 338225784
            },
            {
                "prioritizationFee": 166667,
                "slot": 338225785
            },
            {
                "prioritizationFee": 166667,
                "slot": 338225786
            },
            {
                "prioritizationFee": 0,
                "slot": 338225787
            },
            {
                "prioritizationFee": 0,
                "slot": 338225788
            },
            {
                "prioritizationFee": 0,
                "slot": 338225789
            },
            {
                "prioritizationFee": 0,
                "slot": 338225790
            },
            {
                "prioritizationFee": 0,
                "slot": 338225791
            },
            {
                "prioritizationFee": 0,
                "slot": 338225792
            },
            {
                "prioritizationFee": 0,
                "slot": 338225793
            },
            {
                "prioritizationFee": 494120,
                "slot": 338225794
            },
            {
                "prioritizationFee": 0,
                "slot": 338225795
            },
            {
                "prioritizationFee": 0,
                "slot": 338225796
            },
            {
                "prioritizationFee": 202136,
                "slot": 338225797
            },
            {
                "prioritizationFee": 0,
                "slot": 338225798
            },
            {
                "prioritizationFee": 0,
                "slot": 338225799
            },
            {
                "prioritizationFee": 202136,
                "slot": 338225800
            },
            {
                "prioritizationFee": 0,
                "slot": 338225801
            },
            {
                "prioritizationFee": 0,
                "slot": 338225802
            },
            {
                "prioritizationFee": 10001,
                "slot": 338225803
            },
            {
                "prioritizationFee": 0,
                "slot": 338225804
            },
            {
                "prioritizationFee": 0,
                "slot": 338225805
            },
            {
                "prioritizationFee": 0,
                "slot": 338225806
            },
            {
                "prioritizationFee": 0,
                "slot": 338225807
            },
            {
                "prioritizationFee": 202136,
                "slot": 338225808
            },
            {
                "prioritizationFee": 0,
                "slot": 338225809
            },
            {
                "prioritizationFee": 202136,
                "slot": 338225810
            },
            {
                "prioritizationFee": 0,
                "slot": 338225811
            },
            {
                "prioritizationFee": 0,
                "slot": 338225812
            },
            {
                "prioritizationFee": 0,
                "slot": 338225813
            },
            {
                "prioritizationFee": 0,
                "slot": 338225814
            },
            {
                "prioritizationFee": 6064097,
                "slot": 338225815
            },
            {
                "prioritizationFee": 0,
                "slot": 338225816
            },
            {
                "prioritizationFee": 0,
                "slot": 338225817
            },
            {
                "prioritizationFee": 0,
                "slot": 338225818
            },
            {
                "prioritizationFee": 517927,
                "slot": 338225819
            },
            {
                "prioritizationFee": 0,
                "slot": 338225820
            },
            {
                "prioritizationFee": 0,
                "slot": 338225821
            },
            {
                "prioritizationFee": 0,
                "slot": 338225822
            },
            {
                "prioritizationFee": 602011,
                "slot": 338225823
            },
            {
                "prioritizationFee": 187015,
                "slot": 338225824
            },
            {
                "prioritizationFee": 50000,
                "slot": 338225825
            },
            {
                "prioritizationFee": 0,
                "slot": 338225826
            },
            {
                "prioritizationFee": 0,
                "slot": 338225827
            },
            {
                "prioritizationFee": 0,
                "slot": 338225828
            },
            {
                "prioritizationFee": 0,
                "slot": 338225829
            },
            {
                "prioritizationFee": 0,
                "slot": 338225830
            },
            {
                "prioritizationFee": 0,
                "slot": 338225831
            },
            {
                "prioritizationFee": 0,
                "slot": 338225832
            },
            {
                "prioritizationFee": 0,
                "slot": 338225833
            },
            {
                "prioritizationFee": 0,
                "slot": 338225834
            },
            {
                "prioritizationFee": 0,
                "slot": 338225835
            },
            {
                "prioritizationFee": 0,
                "slot": 338225836
            },
            {
                "prioritizationFee": 0,
                "slot": 338225837
            },
            {
                "prioritizationFee": 0,
                "slot": 338225838
            },
            {
                "prioritizationFee": 487330,
                "slot": 338225839
            },
            {
                "prioritizationFee": 149432,
                "slot": 338225840
            },
            {
                "prioritizationFee": 0,
                "slot": 338225841
            },
            {
                "prioritizationFee": 0,
                "slot": 338225842
            },
            {
                "prioritizationFee": 68526,
                "slot": 338225843
            },
            {
                "prioritizationFee": 0,
                "slot": 338225844
            },
            {
                "prioritizationFee": 310090,
                "slot": 338225845
            },
            {
                "prioritizationFee": 0,
                "slot": 338225846
            },
            {
                "prioritizationFee": 2173913,
                "slot": 338225847
            },
            {
                "prioritizationFee": 99725,
                "slot": 338225848
            },
            {
                "prioritizationFee": 0,
                "slot": 338225849
            },
            {
                "prioritizationFee": 88441,
                "slot": 338225850
            },
            {
                "prioritizationFee": 0,
                "slot": 338225851
            },
            {
                "prioritizationFee": 400000,
                "slot": 338225852
            },
            {
                "prioritizationFee": 0,
                "slot": 338225853
            },
            {
                "prioritizationFee": 0,
                "slot": 338225854
            },
            {
                "prioritizationFee": 164507,
                "slot": 338225855
            },
            {
                "prioritizationFee": 0,
                "slot": 338225856
            },
            {
                "prioritizationFee": 4898,
                "slot": 338225857
            },
            {
                "prioritizationFee": 0,
                "slot": 338225858
            },
            {
                "prioritizationFee": 0,
                "slot": 338225859
            },
            {
                "prioritizationFee": 142369,
                "slot": 338225860
            },
            {
                "prioritizationFee": 84566,
                "slot": 338225861
            },
            {
                "prioritizationFee": 0,
                "slot": 338225862
            },
            {
                "prioritizationFee": 10001,
                "slot": 338225863
            },
            {
                "prioritizationFee": 187015,
                "slot": 338225864
            },
            {
                "prioritizationFee": 8902,
                "slot": 338225865
            },
            {
                "prioritizationFee": 0,
                "slot": 338225866
            },
            {
                "prioritizationFee": 75000,
                "slot": 338225867
            },
            {
                "prioritizationFee": 0,
                "slot": 338225868
            },
            {
                "prioritizationFee": 0,
                "slot": 338225869
            },
            {
                "prioritizationFee": 1771477,
                "slot": 338225870
            },
            {
                "prioritizationFee": 1110536,
                "slot": 338225871
            },
            {
                "prioritizationFee": 215920,
                "slot": 338225872
            },
            {
                "prioritizationFee": 68408,
                "slot": 338225873
            },
            {
                "prioritizationFee": 0,
                "slot": 338225874
            },
            {
                "prioritizationFee": 260520,
                "slot": 338225875
            },
            {
                "prioritizationFee": 2143332,
                "slot": 338225876
            },
            {
                "prioritizationFee": 0,
                "slot": 338225877
            },
            {
                "prioritizationFee": 84168,
                "slot": 338225878
            },
            {
                "prioritizationFee": 0,
                "slot": 338225879
            },
            {
                "prioritizationFee": 0,
                "slot": 338225880
            },
            {
                "prioritizationFee": 501111,
                "slot": 338225881
            },
            {
                "prioritizationFee": 88060,
                "slot": 338225882
            },
            {
                "prioritizationFee": 10001,
                "slot": 338225883
            },
            {
                "prioritizationFee": 171521,
                "slot": 338225884
            },
            {
                "prioritizationFee": 0,
                "slot": 338225885
            },
            {
                "prioritizationFee": 6064097,
                "slot": 338225886
            },
            {
                "prioritizationFee": 6064097,
                "slot": 338225887
            },
            {
                "prioritizationFee": 0,
                "slot": 338225888
            },
            {
                "prioritizationFee": 7578,
                "slot": 338225889
            },
            {
                "prioritizationFee": 0,
                "slot": 338225890
            },
            {
                "prioritizationFee": 0,
                "slot": 338225891
            },
            {
                "prioritizationFee": 202136,
                "slot": 338225892
            },
            {
                "prioritizationFee": 106090,
                "slot": 338225893
            },
            {
                "prioritizationFee": 80776,
                "slot": 338225894
            },
            {
                "prioritizationFee": 111939,
                "slot": 338225895
            },
            {
                "prioritizationFee": 75000,
                "slot": 338225896
            },
            {
                "prioritizationFee": 0,
                "slot": 338225897
            },
            {
                "prioritizationFee": 0,
                "slot": 338225898
            },
            {
                "prioritizationFee": 0,
                "slot": 338225899
            },
            {
                "prioritizationFee": 0,
                "slot": 338225900
            },
            {
                "prioritizationFee": 0,
                "slot": 338225901
            },
            {
                "prioritizationFee": 183582,
                "slot": 338225902
            },
            {
                "prioritizationFee": 0,
                "slot": 338225903
            },
            {
                "prioritizationFee": 0,
                "slot": 338225904
            },
            {
                "prioritizationFee": 0,
                "slot": 338225905
            },
            {
                "prioritizationFee": 535775,
                "slot": 338225906
            },
            {
                "prioritizationFee": 65038,
                "slot": 338225907
            },
            {
                "prioritizationFee": 0,
                "slot": 338225908
            },
            {
                "prioritizationFee": 0,
                "slot": 338225909
            },
            {
                "prioritizationFee": 0,
                "slot": 338225910
            },
            {
                "prioritizationFee": 0,
                "slot": 338225911
            },
            {
                "prioritizationFee": 0,
                "slot": 338225912
            },
            {
                "prioritizationFee": 0,
                "slot": 338225913
            },
            {
                "prioritizationFee": 0,
                "slot": 338225914
            },
            {
                "prioritizationFee": 0,
                "slot": 338225915
            }
        ],
        "id": Id::from(ConstantSizeId::ZERO),
        }
    ))
}

fn get_signatures_for_address_response() -> JsonRpcResponse {
    JsonRpcResponse::from(json!({
        "id": Id::from(ConstantSizeId::ZERO),
        "jsonrpc": "2.0",
        "result": [
            {
                "signature": "3jPA8CnZb9sfs4zVAypa9KB7VAGwrTdXB6mg9H1H9XpATN6Y8iek4Y21Nb9LjbrpYACbF9USV8RBWvXFFhVoQUAs",
                "confirmationStatus": "finalized",
                "memo": null,
                "slot": 340_372_399,
                "err": null,
                "blockTime": 1_747_389_084,
            },
            {
                "signature": "3WM42nYDQAHgBWFd6SbJ3pj1AGgiTJfxXJ2d5dHu49GgqSUui5qdh64S5yLCN1cMKcLMFVKKo776GrtVhfatLqP6",
                "confirmationStatus": "finalized",
                "memo": null,
                "slot": 340_372_399,
                "err": null,
                "blockTime": 1_747_389_084,
            },
            {
                "signature": "5iByUT1gTNXDY24hRx25YmQeebvUMD6jsNpGcu2jh1yjKmYwdo5GtRrYozyhdtdcn8SurwHq6EMp4YTpHgdansjc",
                "confirmationStatus": "finalized",
                "memo": null,
                "slot": 340_372_399,
                "err": null,
                "blockTime": 1_747_389_084,
            },
            {
                "signature": "2Zuhxr6qMGwBrpV611Ema7pZAy1WGSkQyurTcbfyoXwFMNuziUJbM6FCyoL8WxTRG6G3fEik2wSFeN76miUeUnmJ",
                "confirmationStatus": "finalized",
                "memo": null,
                "slot": 340_372_399,
                "err": null,
                "blockTime": 1_747_389_084,
            },
            {
                "signature": "4V1j8jZvXjcUdRoWQBRzxFVigfr61bJdHGsCFAkTm5h4z28FkrDczuTpcvwTRamiwiGm7E77EB5DKRBwG1mUEC8f",
                "confirmationStatus": "finalized",
                "memo": null,
                "slot": 340_372_399,
                "err": {
                    "InstructionError" : [ 3, { "Custom" : 6_001 } ],
                },
                "blockTime": 1_747_389_084,
            },
        ]
    }))
}

fn get_signature_statuses_response(slot: Slot) -> JsonRpcResponse {
    JsonRpcResponse::from(json!({
        "id": Id::from(ConstantSizeId::ZERO),
        "jsonrpc": "2.0",
        "result": {
            // context should be filtered out by transform
            "context": { "slot": slot, "apiVersion": "2.1.9" },
            "value": [
                  {
                    "slot": 48,
                    // confirmations should be filtered out by transform
                    "confirmations": (slot >> 32) as u32,
                    "err": null,
                    "status": { "Ok": null },
                    "confirmationStatus": "finalized"
                  },
                  null
            ]
        },
    }))
}

fn get_slot_response(slot: Slot) -> JsonRpcResponse {
    JsonRpcResponse::from(json!({
        "id": Id::from(ConstantSizeId::ZERO),
        "jsonrpc": "2.0",
        "result": slot,
    }))
}

fn get_token_account_balance_response(slot: Slot) -> JsonRpcResponse {
    JsonRpcResponse::from(json!({
        "id": Id::from(ConstantSizeId::ZERO),
        "jsonrpc": "2.0",
        "result": {
            // context should be filtered out by transform
            "context": { "slot": slot, "apiVersion": "2.1.9" },
            "value": {
                "amount": "9864",
                "decimals": 2,
                "uiAmount": 98.64,
                "uiAmountString": "98.64",
            }
        },
    }))
}

fn get_transaction_response() -> JsonRpcResponse {
    JsonRpcResponse::from(json!({
        "id": Id::from(ConstantSizeId::ZERO),
        "jsonrpc": "2.0",
        "result": {
            "blockTime": 1758792475,
            "meta": {
                "computeUnitsConsumed": 410,
                "costUnits": 2084,
                "err": null,
                "fee": 5000,
                "innerInstructions": [],
                "loadedAddresses": {
                    "readonly": [],
                    "writable": []
                },
                "logMessages": [
                    "Program ComputeBudget111111111111111111111111111111 invoke [1]",
                    "Program ComputeBudget111111111111111111111111111111 success",
                    "Program E2uCGJ4TtYyKPGaK57UMfbs9sgaumwDEZF1aAY6fF3mS invoke [1]",
                    "Program E2uCGJ4TtYyKPGaK57UMfbs9sgaumwDEZF1aAY6fF3mS consumed 110 of 270 compute units",
                    "Program E2uCGJ4TtYyKPGaK57UMfbs9sgaumwDEZF1aAY6fF3mS success",
                    "Program 11111111111111111111111111111111 invoke [1]",
                    "Program 11111111111111111111111111111111 success"
                ],
                "postBalances": [
                    463360314850_u64,
                    6609068,
                    2060160,
                    1,
                    1,
                    1141440
                ],
                "postTokenBalances": [],
                "preBalances": [
                    463360320850_u64,
                    6608068,
                    2060160,
                    1,
                    1,
                    1141440
                ],
                "preTokenBalances": [],
                "rewards": [],
                "status": {
                    "Ok": null
                }
            },
            "slot": 369139986,
            "transaction": [
                "ARAJPXmph5xbnfO74gv8tBIwTA0yw0BuRZvqrr113O9BTj0T4kXejUz3jh1RCasjsZkr2do/ZjMIOg56TTvRlQgBAAMGDEiA3o3u6XvTb57cHKZkhrHuNhISrOgMMafRPe48Q4QgJhAewgMolkoyq6sTbFQFuR86447k9ky2veh5uGg40kK5Pth9DxkikievxiovoyrY6lRfLhWKUZINPu2s+AlMAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADBkZv5SEXMv/srbpyw5vnvIzlu8X3EmssQ5s6QAAAAMGkhusDr3enQhfGliLPnjUOYbtCSz9fET+Twnd+37hJkr+3Zt+dBsrfJ0eCM1bDr9NITRuvFbzpE4a9q1ZEXggDBAAFAqQBAAAFAgACqAELVaozzA/wZnC9ckuJIt1EqfSq6QAzzGYyZzOAmQEAAHF0Ee4i3YhEjwv/FswzZpkBBxEiM0RVZneImaq7zN3u/wCqVTPMZpkSNFZ4mrze8BI0VniavN7wEjRWeJq83vASNFZ4mrze8BI0VniavN7wEjRWeJq83vASNFZ4mrze8BI0VniavN7wEjRWeJq83vASNFZ4mrze8AxIgN6N7ul702+e3BymZIYDAgABDAIAAADoAwAAAAAAAA==",
                "base64"
            ]
        },
    }))
}

fn get_version_response() -> JsonRpcResponse {
    JsonRpcResponse::from(json!({
        "jsonrpc": "2.0",
        "result": {
            "feature-set": 3640012085_u64,
            "solana-core": "2.3.6"
        },
        "id": 0
    }))
}

fn send_transaction_response() -> JsonRpcResponse {
    JsonRpcResponse::from(json!({
        "id": Id::from(ConstantSizeId::ZERO),
        "jsonrpc": "2.0",
        "result": some_signature().to_string(),
    }))
}

fn not_found_response() -> JsonRpcResponse {
    JsonRpcResponse::from(json!({"id": 0, "jsonrpc": "2.0", "result": null}))
}

fn mock_for_ids(
    request: impl Fn() -> JsonRpcRequestMatcher,
    response: impl Fn() -> JsonRpcResponse,
    ids: impl IntoIterator<Item = u64>,
) -> MockHttpOutcallsBuilder {
    let mut mocks = MockHttpOutcallsBuilder::new();
    for id in ids {
        mocks = mocks
            .given(request().with_id(id))
            .respond_with(response().with_id(id))
    }
    mocks
}

fn mock_with_response_slots_for_ids(
    request: impl Fn() -> JsonRpcRequestMatcher,
    response: impl Fn(Slot) -> JsonRpcResponse,
    slots: impl IntoIterator<Item = Slot>,
    ids: impl IntoIterator<Item = u64>,
) -> MockHttpOutcallsBuilder {
    let mut mocks = MockHttpOutcallsBuilder::new();
    for (slot, id) in zip(slots, ids) {
        mocks = mocks
            .given(request().with_id(id))
            .respond_with(response(slot).with_id(id))
    }
    mocks
}
