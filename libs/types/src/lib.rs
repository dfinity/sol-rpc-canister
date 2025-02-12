//! Candid types used by the candid interface of the SOL RPC canister.

#![forbid(unsafe_code)]
#![forbid(missing_docs)]

mod rpc_client;

use candid::CandidType;
use serde::{Deserialize, Serialize};

/// A dummy request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CandidType)]
pub struct DummyRequest {
    /// Input
    pub input: String,
}

/// A dummy response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CandidType)]
pub struct DummyResponse {
    /// Output
    pub output: String,
}
