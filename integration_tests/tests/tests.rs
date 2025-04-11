use assert_matches::*;
use candid::CandidType;
use canhttp::http::json::{ConstantSizeId, Id};
use const_format::formatcp;
use ic_cdk::api::management_canister::http_request::HttpHeader;
use pocket_ic::common::rest::CanisterHttpMethod;
use serde::de::DeserializeOwned;
use serde_json::json;
use sol_rpc_canister::constants::*;
use sol_rpc_client::{RequestBuilder, SolRpcEndpoint};
use sol_rpc_int_tests::{
    mock::MockOutcallBuilder, PocketIcRuntime, Setup, SolRpcTestClient, DEFAULT_CALLER_TEST_ID,
};
use sol_rpc_types::{
    CommitmentLevel, GetAccountInfoParams, GetSlotParams, InstallArgs, Mode, ProviderError,
    RpcAccess, RpcAuth, RpcConfig, RpcEndpoint, RpcError, RpcResult, RpcSource, RpcSources,
    SolanaCluster, SupportedRpcProvider, SupportedRpcProviderId,
};
use solana_account_decoder_client_types::{UiAccount, UiAccountData, UiAccountEncoding};
use solana_signature::Signature;
use solana_signer::Signer;
use std::{fmt::Debug, iter::zip, str::FromStr};
use strum::IntoEnumIterator;

const MOCK_REQUEST_URL: &str = "https://api.devnet.solana.com/";
const MOCK_RESPONSE_RESULT: &str = r#"{"feature-set":2891131721,"solana-core":"1.16.7"}"#;
const MOCK_RESPONSE: &str = formatcp!(
    "{{\"jsonrpc\":\"2.0\",\"id\":\"00000000000000000000\",\"result\":{}}}",
    MOCK_RESPONSE_RESULT
);
const MOCK_REQUEST_MAX_RESPONSE_BYTES: u64 = 1000;

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
            .with_rpc_config(RpcConfig {
                response_size_estimate: Some(MOCK_REQUEST_MAX_RESPONSE_BYTES),
                ..RpcConfig::default()
            })
            .with_rpc_sources(RpcSources::Custom(vec![RpcSource::Custom(RpcEndpoint {
                url: MOCK_REQUEST_URL.to_string(),
                headers: Some(vec![HttpHeader {
                    name: "custom".to_string(),
                    value: "Value".to_string(),
                }]),
            })]));
        let expected_result: serde_json::Value = serde_json::from_str(MOCK_RESPONSE).unwrap();
        assert_matches!(
            client
                .mock_http(builder_fn(MockOutcallBuilder::new(200, MOCK_RESPONSE))).build()
                .json_request(get_version_request())
                .with_cycles(0)
                .send()
                .await,
            sol_rpc_types::MultiRpcResult::Consistent(Ok(msg)) if msg == serde_json::Value::to_string(&expected_result["result"])
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

        assert_eq!(providers.len(), 9);

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
    use canhttp::http::json::Id;

    #[tokio::test]
    async fn should_get_account_info() {
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
                    }),
                )
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

mod get_slot_tests {
    use super::*;

    #[tokio::test]
    async fn should_get_slot_with_full_params() {
        fn request_body(id: u8) -> serde_json::Value {
            let id = ConstantSizeId::from(id).to_string();
            json!({ "jsonrpc": "2.0", "id": id, "method": "getSlot", "params": [{"commitment": "processed", "minContextSlot": 100}] })
        }

        fn response_body(id: u8) -> serde_json::Value {
            let id = ConstantSizeId::from(id).to_string();
            json!({ "id": id, "jsonrpc": "2.0", "result": 1234, })
        }

        let setup = Setup::new().await.with_mock_api_keys().await;
        let client = setup.client();

        let slot = client
            .mock_http_sequence(vec![
                MockOutcallBuilder::new(200, response_body(0)).with_request_body(request_body(0)),
                MockOutcallBuilder::new(200, response_body(1)).with_request_body(request_body(1)),
                MockOutcallBuilder::new(200, response_body(2)).with_request_body(request_body(2)),
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
                .mock_sequential_json_rpc_responses::<3>(
                    200,
                    json!({
                        "id": Id::from(ConstantSizeId::from(first_id)),
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
        }

        setup.drop().await;
    }

    #[tokio::test]
    async fn should_get_consistent_result_with_rounding() {
        let setup = Setup::new().await.with_mock_api_keys().await;

        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let responses = [1234, 1229, 1237]
                .iter()
                .enumerate()
                .map(|(id, slot)| {
                    MockOutcallBuilder::new(
                        200,
                        json!({
                            "id": Id::from(ConstantSizeId::from(id as u64 + first_id as u64)),
                            "jsonrpc": "2.0",
                            "result": slot,
                        }),
                    )
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
            let responses = [1234, 1229, 1237]
                .iter()
                .enumerate()
                .map(|(id, slot)| {
                    MockOutcallBuilder::new(
                        200,
                        json!({
                            "id": Id::from(ConstantSizeId::from(id as u64 + first_id as u64)),
                            "jsonrpc": "2.0",
                            "result": slot,
                        }),
                    )
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

mod send_transaction_tests {
    use super::*;

    #[tokio::test]
    async fn should_send_transaction() {
        let setup = Setup::new().await.with_mock_api_keys().await;
        let signature = "2vC221MDR312jrFzh5TRnMfUCHrCiG4cBuzHmagdgrQSsdLHaq65uJVLCWmubw4FkBDUxhRpQma785MpMwRS6ob7";

        for (sources, first_id) in zip(rpc_sources(), vec![0_u8, 3, 6]) {
            let client = setup.client().with_rpc_sources(sources);

            let results = client
                .mock_sequential_json_rpc_responses::<3>(
                    200,
                    json!({
                        "id": Id::from(ConstantSizeId::from(first_id)),
                        "jsonrpc": "2.0",
                        "result": signature
                    }),
                )
                .build()
                .send_transaction(some_transaction())
                .send()
                .await
                .expect_consistent();

            assert_eq!(results, Ok(Signature::from_str(signature).unwrap()));
        }

        setup.drop().await;
    }
}

mod generic_request_tests {
    use super::*;
    use canhttp::http::json::Id;

    #[tokio::test]
    async fn request_should_require_cycles() {
        let setup = Setup::new().await.with_mock_api_keys().await;
        let client = setup.client().build();

        let results = client
            .json_request(get_version_request())
            .with_cycles(0)
            .send()
            .await
            // The result is expected to be inconsistent because the different provider URLs means
            // the request and hence expected number of cycles for each provider is different.
            .expect_inconsistent();

        for (_provider, result) in results {
            assert_matches!(
                result,
                Err(RpcError::ProviderError(ProviderError::TooFewCycles {
                    expected: _,
                    received: 0
                }))
            );
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
                    "result": serde_json::Value::from_str(MOCK_RESPONSE_RESULT).unwrap()
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

fn get_version_request() -> serde_json::Value {
    json!({"jsonrpc": "2.0", "id": Id::from(ConstantSizeId::ZERO), "method": "getVersion"})
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
                SolRpcEndpoint::GetAccountInfo => {
                    check(client.get_account_info(GetAccountInfoParams::from(some_pubkey()))).await;
                }
                SolRpcEndpoint::GetSlot => {
                    check(client.get_slot().with_params(GetSlotParams::default())).await;
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
                SolRpcEndpoint::GetAccountInfo => {
                    check(client.get_account_info(GetAccountInfoParams::from(some_pubkey()))).await;
                }
                SolRpcEndpoint::GetSlot => {
                    check(client.get_slot().with_params(GetSlotParams::default())).await;
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
                sol_rpc_types::MultiRpcResult<CandidOutput>,
                sol_rpc_types::MultiRpcResult<Output>,
            >,
            expected_cycles_cost: u128,
        ) where
            Config: CandidType + Clone + Send,
            Params: CandidType + Clone + Send,
            CandidOutput: CandidType + DeserializeOwned,
            Output: Debug,
            sol_rpc_types::MultiRpcResult<CandidOutput>:
                Into<sol_rpc_types::MultiRpcResult<Output>>,
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

            // Same request with less cycles should fail.
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
                        client.get_account_info(GetAccountInfoParams::from(some_pubkey())),
                        1_793_744_800,
                    )
                    .await;
                }
                SolRpcEndpoint::GetSlot => {
                    check(
                        &setup,
                        client.get_slot().with_params(GetSlotParams::default()),
                        1_792_548_000,
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
                    .await;
                }
            }
        }

        setup.drop().await;
    }
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

fn some_pubkey() -> solana_pubkey::Pubkey {
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        .parse::<solana_pubkey::Pubkey>()
        .unwrap()
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
