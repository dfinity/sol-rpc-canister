use sol_rpc_canister::constants::*;
use sol_rpc_int_tests::{Setup, SolRpcTestClient, ADDITIONAL_TEST_ID};
use sol_rpc_types::{
    InstallArgs, Mode, Provider, ProviderError, RpcAccess, RpcApi, RpcAuth, RpcError, RpcService,
    SolMainnetService, SolanaCluster,
};

const MOCK_REQUEST_URL: &str = "https://api.devnet.solana.com";
const MOCK_REQUEST_PAYLOAD: &str = r#"{"jsonrpc":"2.0","id":1,"method":"getVersion"}"#;
const MOCK_REQUEST_RESPONSE: &str =
    r#"{"id":1,"jsonrpc":"2.0","result":{"feature-set":2891131721,"solana-core":"1.16.7"}}"#;
const MOCK_REQUEST_MAX_RESPONSE_BYTES: u64 = 1000;

mod mock_request_tests {
    use super::*;
    use assert_matches::*;
    use ic_cdk::api::management_canister::http_request::HttpHeader;
    use pocket_ic::common::rest::CanisterHttpMethod;
    use sol_rpc_int_tests::mock::*;

    async fn mock_request(builder_fn: impl Fn(MockOutcallBuilder) -> MockOutcallBuilder) {
        let setup = Setup::with_args(InstallArgs {
            mode: Some(Mode::Demo),
            ..Default::default()
        })
        .await;
        let client = setup.client();
        assert_matches!(
            client
                .request(
                    RpcService::Custom(RpcApi {
                        url: MOCK_REQUEST_URL.to_string(),
                        headers: Some(vec![HttpHeader {
                            name: "Custom".to_string(),
                            value: "Value".to_string(),
                        }]),
                    }),
                    MOCK_REQUEST_PAYLOAD,
                    MOCK_REQUEST_MAX_RESPONSE_BYTES,
                )
                .await
                .mock_http(builder_fn(MockOutcallBuilder::new(
                    200,
                    MOCK_REQUEST_RESPONSE
                )))
                .await
                .wait()
                .await,
            Ok(_)
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
                ("Custom", "Value"),
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
                    ("Custom", "Value"),
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
        let client = setup.client();
        let providers = client.get_providers().await;

        assert_eq!(providers.len(), 5);

        assert_eq!(
            providers[0],
            Provider {
                provider_id: "alchemy-mainnet".to_string(),
                cluster: SolanaCluster::Mainnet,
                access: RpcAccess::Authenticated {
                    auth: RpcAuth::BearerToken {
                        url: "https://solana-mainnet.g.alchemy.com/v2".to_string(),
                    },
                    public_url: Some("https://solana-mainnet.g.alchemy.com/v2/demo".to_string()),
                },
                alias: Some(RpcService::SolMainnet(SolMainnetService::Alchemy)),
            }
        );

        setup.drop().await;
    }
}

mod generic_request_tests {
    use super::*;
    use assert_matches::*;
    use sol_rpc_int_tests::mock::MockOutcallBuilder;

    #[tokio::test]
    async fn request_should_require_cycles() {
        let setup = Setup::new().await;
        let client = setup.client();

        let result = client
            .request(
                RpcService::SolMainnet(SolMainnetService::Alchemy),
                MOCK_REQUEST_PAYLOAD,
                MOCK_REQUEST_MAX_RESPONSE_BYTES,
            )
            .await
            .wait()
            .await;

        assert_matches!(
            result,
            Err(RpcError::ProviderError(ProviderError::TooFewCycles {
                expected: _,
                received: 0
            }))
        );

        setup.drop().await;
    }

    #[tokio::test]
    async fn request_should_succeed_in_demo_mode() {
        let setup = Setup::with_args(InstallArgs {
            mode: Some(Mode::Demo),
            ..Default::default()
        })
        .await;
        let client = setup.client();

        let result = client
            .request(
                RpcService::SolMainnet(SolMainnetService::Alchemy),
                MOCK_REQUEST_PAYLOAD,
                MOCK_REQUEST_MAX_RESPONSE_BYTES,
            )
            .await
            .mock_http(MockOutcallBuilder::new(200, MOCK_REQUEST_RESPONSE))
            .await
            .wait()
            .await;

        assert_matches!(result, Ok(msg) if msg == MOCK_REQUEST_RESPONSE);

        setup.drop().await;
    }
}

mod retrieve_logs_tests {
    use super::*;

    #[tokio::test]
    async fn should_retrieve_logs() {
        let setup = Setup::new().await;
        let client = setup.client();
        assert_eq!(client.retrieve_logs("DEBUG").await, vec![]);
        assert_eq!(client.retrieve_logs("INFO").await, vec![]);

        // Generate some log
        setup
            .client()
            .with_caller(setup.controller())
            .update_api_keys(&[(
                "alchemy-mainnet".to_string(),
                Some("unauthorized-api-key".to_string()),
            )])
            .await;

        assert_eq!(client.retrieve_logs("DEBUG").await, vec![]);
        assert!(client.retrieve_logs("INFO").await[0]
            .message
            .contains("Updating API keys"));
    }
}

mod update_api_key_tests {
    use super::*;

    #[tokio::test]
    async fn should_update_api_key() {
        let authorized_caller = ADDITIONAL_TEST_ID;
        let setup = Setup::with_args(InstallArgs {
            manage_api_keys: Some(vec![authorized_caller]),
            ..Default::default()
        })
        .await;

        let provider_id = "alchemy-mainnet";
        let api_key = "test-api-key";
        let client = setup.client().with_caller(authorized_caller);
        client
            .update_api_keys(&[(provider_id.to_string(), Some(api_key.to_string()))])
            .await;
        client
            .verify_api_key((provider_id.to_string(), Some(api_key.to_string())))
            .await;

        client
            .update_api_keys(&[(provider_id.to_string(), None)])
            .await;
        client.verify_api_key((provider_id.to_string(), None)).await;
    }

    #[tokio::test]
    #[should_panic(expected = "You are not authorized")]
    async fn should_prevent_unauthorized_update_api_keys() {
        let setup = Setup::new().await;
        setup
            .client()
            .update_api_keys(&[(
                "alchemy-mainnet".to_string(),
                Some("unauthorized-api-key".to_string()),
            )])
            .await;
    }

    #[tokio::test]
    #[should_panic(expected = "Trying to set API key for unauthenticated provider")]
    async fn should_prevent_unauthenticated_update_api_keys() {
        let setup = Setup::new().await;
        setup
            .client()
            .with_caller(setup.controller())
            .update_api_keys(&[(
                "publicnode-mainnet".to_string(),
                Some("invalid-api-key".to_string()),
            )])
            .await;
    }
}

mod canister_upgrade_tests {
    use super::*;

    #[tokio::test]
    async fn upgrade_should_keep_api_keys() {
        let setup = Setup::new().await;
        let provider_id = "alchemy-mainnet";
        let api_key = "test-api-key";
        let client = setup.client().with_caller(setup.controller());
        client
            .update_api_keys(&[(provider_id.to_string(), Some(api_key.to_string()))])
            .await;
        client
            .verify_api_key((provider_id.to_string(), Some(api_key.to_string())))
            .await;

        setup.upgrade_canister(InstallArgs::default()).await;

        client
            .verify_api_key((provider_id.to_string(), Some(api_key.to_string())))
            .await;
    }

    #[tokio::test]
    async fn upgrade_should_keep_manage_api_key_principals() {
        let authorized_caller = ADDITIONAL_TEST_ID;
        let setup = Setup::with_args(InstallArgs {
            manage_api_keys: Some(vec![authorized_caller]),
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
            .with_caller(authorized_caller)
            .update_api_keys(&[(
                "alchemy-mainnet".to_string(),
                Some("authorized-api-key".to_string()),
            )])
            .await;
    }

    #[tokio::test]
    #[should_panic(expected = "You are not authorized")]
    async fn upgrade_should_change_manage_api_key_principals() {
        let deauthorized_caller = ADDITIONAL_TEST_ID;
        let setup = Setup::with_args(InstallArgs {
            manage_api_keys: Some(vec![deauthorized_caller]),
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
            .with_caller(deauthorized_caller)
            .update_api_keys(&[(
                "alchemy-mainnet".to_string(),
                Some("unauthorized-api-key".to_string()),
            )])
            .await;
    }
}
