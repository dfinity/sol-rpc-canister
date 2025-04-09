use super::PROVIDERS;
use crate::constants::API_KEY_REPLACE_STRING;
use sol_rpc_types::{RpcAccess, RpcAuth, SupportedRpcProvider, SupportedRpcProviderId};

#[test]
fn test_rpc_provider_url_patterns() {
    PROVIDERS.with(|providers| {
        for (provider, SupportedRpcProvider { access, .. }) in providers {
            fn assert_not_url_pattern(url: &str, provider: &SupportedRpcProviderId) {
                assert!(
                    !url.contains(API_KEY_REPLACE_STRING),
                    "Unexpected API key in URL for provider: {:?}",
                    provider
                )
            }
            fn assert_url_pattern(url: &str, provider: &SupportedRpcProviderId) {
                assert!(
                    url.contains(API_KEY_REPLACE_STRING),
                    "Missing API key in URL pattern for provider: {:?}",
                    provider
                )
            }
            match access {
                RpcAccess::Authenticated { auth, public_url } => {
                    match auth {
                        RpcAuth::BearerToken { url } => assert_not_url_pattern(url, provider),
                        RpcAuth::UrlParameter { url_pattern } => {
                            assert_url_pattern(url_pattern, provider)
                        }
                    }
                    if let Some(public_url) = public_url {
                        assert_not_url_pattern(public_url, provider);
                    }
                }
                RpcAccess::Unauthenticated { public_url } => {
                    assert_not_url_pattern(public_url, provider);
                }
            }
        }
    })
}

#[test]
fn should_have_consistent_name_for_cluster() {
    PROVIDERS.with(|providers| {
        for (provider_id, provider) in providers {
            assert!(provider_id
                .to_string()
                .ends_with(&provider.cluster.to_string()));
        }
    })
}

mod providers_new {
    use crate::providers::Providers;
    use assert_matches::assert_matches;
    use maplit::btreeset;
    use sol_rpc_types::{
        ConsensusStrategy, ProviderError, RpcSource, RpcSources, SolanaCluster,
        SupportedRpcProviderId,
    };

    #[test]
    fn should_fail_when_providers_explicitly_set_to_empty() {
        assert_matches!(
            Providers::new(RpcSources::Custom(vec![]), ConsensusStrategy::default()),
            Err(ProviderError::InvalidRpcConfig(_))
        );
    }

    #[test]
    fn should_use_default_providers() {
        for cluster in [SolanaCluster::Mainnet, SolanaCluster::Devnet] {
            let providers =
                Providers::new(RpcSources::Default(cluster), ConsensusStrategy::default()).unwrap();
            assert!(!providers.sources.is_empty());
        }
    }

    #[test]
    fn should_use_specified_provider() {
        let provider1 = SupportedRpcProviderId::AlchemyMainnet;
        let provider2 = SupportedRpcProviderId::PublicNodeMainnet;

        let providers = Providers::new(
            RpcSources::Custom(vec![
                RpcSource::Supported(provider1),
                RpcSource::Supported(provider2),
            ]),
            ConsensusStrategy::default(),
        )
        .unwrap();

        assert_eq!(
            providers.sources,
            btreeset! {
                RpcSource::Supported(provider1),
                RpcSource::Supported(provider2),
            }
        );
    }
}
