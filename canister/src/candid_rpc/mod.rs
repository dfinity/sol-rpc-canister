use crate::{
    add_metric_entry,
    metrics::MetricRpcMethod,
    providers::get_provider,
    rpc_client::{MultiRpcRequest, ReducedResult},
    util::hostname_from_url,
};
use canhttp::multi::ReductionError;
use serde::{de::DeserializeOwned, Serialize};
use sol_rpc_types::{
    MultiRpcResult, ProviderError, RpcAccess, RpcAuth, RpcError, RpcResult, RpcSource,
    SupportedRpcProvider,
};
use std::fmt::Debug;

pub async fn send_multi<Params, Output, Error>(
    request: Result<MultiRpcRequest<Params, Output>, Error>,
) -> MultiRpcResult<Output>
where
    Params: Serialize + Clone + Debug,
    Output: Debug + DeserializeOwned + PartialEq + Serialize,
    Error: Into<RpcError>,
{
    match request {
        Ok(request) => {
            let method = request.method().to_string();
            let result = request.send_and_reduce().await;
            process_result(method, result)
        }
        Err(e) => process_error(e),
    }
}

fn process_result<T>(
    method: impl Into<MetricRpcMethod>,
    result: ReducedResult<T>,
) -> MultiRpcResult<T> {
    match result {
        Ok(value) => MultiRpcResult::Consistent(Ok(value)),
        Err(err) => match err {
            ReductionError::ConsistentError(err) => MultiRpcResult::Consistent(Err(err)),
            ReductionError::InconsistentResults(multi_call_results) => {
                let results: Vec<_> = multi_call_results.into_iter().collect();
                observe_inconsistent(method.into(), &results);
                MultiRpcResult::Inconsistent(results)
            }
        },
    }
}

fn process_error<T, E: Into<RpcError>>(error: E) -> MultiRpcResult<T> {
    MultiRpcResult::Consistent(Err(error.into()))
}

fn observe_inconsistent<T>(method: MetricRpcMethod, results: &[(RpcSource, RpcResult<T>)]) {
    // Generally, `ProviderError::TooFewCycles` errors are expected to result in an inconsistent
    // response since the required number of cycles is different for each provider (due e.g. to
    // different request URL lengths). Therefore, do not increment inconsistent responses metrics
    // in the case of `ProviderError::TooFewCycles` results.
    if results.iter().all(|(_source, result)| {
        matches!(
            result,
            Err(RpcError::ProviderError(ProviderError::TooFewCycles { .. }))
        )
    }) {
        return;
    }
    // Otherwise, increment inconsistent responses metrics normally.
    results.iter().for_each(|(source, _service_result)| {
        if let RpcSource::Supported(provider_id) = source {
            if let Some(provider) = get_provider(provider_id) {
                if let Some(host) = hostname(provider.clone()) {
                    add_metric_entry!(inconsistent_responses, (method.clone(), host.into()), 1)
                }
            }
        }
    });
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
