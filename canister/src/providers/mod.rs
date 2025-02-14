#[cfg(test)]
mod tests;

use sol_rpc_types::{Provider, ProviderId, RpcAccess, RpcAuth};
use sol_rpc_types::{RpcService, SolDevnetService, SolMainnetService, SolanaCluster};
use std::collections::HashMap;

thread_local! {
    pub static PROVIDERS: Vec<Provider> = vec![
        Provider {
            provider_id: String::from("alchemy-mainnet"),
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::BearerToken {
                    url: String::from("https://solana-mainnet.g.alchemy.com/v2"),
                },
                public_url: Some(String::from("https://solana-mainnet.g.alchemy.com/v2/demo")),
            },
            alias: Some(RpcService::SolMainnet(SolMainnetService::Alchemy)),
        },
        Provider {
            provider_id: String::from("alchemy-devnet"),
            cluster: SolanaCluster::Devnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::BearerToken {
                    url: String::from("https://solana-devnet.g.alchemy.com/v2"),
                },
                public_url: Some(String::from("https://solana-devnet.g.alchemy.com/v2/demo")),
            },
            alias: Some(RpcService::SolDevnet(SolDevnetService::Alchemy)),
        },
        Provider {
            provider_id: String::from("ankr-mainnet"),
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: String::from("https://rpc.ankr.com/solana/{API_KEY}"),
                },
                public_url: None,
            },
            alias: Some(RpcService::SolMainnet(SolMainnetService::Ankr)),
        },
        Provider {
            provider_id: String::from("ankr-devnet"),
            cluster: SolanaCluster::Devnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: String::from("https://rpc.ankr.com/solana_devnet/{API_KEY}"),
                },
                public_url: Some(String::from("https://rpc.ankr.com/solana_devnet/")),
            },
            alias: Some(RpcService::SolDevnet(SolDevnetService::Ankr)),
        },
        Provider {
            provider_id: String::from("publicnode-mainnet"),
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Unauthenticated {
                public_url: String::from("https://solana-rpc.publicnode.com"),
            },
            alias: Some(RpcService::SolMainnet(SolMainnetService::PublicNode)),
        },
    ];

    pub static PROVIDER_MAP: HashMap<ProviderId, Provider> = PROVIDERS.with(|providers| {
        providers
            .iter()
            .map(|provider| (provider.provider_id.clone(), provider.clone()))
            .collect()
    });

    pub static SERVICE_PROVIDER_MAP: HashMap<RpcService, ProviderId> = PROVIDERS.with(|providers| {
        providers
            .iter()
            .filter_map(|provider| Some((provider.alias.clone()?, provider.provider_id.clone())))
            .collect()
    });
}
