//! Candid types used by the candid interface of the SOL RPC canister.

#![forbid(unsafe_code)]
#![forbid(missing_docs)]

mod lifecycle;
mod rpc_client;

pub use lifecycle::{InstallArgs, Mode, NumSubnetNodes};
pub use rpc_client::{
    ConsensusStrategy, HttpHeader, OverrideProvider, RegexString, RegexSubstitution, RpcAccess,
    RpcAuth, RpcConfig, RpcEndpoint, RpcSource, RpcSources, SolanaCluster, SupportedRpcProvider,
    SupportedRpcProviderId,
};
