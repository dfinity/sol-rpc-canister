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
use serde_json::from_value;
use solana_account::Account;
use solana_account_decoder_client_types::UiAccount;
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
    GetAccountInfo,
    #[n(1)]
    GetSlot,
    #[n(2)]
    Raw,
}

impl ResponseTransform {
    fn apply(&self, body_bytes: &mut Vec<u8>) {
        use serde_json::{from_slice, to_vec, Value};

        fn canonicalize_get_account_info_response<T>(body: &mut Vec<u8>) -> Option<Vec<u8>>
        where
            T: Debug + Serialize + DeserializeOwned,
        {
            let response = from_slice::<JsonRpcResponse<Value>>(body)
                .ok()?
                .map(|result| {
                    from_value::<UiAccount>(result["value"].clone())
                        .unwrap()
                        .decode::<Account>()
                        .unwrap()
                });
            to_vec(&response).ok()
        }

        fn canonicalize_json_rpc_response<T>(body: &mut Vec<u8>)
        where
            T: Serialize + DeserializeOwned,
        {
            let response: JsonRpcResponse<T> = match from_slice(body) {
                Ok(response) => response,
                Err(_) => return,
            };
            *body = to_vec(&response).expect("BUG: failed to serialize response");
        }

        fn canonicalize_raw(text: &[u8]) -> Option<Vec<u8>> {
            let json = from_slice::<Value>(text).ok()?;
            to_vec(&json).ok()
        }

        match self {
            Self::GetAccountInfo => {
                if let Some(bytes) = canonicalize_get_account_info_response::<Account>(body_bytes) {
                    *body_bytes = bytes
                }
            }
            // TODO XC-292: Add rounding to the response transform and
            //  add a unit test simulating consensus when the providers
            //  return slightly differing results.
            Self::GetSlot => canonicalize_json_rpc_response::<Slot>(body_bytes),
            Self::Raw => {
                if let Some(bytes) = canonicalize_raw(body_bytes) {
                    *body_bytes = bytes
                }
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
