mod ed25519;
pub mod solana_wallet;
pub mod spl;
pub mod state;

use crate::state::{read_state, State};
use candid::{CandidType, Deserialize, Principal};
use sol_rpc_client::{ed25519::Ed25519KeyId, IcRuntime, SolRpcClient};
use sol_rpc_types::{
    CommitmentLevel, ConsensusStrategy, MultiRpcResult, RpcConfig, RpcSources, SolanaCluster,
};
use solana_hash::Hash;
use std::str::FromStr;

// Fetch a recent blockhash using the Solana `getSlot` and `getBlock` methods.
// Since the `getSlot` method might fail due to Solana's fast blocktime, and some slots do not
// have blocks, we retry the RPC calls several times in case of failure to find a recent block.
pub async fn get_recent_blockhash(rpc_client: &SolRpcClient<IcRuntime>) -> Hash {
    let num_tries = 3;
    let mut errors = Vec::with_capacity(num_tries);
    loop {
        if errors.len() >= num_tries {
            panic!("Failed to get recent block hash after {num_tries} tries: {errors:?}");
        }
        match rpc_client.get_slot().send().await {
            MultiRpcResult::Consistent(Ok(slot)) => match rpc_client.get_block(slot).send().await {
                MultiRpcResult::Consistent(Ok(Some(block))) => {
                    return Hash::from_str(&block.blockhash).expect("Unable to parse blockhash")
                }
                MultiRpcResult::Consistent(Ok(None)) => {
                    errors.push(format!("No block for slot {slot}"));
                    continue;
                }
                MultiRpcResult::Inconsistent(results) => {
                    errors.push(format!(
                        "Inconsistent results for block with slot {slot}: {:?}",
                        results
                    ));
                    continue;
                }
                MultiRpcResult::Consistent(Err(e)) => {
                    errors.push(format!("Failed to get block with slot {slot}: {:?}", e));
                    continue;
                }
            },
            MultiRpcResult::Inconsistent(results) => {
                errors.push(format!("Failed to retrieved last slot: {:?}", results));
                continue;
            }
            MultiRpcResult::Consistent(Err(e)) => {
                errors.push(format!("Failed to retrieve slot: {:?}", e));
                continue;
            }
        }
    }
}

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
    let principal = ic_cdk::caller();
    if principal == Principal::anonymous() {
        panic!("anonymous principal is not allowed");
    }
    principal
}
