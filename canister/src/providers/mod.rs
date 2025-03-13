#[cfg(test)]
mod tests;

use maplit::btreemap;
use sol_rpc_types::{RpcAccess, RpcAuth, SupportedRpcProvider, SolanaCluster, SupportedRpcProviderId};
use std::collections::BTreeMap;

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
        SupportedRpcProviderId::PublicNodeMainnet => SupportedRpcProvider {
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Unauthenticated {
                public_url: "https://solana-rpc.publicnode.com".to_string(),
            }
        },
    };
}

pub fn get_provider(provider_id: &SupportedRpcProviderId) -> Option<SupportedRpcProvider> {
    PROVIDERS.with(|providers| providers.get(provider_id).cloned())
}
