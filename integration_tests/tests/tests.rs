use sol_rpc_int_tests::Setup;
use sol_rpc_types::{Provider, RpcAccess, RpcAuth, RpcService, SolMainnetService, SolanaCluster};

#[tokio::test]
async fn should_get_providers() {
    let setup = Setup::new().await;
    let client = setup.client();
    let providers = client.get_providers().await;

    assert_eq!(providers.len(), 5);

    assert_eq!(
        providers[0],
        Provider {
            provider_id: String::from("alchemy-mainnet"),
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::BearerToken {
                    url: String::from("https://solana-mainnet.g.alchemy.com/v2"),
                },
                public_url: Some(String::from("https://solana-mainnet.g.alchemy.com/v2/demo")),
            },
            alias: Some(RpcService::SolMainnet(SolMainnetService::Alchemy)),
        }
    );

    setup.drop().await;
}
