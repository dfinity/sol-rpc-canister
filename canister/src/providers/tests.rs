use super::PROVIDERS;
use crate::constants::API_KEY_REPLACE_STRING;
use sol_rpc_types::{RpcAccess, RpcAuth, RpcProvider, SupportedProvider};

#[test]
fn test_rpc_provider_url_patterns() {
    PROVIDERS.with(|providers| {
        for (provider, RpcProvider { access, .. }) in providers {
            fn assert_not_url_pattern(url: &str, provider: &SupportedProvider) {
                assert!(
                    !url.contains(API_KEY_REPLACE_STRING),
                    "Unexpected API key in URL for provider: {:?}",
                    provider
                )
            }
            fn assert_url_pattern(url: &str, provider: &SupportedProvider) {
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
