mod sol_rpc_client {
    use crate::rpc_client::SolRpcClient;
    use assert_matches::assert_matches;
    use maplit::btreeset;
    use sol_rpc_types::{
        ProviderError, RpcSource, RpcSources, SolanaCluster, SupportedRpcProviderId,
    };

    #[test]
    fn should_fail_when_providers_explicitly_set_to_empty() {
        assert_matches!(
            SolRpcClient::new(RpcSources::Custom(vec![]), None, None),
            Err(ProviderError::InvalidRpcConfig(_))
        );
    }

    #[test]
    fn should_use_default_providers() {
        for cluster in [SolanaCluster::Mainnet, SolanaCluster::Devnet] {
            let client = SolRpcClient::new(RpcSources::Default(cluster), None, None).unwrap();
            assert!(!client.providers().is_empty());
        }
    }

    #[test]
    fn should_use_specified_provider() {
        let provider1 = SupportedRpcProviderId::AlchemyMainnet;
        let provider2 = SupportedRpcProviderId::PublicNodeMainnet;

        let client = SolRpcClient::new(
            RpcSources::Custom(vec![
                RpcSource::Supported(provider1),
                RpcSource::Supported(provider2),
            ]),
            None,
            None,
        )
        .unwrap();

        assert_eq!(
            client.providers(),
            &btreeset! {
                RpcSource::Supported(provider1),
                RpcSource::Supported(provider2),
            }
        );
    }
}
