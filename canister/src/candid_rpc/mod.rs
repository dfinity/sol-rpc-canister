use crate::rpc_client::{ReducedResult, SolRpcClient};
use canhttp::multi::ReductionError;
use serde::Serialize;
use sol_rpc_types::{GetSlotParams, MultiRpcResult, RpcConfig, RpcResult, RpcSources};
use solana_clock::Slot;
use std::fmt::Debug;

fn process_result<T>(result: ReducedResult<T>) -> MultiRpcResult<T> {
    match result {
        Ok(value) => MultiRpcResult::Consistent(Ok(value)),
        Err(err) => match err {
            ReductionError::ConsistentError(err) => MultiRpcResult::Consistent(Err(err)),
            ReductionError::InconsistentResults(multi_call_results) => {
                let results: Vec<_> = multi_call_results.into_iter().collect();
                results.iter().for_each(|(_service, _service_result)| {
                    // TODO XC-296: Add metrics for inconsistent providers
                });
                MultiRpcResult::Inconsistent(results)
            }
        },
    }
}

/// Adapt the `EthRpcClient` to the `Candid` interface used by the EVM-RPC canister.
pub struct CandidRpcClient {
    client: SolRpcClient,
}

impl CandidRpcClient {
    pub fn new(source: RpcSources, config: Option<RpcConfig>) -> RpcResult<Self> {
        Ok(Self {
            client: SolRpcClient::new(source, config)?,
        })
    }

    pub async fn get_slot(&self, params: GetSlotParams) -> MultiRpcResult<Slot> {
        process_result(self.client.get_slot(params).await)
    }

    pub async fn raw_request<I>(
        &self,
        request: canhttp::http::json::JsonRpcRequest<I>,
    ) -> MultiRpcResult<serde_json::Value>
    where
        I: Serialize + Clone + Debug,
    {
        process_result(self.client.raw_request(request).await)
    }
}
