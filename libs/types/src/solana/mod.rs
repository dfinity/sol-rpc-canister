use serde::{Deserialize, Serialize};

/// The parameters for a Solana [`getSlot`](https://solana.com/docs/rpc/http/getslot) RPC method call.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GetSlotParams {
    /// The request returns the slot that has reached this or the default commitment level.
    commitment: Option<String>,
    /// The minimum slot that the request can be evaluated at.
    #[serde(rename = "minContextSlot")]
    min_context_slot: Option<u64>,
}
