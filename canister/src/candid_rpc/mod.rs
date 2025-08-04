use crate::{rpc_client::MultiRpcRequest, util::hostname_from_url};
use canhttp::multi::ReductionError;
use serde::{de::DeserializeOwned, Serialize};
use sol_rpc_types::{MultiRpcResult, RpcAccess, RpcAuth, RpcError, SupportedRpcProvider};
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
        Ok(request) => match request.send_and_reduce().await {
            Ok(value) => MultiRpcResult::Consistent(Ok(value)),
            Err(err) => match err {
                ReductionError::ConsistentError(err) => MultiRpcResult::Consistent(Err(err)),
                ReductionError::InconsistentResults(multi_call_results) => {
                    let results: Vec<_> = multi_call_results.into_iter().collect();
                    MultiRpcResult::Inconsistent(results)
                }
            },
        },
        Err(e) => process_error(e),
    }
}

fn process_error<T, E: Into<RpcError>>(error: E) -> MultiRpcResult<T> {
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
