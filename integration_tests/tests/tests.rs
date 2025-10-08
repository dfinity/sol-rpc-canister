#![recursion_limit = "512"]
use assert_matches::*;
use candid::CandidType;
use canhttp::http::json::{ConstantSizeId, Id};
use const_format::formatcp;
use ic_cdk::api::{call::RejectionCode, management_canister::http_request::HttpHeader};
use pocket_ic::common::rest::CanisterHttpMethod;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use sol_rpc_canister::constants::*;
use sol_rpc_client::{RequestBuilder, SolRpcClient, SolRpcConfig, SolRpcEndpoint};
use sol_rpc_int_tests::{
    json_rpc_sequential_id, mock::MockOutcallBuilder, PocketIcRuntime, Setup, SolRpcTestClient,
    DEFAULT_CALLER_TEST_ID,
};
use sol_rpc_types::{
    CommitmentLevel, ConfirmedTransactionStatusWithSignature, ConsensusStrategy,
    GetSignaturesForAddressLimit, GetSlotParams, GetTransactionEncoding, HttpOutcallError,
    InstallArgs, InstructionError, Mode, MultiRpcResult, PrioritizationFee, ProviderError,
    RpcAccess, RpcAuth, RpcEndpoint, RpcError, RpcResult, RpcSource, RpcSources, Slot,
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

const MOCK_REQUEST_URL: &str = "https://api.devnet.solana.com/";
const MOCK_RESPONSE_RESULT: &str = r#"{"feature-set":2891131721,"solana-core":"1.16.7"}"#;
const MOCK_RESPONSE: &str = formatcp!(
    "{{\"jsonrpc\":\"2.0\",\"id\":\"00000000000000000000\",\"result\":{}}}",
    MOCK_RESPONSE_RESULT
);
const MOCK_REQUEST_MAX_RESPONSE_BYTES: u64 = 1000;
const USDC_PUBLIC_KEY: solana_pubkey::Pubkey =
    pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
// See: https://internetcomputer.org/docs/references/cycles-cost-formulas#https-outcalls
const HTTP_OUTCALL_BASE_FEE: u128 = (3_000_000 + 60_000 * 34) * 34;

mod mock_request_tests {
    use super::*;

    async fn mock_request(builder_fn: impl Fn(MockOutcallBuilder) -> MockOutcallBuilder) {
        let setup = Setup::with_args(InstallArgs {
            mode: Some(Mode::Demo),
            ..Default::default()
        })
        .await;
        let client = setup
            .client()
            .with_response_size_estimate(MOCK_REQUEST_MAX_RESPONSE_BYTES)
            .with_rpc_sources(RpcSources::Custom(vec![RpcSource::Custom(RpcEndpoint {
                url: MOCK_REQUEST_URL.to_string(),
                headers: Some(vec![HttpHeader {
                    name: "custom".to_string(),
                    value: "Value".to_string(),
                }]),
            })]));
        let expected_result: Value = serde_json::from_str(MOCK_RESPONSE).unwrap();
        assert_matches!(
            client
                .mock_http(builder_fn(MockOutcallBuilder::new(200, MOCK_RESPONSE))).build()
                .json_request(get_version_request())
                .with_cycles(0)
                .send()
                .await,
            MultiRpcResult::Consistent(Ok(msg)) if msg == Value::to_string(&expected_result["result"])
        );
    }

    #[tokio::test]
    async fn mock_request_should_succeed() {
        mock_request(|builder| builder).await
    }

    #[tokio::test]
    async fn mock_request_should_succeed_with_url() {
        mock_request(|builder| builder.with_url(MOCK_REQUEST_URL)).await
    }

    #[tokio::test]
    async fn mock_request_should_succeed_with_method() {
        mock_request(|builder| builder.with_method(CanisterHttpMethod::POST)).await
    }

    #[tokio::test]
    async fn mock_request_should_succeed_with_request_headers() {
        mock_request(|builder| {
            builder.with_request_headers(vec![
                (CONTENT_TYPE_HEADER_LOWERCASE, CONTENT_TYPE_VALUE),
                ("custom", "Value"),
            ])
        })
        .await
    }

    #[tokio::test]
    async fn mock_request_should_succeed_with_request_body() {
        mock_request(|builder| builder.with_request_body(get_version_request())).await
    }

    #[tokio::test]
    async fn mock_request_should_succeed_with_max_response_bytes() {
        mock_request(|builder| builder.with_max_response_bytes(MOCK_REQUEST_MAX_RESPONSE_BYTES))
            .await
    }

    #[tokio::test]
    async fn mock_request_should_succeed_with_all() {
        mock_request(|builder| {
            builder
                .with_url(MOCK_REQUEST_URL)
                .with_method(CanisterHttpMethod::POST)
                .with_request_headers(vec![
                    (CONTENT_TYPE_HEADER_LOWERCASE, CONTENT_TYPE_VALUE),
                    ("custom", "Value"),
                ])
                .with_request_body(get_version_request())
        })
        .await
    }
}

mod get_provider_tests {
    use super::*;

    #[tokio::test]
    async fn should_get_providers() {
        let setup = Setup::new().await;
        let client = setup.client().build();
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

        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let client = setup.client().with_rpc_sources(sources);
            let pubkey =
                solana_pubkey::Pubkey::from_str("11111111111111111111111111111111").unwrap();

            let results = client
                .mock_sequential_json_rpc_responses::<3>(200, get_account_info_response(first_id))
                .build()
                .get_account_info(pubkey)
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

        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let client = setup.client().with_rpc_sources(sources);
            let pubkey =
                solana_pubkey::Pubkey::from_str("11111111111111111111111111111111").unwrap();

            let results = client
                .mock_sequential_json_rpc_responses::<3>(
                    200,
                    json!({
                        "id": Id::from(ConstantSizeId::from(first_id)),
                        "jsonrpc": "2.0",
                        "result": {
                            "context": { "apiVersion": "2.0.15", "slot": 341197053 }
                        },
                    }),
                )
                .build()
                .get_account_info(pubkey)
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

        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let client = setup.client().with_rpc_sources(sources);
            let slot: Slot = 123;

            let results = client
                .mock_sequential_json_rpc_responses::<3>(200, get_block_response(first_id))
                .build()
                .get_block(slot)
                .send()
                .await
                .expect_consistent();

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

        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let client = setup.client().with_rpc_sources(sources);
            let slot: Slot = 123;

            let results = client
                .mock_sequential_json_rpc_responses::<3>(
                    200,
                    json!({
                        "id": Id::from(ConstantSizeId::from(first_id)),
                        "jsonrpc": "2.0",
                        "result": null
                    }),
                )
                .build()
                .get_block(slot)
                .send()
                .await
                .expect_consistent();

            assert_eq!(results, Ok(None));
        }

        setup.drop().await;
    }
}

mod get_slot_tests {
    use super::*;

    #[tokio::test]
    async fn should_get_slot_with_full_params() {
        fn request_body(id: u8) -> Value {
            let id = ConstantSizeId::from(id).to_string();
            json!({ "jsonrpc": "2.0", "id": id, "method": "getSlot", "params": [{"commitment": "processed", "minContextSlot": 100}] })
        }

        let setup = Setup::new().await.with_mock_api_keys().await;
        let client = setup.client();

        let slot = client
            .mock_http_sequence(vec![
                MockOutcallBuilder::new(200, get_slot_response(0, 1234))
                    .with_request_body(request_body(0)),
                MockOutcallBuilder::new(200, get_slot_response(1, 1234))
                    .with_request_body(request_body(1)),
                MockOutcallBuilder::new(200, get_slot_response(2, 1234))
                    .with_request_body(request_body(2)),
            ])
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

        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let client = setup.client().with_rpc_sources(sources);

            let results = client
                .mock_sequential_json_rpc_responses::<3>(200, get_slot_response(first_id, 1234))
                .build()
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

        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let responses = [1234_u64, 1229, 1237]
                .into_iter()
                .enumerate()
                .map(|(id, slot)| {
                    MockOutcallBuilder::new(200, get_slot_response(id as u8 + first_id, slot))
                })
                .collect();
            let client = setup.client().with_rpc_sources(sources);

            let results = client
                .mock_http_sequence(responses)
                .build()
                .get_slot()
                .send()
                .await
                .expect_consistent();

            assert_eq!(results, Ok(1220));
        }

        setup.drop().await;
    }

    #[tokio::test]
    async fn should_get_inconsistent_result_without_rounding() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let responses = [1234_u64, 1229, 1237]
                .into_iter()
                .enumerate()
                .map(|(id, slot)| {
                    MockOutcallBuilder::new(200, get_slot_response(id as u8 + first_id, slot))
                })
                .collect();
            let client = setup.client().with_rpc_sources(sources);

            let results: Vec<RpcResult<_>> = client
                .mock_http_sequence(responses)
                .build()
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
        fn request_body(id: u8) -> Value {
            let id = ConstantSizeId::from(id).to_string();
            json!( { "jsonrpc": "2.0", "id": id, "method": "getRecentPrioritizationFees", "params": [ [ USDC_PUBLIC_KEY.to_string() ] ] } )
        }

        let setup = Setup::new().await.with_mock_api_keys().await;
        let client = setup.client();
        let fees = client
            .mock_http_sequence(vec![
                MockOutcallBuilder::new(200, get_recent_prioritization_fees_response(0))
                    .with_request_body(request_body(0)),
                MockOutcallBuilder::new(200, get_recent_prioritization_fees_response(1))
                    .with_request_body(request_body(1)),
                MockOutcallBuilder::new(200, get_recent_prioritization_fees_response(2))
                    .with_request_body(request_body(2)),
            ])
            .build()
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
        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let client = setup.client().with_rpc_sources(sources);

            let results = client
                .mock_sequential_json_rpc_responses::<3>(
                    200,
                    send_transaction_response(first_id, some_signature()),
                )
                .build()
                .send_transaction(some_transaction())
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
        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let client = setup.client().with_rpc_sources(sources);

            let results = client
                .mock_sequential_json_rpc_responses::<3>(200, get_transaction_response(first_id))
                .build()
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
            request: RequestBuilder<PocketIcRuntime<'_>, Config, Params, CandidOutput, Output>,
        ) where
            Config: CandidType + Clone + Send,
            Params: CandidType + Clone + Send,
            CandidOutput: Into<Output> + CandidType + DeserializeOwned,
        {
            let result = request
                .with_cycles(HTTP_OUTCALL_BASE_FEE - 1)
                .try_send()
                .await;
            assert!(result.is_err_and(|(_code, message)| message.contains("Not enough cycles")));
        }

        let setup = Setup::new().await.with_mock_api_keys().await;
        let client = setup.client().build();

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
                    check(client.json_request(get_version_request())).await;
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
            request: RequestBuilder<PocketIcRuntime<'_>, Config, Params, CandidOutput, Output>,
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
        let client = setup
            .client()
            // We always return a dummy response so that individual responses
            // do not need to be mocked.
            .mock_http(MockOutcallBuilder::new(403, json!({})))
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
                    check(client.json_request(get_version_request())).await;
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
        let client = setup.client();

        let result = client
            .mock_sequential_json_rpc_responses::<3>(
                200,
                json!({
                    "id": Id::from(ConstantSizeId::ZERO),
                    "jsonrpc": "2.0",
                    "result": Value::from_str(MOCK_RESPONSE_RESULT).unwrap()
                }),
            )
            .build()
            .json_request(get_version_request())
            .with_cycles(0)
            .send()
            .await
            .expect_consistent();

        assert_matches!(result, Ok(msg) if msg == MOCK_RESPONSE_RESULT);

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
            .client()
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
    use candid::{encode_args, Principal};
    use pocket_ic::{ErrorCode, RejectCode, RejectResponse};

    #[tokio::test]
    async fn should_update_api_key() {
        let setup = Setup::with_args(InstallArgs {
            manage_api_keys: Some(vec![DEFAULT_CALLER_TEST_ID]),
            ..Default::default()
        })
        .await;

        let provider = SupportedRpcProviderId::AlchemyMainnet;
        let api_key = "test-api-key";
        let client = setup.client().build();
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
            .client()
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
            .client()
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
                    reject_code: RejectCode::CanisterReject,
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
        let client = setup.client().build();
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
            .client()
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
            .client()
            .build()
            .update_api_keys(&[(
                SupportedRpcProviderId::AlchemyMainnet,
                Some("unauthorized-api-key".to_string()),
            )])
            .await;
    }
}

fn get_version_request() -> Value {
    get_version_request_with_id(0)
}

fn get_version_request_with_id(id: u8) -> Value {
    json!({"jsonrpc": "2.0", "id": Id::from(ConstantSizeId::from(id)), "method": "getVersion"})
}

fn get_version_response(id: u8) -> Value {
    json!({
        "jsonrpc": "2.0",
        "result": {
            "feature-set": 3640012085_u64,
            "solana-core": "2.3.6"
        },
        "id": Id::from(ConstantSizeId::from(id))
    })
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

    #[tokio::test]
    async fn should_be_idempotent() {
        async fn check<Config, Params, CandidOutput, Output>(
            request: RequestBuilder<PocketIcRuntime<'_>, Config, Params, CandidOutput, Output>,
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
        let client = setup.client().build();

        for endpoint in SolRpcEndpoint::iter() {
            match endpoint {
                SolRpcEndpoint::GetSlot => {
                    check(client.get_slot()).await;
                }
                SolRpcEndpoint::JsonRequest => {
                    check(client.json_request(get_version_request())).await;
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
            request: RequestBuilder<PocketIcRuntime<'_>, Config, Params, CandidOutput, Output>,
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
        let client = setup.client().build();

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
                    check(client.json_request(get_version_request())).await;
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
                PocketIcRuntime<'_>,
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
        let client = setup
            .client()
            // The exact cycles cost of an HTTPs outcall is independent of the response,
            // so we always return a dummy response so that individual responses
            // do not need to be mocked.
            .mock_http(MockOutcallBuilder::new(403, json!({})))
            .build();

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
                        client.json_request(get_version_request()),
                        1_790_956_800,
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
        async fn check<F, Config, Params, CandidOutput, Output>(setup: &Setup, request: F)
        where
            F: Fn(
                SolRpcClient<PocketIcRuntime<'_>>,
            ) -> RequestBuilder<
                PocketIcRuntime<'_>,
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
            let client = setup
                .client()
                .with_rpc_sources(RpcSources::Custom(vec![RpcSource::Supported(
                    SupportedRpcProviderId::AlchemyMainnet,
                )]))
                .mock_http_once(
                    MockOutcallBuilder::new_error(RejectionCode::SysFatal, "Unrecoverable error!")
                        .with_max_response_bytes(1_999_999),
                )
                .build();
            let result = request(client)
                .with_response_size_estimate(1_999_999)
                .with_cycles(1_000_000_000_000)
                .send()
                .await;
            assert_eq!(
                result,
                MultiRpcResult::Consistent(Err(RpcError::HttpOutcallError(
                    HttpOutcallError::IcError {
                        code: RejectionCode::SysFatal,
                        message: "Unrecoverable error!".to_string()
                    }
                )))
            );
        }

        let setup = Setup::new().await.with_mock_api_keys().await;
        for endpoint in SolRpcEndpoint::iter() {
            match endpoint {
                SolRpcEndpoint::GetAccountInfo => {
                    check(&setup, |client| client.get_account_info(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetBalance => {
                    check(&setup, |client| client.get_balance(USDC_PUBLIC_KEY)).await;
                }
                SolRpcEndpoint::GetBlock => check(&setup, |client| client.get_block(577996)).await,
                SolRpcEndpoint::GetRecentPrioritizationFees => {
                    check(&setup, |client| {
                        client.get_recent_prioritization_fees(&[]).unwrap()
                    })
                    .await;
                }
                SolRpcEndpoint::GetSignaturesForAddress => {
                    check(&setup, |client| {
                        client.get_signatures_for_address(USDC_PUBLIC_KEY)
                    })
                    .await;
                }
                SolRpcEndpoint::GetSignatureStatuses => {
                    check(&setup, |client| {
                        client.get_signature_statuses(&[some_signature()]).unwrap()
                    })
                    .await;
                }
                SolRpcEndpoint::GetSlot => {
                    check(&setup, |client| client.get_slot()).await;
                }
                SolRpcEndpoint::GetTokenAccountBalance => {
                    check(&setup, |client| {
                        client.get_token_account_balance(USDC_PUBLIC_KEY)
                    })
                    .await;
                }
                SolRpcEndpoint::GetTransaction => {
                    check(&setup, |client| client.get_transaction(some_signature())).await;
                }
                SolRpcEndpoint::JsonRequest => {
                    check(&setup, |client| client.json_request(get_version_request())).await;
                }
                SolRpcEndpoint::SendTransaction => {
                    check(&setup, |client| client.send_transaction(some_transaction())).await
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
            ok_result: Value,
        ) where
            F: Fn(
                SolRpcClient<PocketIcRuntime<'_>>,
            ) -> RequestBuilder<
                PocketIcRuntime<'_>,
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
            let [ok_result_0, ok_result_1, _, ok_result_3, ok_result_4] =
                json_rpc_sequential_id(ok_result);

            let client = setup
                .client()
                .with_rpc_sources(RpcSources::Custom(vec![
                    RpcSource::Supported(SupportedRpcProviderId::AlchemyMainnet),
                    RpcSource::Supported(SupportedRpcProviderId::AnkrMainnet),
                    RpcSource::Supported(SupportedRpcProviderId::PublicNodeMainnet),
                ]))
                .mock_http_sequence(vec![
                    MockOutcallBuilder::new(200, ok_result_0),
                    MockOutcallBuilder::new(200, ok_result_1),
                    MockOutcallBuilder::new_error(RejectionCode::SysFatal, "Some error!"),
                ])
                .build();

            let result = request(client.clone())
                .with_response_consensus(ConsensusStrategy::Equality)
                .send()
                .await;
            assert_matches!(result, MultiRpcResult::Inconsistent(_));

            let client = setup
                .client()
                .with_rpc_sources(RpcSources::Custom(vec![
                    RpcSource::Supported(SupportedRpcProviderId::AlchemyMainnet),
                    RpcSource::Supported(SupportedRpcProviderId::AnkrMainnet),
                    RpcSource::Supported(SupportedRpcProviderId::PublicNodeMainnet),
                ]))
                .mock_http_sequence(vec![
                    MockOutcallBuilder::new(200, ok_result_3),
                    MockOutcallBuilder::new(200, ok_result_4),
                    MockOutcallBuilder::new_error(RejectionCode::SysFatal, "Some error!"),
                ])
                .build();

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
        for endpoint in SolRpcEndpoint::iter() {
            match endpoint {
                SolRpcEndpoint::GetAccountInfo => {
                    check(
                        &setup,
                        |client| client.get_account_info(USDC_PUBLIC_KEY),
                        get_account_info_response(0),
                    )
                    .await;
                }
                SolRpcEndpoint::GetBalance => {
                    check(
                        &setup,
                        |client| client.get_balance(USDC_PUBLIC_KEY),
                        get_balance_response(6),
                    )
                    .await;
                }
                SolRpcEndpoint::GetBlock => {
                    check(
                        &setup,
                        |client| client.get_block(577996),
                        get_block_response(12),
                    )
                    .await
                }
                SolRpcEndpoint::GetRecentPrioritizationFees => {
                    check(
                        &setup,
                        |client| {
                            client
                                .get_recent_prioritization_fees(&[])
                                .unwrap()
                                .with_max_slot_rounding_error(10)
                                .with_max_length(NonZeroU8::new(5).unwrap())
                        },
                        get_recent_prioritization_fees_response(18),
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
                        get_signatures_for_address_response(24),
                    )
                    .await;
                }
                SolRpcEndpoint::GetSignatureStatuses => {
                    check(
                        &setup,
                        |client| client.get_signature_statuses(&[some_signature()]).unwrap(),
                        get_signature_statuses_response(30),
                    )
                    .await;
                }
                SolRpcEndpoint::GetSlot => {
                    check(
                        &setup,
                        |client| client.get_slot(),
                        get_slot_response(36, 1234),
                    )
                    .await;
                }
                SolRpcEndpoint::GetTokenAccountBalance => {
                    check(
                        &setup,
                        |client| client.get_token_account_balance(USDC_PUBLIC_KEY),
                        get_token_account_balance_response(42),
                    )
                    .await;
                }
                SolRpcEndpoint::GetTransaction => {
                    check(
                        &setup,
                        |client| client.get_transaction(some_signature()),
                        get_transaction_response(48),
                    )
                    .await;
                }
                SolRpcEndpoint::JsonRequest => {
                    check(
                        &setup,
                        |client| client.json_request(get_version_request_with_id(54)),
                        get_version_response(54),
                    )
                    .await;
                }
                SolRpcEndpoint::SendTransaction => {
                    check(
                        &setup,
                        |client| client.send_transaction(some_transaction()),
                        send_transaction_response(60, some_signature()),
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
        fn request_body(id: u8) -> serde_json::Value {
            json!({
                "id": Id::from(ConstantSizeId::from(id)),
                "jsonrpc": "2.0",
                "method": "getBalance",
                "params": [
                    USDC_PUBLIC_KEY.to_string(),
                    {
                        "commitment": "confirmed",
                        "minContextSlot": 100
                    }
                ]
            })
        }

        let setup = Setup::new().await.with_mock_api_keys().await;
        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let client = setup.client().with_rpc_sources(sources);

            let results = client
                .mock_http_sequence(vec![
                    MockOutcallBuilder::new(200, get_balance_response(first_id))
                        .with_request_body(request_body(first_id)),
                    MockOutcallBuilder::new(200, get_balance_response(first_id + 1))
                        .with_request_body(request_body(first_id + 1)),
                    MockOutcallBuilder::new(200, get_balance_response(first_id + 2))
                        .with_request_body(request_body(first_id + 2)),
                ])
                .build()
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
        fn request_body(id: u8) -> Value {
            json!({
                "jsonrpc": "2.0",
                "id": Id::from(ConstantSizeId::from(id)),
                "method": "getTokenAccountBalance",
                "params": [
                    USDC_PUBLIC_KEY.to_string(),
                    {
                        "commitment": "confirmed",
                    }
                ]
            })
        }

        let setup = Setup::new().await.with_mock_api_keys().await;
        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let client = setup.client().with_rpc_sources(sources);

            let results = client
                .mock_http_sequence(vec![
                    MockOutcallBuilder::new(200, get_token_account_balance_response(first_id))
                        .with_request_body(request_body(first_id)),
                    MockOutcallBuilder::new(200, get_token_account_balance_response(first_id + 1))
                        .with_request_body(request_body(first_id + 1)),
                    MockOutcallBuilder::new(200, get_token_account_balance_response(first_id + 2))
                        .with_request_body(request_body(first_id + 2)),
                ])
                .build()
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
        fn request_body(id: u8) -> Value {
            json!({
                "jsonrpc": "2.0",
                "id": Id::from(ConstantSizeId::from(id)),
                "method": "getSignatureStatuses",
                "params": [
                    [some_signature().to_string(), another_signature().to_string()],
                    {
                        "searchTransactionHistory": true
                    }
                ],
            })
        }

        let setup = Setup::new().await.with_mock_api_keys().await;
        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let client = setup.client().with_rpc_sources(sources);

            let results = client
                .mock_http_sequence(vec![
                    MockOutcallBuilder::new(200, get_signature_statuses_response(first_id))
                        .with_request_body(request_body(first_id)),
                    MockOutcallBuilder::new(200, get_signature_statuses_response(first_id + 1))
                        .with_request_body(request_body(first_id + 1)),
                    MockOutcallBuilder::new(200, get_signature_statuses_response(first_id + 2))
                        .with_request_body(request_body(first_id + 2)),
                ])
                .build()
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
        fn request_body(id: u8) -> Value {
            json!({
                "jsonrpc": "2.0",
                "id": Id::from(ConstantSizeId::from(id)),
                "method": "getSignaturesForAddress",
                "params": [
                    USDC_PUBLIC_KEY.to_string(),
                    {
                        "limit": 5,
                    },
                ],
            })
        }

        let setup = Setup::new().await.with_mock_api_keys().await;
        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let client = setup.client().with_rpc_sources(sources);

            let results = client
                .mock_http_sequence(vec![
                    MockOutcallBuilder::new(200, get_signatures_for_address_response(first_id))
                        .with_request_body(request_body(first_id)),
                    MockOutcallBuilder::new(200, get_signatures_for_address_response(first_id + 1))
                        .with_request_body(request_body(first_id + 1)),
                    MockOutcallBuilder::new(200, get_signatures_for_address_response(first_id + 2))
                        .with_request_body(request_body(first_id + 2)),
                ])
                .build()
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

    #[tokio::test]
    async fn should_retrieve_metrics() {
        let setup = Setup::new().await.with_mock_api_keys().await;
        let client = setup
            .client()
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
            ]));

        let client = client
            .mock_http_sequence(vec![
                MockOutcallBuilder::new(
                    200,
                    json!({
                        "id": Id::from(ConstantSizeId::from(0_u8)),
                        "jsonrpc": "2.0",
                        "result": 1_450_305,
                    }),
                ),
                MockOutcallBuilder::new(
                    200,
                    json!({
                        "id": Id::from(ConstantSizeId::from(1_u8)),
                        "jsonrpc": "2.0",
                        "result": 1_450_305,
                    }),
                ),
                MockOutcallBuilder::new(
                    200,
                    json!({
                      "jsonrpc": "2.0",
                      "error": {
                          "code": -32603,
                          "message": "Internal error: failed to get slot: Node is behind",
                          "data": null
                      },
                      "id": Id::from(ConstantSizeId::from(2_u8)),
                    }),
                ),
                MockOutcallBuilder::new(429, json!({})),
                MockOutcallBuilder::new(500, json!({})),
                MockOutcallBuilder::new_error(RejectionCode::SysFatal, "Fatal error!"),
            ])
            .build();

        let result = client.get_slot().send().await;
        assert_eq!(result, MultiRpcResult::Consistent(Ok(1_450_300)));

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
            // `solrpc_latencies` latency histograms
            .assert_contains_metric_matching(r#"solrpc_latencies_bucket\{method="getSlot",host="solana-mainnet.g.alchemy.com",le="\d+"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_latencies_bucket\{method="getSlot",host="rpc.ankr.com",le="\d+"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_latencies_bucket\{method="getSlot",host="solana-mainnet.core.chainstack.com",le="\d+"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_latencies_bucket\{method="getSlot",host="lb.drpc.org",le="\d+"\} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_latencies_bucket\{method="getSlot",host="mainnet.helius-rpc.com",le="\d+"\} 1 \d+"#)
            .assert_does_not_contain_metric_matching(r#"solrpc_latencies\{method="getSlot",host="solana-rpc.publicnode.com",le="\d+"\} 1 \d+"#)
            // `solrpc_inconsistent_responses` counters: inconsistent results
            .assert_contains_metric_matching(r#"solrpc_inconsistent_responses\{method="getSlot",host="solana-mainnet.g.alchemy.com"} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_inconsistent_responses\{method="getSlot",host="rpc.ankr.com"} 1 \d+"#)
            .assert_contains_metric_matching(r#"solrpc_inconsistent_responses\{method="getSlot",host="solana-mainnet.core.chainstack.com"} 1 \d+"#)
            .assert_does_not_contain_metric_matching(r#"solrpc_inconsistent_responses\{method="getSlot",host="lb.drpc.org"} 1 \d+"#)
            .assert_does_not_contain_metric_matching(r#"solrpc_inconsistent_responses\{method="getSlot",host="mainnet.helius-rpc.com"} 1 \d+"#)
            .assert_does_not_contain_metric_matching(r#"solrpc_inconsistent_responses\{method="getSlot",host="solana-rpc.publicnode.com"} 1 \d+"#);
    }

    #[tokio::test]
    async fn should_not_record_metrics_when_not_enough_cycles() {
        let setup = Setup::new().await.with_mock_api_keys().await;
        let client = setup.client().build();

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
        .client()
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

    let client = setup
        .client()
        .with_rpc_sources(RpcSources::Custom(vec![RpcSource::Supported(
            SupportedRpcProviderId::AlchemyMainnet,
        )]));

    let results = client
        .mock_sequential_json_rpc_responses::<1>(
            200,
            json!({
                "id": Id::from(ConstantSizeId::ZERO),
                "jsonrpc": "2.0",
                "result": 1234,
            }),
        )
        .build()
        .get_slot()
        .with_rounding_error(0)
        .send()
        .await
        .expect_consistent();
    assert_eq!(results, Ok(1234));

    let logs = setup.retrieve_logs("TRACE_HTTP").await;
    assert_eq!(logs.len(), 2, "Unexpected amount of logs: {logs:?}");

    assert_eq!(logs[0].message, "JSON-RPC request with id `00000000000000000000` to solana-mainnet.g.alchemy.com: JsonRpcRequest { jsonrpc: V2, method: \"getSlot\", id: String(\"00000000000000000000\"), params: Some(GetSlotParams(None)) }");
    assert_eq!(logs[1].message, "Got response for request with id `00000000000000000000`. Response with status 200 OK: JsonRpcResponse { jsonrpc: V2, id: String(\"00000000000000000000\"), result: Ok(1234) }");

    setup.drop().await;
}

#[tokio::test]
async fn should_change_default_providers_when_one_keeps_failing() {
    fn request_body(id: u8) -> Value {
        let id = ConstantSizeId::from(id).to_string();
        json!({ "jsonrpc": "2.0", "id": id, "method": "getSlot", "params": [null] })
    }

    fn response_body(id: u8) -> Value {
        let id = ConstantSizeId::from(id).to_string();
        json!({ "id": id, "jsonrpc": "2.0", "result": 1200, })
    }

    let setup = Setup::new().await.with_mock_api_keys().await;
    let client = setup.client();
    let slot = client
        .with_consensus_strategy(ConsensusStrategy::Threshold {
            min: 2,
            total: Some(3),
        })
        .mock_http_sequence(vec![
            MockOutcallBuilder::new(200, response_body(0))
                .with_request_body(request_body(0))
                .with_host("solana-mainnet.g.alchemy.com"),
            MockOutcallBuilder::new(500, "error")
                .with_request_body(request_body(1))
                .with_host("lb.drpc.org"),
            MockOutcallBuilder::new(200, response_body(2))
                .with_request_body(request_body(2))
                .with_host("mainnet.helius-rpc.com"),
        ])
        .build()
        .get_slot()
        .send()
        .await
        .expect_consistent();
    assert_eq!(slot, Ok(1200));

    let client = setup.client();
    let slot = client
        .with_consensus_strategy(ConsensusStrategy::Equality)
        .with_rpc_sources(RpcSources::Custom(vec![RpcSource::Supported(
            SupportedRpcProviderId::AnkrMainnet,
        )]))
        .mock_http_sequence(vec![MockOutcallBuilder::new(200, response_body(3))
            .with_request_body(request_body(3))
            .with_host("rpc.ankr.com")])
        .build()
        .get_slot()
        .send()
        .await
        .expect_consistent();
    assert_eq!(slot, Ok(1200));

    let client = setup.client();
    let slot = client
        .with_consensus_strategy(ConsensusStrategy::Threshold {
            min: 3,
            total: Some(3),
        })
        .mock_http_sequence(vec![
            MockOutcallBuilder::new(200, response_body(4))
                .with_request_body(request_body(4))
                .with_host("solana-mainnet.g.alchemy.com"),
            MockOutcallBuilder::new(200, response_body(5))
                .with_request_body(request_body(5))
                .with_host("rpc.ankr.com"),
            MockOutcallBuilder::new(200, response_body(6))
                .with_request_body(request_body(6))
                .with_host("mainnet.helius-rpc.com"),
        ])
        .build()
        .get_slot()
        .send()
        .await
        .expect_consistent();
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

fn get_account_info_response(id: u8) -> Value {
    json!({
        "id": Id::from(ConstantSizeId::from(id)),
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
    })
}

fn get_balance_response(id: u8) -> Value {
    json!({
        "id": Id::from(ConstantSizeId::from(id)),
        "jsonrpc": "2.0",
        "result": {
            // context should be filtered out by transform
            "context": { "slot": 334048531 + id as u64, "apiVersion": "2.1.9" },
            "value": 389086612571_u64
        },
    })
}

fn get_block_response(id: u8) -> Value {
    json!({
        "id": Id::from(ConstantSizeId::from(id)),
        "jsonrpc": "2.0",
        "result":{
            "blockHeight": 360854634,
            "blockTime": 1744122369,
            "parentSlot": 372877611,
            "blockhash": "8QeCusqSTKeC23NwjTKRBDcPuEfVLtszkxbpL6mXQEp4",
            "previousBlockhash": "4Pcj2yJkCYyhnWe8Ze3uK2D2EtesBxhAevweDoTcxXf3"}
    })
}

fn get_recent_prioritization_fees_response(id: u8) -> Value {
    json!({
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
        "id": Id::from(ConstantSizeId::from(id))
        }
    )
}

fn get_signatures_for_address_response(id: u8) -> Value {
    json!({
        "id": Id::from(ConstantSizeId::from(id)),
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
    })
}

fn get_signature_statuses_response(id: u8) -> Value {
    json!({
        "id": Id::from(ConstantSizeId::from(id)),
        "jsonrpc": "2.0",
        "result": {
            // context should be filtered out by transform
            "context": { "slot": 334048531 + id as u64, "apiVersion": "2.1.9" },
            "value": [
                  {
                    "slot": 48,
                    // confirmations should be filtered out by transform
                    "confirmations": id,
                    "err": null,
                    "status": { "Ok": null },
                    "confirmationStatus": "finalized"
                  },
                  null
            ]
        },
    })
}

fn get_slot_response(id: u8, slot: u64) -> Value {
    json!({
        "id": Id::from(ConstantSizeId::from(id)),
        "jsonrpc": "2.0",
        "result": slot,
    })
}

fn get_token_account_balance_response(id: u8) -> Value {
    json!({
        "id": Id::from(ConstantSizeId::from(id)),
        "jsonrpc": "2.0",
        "result": {
            // context should be filtered out by transform
            "context": { "slot": 334048531 + id as u64, "apiVersion": "2.1.9" },
            "value": {
                "amount": "9864",
                "decimals": 2,
                "uiAmount": 98.64,
                "uiAmountString": "98.64",
            }
        },
    })
}

fn get_transaction_response(id: u8) -> Value {
    json!({
        "id": Id::from(ConstantSizeId::from(id)),
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
    })
}

fn send_transaction_response(id: u8, transaction_signature: solana_signature::Signature) -> Value {
    json!({
        "id": Id::from(ConstantSizeId::from(id)),
        "jsonrpc": "2.0",
        "result": transaction_signature.to_string(),
    })
}
