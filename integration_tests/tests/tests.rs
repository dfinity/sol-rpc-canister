use sol_rpc_int_tests::{Setup, SolRpcTestClient, ADDITIONAL_TEST_ID};
use sol_rpc_types::{
    InstallArgs, Provider, RpcAccess, RpcAuth, RpcService, SolMainnetService, SolanaCluster,
};

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
