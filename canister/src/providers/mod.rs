#[cfg(test)]
mod tests;

use maplit::btreemap;
use sol_rpc_types::{ProviderId, RpcAccess, RpcAuth, RpcProvider, SolanaCluster};
use std::collections::BTreeMap;

thread_local! {
    pub static PROVIDERS: BTreeMap<RpcProvider, RpcAccess> = btreemap! {
        (ProviderId::Alchemy, SolanaCluster::Mainnet) => RpcAccess::Authenticated {
            auth: RpcAuth::BearerToken {
                url: "https://solana-mainnet.g.alchemy.com/v2".to_string(),
            },
            public_url: Some("https://solana-mainnet.g.alchemy.com/v2/demo".to_string()),
        },
        (ProviderId::Alchemy, SolanaCluster::Devnet) => RpcAccess::Authenticated {
            auth: RpcAuth::BearerToken {
                url: "https://solana-devnet.g.alchemy.com/v2".to_string(),
            },
            public_url: Some("https://solana-devnet.g.alchemy.com/v2/demo".to_string()),
        },
        (ProviderId::Ankr, SolanaCluster::Mainnet) => RpcAccess::Authenticated {
            auth: RpcAuth::UrlParameter {
                url_pattern: "https://rpc.ankr.com/solana/{API_KEY}".to_string(),
            },
            public_url: None,
        },
        (ProviderId::Ankr, SolanaCluster::Devnet) => RpcAccess::Authenticated {
            auth: RpcAuth::UrlParameter {
                url_pattern: "https://rpc.ankr.com/solana_devnet/{API_KEY}".to_string(),
            },
            public_url: Some("https://rpc.ankr.com/solana_devnet/".to_string()),
        },
        (ProviderId::PublicNode, SolanaCluster::Mainnet) => RpcAccess::Unauthenticated {
            public_url: "https://solana-rpc.publicnode.com".to_string(),
        },
    };
}

pub fn get_provider(provider: &RpcProvider) -> Option<RpcAccess> {
    PROVIDERS.with(|providers| providers.get(provider).cloned())
}
