use serde::{Deserialize, Serialize};
use sol_rpc_types::{
    CommitmentLevel, DataSlice, GetAccountInfoEncoding, GetBlockCommitmentLevel, Slot,
};
use solana_transaction_status_client_types::{TransactionDetails, UiTransactionEncoding};

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
    pub commitment: Option<CommitmentLevel>,
    pub encoding: Option<GetAccountInfoEncoding>,
    #[serde(rename = "dataSlice")]
    pub data_slice: Option<DataSlice>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(into = "(Slot, Option<GetBlockConfig>)")]
pub struct GetBlockParams(Slot, Option<GetBlockConfig>);

impl From<sol_rpc_types::GetBlockParams> for GetBlockParams {
    fn from(params: sol_rpc_types::GetBlockParams) -> Self {
        // TODO XC-289: Check if all config fields are null, and if so, serialize it as null.
        //  Currently, we do not want it to be null since e.g. `"transaction_Details": "none"`
        //  is not the default value.
        let config = Some(GetBlockConfig {
            encoding: None,
            transaction_details: Some(TransactionDetails::None),
            rewards: Some(false),
            commitment: params.commitment,
            max_supported_transaction_version: params.max_supported_transaction_version,
        });
        Self(params.slot, config)
    }
}

impl From<GetBlockParams> for (Slot, Option<GetBlockConfig>) {
    fn from(params: GetBlockParams) -> Self {
        (params.0, params.1)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
// TODO XC-289: Use values for `rewards`, `encoding` and `transactionDetails` fields.
pub struct GetBlockConfig {
    pub encoding: Option<UiTransactionEncoding>,
    #[serde(rename = "transactionDetails")]
    pub transaction_details: Option<TransactionDetails>,
    pub rewards: Option<bool>,
    pub commitment: Option<GetBlockCommitmentLevel>,
    #[serde(rename = "maxSupportedTransactionVersion")]
    pub max_supported_transaction_version: Option<u8>,
}
