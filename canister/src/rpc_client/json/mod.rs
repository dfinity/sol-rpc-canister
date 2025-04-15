use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sol_rpc_types::{
    CommitmentLevel, DataSlice, GetAccountInfoEncoding, GetBlockCommitmentLevel,
    GetTransactionEncoding, SendTransactionEncoding, Slot, TransactionDetails,
};
use solana_transaction_status_client_types::UiTransactionEncoding;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(into = "(Option<GetSlotConfig>,)")]
pub struct GetSlotParams(Option<GetSlotConfig>);

impl From<sol_rpc_types::GetSlotParams> for GetSlotParams {
    fn from(params: sol_rpc_types::GetSlotParams) -> Self {
        let config = if params.is_default_config() {
            None
        } else {
            Some(GetSlotConfig {
                commitment: params.commitment,
                min_context_slot: params.min_context_slot,
            })
        };
        Self(config)
    }
}

impl From<GetSlotParams> for (Option<GetSlotConfig>,) {
    fn from(params: GetSlotParams) -> Self {
        (params.0,)
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetSlotConfig {
    pub commitment: Option<CommitmentLevel>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
}

#[skip_serializing_none]
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

#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GetAccountInfoConfig {
    pub commitment: Option<CommitmentLevel>,
    pub encoding: Option<GetAccountInfoEncoding>,
    #[serde(rename = "dataSlice")]
    pub data_slice: Option<DataSlice>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(into = "(Slot, Option<GetBlockConfig>)")]
pub struct GetBlockParams(Slot, Option<GetBlockConfig>);

impl From<sol_rpc_types::GetBlockParams> for GetBlockParams {
    fn from(params: sol_rpc_types::GetBlockParams) -> Self {
        // TODO XC-342: Check if all config fields are null, and if so, serialize it as null.
        //  Currently, we do not want it to be null since rewards=false is not the default value.
        let config = Some(GetBlockConfig {
            encoding: None,
            // Always specify since the default value is `full` which we do not support yet
            transaction_details: Some(params.transaction_details.unwrap_or_default()),
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
// TODO XC-342: Use values for `rewards`, `encoding` and `transactionDetails` fields.
pub struct GetBlockConfig {
    pub encoding: Option<UiTransactionEncoding>,
    #[serde(rename = "transactionDetails")]
    pub transaction_details: Option<TransactionDetails>,
    pub rewards: Option<bool>,
    pub commitment: Option<GetBlockCommitmentLevel>,
    #[serde(rename = "maxSupportedTransactionVersion")]
    pub max_supported_transaction_version: Option<u8>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(into = "(String, Option<GetTransactionConfig>)")]
pub struct GetTransactionParams(String, Option<GetTransactionConfig>);

impl From<sol_rpc_types::GetTransactionParams> for GetTransactionParams {
    fn from(params: sol_rpc_types::GetTransactionParams) -> Self {
        let config = if params.is_default_config() {
            None
        } else {
            Some(GetTransactionConfig {
                commitment: params.commitment,
                max_supported_transaction_version: params.max_supported_transaction_version,
                encoding: params.encoding,
            })
        };
        Self(params.signature, config)
    }
}

impl From<GetTransactionParams> for (String, Option<GetTransactionConfig>) {
    fn from(params: GetTransactionParams) -> Self {
        (params.0.to_string(), params.1)
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetTransactionConfig {
    pub commitment: Option<CommitmentLevel>,
    #[serde(rename = "maxSupportedTransactionVersion")]
    pub max_supported_transaction_version: Option<u8>,
    pub encoding: Option<GetTransactionEncoding>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(into = "(String, Option<SendTransactionConfig>)")]
pub struct SendTransactionParams(String, Option<SendTransactionConfig>);

impl From<sol_rpc_types::SendTransactionParams> for SendTransactionParams {
    fn from(params: sol_rpc_types::SendTransactionParams) -> Self {
        let transaction = params.get_transaction().to_string();
        let config = if params.is_default_config() {
            None
        } else {
            Some(SendTransactionConfig {
                encoding: params.get_encoding().cloned(),
                skip_preflight: params.skip_preflight,
                preflight_commitment: params.preflight_commitment,
                max_retries: params.max_retries,
                min_context_slot: params.min_context_slot,
            })
        };
        Self(transaction, config)
    }
}

impl From<SendTransactionParams> for (String, Option<SendTransactionConfig>) {
    fn from(params: SendTransactionParams) -> Self {
        (params.0.to_string(), params.1)
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SendTransactionConfig {
    pub encoding: Option<SendTransactionEncoding>,
    #[serde(rename = "skipPreflight")]
    pub skip_preflight: Option<bool>,
    #[serde(rename = "preflightCommitment")]
    pub preflight_commitment: Option<CommitmentLevel>,
    #[serde(rename = "maxRetries")]
    pub max_retries: Option<u32>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
}
