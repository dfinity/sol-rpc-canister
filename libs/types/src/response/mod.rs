use crate::{RpcResult, RpcSource};
use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Represents an aggregated result from multiple RPC calls to different RPC providers.
/// The results are aggregated using a [`crate::ConsensusStrategy`].
#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
pub enum MultiRpcResult<T> {
    /// The results from the different providers were consistent.
    Consistent(RpcResult<T>),
    /// The results from the different providers were not consistent.
    Inconsistent(Vec<(RpcSource, RpcResult<T>)>),
}

impl<T> From<RpcResult<T>> for MultiRpcResult<T> {
    fn from(result: RpcResult<T>) -> Self {
        MultiRpcResult::Consistent(result)
    }
}

impl<T> MultiRpcResult<T> {
    /// Maps a `MultiRpcResult` containing values of type `T` to a `MultiRpcResult` containing values
    /// of type `R`.
    pub fn map<R, F>(self, f: F) -> MultiRpcResult<R>
    where
        F: FnOnce(T) -> R + Clone,
    {
        match self {
            MultiRpcResult::Consistent(result) => MultiRpcResult::Consistent(result.map(f)),
            MultiRpcResult::Inconsistent(results) => MultiRpcResult::Inconsistent(
                results
                    .into_iter()
                    .map(|(source, result)| (source, result.map(f.clone())))
                    .collect(),
            ),
        }
    }
}

/// blah
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GenericRequestResult {
    /// blah
    pub body: String,
}
