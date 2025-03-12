//! Candid types used by the candid interface of the SOL RPC canister.

#![forbid(unsafe_code)]
#![forbid(missing_docs)]

mod lifecycle;
mod rpc_client;

pub use lifecycle::InstallArgs;
pub use rpc_client::{
    ConsensusStrategy, HttpHeader, OverrideProvider, ProviderId, RegexString, RegexSubstitution,
    RpcAccess, RpcAuth, RpcConfig, RpcEndpoint, RpcProvider, RpcSource, RpcSources, SolanaCluster,
};
