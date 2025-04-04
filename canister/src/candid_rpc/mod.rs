use crate::{
    add_metric_entry, metrics::RpcMethod, providers::get_provider, rpc_client::ReducedResult,
    util::hostname_from_url,
};
use canhttp::multi::ReductionError;
use sol_rpc_types::{
    MultiRpcResult, RpcAccess, RpcAuth, RpcError, RpcSource, SupportedRpcProvider,
};

pub fn process_result<T>(method: RpcMethod, result: ReducedResult<T>) -> MultiRpcResult<T> {
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

pub fn process_error<T, E: Into<RpcError>>(error: E) -> MultiRpcResult<T> {
    MultiRpcResult::Consistent(Err(error.into()))
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
