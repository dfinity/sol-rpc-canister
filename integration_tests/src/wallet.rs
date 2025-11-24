//! Module to interact with a [cycles wallet](https://github.com/dfinity/cycles-wallet) canister.

use crate::{decode_call_response, encode_args};
use candid::{utils::ArgumentEncoder, CandidType, Principal};
use ic_canister_runtime::IcError;
use ic_error_types::RejectCode;
use ic_management_canister_types_pocket_ic::CanisterId;
use regex::Regex;
use serde::{de::DeserializeOwned, Deserialize};

/// Argument to the cycles wallet canister `wallet_call128` method.
#[derive(CandidType, Deserialize)]
pub struct CallCanisterArgs {
    canister: Principal,
    method_name: String,
    #[serde(with = "serde_bytes")]
    args: Vec<u8>,
    cycles: u128,
}

impl CallCanisterArgs {
    pub fn new<In: ArgumentEncoder>(
        canister_id: CanisterId,
        method: impl ToString,
        args: In,
        cycles: u128,
    ) -> Self {
        Self {
            canister: canister_id,
            method_name: method.to_string(),
            args: encode_args(args),
            cycles,
        }
    }
}

/// Return type of the cycles wallet canister `wallet_call128` method.
#[derive(CandidType, Deserialize)]
pub struct CallResult {
    #[serde(with = "serde_bytes", rename = "return")]
    pub bytes: Vec<u8>,
}

/// The cycles wallet canister formats the rejection code and error message from the target
/// canister into a single string. Extract them back from the formatted string.
pub fn decode_cycles_wallet_response<Out>(response: Vec<u8>) -> Result<Out, IcError>
where
    Out: CandidType + DeserializeOwned,
{
    match decode_call_response::<Result<CallResult, String>>(response)? {
        Ok(CallResult { bytes }) => decode_call_response(bytes),
        Err(message) => {
            match Regex::new(r"^An error happened during the call: (\d+): (.*)$")
                .unwrap()
                .captures(&message)
            {
                Some(captures) => {
                    let (_, [code, message]) = captures.extract();
                    Err(IcError::CallRejected {
                        code: code.parse::<u64>().unwrap().try_into().unwrap(),
                        message: message.to_string(),
                    })
                }
                None => Err(IcError::CallRejected {
                    code: RejectCode::SysFatal,
                    message,
                }),
            }
        }
    }
}
