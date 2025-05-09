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
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{from_slice, to_vec, Value};
use solana_clock::Slot;
use solana_transaction_status_client_types::TransactionStatus;
use std::fmt::Debug;

/// Describes a payload transformation to execute before passing the HTTP response to consensus.
/// The purpose of these transformations is to ensure that the response encoding is deterministic
/// (the field order is the same).
#[derive(Clone, Debug, Decode, Encode)]
pub enum ResponseTransform {
    #[n(0)]
    GetAccountInfo,
    #[n(1)]
    GetBalance,
    #[n(2)]
    GetBlock,
    #[n(3)]
    GetSignatureStatuses,
    #[n(4)]
    GetSlot(#[n(0)] RoundingError),
    #[n(5)]
    GetTokenAccountBalance,
    #[n(6)]
    GetTransaction,
    #[n(7)]
    SendTransaction,
    #[n(8)]
    Raw,
}

impl ResponseTransform {
    fn apply(&self, body_bytes: &mut Vec<u8>) {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub struct SolanaRpcResult<T> {
            // This field is always ignored since it contains the fast-changing current
            // slot value for which consensus cannot generally be reached across nodes.
            context: Value,
            pub value: T,
        }

        fn ignore_context<T>(value: SolanaRpcResult<T>) -> T {
            value.value
        }

        fn canonicalize_response<T, R>(body_bytes: &mut Vec<u8>, f: impl FnOnce(T) -> R)
        where
            T: Serialize + DeserializeOwned + Debug,
            R: Serialize + DeserializeOwned,
        {
            if let Ok(response) = from_slice::<JsonRpcResponse<T>>(body_bytes) {
                if let Ok(bytes) = to_vec(&response.map(f)) {
                    *body_bytes = bytes
                }
            }
        }

        match self {
            Self::GetAccountInfo => {
                canonicalize_response::<SolanaRpcResult<Option<Value>>, Option<Value>>(
                    body_bytes,
                    ignore_context,
                );
            }
            Self::GetBalance => {
                canonicalize_response::<SolanaRpcResult<Value>, Value>(body_bytes, ignore_context);
            }
            Self::GetBlock => {
                canonicalize_response::<Value, Option<Value>>(body_bytes, |result| match result {
                    Value::Null => None,
                    value => Some(value),
                });
            }
            Self::GetSignatureStatuses => {
                canonicalize_response::<
                    SolanaRpcResult<Vec<Option<TransactionStatus>>>,
                    Vec<Option<TransactionStatus>>,
                >(body_bytes, |statuses| {
                    statuses
                        .value
                        .into_iter()
                        .map(|maybe_status| {
                            maybe_status.map(|mut status| {
                                status.confirmations = None;
                                status
                            })
                        })
                        .collect()
                });
            }
            Self::GetSlot(rounding_error) => {
                canonicalize_response::<Slot, Slot>(body_bytes, |slot| rounding_error.round(slot));
            }
            Self::GetTransaction => {
                canonicalize_response::<Value, Option<Value>>(body_bytes, |result| match result {
                    Value::Null => None,
                    value => Some(value),
                });
            }
            Self::GetTokenAccountBalance => {
                canonicalize_response::<SolanaRpcResult<Value>, Value>(body_bytes, ignore_context);
            }
            Self::SendTransaction => {
                canonicalize_response::<String, String>(body_bytes, std::convert::identity);
            }
            Self::Raw => {
                canonicalize_response::<Value, Value>(body_bytes, std::convert::identity);
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
