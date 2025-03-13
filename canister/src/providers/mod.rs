#[cfg(test)]
mod tests;

use maplit::btreemap;
use sol_rpc_types::{SupportedProvider, RpcAccess, RpcAuth, RpcProvider, SolanaCluster};
use std::collections::BTreeMap;

thread_local! {
    pub static PROVIDERS: BTreeMap<SupportedProvider, RpcProvider> = btreemap! {
        SupportedProvider::AlchemyMainnet => RpcProvider {
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::BearerToken {
                    url: "https://solana-mainnet.g.alchemy.com/v2".to_string(),
                },
                public_url: Some("https://solana-mainnet.g.alchemy.com/v2/demo".to_string()),
            }
        },
        SupportedProvider::AlchemyDevnet => RpcProvider {
            cluster: SolanaCluster::Devnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::BearerToken {
                    url: "https://solana-devnet.g.alchemy.com/v2".to_string(),
                },
                public_url: Some("https://solana-devnet.g.alchemy.com/v2/demo".to_string()),
            }
        },
        SupportedProvider::AnkrMainnet => RpcProvider {
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: "https://rpc.ankr.com/solana/{API_KEY}".to_string(),
                },
                public_url: None,
            }
        },
        SupportedProvider::AnkrDevnet => RpcProvider {
            cluster: SolanaCluster::Devnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: "https://rpc.ankr.com/solana_devnet/{API_KEY}".to_string(),
                },
                public_url: Some("https://rpc.ankr.com/solana_devnet/".to_string()),
            }
        },
        SupportedProvider::PublicNodeMainnet => RpcProvider {
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Unauthenticated {
                public_url: "https://solana-rpc.publicnode.com".to_string(),
            }
        },
    };
}

pub fn get_provider(provider_id: &SupportedProvider) -> Option<RpcProvider> {
    PROVIDERS.with(|providers| providers.get(provider_id).cloned())
}

pub fn get_access(provider_id: &SupportedProvider) -> Option<RpcAccess> {
    get_provider(provider_id).map(|provider| provider.access)
}
