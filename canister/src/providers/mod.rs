#[cfg(test)]
mod tests;

use crate::types::{Provider, ProviderId, RpcAccess, RpcAuth};
use sol_rpc_types::{RpcService, SolDevnetService, SolMainnetService, SolanaCluster};
use std::collections::HashMap;

pub const PROVIDERS: &[Provider] = &[
    Provider {
        provider_id: "alchemy-mainnet",
        cluster: SolanaCluster::Mainnet,
        access: RpcAccess::Authenticated {
            auth: RpcAuth::BearerToken {
                url: "https://solana-mainnet.g.alchemy.com/v2",
            },
            public_url: Some("https://solana-mainnet.g.alchemy.com/v2/demo"),
        },
        alias: Some(RpcService::SolMainnet(SolMainnetService::Alchemy)),
    },
    Provider {
        provider_id: "alchemy-devnet",
        cluster: SolanaCluster::Devnet,
        access: RpcAccess::Authenticated {
            auth: RpcAuth::BearerToken {
                url: "https://solana-devnet.g.alchemy.com/v2",
            },
            public_url: Some("https://solana-devnet.g.alchemy.com/v2/demo"),
        },
        alias: Some(RpcService::SolDevnet(SolDevnetService::Alchemy)),
    },
    Provider {
        provider_id: "ankr-mainnet",
        cluster: SolanaCluster::Mainnet,
        access: RpcAccess::Authenticated {
            auth: RpcAuth::UrlParameter {
                url_pattern: "https://rpc.ankr.com/solana/{API_KEY}",
            },
            public_url: None,
        },
        alias: Some(RpcService::SolMainnet(SolMainnetService::Ankr)),
    },
    Provider {
        provider_id: "ankr-devnet",
        cluster: SolanaCluster::Devnet,
        access: RpcAccess::Authenticated {
            auth: RpcAuth::UrlParameter {
                url_pattern: "https://rpc.ankr.com/solana_devnet/{API_KEY}",
            },
            public_url: Some("https://rpc.ankr.com/solana_devnet/"),
        },
        alias: Some(RpcService::SolDevnet(SolDevnetService::Ankr)),
    },
    Provider {
        provider_id: "publicnode-mainnet",
        cluster: SolanaCluster::Mainnet,
        access: RpcAccess::Unauthenticated {
            public_url: "https://solana-rpc.publicnode.com",
        },
        alias: Some(RpcService::SolMainnet(SolMainnetService::PublicNode)),
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
