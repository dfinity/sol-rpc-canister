use crate::{
    constants::{SOL_DEVNET_CHAIN_ID, SOL_MAINNET_CHAIN_ID},
    types::{Provider, ProviderId, RpcAccess, RpcAuth},
};
use sol_rpc_types::{RpcService, SolDevnetService, SolMainnetService};
use std::collections::HashMap;

pub const PROVIDERS: &[Provider] = &[
    Provider {
        provider_id: 1,
        chain_id: SOL_MAINNET_CHAIN_ID,
        access: RpcAccess::Authenticated {
            auth: RpcAuth::UrlParameter {
                url_pattern: "https://rpc.ankr.com/eth/{API_KEY}",
            },
            public_url: Some("https://rpc.ankr.com/eth"),
        },
        alias: Some(RpcService::SolMainnet(SolMainnetService::Ankr)),
    },
    Provider {
        provider_id: 2,
        chain_id: SOL_MAINNET_CHAIN_ID,
        access: RpcAccess::Unauthenticated {
            public_url: "https://ethereum-rpc.publicnode.com",
        },
        alias: Some(RpcService::SolMainnet(SolMainnetService::PublicNode)),
    },
    Provider {
        provider_id: 5,
        chain_id: SOL_DEVNET_CHAIN_ID,
        access: RpcAccess::Authenticated {
            auth: RpcAuth::UrlParameter {
                url_pattern: "https://rpc.ankr.com/eth_sepolia/{API_KEY}",
            },
            public_url: Some("https://rpc.ankr.com/eth_sepolia"),
        },
        alias: Some(RpcService::SolDevnet(SolDevnetService::Ankr)),
    },
    Provider {
        provider_id: 8,
        chain_id: SOL_MAINNET_CHAIN_ID,
        access: RpcAccess::Authenticated {
            auth: RpcAuth::BearerToken {
                url: "https://eth-mainnet.g.alchemy.com/v2",
            },
            public_url: Some("https://eth-mainnet.g.alchemy.com/v2/demo"),
        },
        alias: Some(RpcService::SolMainnet(SolMainnetService::Alchemy)),
    },
    Provider {
        provider_id: 9,
        chain_id: SOL_DEVNET_CHAIN_ID,
        access: RpcAccess::Authenticated {
            auth: RpcAuth::BearerToken {
                url: "https://eth-sepolia.g.alchemy.com/v2",
            },
            public_url: Some("https://eth-sepolia.g.alchemy.com/v2/demo"),
        },
        alias: Some(RpcService::SolDevnet(SolDevnetService::Alchemy)),
    },
];

thread_local! {
    pub static PROVIDER_MAP: HashMap<ProviderId, Provider> =
        PROVIDERS.iter()
            .map(|provider| (provider.provider_id, provider.clone())).collect();

    pub static SERVICE_PROVIDER_MAP: HashMap<RpcService, ProviderId> =
        PROVIDERS.iter()
            .filter_map(|provider| Some((provider.alias.clone()?, provider.provider_id)))
            .collect();
}

#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet};

    use crate::{
        constants::API_KEY_REPLACE_STRING,
        types::{Provider, RpcAccess, RpcAuth},
    };

    use super::{PROVIDERS, SERVICE_PROVIDER_MAP};

    #[test]
    fn test_provider_id_sequence() {
        for (i, provider) in PROVIDERS.iter().enumerate() {
            assert_eq!(provider.provider_id, i as u64);
        }
    }

    #[test]
    fn test_rpc_provider_url_patterns() {
        for provider in PROVIDERS {
            fn assert_not_url_pattern(url: &str, provider: &Provider) {
                assert!(
                    !url.contains(API_KEY_REPLACE_STRING),
                    "Unexpected API key in URL for provider: {}",
                    provider.provider_id
                )
            }
            fn assert_url_pattern(url: &str, provider: &Provider) {
                assert!(
                    url.contains(API_KEY_REPLACE_STRING),
                    "Missing API key in URL pattern for provider: {}",
                    provider.provider_id
                )
            }
            match &provider.access {
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
    }

    #[test]
    fn test_no_duplicate_service_providers() {
        SERVICE_PROVIDER_MAP.with(|map| {
            assert_eq!(
                map.len(),
                map.keys().collect::<HashSet<_>>().len(),
                "Duplicate service in mapping"
            );
            assert_eq!(
                map.len(),
                map.values().collect::<HashSet<_>>().len(),
                "Duplicate provider in mapping"
            );
        })
    }

    #[test]
    fn test_service_provider_coverage() {
        SERVICE_PROVIDER_MAP.with(|map| {
            let inverse_map: HashMap<_, _> = map.iter().map(|(k, v)| (v, k)).collect();
            for provider in PROVIDERS {
                assert!(
                    inverse_map.contains_key(&provider.provider_id),
                    "Missing service mapping for provider with ID: {}",
                    provider.provider_id
                );
            }
        })
    }
}
