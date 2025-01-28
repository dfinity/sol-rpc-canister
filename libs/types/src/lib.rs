#[cfg(test)]
mod tests;

use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CandidType)]
pub struct DummyRequest {
    pub input: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, CandidType)]
pub struct DummyResponse {
    pub output: String,
}
