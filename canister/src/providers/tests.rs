use super::PROVIDERS;
use crate::constants::API_KEY_REPLACE_STRING;
use sol_rpc_types::{ProviderId, RpcAccess, RpcAuth, SolanaCluster};

#[test]
fn test_rpc_provider_url_patterns() {
    PROVIDERS.with(|providers| {
        for ((provider, cluster), access) in providers {
            fn assert_not_url_pattern(url: &str, provider: &ProviderId, cluster: &SolanaCluster) {
                assert!(
                    !url.contains(API_KEY_REPLACE_STRING),
                    "Unexpected API key in URL for provider: {:?} ({:?})",
                    provider,
                    cluster
                )
            }
            fn assert_url_pattern(url: &str, provider: &ProviderId, cluster: &SolanaCluster) {
                assert!(
                    url.contains(API_KEY_REPLACE_STRING),
                    "Missing API key in URL pattern for provider: {:?} ({:?})",
                    provider,
                    cluster
                )
            }
            match access {
                RpcAccess::Authenticated { auth, public_url } => {
                    match auth {
                        RpcAuth::BearerToken { url } => {
                            assert_not_url_pattern(url, provider, cluster)
                        }
                        RpcAuth::UrlParameter { url_pattern } => {
                            assert_url_pattern(url_pattern, provider, cluster)
                        }
                    }
                    if let Some(public_url) = public_url {
                        assert_not_url_pattern(public_url, provider, cluster);
                    }
                }
                RpcAccess::Unauthenticated { public_url } => {
                    assert_not_url_pattern(public_url, provider, cluster);
                }
            }
        }
    })
}
