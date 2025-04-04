use assert_matches::*;
use const_format::formatcp;
use ic_cdk::api::management_canister::http_request::HttpHeader;
use pocket_ic::common::rest::CanisterHttpMethod;
use serde_json::json;
use sol_rpc_canister::constants::*;
use sol_rpc_int_tests::{
    mock::MockOutcallBuilder, Setup, SolRpcTestClient, DEFAULT_CALLER_TEST_ID,
};
use sol_rpc_types::{
    GetSlotParams, InstallArgs, Mode, ProviderError, RpcAccess, RpcAuth, RpcConfig, RpcEndpoint,
    RpcError, RpcResult, RpcSource, RpcSources, SolanaCluster, SupportedRpcProvider,
    SupportedRpcProviderId,
};
use std::str::FromStr;

const MOCK_REQUEST_URL: &str = "https://api.devnet.solana.com/";
const MOCK_REQUEST_PAYLOAD: &str = r#"{"jsonrpc":"2.0","id":0,"method":"getVersion"}"#;
const MOCK_RESPONSE_RESULT: &str = r#"{"feature-set":2891131721,"solana-core":"1.16.7"}"#;
const MOCK_RESPONSE: &str = formatcp!(
    "{{\"jsonrpc\":\"2.0\",\"id\":0,\"result\":{}}}",
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
                .raw_request(get_version_request())
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
        mock_request(|builder| builder.with_raw_request_body(MOCK_REQUEST_PAYLOAD)).await
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
                .with_raw_request_body(MOCK_REQUEST_PAYLOAD)
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

mod get_slot_tests {
    use super::*;
    use sol_rpc_types::{CommitmentLevel, GetSlotParams};
    use std::iter::zip;

    #[tokio::test]
    async fn should_get_slot_with_full_params() {
        fn request_body(id: u8) -> serde_json::Value {
            json!({ "jsonrpc": "2.0", "id": id, "method": "getSlot", "params": [{"commitment": "processed", "minContextSlot": 100}] })
        }

        fn response_body(id: u8) -> serde_json::Value {
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
                        "id": first_id,
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
                            "id": id + first_id as usize,
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
                            "id": id + first_id as usize,
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

mod generic_request_tests {
    use super::*;

    #[tokio::test]
    async fn request_should_require_cycles() {
        let setup = Setup::new().await.with_mock_api_keys().await;
        let client = setup.client().build();

        let results = client
            .raw_request(get_version_request())
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
                    "id": 0,
                    "jsonrpc": "2.0",
                    "result": serde_json::Value::from_str(MOCK_RESPONSE_RESULT).unwrap()
                }),
            )
            .build()
            .raw_request(get_version_request())
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
    json!({"jsonrpc": "2.0", "id": 0, "method": "getVersion"})
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

#[tokio::test]
async fn should_get_slot() {
    let setup = Setup::new().await.with_mock_api_keys().await;
    let client = setup
        .client()
        .mock_http_sequence(vec![
            MockOutcallBuilder::new(
                200,
                json!({ "jsonrpc": "2.0", "result": 371059358, "id": 0 }),
            ),
            MockOutcallBuilder::new(
                200,
                json!({ "jsonrpc": "2.0", "result": 371059358, "id": 1 }),
            ),
            MockOutcallBuilder::new(
                200,
                json!({ "jsonrpc": "2.0", "result": 371059358, "id": 2 }),
            ),
        ])
        .build();

    let five_percents = 5_u8;
    let request = client.get_slot().with_params(GetSlotParams::default());

    let cycles_cost = request.clone().request_cost().send().await.unwrap();
    assert_within(cycles_cost, 1_792_548_000, five_percents);

    let cycles_before = setup.sol_rpc_canister_cycles_balance().await;
    let slot = request
        .clone()
        .with_cycles(cycles_cost)
        .send()
        .await
        .expect_consistent()
        .unwrap();
    let cycles_after = setup.sol_rpc_canister_cycles_balance().await;
    let cycles_consumed = cycles_before + cycles_cost - cycles_after;
    assert_within(cycles_consumed, 841_708_745, five_percents);

    assert_eq!(slot, 371059340);
    assert!(
        cycles_after > cycles_before,
        "BUG: not enough cycles requested. Requested {cycles_cost} cycles, but consumed {cycles_consumed} cycles"
    );

    let client = setup
        .client()
        .mock_http_sequence(vec![
            MockOutcallBuilder::new(
                200,
                json!({ "jsonrpc": "2.0", "result": 371059358, "id": 3 }),
            ),
            MockOutcallBuilder::new(
                200,
                json!({ "jsonrpc": "2.0", "result": 371059358, "id": 4 }),
            ),
        ])
        .build();

    let results = client
        .get_slot()
        .with_params(GetSlotParams::default())
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
