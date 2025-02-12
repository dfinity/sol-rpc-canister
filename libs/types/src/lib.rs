//! Candid types used by the candid interface of the SOL RPC canister.

#![forbid(unsafe_code)]
#![forbid(missing_docs)]

mod rpc_client;

pub use rpc_client::{
    HttpHeader, Provider, RpcAccess, RpcAuth, RpcService, SolDevnetService, SolMainnetService,
};
