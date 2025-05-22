mod ed25519;
pub mod solana_wallet;
pub mod spl;
pub mod state;

use crate::state::{read_state, State};
use candid::{CandidType, Deserialize, Principal};
use sol_rpc_client::{IcRuntime, SolRpcClient};
use sol_rpc_types::{CommitmentLevel, RpcSources, SolanaCluster};
use std::fmt::Display;

pub fn client() -> SolRpcClient<IcRuntime> {
    read_state(|state| state.sol_rpc_canister_id())
        .map(|canister_id| SolRpcClient::builder(IcRuntime, canister_id))
        .unwrap_or(SolRpcClient::builder_for_ic())
        .with_rpc_sources(RpcSources::Default(
            read_state(|state| state.solana_network()).into(),
        ))
        .with_default_commitment_level(read_state(State::solana_commitment_level))
        .build()
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct InitArg {
    pub sol_rpc_canister_id: Option<Principal>,
    pub solana_network: Option<SolanaNetwork>,
    pub ed25519_key_name: Option<Ed25519KeyName>,
    pub solana_commitment_level: Option<CommitmentLevel>,
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum SolanaNetwork {
    Mainnet,
    #[default]
    Devnet,
    Testnet,
}

impl From<SolanaNetwork> for SolanaCluster {
    fn from(network: SolanaNetwork) -> Self {
        match network {
            SolanaNetwork::Mainnet => Self::Mainnet,
            SolanaNetwork::Devnet => Self::Devnet,
            SolanaNetwork::Testnet => Self::Testnet,
        }
    }
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Ed25519KeyName {
    #[default]
    TestKeyLocalDevelopment,
    TestKey1,
    ProductionKey1,
}

impl Display for Ed25519KeyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Ed25519KeyName::TestKeyLocalDevelopment => "dfx_test_key",
            Ed25519KeyName::TestKey1 => "test_key_1",
            Ed25519KeyName::ProductionKey1 => "key_1",
        }
        .to_string();
        write!(f, "{}", str)
    }
}

pub fn validate_caller_not_anonymous() -> Principal {
    let principal = ic_cdk::caller();
    if principal == Principal::anonymous() {
        panic!("anonymous principal is not allowed");
    }
    principal
}
