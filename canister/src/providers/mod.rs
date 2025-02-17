#[cfg(test)]
mod tests;

use sol_rpc_types::{Provider, ProviderId, RpcAccess, RpcAuth};
use sol_rpc_types::{RpcService, SolDevnetService, SolMainnetService, SolanaCluster};
use std::collections::HashMap;

thread_local! {
    pub static PROVIDERS: [Provider; 5] = [
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
            provider_id: "publicnode-mainnet".to_string(),
            cluster: SolanaCluster::Mainnet,
            access: RpcAccess::Unauthenticated {
                public_url: "https://solana-rpc.publicnode.com".to_string(),
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
