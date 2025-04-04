use serde::{Deserialize, Serialize};
use sol_rpc_types::{CommitmentLevel, DataSlice, GetAccountInfoEncoding};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetAccountInfoConfig {
    /// The request returns the slot that has reached this or the default commitment level.
    pub commitment: Option<CommitmentLevel>,
    /// Encoding format for Account data.
    pub encoding: Option<GetAccountInfoEncoding>,
    /// Request a slice of the account's data.
    #[serde(rename = "dataSlice")]
    pub data_slice: Option<DataSlice>,
    /// The minimum slot that the request can be evaluated at.
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
}