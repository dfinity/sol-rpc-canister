#[cfg(test)]
mod tests;

use candid::candid_method;
use canhttp::http::json::JsonRpcResponse;
use ic_cdk::{
    api::management_canister::http_request::{HttpResponse, TransformArgs},
    query,
};
use minicbor::{Decode, Encode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{from_slice, Value};
use sol_rpc_types::{PrioritizationFee, RoundingError};
use solana_clock::Slot;
use solana_transaction_status_client_types::TransactionStatus;
use std::fmt::Debug;
use std::num::NonZeroU8;

/// Describes a payload transformation to execute before passing the HTTP response to consensus.
/// The purpose of these transformations is to ensure that the response encoding is deterministic
/// (the field order is the same).
#[derive(Clone, Debug, Decode, Encode)]
#[cfg_attr(test, derive(strum::EnumDiscriminants))]
#[cfg_attr(test, strum_discriminants(derive(strum::EnumIter)))]
pub enum ResponseTransform {
    #[n(0)]
    GetAccountInfo,
    #[n(1)]
    GetBalance,
    #[n(2)]
    GetBlock,
    #[n(3)]
    GetRecentPrioritizationFees {
        #[cbor(n(0), with = "crate::rpc_client::cbor::rounding_error")]
        max_slot_rounding_error: RoundingError,
        #[n(1)]
        max_length: NonZeroU8,
    },
    #[n(4)]
    GetSignaturesForAddress,
    #[n(5)]
    GetSignatureStatuses,
    #[n(6)]
    GetSlot(#[cbor(n(0), with = "crate::rpc_client::cbor::rounding_error")] RoundingError),
    #[n(7)]
    GetTokenAccountBalance,
    #[n(8)]
    GetTransaction,
    #[n(9)]
    SendTransaction,
    #[n(10)]
    Raw,
}

impl ResponseTransform {
    fn apply(&self, body_bytes: &mut Vec<u8>) {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub struct SolanaRpcResult<T> {
            // This field is always ignored since it contains the fast-changing current
            // slot value for which consensus cannot generally be reached across nodes.
            context: Value,
            value: T,
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
            Self::GetRecentPrioritizationFees {
                max_slot_rounding_error,
                max_length,
            } => {
                canonicalize_response::<Vec<PrioritizationFee>, Vec<PrioritizationFee>>(
                    body_bytes,
                    |mut fees| {
                        // The exact number of elements for the returned priority fees is not really specified in the
                        // [API](https://solana.com/de/docs/rpc/http/getrecentprioritizationfees),
                        // which simply mentions
                        // "Currently, a node's prioritization-fee cache stores data from up to 150 blocks."
                        // Manual testing shows that the result seems to always contain 150 elements on mainnet (also for not used addresses)
                        // but not necessarily when using a local validator.
                        if fees.is_empty() {
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
                            .take(max_length.get() as usize)
                            .collect::<Vec<_>>()
                            .into_iter()
                            .rev()
                            .collect()
                    },
                );
            }
            Self::GetSignaturesForAddress => {
                canonicalize_response::<Value, Value>(body_bytes, std::convert::identity);
            }
            Self::GetSignatureStatuses => {
                canonicalize_response::<
                    SolanaRpcResult<Vec<Option<TransactionStatus>>>,
                    Vec<Option<TransactionStatus>>,
                >(body_bytes, |statuses| {
                    ignore_context(statuses)
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
