use serde::{Deserialize, Serialize};
use sol_rpc_types::{CommitmentLevel, DataSlice, GetAccountInfoEncoding};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(into = "(String, Option<GetAccountInfoConfig>)")]
pub struct GetAccountInfoParams(String, Option<GetAccountInfoConfig>);

impl From<sol_rpc_types::GetAccountInfoParams> for GetAccountInfoParams {
    fn from(params: sol_rpc_types::GetAccountInfoParams) -> Self {
        let config = if params.is_default_config() {
            None
        } else {
            Some(GetAccountInfoConfig {
                commitment: params.commitment,
                encoding: params.encoding,
                data_slice: params.data_slice,
                min_context_slot: params.min_context_slot,
            })
        };
        Self(params.pubkey, config)
    }
}

impl From<GetAccountInfoParams> for (String, Option<GetAccountInfoConfig>) {
    fn from(params: GetAccountInfoParams) -> Self {
        (params.0.to_string(), params.1)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
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
