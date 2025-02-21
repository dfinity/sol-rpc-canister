#[cfg(test)]
mod tests;

use sol_rpc_types::{Provider, ProviderId, RpcAccess, RpcAuth};
use sol_rpc_types::{RpcService, SolDevnetService, SolMainnetService, SolanaCluster};
use std::collections::HashMap;

thread_local! {
    pub static PROVIDERS: [Provider; 10] = [
        Provider {
            provider_id: "alchemy-mainnet".to_string(),
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::BearerToken {
                    url: "https://solana-mainnet.g.alchemy.com/v2".to_string(),
                },
                public_url: Some("https://solana-mainnet.g.alchemy.com/v2/demo".to_string()),
            },
            alias: Some(RpcService::SolMainnet(SolMainnetService::Alchemy)),
        },
        Provider {
            provider_id: "alchemy-devnet".to_string(),
            cluster: SolanaCluster::Devnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::BearerToken {
                    url: "https://solana-devnet.g.alchemy.com/v2".to_string(),
                },
                public_url: Some("https://solana-devnet.g.alchemy.com/v2/demo".to_string()),
            },
            alias: Some(RpcService::SolDevnet(SolDevnetService::Alchemy)),
        },
        Provider {
            provider_id: "ankr-mainnet".to_string(),
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: "https://rpc.ankr.com/solana/{API_KEY}".to_string(),
                },
                public_url: None,
            },
            alias: Some(RpcService::SolMainnet(SolMainnetService::Ankr)),
        },
        Provider {
            provider_id: "ankr-devnet".to_string(),
            cluster: SolanaCluster::Devnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: "https://rpc.ankr.com/solana_devnet/{API_KEY}".to_string(),
                },
                public_url: Some("https://rpc.ankr.com/solana_devnet/".to_string()),
            },
            alias: Some(RpcService::SolDevnet(SolDevnetService::Ankr)),
        },
        Provider {
            provider_id: "drpc-mainnet".to_string(),
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Unauthenticated {
                public_url: "https://solana.drpc.org".to_string(),
            },
            alias: Some(RpcService::SolMainnet(SolMainnetService::DRPC)),
        },
        Provider {
            provider_id: "drpc-devnet".to_string(),
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Unauthenticated {
                public_url: "https://solana-devnet.drpc.org".to_string(),
            },
            alias: Some(RpcService::SolMainnet(SolMainnetService::DRPC)),
        },
        Provider {
            provider_id: "helius-mainnet".to_string(),
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: " https://devnet.helius-rpc.com/?api-key={API_KEY}".to_string(),
                },
                public_url: None,
            },
            alias: Some(RpcService::SolMainnet(SolMainnetService::Helius)),
        },
        Provider {
            provider_id: "helius-devnet".to_string(),
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: " https://mainnet.helius-rpc.com/?api-key={API_KEY}".to_string(),
                },
                public_url: None,
            },
            alias: Some(RpcService::SolMainnet(SolMainnetService::Helius)),
        },
        Provider {
            provider_id: "publicnode-mainnet".to_string(),
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Unauthenticated {
                public_url: "https://solana-rpc.publicnode.com".to_string(),
            },
            alias: Some(RpcService::SolMainnet(SolMainnetService::PublicNode)),
        },
        Provider {
            provider_id: "lava-network-mainnet".to_string(),
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Authenticated {
                auth: RpcAuth::UrlParameter {
                    url_pattern: "https://g.w.lavanet.xyz:443/gateway/solana/rpc-http/{API_KEY}".to_string(),
                },
                public_url: None,
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

pub fn find_provider(f: impl Fn(&Provider) -> bool) -> Option<Provider> {
    PROVIDERS.with(|providers| providers.iter().find(|&provider| f(provider)).cloned())
}
