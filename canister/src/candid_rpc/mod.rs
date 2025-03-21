use crate::{
    add_metric_entry,
    metrics::{MetricRpcHost, RpcMethod},
    providers::get_provider,
    rpc_client::{MultiCallError, SolRpcClient},
    types::MultiRpcResult,
    util::hostname_from_url,
};
use sol_rpc_types::{
    GetSlotParams, RpcAccess, RpcAuth, RpcConfig, RpcResult, RpcSource, RpcSources,
    SupportedRpcProvider,
};
use solana_clock::Slot;

fn process_result<T>(method: RpcMethod, result: Result<T, MultiCallError<T>>) -> MultiRpcResult<T> {
    match result {
        Ok(value) => MultiRpcResult::Consistent(Ok(value)),
        Err(err) => match err {
            MultiCallError::ConsistentError(err) => MultiRpcResult::Consistent(Err(err)),
            MultiCallError::InconsistentResults(multi_call_results) => {
                let results = multi_call_results.into_vec();
                results.iter().for_each(|(source, _service_result)| {
                    if let RpcSource::Supported(provider_id) = source {
                        if let Some(provider) = get_provider(provider_id) {
                            add_metric_entry!(
                                inconsistent_responses,
                                (method.into(), MetricRpcHost(hostname(provider.clone()))),
                                1
                            )
                        }
                    }
                });
                MultiRpcResult::Inconsistent(results)
            }
        },
    }
}

pub fn hostname(provider: SupportedRpcProvider) -> String {
    let url = match provider.access {
        RpcAccess::Authenticated { auth, .. } => match auth {
            RpcAuth::BearerToken { url } => url,
            RpcAuth::UrlParameter { url_pattern } => url_pattern,
        },
        RpcAccess::Unauthenticated { public_url } => public_url,
    };
    hostname_from_url(url.as_str()).unwrap_or_else(|| "(unknown)".to_string())
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
        process_result(RpcMethod::GetSlot, self.client.get_slot(params).await)
    }
}
