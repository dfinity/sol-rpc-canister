#[cfg(test)]
mod tests;

use crate::types::RoundingError;
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
use std::fmt::Debug;

/// Describes a payload transformation to execute before passing the HTTP response to consensus.
/// The purpose of these transformations is to ensure that the response encoding is deterministic
/// (the field order is the same).
#[derive(Clone, Debug, Decode, Encode)]
pub enum ResponseTransform {
    #[n(0)]
    GetSlot(#[n(1)] RoundingError),
    #[n(2)]
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

        match self {
            Self::GetSlot(rounding_error) => {
                canonicalize::<JsonRpcResponse<Slot>>(body_bytes, |response| {
                    response.map(|slot| rounding_error.round(slot))
                });
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
