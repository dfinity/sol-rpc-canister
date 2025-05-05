#[cfg(test)]
mod tests;

use crate::types::RoundingError;
use candid::candid_method;
use canhttp::http::json::{JsonRpcResponse, JsonRpcResult};
use ic_cdk::{
    api::management_canister::http_request::{HttpResponse, TransformArgs},
    query,
};
use minicbor::{Decode, Encode};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_slice, to_vec, Value};
use sol_rpc_types::PrioritizationFee;
use solana_clock::Slot;
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
    GetRecentPrioritizationFees {
        #[n(0)]
        max_slot_rounding_error: RoundingError,
        #[n(1)]
        max_num_slots: u8,
    },
    #[n(4)]
    GetSlot(#[n(0)] RoundingError),
    #[n(5)]
    GetTransaction,
    #[n(6)]
    SendTransaction,
    #[n(7)]
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
                if let Ok(bytes) = to_vec(&response.map(f)) {
                    *body_bytes = bytes
                }
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
                max_num_slots: num_slots,
            } => {
                assert!(
                    &1_u8 <= num_slots && num_slots <= &150_u8,
                    "BUG: expected number of slots to be between 1 and 150, but got {num_slots}"
                );
                if let Ok(response) =
                    from_slice::<JsonRpcResponse<Vec<PrioritizationFee>>>(body_bytes)
                {
                    let (id, result) = response.into_parts();
                    match result {
                        Ok(mut fees) => {
                            // The order of the prioritization fees in the response is not specified in the
                            // [API](https://solana.com/de/docs/rpc/http/getrecentprioritizationfees),
                            // although examples and manual testing show that the response is sorted by increasing number of slot.
                            // To avoid any problem, we enforce the sorting.
                            fees.sort_unstable_by_key(|fee| fee.slot);
                            // Currently, a node's prioritization-fee cache stores data from up to 150 blocks.
                            if fees.len() <= 150 {
                                *body_bytes = serde_json::to_vec(&JsonRpcResponse::from_ok( id, fees ))
                                    .expect(
                                        "BUG: failed to serialize previously deserialized JsonRpcResponse",
                                    );
                                return;
                            }
                            let max_slot = max_slot_rounding_error.round(
                                fees.last()
                                    .expect("BUG: recent prioritization fees should contain at least 150 elements")
                                    .slot,
                            );
                            let min_slot = max_slot
                                .checked_sub((num_slots - 1) as u64)
                                .expect("ERROR: ");
                            fees.retain(|fee| min_slot <= fee.slot && fee.slot <= max_slot);
                            assert_eq!(fees.len(), *num_slots as usize);

                            *body_bytes = serde_json::to_vec(&JsonRpcResponse::from_ok(id, fees))
                                .expect(
                                "BUG: failed to serialize previously deserialized JsonRpcResponse",
                            );
                        }
                        Err(json_rpc_error) => {
                            // canonicalize json representation
                            *body_bytes = serde_json::to_vec(&JsonRpcResponse::<
                                Vec<PrioritizationFee>,
                            >::from_error(
                                id, json_rpc_error
                            ))
                            .expect(
                                "BUG: failed to serialize previously deserialized JsonRpcResponse",
                            )
                        }
                    }
                }
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
