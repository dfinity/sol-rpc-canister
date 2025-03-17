use crate::{
    rpc_client::{MultiCallError, Slot, SolRpcClient},
    types::MultiRpcResult,
};
use sol_rpc_types::{GetSlotParams, RpcConfig, RpcResult, RpcSources};

fn process_result<T>(result: Result<T, MultiCallError<T>>) -> MultiRpcResult<T> {
    match result {
        Ok(value) => MultiRpcResult::Consistent(Ok(value)),
        Err(err) => match err {
            MultiCallError::ConsistentError(err) => MultiRpcResult::Consistent(Err(err)),
            MultiCallError::InconsistentResults(multi_call_results) => {
                let results = multi_call_results.into_vec();
                results.iter().for_each(|(_service, _service_result)| {
                    // TODO XC-292: Add metrics for inconsistent providers
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

    pub async fn get_slot(&self, args: GetSlotParams) -> MultiRpcResult<u64> {
        process_result(self.client.get_slot(args).await).map(|Slot(slot)| slot)
    }
}
