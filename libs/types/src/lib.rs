//! Candid types used by the candid interface of the SOL RPC canister.

#![forbid(unsafe_code)]
#![forbid(missing_docs)]

mod lifecycle;
mod regex;
mod rpc_client;

pub use lifecycle::InstallArgs;
pub use regex::{RegexString, RegexSubstitution};
pub use rpc_client::{
    HttpHeader, OverrideProvider, Provider, ProviderId, RpcAccess, RpcApi, RpcAuth, RpcService,
    SolDevnetService, SolMainnetService, SolanaCluster,
};
