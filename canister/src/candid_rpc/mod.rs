use crate::{
    add_metric_entry,
    metrics::RpcMethod,
    providers::get_provider,
    rpc_client::{ReducedResult, SolRpcClient},
    types::RoundingError,
    util::hostname_from_url,
};
use canhttp::multi::ReductionError;
use serde::Serialize;
use sol_rpc_types::{
    AccountInfo, GetAccountInfoParams, GetSlotParams, MultiRpcResult, RpcAccess, RpcAuth,
    RpcConfig, RpcResult, RpcSource, RpcSources, Slot, SupportedRpcProvider,
};
use std::fmt::Debug;

fn process_result<T>(method: RpcMethod, result: ReducedResult<T>) -> MultiRpcResult<T> {
    match result {
        Ok(value) => MultiRpcResult::Consistent(Ok(value)),
        Err(err) => match err {
            ReductionError::ConsistentError(err) => MultiRpcResult::Consistent(Err(err)),
            ReductionError::InconsistentResults(multi_call_results) => {
                let results: Vec<_> = multi_call_results.into_iter().collect();
                results.iter().for_each(|(source, _service_result)| {
                    if let RpcSource::Supported(provider_id) = source {
                        if let Some(provider) = get_provider(provider_id) {
                            if let Some(host) = hostname(provider.clone()) {
                                add_metric_entry!(
                                    inconsistent_responses,
                                    (method.into(), host.into()),
                                    1
                                )
                            }
                        }
                    }
                });
                MultiRpcResult::Inconsistent(results)
            }
        },
    }
}

pub fn hostname(provider: SupportedRpcProvider) -> Option<String> {
    let url = match provider.access {
        RpcAccess::Authenticated { auth, .. } => match auth {
            RpcAuth::BearerToken { url } => url,
            RpcAuth::UrlParameter { url_pattern } => url_pattern,
        },
        RpcAccess::Unauthenticated { public_url } => public_url,
    };
    hostname_from_url(url.as_str())
}

/// Adapt the `SolRpcClient` to the `Candid` interface used by the SOL-RPC canister.
pub struct CandidRpcClient {
    client: SolRpcClient,
}

impl CandidRpcClient {
    pub fn new(source: RpcSources, config: Option<RpcConfig>) -> RpcResult<Self> {
        Self::new_with_rounding_error(source, config, None)
    }

    pub fn new_with_rounding_error(
        source: RpcSources,
        config: Option<RpcConfig>,
        rounding_error: Option<RoundingError>,
    ) -> RpcResult<Self> {
        Ok(Self {
            client: SolRpcClient::new(source, config, rounding_error)?,
        })
    }

    pub async fn get_account_info(
        &self,
        params: GetAccountInfoParams,
    ) -> MultiRpcResult<Option<AccountInfo>> {
        process_result(
            RpcMethod::GetAccountInfo,
            self.client.get_account_info(params.into()).await,
        )
        .map(|maybe_account| maybe_account.map(AccountInfo::from))
    }

    pub async fn get_slot(&self, params: Option<GetSlotParams>) -> MultiRpcResult<Slot> {
        process_result(RpcMethod::GetSlot, self.client.get_slot(params).await)
    }

    pub async fn raw_request<I>(
        &self,
        request: canhttp::http::json::JsonRpcRequest<I>,
    ) -> MultiRpcResult<String>
    where
        I: Serialize + Clone + Debug,
    {
        process_result(RpcMethod::Generic, self.client.raw_request(request).await)
            .map(|value| value.to_string())
    }
}
