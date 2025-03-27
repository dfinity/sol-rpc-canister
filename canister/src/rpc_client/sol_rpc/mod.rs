#[cfg(test)]
mod tests;

use candid::candid_method;
use canhttp::http::json::JsonRpcResponse;
use ic_cdk::{
    api::management_canister::http_request::{HttpResponse, TransformArgs},
    query,
};
use minicbor::{Decode, Encode};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_slice, to_vec};
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
    #[n(1)]
    Raw,
}

impl ResponseTransform {
    fn apply(&self, body_bytes: &mut Vec<u8>) {
        fn canonicalize<T>(body_bytes: &mut Vec<u8>, f: impl FnOnce(T) -> T)
        where
            T: Serialize + DeserializeOwned,
        {
            if let Ok(Ok(bytes)) = from_slice::<T>(body_bytes).map(f).as_ref().map(to_vec) {
                *body_bytes = bytes
            }
        }

        // TODO: Add `map_result` method to `JsonRpcResponse` instead
        fn map_json_rpc_result<T>(
            f: fn(T) -> T,
        ) -> impl FnOnce(JsonRpcResponse<T>) -> JsonRpcResponse<T> {
            move |response: JsonRpcResponse<T>| {
                JsonRpcResponse::from_parts(response.id().clone(), response.into_result().map(f))
            }
        }

        match self {
            // TODO XC-292: Add rounding to the response transform and
            //  add a unit test simulating consensus when the providers
            //  return slightly differing results.
            Self::GetSlot => {
                canonicalize::<JsonRpcResponse<Slot>>(
                    body_bytes,
                    map_json_rpc_result(|slot: Slot| slot),
                );
            }
            Self::Raw => {
                canonicalize::<serde_json::Value>(body_bytes, std::convert::identity);
            }
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
