use candid::CandidType;
use serde::{Deserialize, Serialize};

/// A Solana [slot](https://solana.com/docs/references/terminology#slot): the period of time for
/// which each leader ingests transactions and produces a block.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct Slot(pub u64);

/// The parameters for a Solana [`getSlot`](https://solana.com/docs/rpc/http/getslot) RPC method call.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GetSlotParams {
    /// The request returns the slot that has reached this or the default commitment level.
    commitment: Option<String>,
    /// The minimum slot that the request can be evaluated at.
    #[serde(rename = "minContextSlot")]
    min_context_slot: Option<u64>,
}
