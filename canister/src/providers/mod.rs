#[cfg(test)]
mod tests;

use crate::{constants::API_KEY_REPLACE_STRING, state::read_state, types::OverrideProvider};
use ic_cdk::api::management_canister::http_request::HttpHeader;
use maplit::btreemap;
use sol_rpc_types::{
    ConsensusStrategy, ProviderError, RpcAccess, RpcAuth, RpcEndpoint, RpcError, RpcResult,
    RpcSource, RpcSources, SolanaCluster, SupportedRpcProvider, SupportedRpcProviderId,
};
use std::collections::{BTreeMap, BTreeSet};

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
        SupportedRpcProviderId::DrpcMainnet => SupportedRpcProvider {
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Unauthenticated {
                public_url: "https://solana.drpc.org".to_string(),
            },
        },
        SupportedRpcProviderId::DrpcDevnet => SupportedRpcProvider {
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Unauthenticated {
                public_url: "https://solana-devnet.drpc.org".to_string(),
            },
        },
        SupportedRpcProviderId::HeliusMainnet => SupportedRpcProvider {
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: "https://devnet.helius-rpc.com/?api-key={API_KEY}".to_string(),
                },
                public_url: None,
            },
        },
        SupportedRpcProviderId::HeliusDevnet => SupportedRpcProvider {
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: "https://mainnet.helius-rpc.com/?api-key={API_KEY}".to_string(),
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
    // they are specified (taking the default ones first, followed by the non default ones if necessary)
    // if the providers are not explicitly specified by the caller.
    const DEFAULT_MAINNET_SUPPORTED_PROVIDERS: &'static [SupportedRpcProviderId] = &[
        SupportedRpcProviderId::AlchemyMainnet,
        SupportedRpcProviderId::AnkrMainnet,
        SupportedRpcProviderId::PublicNodeMainnet,
        SupportedRpcProviderId::DrpcMainnet,
    ];
    const NON_DEFAULT_MAINNET_SUPPORTED_PROVIDERS: &'static [SupportedRpcProviderId] =
        &[SupportedRpcProviderId::HeliusMainnet];

    const DEFAULT_DEVNET_SUPPORTED_PROVIDERS: &'static [SupportedRpcProviderId] = &[
        SupportedRpcProviderId::AlchemyDevnet,
        SupportedRpcProviderId::AnkrDevnet,
        SupportedRpcProviderId::DrpcDevnet,
    ];
    const NON_DEFAULT_DEVNET_SUPPORTED_PROVIDERS: &'static [SupportedRpcProviderId] =
        &[SupportedRpcProviderId::HeliusDevnet];

    pub fn new(source: RpcSources, strategy: ConsensusStrategy) -> Result<Self, ProviderError> {
        fn get_sources(provider_ids: &[SupportedRpcProviderId]) -> Vec<RpcSource> {
            provider_ids
                .iter()
                .map(|provider| RpcSource::Supported(*provider))
                .collect()
        }

        let providers: BTreeSet<_> = match source {
            RpcSources::Custom(sources) => {
                choose_providers(Some(sources), vec![], vec![], strategy)?
            }
            RpcSources::Default(cluster) => match cluster {
                SolanaCluster::Mainnet => choose_providers(
                    None,
                    get_sources(Self::DEFAULT_MAINNET_SUPPORTED_PROVIDERS),
                    get_sources(Self::NON_DEFAULT_MAINNET_SUPPORTED_PROVIDERS),
                    strategy,
                )?,
                SolanaCluster::Devnet => choose_providers(
                    None,
                    get_sources(Self::DEFAULT_DEVNET_SUPPORTED_PROVIDERS),
                    get_sources(Self::NON_DEFAULT_DEVNET_SUPPORTED_PROVIDERS),
                    strategy,
                )?,
                cluster => return Err(ProviderError::UnsupportedCluster(format!("{:?}", cluster))),
            },
        };

        if providers.is_empty() {
            return Err(ProviderError::InvalidRpcConfig(
                "No matching providers found".to_string(),
            ));
        }

        Ok(Self { sources: providers })
    }
}

fn choose_providers(
    user_input: Option<Vec<RpcSource>>,
    default_providers: Vec<RpcSource>,
    non_default_providers: Vec<RpcSource>,
    strategy: ConsensusStrategy,
) -> Result<BTreeSet<RpcSource>, ProviderError> {
    match strategy {
        ConsensusStrategy::Equality => Ok(user_input
            .unwrap_or_else(|| default_providers.to_vec())
            .into_iter()
            .collect()),
        ConsensusStrategy::Threshold { total, min } => {
            // Ensure that
            // 0 < min <= total <= all_providers.len()
            if min == 0 {
                return Err(ProviderError::InvalidRpcConfig(
                    "min must be greater than 0".to_string(),
                ));
            }
            match user_input {
                None => {
                    let all_providers_len = default_providers.len() + non_default_providers.len();
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
                    let providers: BTreeSet<_> = default_providers
                        .iter()
                        .chain(non_default_providers.iter())
                        .take(total as usize)
                        .cloned()
                        .collect();
                    assert_eq!(providers.len(), total as usize, "BUG: duplicate providers");
                    Ok(providers)
                }
                Some(providers) => {
                    if min > providers.len() as u8 {
                        return Err(ProviderError::InvalidRpcConfig(format!(
                            "min {} is greater than the number of specified providers {}",
                            min,
                            providers.len()
                        )));
                    }
                    if let Some(total) = total {
                        if total != providers.len() as u8 {
                            return Err(ProviderError::InvalidRpcConfig(format!(
                                "total {} is different than the number of specified providers {}",
                                total,
                                providers.len()
                            )));
                        }
                    }
                    Ok(providers.into_iter().collect())
                }
            }
        }
    }
}

pub fn resolve_rpc_provider(service: RpcSource) -> RpcEndpoint {
    match service {
        RpcSource::Supported(provider_id) => PROVIDERS
            .with(|providers| providers.get(&provider_id).cloned())
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
