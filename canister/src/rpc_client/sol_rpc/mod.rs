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
use serde_json::{from_slice, Value};
use sol_rpc_types::PrioritizationFee;
use solana_clock::Slot;
use std::fmt::Debug;
use strum::EnumIter;

/// Describes a payload transformation to execute before passing the HTTP response to consensus.
/// The purpose of these transformations is to ensure that the response encoding is deterministic
/// (the field order is the same).
#[derive(Clone, Debug, Decode, Encode, EnumIter)]
pub enum ResponseTransform {
    #[n(0)]
    GetAccountInfo,
    #[n(1)]
    GetBalance,
    #[n(2)]
    GetBlock,
    #[n(3)]
    GetRecentPrioritizationFees {
        #[n(0)]
        max_slot_rounding_error: RoundingError,
        #[n(1)]
        max_length: u8,
    },
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
        fn canonicalize_response<T, R>(body_bytes: &mut Vec<u8>, f: impl FnOnce(T) -> R)
        where
            T: Serialize + DeserializeOwned,
            R: Serialize + DeserializeOwned,
        {
            if let Ok(response) = from_slice::<JsonRpcResponse<T>>(body_bytes) {
                if let Ok(bytes) = serde_json::to_vec(&response.map(f)) {
                    *body_bytes = bytes
                }
                // If the serialization fails, this would typically be the sign of a bug,
                // since deserialization was successfully done before calling that method.
                // However, since this code path is called in a query method as part of the HTTPs transform,
                // we prefer avoiding panicking since this would be hard to debug and could theoretically affect
                // all calls.
            }
        }

        match self {
            Self::GetAccountInfo => {
                canonicalize_response::<Value, Option<Value>>(body_bytes, |result| {
                    match result["value"].clone() {
                        Value::Null => None,
                        value => Some(value),
                    }
                });
            }
            Self::GetBalance => {
                canonicalize_response::<Value, Value>(body_bytes, |result| result["value"].clone());
            }
            Self::GetBlock => {
                canonicalize_response::<Value, Option<Value>>(body_bytes, |result| match result {
                    Value::Null => None,
                    value => Some(value),
                });
            }
            Self::GetRecentPrioritizationFees {
                max_slot_rounding_error,
                max_length,
            } => {
                canonicalize_response::<Vec<PrioritizationFee>, Vec<PrioritizationFee>>(
                    body_bytes,
                    |mut fees| {
                        // actual processing here
                        // The exact number of elements for the returned priority fees is not really specified in the
                        // [API](https://solana.com/de/docs/rpc/http/getrecentprioritizationfees),
                        // which simply mentions
                        // "Currently, a node's prioritization-fee cache stores data from up to 150 blocks."
                        // Manual testing shows that the result seems to always contain 150 elements on mainnet (also for not used addresses)
                        // but not necessarily when using a local validator.
                        if fees.is_empty() || max_length == &0 {
                            return Vec::default();
                        }
                        // The order of the prioritization fees in the response is not specified in the
                        // [API](https://solana.com/de/docs/rpc/http/getrecentprioritizationfees),
                        // although examples and manual testing show that the response is sorted by increasing number of slot.
                        // To avoid any problem, we enforce the sorting.
                        fees.sort_unstable_by(|fee, other_fee| {
                            other_fee.slot.cmp(&fee.slot) //sort by decreasing order of slot
                        });
                        let max_rounded_slot = max_slot_rounding_error.round(
                            fees.first()
                                .expect("BUG: recent prioritization fees should be non-empty")
                                .slot,
                        );

                        fees.into_iter()
                            .skip_while(|fee| fee.slot > max_rounded_slot)
                            .take(*max_length as usize)
                            .collect::<Vec<_>>()
                            .into_iter()
                            .rev()
                            .collect()
                    },
                );
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
                canonicalize_response::<Value, Value>(body_bytes, |result| result["value"].clone());
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
