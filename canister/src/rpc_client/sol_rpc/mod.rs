#[cfg(test)]
mod tests;

use crate::{
    http::http_client,
    metrics::MetricRpcMethod,
    providers::{request_builder, resolve_rpc_provider},
    state::read_state,
};
use candid::candid_method;
use canhttp::{
    http::json::{JsonRpcRequest, JsonRpcResponse},
    MaxResponseBytesRequestExtension, TransformContextRequestExtension,
};
use ic_cdk::{
    api::management_canister::http_request::{HttpResponse, TransformArgs, TransformContext},
    query,
};
use minicbor::{Decode, Encode};
use serde::{de::DeserializeOwned, Serialize};
use sol_rpc_types::{JsonRpcError, RpcError, RpcSource};
use solana_clock::Slot;
use std::{fmt, fmt::Debug};

// This constant is our approximation of the expected header size.
// The HTTP standard doesn't define any limit, and many implementations limit
// the headers size to 8 KiB. We chose a lower limit because headers observed on most providers
// fit in the constant defined below, and if there is a spike, then the payload size adjustment
// should take care of that.
pub const HEADER_SIZE_LIMIT: u64 = 2 * 1024;

// This constant comes from the IC specification:
// > If provided, the value must not exceed 2MB
const HTTP_MAX_SIZE: u64 = 2_000_000;

pub const MAX_PAYLOAD_SIZE: u64 = HTTP_MAX_SIZE - HEADER_SIZE_LIMIT;

/// Describes a payload transformation to execute before passing the HTTP response to consensus.
/// The purpose of these transformations is to ensure that the response encoding is deterministic
/// (the field order is the same).
#[derive(Debug, Decode, Encode)]
pub enum ResponseTransform {
    #[n(0)]
    GetSlot,
}

impl ResponseTransform {
    fn apply(&self, body_bytes: &mut Vec<u8>) {
        fn redact_response<T>(body: &mut Vec<u8>)
        where
            T: Serialize + DeserializeOwned,
        {
            let response: JsonRpcResponse<T> = match serde_json::from_slice(body) {
                Ok(response) => response,
                Err(_) => return,
            };
            *body = serde_json::to_vec(&response).expect("BUG: failed to serialize response");
        }

        match self {
            // TODO XC-292: Add rounding to the response transform and
            //  add a unit test simulating consensus when the providers
            //  return slightly differing results.
            Self::GetSlot => redact_response::<Slot>(body_bytes),
        }
    }
}

#[query]
#[candid_method(query)]
fn cleanup_response(mut args: TransformArgs) -> HttpResponse {
    args.response.headers.clear();
    let status_ok = args.response.status >= 200u16 && args.response.status < 300u16;
    if status_ok && !args.context.is_empty() {
        let maybe_transform: Result<ResponseTransform, _> = minicbor::decode(&args.context[..]);
        if let Ok(transform) = maybe_transform {
            transform.apply(&mut args.response.body);
        }
    }
    args.response
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResponseSizeEstimate(u64);

impl ResponseSizeEstimate {
    pub fn new(num_bytes: u64) -> Self {
        assert!(num_bytes > 0);
        assert!(num_bytes <= MAX_PAYLOAD_SIZE);
        Self(num_bytes)
    }

    /// Describes the expected (90th percentile) number of bytes in the HTTP response body.
    /// This number should be less than `MAX_PAYLOAD_SIZE`.
    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ResponseSizeEstimate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Calls a JSON-RPC method at the specified URL.
pub async fn call<I, O>(
    provider: &RpcSource,
    method: impl Into<String>,
    params: I,
    response_size_estimate: ResponseSizeEstimate,
    response_transform: &Option<ResponseTransform>,
) -> Result<O, RpcError>
where
    I: Serialize + Clone + Debug,
    O: Debug + DeserializeOwned,
{
    use tower::Service;

    let transform_op = response_transform
        .as_ref()
        .map(|t| {
            let mut buf = vec![];
            minicbor::encode(t, &mut buf).unwrap();
            buf
        })
        .unwrap_or_default();

    let effective_size_estimate = response_size_estimate.get();
    let request = request_builder(
        resolve_rpc_provider(provider.clone()),
        &read_state(|state| state.get_override_provider()),
    )?
    .max_response_bytes(effective_size_estimate)
    .transform_context(TransformContext::from_name(
        "cleanup_response".to_owned(),
        transform_op.clone(),
    ))
    .body(JsonRpcRequest::new(method, params))
    .expect("BUG: invalid request");

    let rpc_method = request.body().method().to_string();
    let mut client = http_client(MetricRpcMethod(rpc_method.clone()), true);
    let response = client.call(request).await?;
    match response.into_body().into_result() {
        Ok(r) => Ok(r),
        Err(canhttp::http::json::JsonRpcError {
            code,
            message,
            data: _,
        }) => Err(JsonRpcError { code, message }.into()),
    }
}
