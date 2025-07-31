#[cfg(test)]
mod tests;

use crate::{
    constants::API_KEY_REPLACE_STRING,
    memory::{rank_providers, read_state},
    types::OverrideProvider,
};
use canhttp::multi::{TimedSizedMap, TimedSizedVec, Timestamp};
use ic_cdk::api::management_canister::http_request::HttpHeader;
use maplit::btreemap;
use sol_rpc_types::{
    ConsensusStrategy, ProviderError, RpcAccess, RpcAuth, RpcEndpoint, RpcError, RpcResult,
    RpcSource, RpcSources, SolanaCluster, SupportedRpcProvider, SupportedRpcProviderId,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroUsize,
    time::Duration,
};

thread_local! {
    pub static PROVIDERS: BTreeMap<SupportedRpcProviderId, SupportedRpcProvider> = btreemap! {
        SupportedRpcProviderId::AlchemyMainnet => SupportedRpcProvider {
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::BearerToken {
                    url: "https://solana-mainnet.g.alchemy.com/v2".to_string(),
                },
                public_url: Some("https://solana-mainnet.g.alchemy.com/v2/demo".to_string()),
            }
        },
        SupportedRpcProviderId::AlchemyDevnet => SupportedRpcProvider {
            cluster: SolanaCluster::Devnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::BearerToken {
                    url: "https://solana-devnet.g.alchemy.com/v2".to_string(),
                },
                public_url: Some("https://solana-devnet.g.alchemy.com/v2/demo".to_string()),
            }
        },
        SupportedRpcProviderId::AnkrMainnet => SupportedRpcProvider {
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: "https://rpc.ankr.com/solana/{API_KEY}".to_string(),
                },
                public_url: None,
            }
        },
        SupportedRpcProviderId::AnkrDevnet => SupportedRpcProvider {
            cluster: SolanaCluster::Devnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: "https://rpc.ankr.com/solana_devnet/{API_KEY}".to_string(),
                },
                public_url: Some("https://rpc.ankr.com/solana_devnet/".to_string()),
            }
        },
        SupportedRpcProviderId::ChainstackMainnet => SupportedRpcProvider {
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: "https://solana-mainnet.core.chainstack.com/{API_KEY}".to_string(),
                },
                public_url: None,
            }
        },
        SupportedRpcProviderId::ChainstackDevnet => SupportedRpcProvider {
            cluster: SolanaCluster::Devnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: "https://solana-devnet.core.chainstack.com/{API_KEY}".to_string(),
                },
                public_url: None,
            }
        },
        SupportedRpcProviderId::DrpcMainnet => SupportedRpcProvider {
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
            auth: RpcAuth::UrlParameter {
                    url_pattern: "https://lb.drpc.org/ogrpc?network=solana&dkey={API_KEY}".to_string()
                },
                public_url: Some("https://solana.drpc.org".to_string()),
            }
        },
        SupportedRpcProviderId::DrpcDevnet => SupportedRpcProvider {
            cluster: SolanaCluster::Devnet,
            access: RpcAccess::Authenticated {
            auth: RpcAuth::UrlParameter {
                    url_pattern: "https://lb.drpc.org/ogrpc?network=solana-devnet&dkey={API_KEY}".to_string()
                },
                public_url: Some("https://solana-devnet.drpc.org".to_string()),
            }
        },
        SupportedRpcProviderId::HeliusMainnet => SupportedRpcProvider {
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: "https://mainnet.helius-rpc.com/?api-key={API_KEY}".to_string(),
                },
                public_url: None,
            },
        },
        SupportedRpcProviderId::HeliusDevnet => SupportedRpcProvider {
            cluster: SolanaCluster::Devnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: "https://devnet.helius-rpc.com/?api-key={API_KEY}".to_string(),
                },
                public_url: None,
            },
        },
        SupportedRpcProviderId::PublicNodeMainnet => SupportedRpcProvider {
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Unauthenticated {
                public_url: "https://solana-rpc.publicnode.com".to_string(),
            },
        },
    };
}

pub fn get_provider(provider_id: &SupportedRpcProviderId) -> Option<SupportedRpcProvider> {
    PROVIDERS.with(|providers| providers.get(provider_id).cloned())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Providers {
    /// *Non-empty* set of providers to query.
    pub sources: BTreeSet<RpcSource>,
}

impl Providers {
    // Order of providers matters!
    // The threshold consensus strategy will consider the first `total` providers in the order
    // they are specified if the providers are not explicitly specified by the caller.
    const MAINNET_PROVIDERS: &'static [SupportedRpcProviderId] = &[
        SupportedRpcProviderId::AlchemyMainnet,
        SupportedRpcProviderId::HeliusMainnet,
        SupportedRpcProviderId::DrpcMainnet,
        SupportedRpcProviderId::AnkrMainnet,
        SupportedRpcProviderId::PublicNodeMainnet,
        SupportedRpcProviderId::ChainstackMainnet,
    ];

    const DEVNET_PROVIDERS: &'static [SupportedRpcProviderId] = &[
        SupportedRpcProviderId::AlchemyDevnet,
        SupportedRpcProviderId::HeliusDevnet,
        SupportedRpcProviderId::DrpcDevnet,
        SupportedRpcProviderId::AnkrDevnet,
        SupportedRpcProviderId::ChainstackDevnet,
    ];

    const DEFAULT_NUM_PROVIDERS_FOR_EQUALITY: usize = 3;

    pub fn new(
        source: RpcSources,
        strategy: ConsensusStrategy,
        now: Timestamp,
    ) -> Result<Self, ProviderError> {
        fn supported_providers(
            cluster: &SolanaCluster,
        ) -> Result<&[SupportedRpcProviderId], ProviderError> {
            match cluster {
                SolanaCluster::Mainnet => Ok(Providers::MAINNET_PROVIDERS),
                SolanaCluster::Devnet => Ok(Providers::DEVNET_PROVIDERS),
                SolanaCluster::Testnet => {
                    Err(ProviderError::UnsupportedCluster(format!("{:?}", cluster)))
                }
            }
        }

        fn supported_rpc_source(supported_provider: SupportedRpcProviderId) -> RpcSource {
            RpcSource::Supported(supported_provider)
        }

        let providers: BTreeSet<_> = match strategy {
            ConsensusStrategy::Equality => match source {
                RpcSources::Custom(custom_providers) => Ok(custom_providers.into_iter().collect()),
                RpcSources::Default(cluster) => {
                    let supported_providers = supported_providers(&cluster)?;
                    assert!(
                        supported_providers.len() >= Self::DEFAULT_NUM_PROVIDERS_FOR_EQUALITY,
                        "BUG: need at least 3 providers, but got {supported_providers:?}"
                    );
                    Ok(rank_providers(supported_providers, now)
                        .into_iter()
                        .take(Self::DEFAULT_NUM_PROVIDERS_FOR_EQUALITY)
                        .map(supported_rpc_source)
                        .collect())
                }
            },
            ConsensusStrategy::Threshold { total, min } => {
                // Ensure that
                // 0 < min <= total <= all_providers.len()
                if min == 0 {
                    return Err(ProviderError::InvalidRpcConfig(
                        "min must be greater than 0".to_string(),
                    ));
                }
                match source {
                    RpcSources::Custom(custom_providers) => {
                        if min > custom_providers.len() as u8 {
                            return Err(ProviderError::InvalidRpcConfig(format!(
                                "min {} is greater than the number of specified providers {}",
                                min,
                                custom_providers.len()
                            )));
                        }
                        if let Some(total) = total {
                            if total != custom_providers.len() as u8 {
                                return Err(ProviderError::InvalidRpcConfig(format!(
                                    "total {} is different than the number of specified providers {}",
                                    total,
                                    custom_providers.len()
                                )));
                            }
                        };
                        Ok(custom_providers.into_iter().collect())
                    }
                    RpcSources::Default(cluster) => {
                        let supported_providers = supported_providers(&cluster)?;
                        let all_providers_len = supported_providers.len();
                        let total = total.ok_or_else(|| {
                            ProviderError::InvalidRpcConfig(
                                "total must be specified when using default providers".to_string(),
                            )
                        })?;

                        if min > total {
                            return Err(ProviderError::InvalidRpcConfig(format!(
                                "min {} is greater than total {}",
                                min, total
                            )));
                        }

                        if total > all_providers_len as u8 {
                            return Err(ProviderError::InvalidRpcConfig(format!(
                                "total {} is greater than the number of all supported providers {}",
                                total, all_providers_len
                            )));
                        }
                        let providers: BTreeSet<_> = rank_providers(supported_providers, now)
                            .into_iter()
                            .take(total as usize)
                            .map(supported_rpc_source)
                            .collect();
                        assert_eq!(providers.len(), total as usize, "BUG: duplicate providers");
                        Ok(providers)
                    }
                }
            }
        }?;

        if providers.is_empty() {
            return Err(ProviderError::InvalidRpcConfig(
                "No matching providers found".to_string(),
            ));
        }

        Ok(Self { sources: providers })
    }
}

pub fn resolve_rpc_provider(service: RpcSource) -> RpcEndpoint {
    match service {
        RpcSource::Supported(provider_id) => get_provider(&provider_id)
            .map(|provider| resolve_api_key(provider.access, provider_id))
            .expect("Unknown provider"),
        RpcSource::Custom(api) => api,
    }
}

fn resolve_api_key(access: RpcAccess, provider: SupportedRpcProviderId) -> RpcEndpoint {
    match &access {
        RpcAccess::Authenticated { auth, public_url } => {
            let api_key = read_state(|s| s.get_api_key(&provider));
            match api_key {
                Some(api_key) => match auth {
                    RpcAuth::BearerToken { url } => RpcEndpoint {
                        url: url.to_string(),
                        headers: Some(vec![HttpHeader {
                            name: "Authorization".to_string(),
                            value: format!("Bearer {}", api_key.read()),
                        }]),
                    },
                    RpcAuth::UrlParameter { url_pattern } => RpcEndpoint {
                        url: url_pattern.replace(API_KEY_REPLACE_STRING, api_key.read()),
                        headers: None,
                    },
                },
                None => RpcEndpoint {
                    url: public_url.clone().unwrap_or_else(|| {
                        panic!("API key not yet initialized for provider: {:?}", provider)
                    }),
                    headers: None,
                },
            }
        }
        RpcAccess::Unauthenticated { public_url } => RpcEndpoint {
            url: public_url.to_string(),
            headers: None,
        },
    }
}

pub fn request_builder(
    endpoint: RpcEndpoint,
    override_provider: &OverrideProvider,
) -> RpcResult<http::request::Builder> {
    let endpoint = override_provider.apply(endpoint).map_err(|regex_error| {
        RpcError::ValidationError(format!(
            "BUG: regex should have been validated when initially set. Error: {regex_error}"
        ))
    })?;
    let mut request_builder = http::Request::post(endpoint.url);
    for HttpHeader { name, value } in endpoint.headers.unwrap_or_default() {
        request_builder = request_builder.header(name, value);
    }
    Ok(request_builder)
}

/// Record when a supported RPC service was used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportedRpcProviderUsage(TimedSizedMap<SupportedRpcProviderId, ()>);

impl Default for SupportedRpcProviderUsage {
    fn default() -> Self {
        Self::new()
    }
}

impl SupportedRpcProviderUsage {
    pub fn new() -> SupportedRpcProviderUsage {
        Self(TimedSizedMap::new(
            Duration::from_secs(20 * 60),
            NonZeroUsize::new(500).unwrap(),
        ))
    }

    pub fn record_evict(&mut self, service: SupportedRpcProviderId, now: Timestamp) {
        self.0.insert_evict(now, service, ());
    }

    pub fn rank_ascending_evict(
        &mut self,
        providers: &[SupportedRpcProviderId],
        now: Timestamp,
    ) -> Vec<SupportedRpcProviderId> {
        fn ascending_num_elements<V>(values: Option<&TimedSizedVec<V>>) -> impl Ord {
            std::cmp::Reverse(values.map(|v| v.len()).unwrap_or_default())
        }

        self.0.evict_expired(providers, now);
        self.0
            .sort_keys_by(providers, ascending_num_elements)
            .copied()
            .collect()
    }
}
