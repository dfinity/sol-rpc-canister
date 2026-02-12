mod ed25519;
pub mod solana_wallet;
pub mod spl;
pub mod state;

use crate::state::{read_state, State};
use candid::{CandidType, Principal};
use ic_canister_runtime::IcRuntime;
use serde::Deserialize;
use sol_rpc_client::{ed25519::Ed25519KeyId, SolRpcClient, SOL_RPC_CANISTER};
use sol_rpc_types::{
    CommitmentLevel, ConsensusStrategy, RpcEndpoint, RpcSource, RpcSources, SolanaCluster,
};

pub fn client() -> SolRpcClient<IcRuntime> {
    let rpc_sources = read_state(|state| state.solana_network().clone()).into();
    let consensus_strategy = match rpc_sources {
        RpcSources::Custom(_) => ConsensusStrategy::Equality,
        RpcSources::Default(_) => ConsensusStrategy::Threshold {
            min: 2,
            total: Some(3),
        },
    };
    SolRpcClient::builder(IcRuntime::new(), sol_rpc_canister_id())
        .with_rpc_sources(rpc_sources)
        .with_consensus_strategy(consensus_strategy)
        .with_default_commitment_level(read_state(State::solana_commitment_level))
        .build()
}

fn sol_rpc_canister_id() -> Principal {
    const ENV_VAR_NAME: &str = "PUBLIC_CANISTER_ID:sol_rpc";
    if ic_cdk::api::env_var_name_exists(ENV_VAR_NAME) {
        Principal::from_text(&ic_cdk::api::env_var_value(ENV_VAR_NAME))
            .expect("Invalid SOL RPC canister ID")
    } else {
        SOL_RPC_CANISTER
    }
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct InitArg {
    pub sol_rpc_canister_id: Option<Principal>,
    pub solana_network: Option<SolanaNetwork>,
    pub ed25519_key_name: Option<Ed25519KeyName>,
    pub solana_commitment_level: Option<CommitmentLevel>,
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub enum SolanaNetwork {
    Mainnet,
    #[default]
    Devnet,
    Custom(RpcEndpoint),
}

impl From<SolanaNetwork> for RpcSources {
    fn from(network: SolanaNetwork) -> Self {
        match network {
            SolanaNetwork::Mainnet => Self::Default(SolanaCluster::Mainnet),
            SolanaNetwork::Devnet => Self::Default(SolanaCluster::Devnet),
            SolanaNetwork::Custom(endpoint) => Self::Custom(vec![RpcSource::Custom(endpoint)]),
        }
    }
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Ed25519KeyName {
    #[default]
    LocalDevelopment,
    MainnetTestKey1,
    MainnetProdKey1,
}

impl From<Ed25519KeyName> for Ed25519KeyId {
    fn from(key_id: Ed25519KeyName) -> Self {
        match key_id {
            Ed25519KeyName::LocalDevelopment => Self::LocalDevelopment,
            Ed25519KeyName::MainnetTestKey1 => Self::MainnetTestKey1,
            Ed25519KeyName::MainnetProdKey1 => Self::MainnetProdKey1,
        }
    }
}

pub fn validate_caller_not_anonymous() -> Principal {
    let principal = ic_cdk::api::msg_caller();
    if principal == Principal::anonymous() {
        panic!("anonymous principal is not allowed");
    }
    principal
}
