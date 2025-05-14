use derive_more::From;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sol_rpc_types::{
    CommitmentLevel, DataSlice, GetAccountInfoEncoding, GetBlockCommitmentLevel,
    GetTransactionEncoding, Pubkey, SendTransactionEncoding, Signature, Slot, TransactionDetails,
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
        Self(params.pubkey.to_string(), config)
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
#[serde(into = "(String, Option<GetBalanceConfig>)")]
pub struct GetBalanceParams(String, Option<GetBalanceConfig>);

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetBalanceConfig {
    pub commitment: Option<CommitmentLevel>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
}

impl From<sol_rpc_types::GetBalanceParams> for GetBalanceParams {
    fn from(
        sol_rpc_types::GetBalanceParams {
            pubkey,
            commitment,
            min_context_slot,
        }: sol_rpc_types::GetBalanceParams,
    ) -> Self {
        let config = if commitment.is_some() || min_context_slot.is_some() {
            Some(GetBalanceConfig {
                commitment,
                min_context_slot,
            })
        } else {
            None
        };
        GetBalanceParams(pubkey.to_string(), config)
    }
}

impl From<GetBalanceParams> for (String, Option<GetBalanceConfig>) {
    fn from(value: GetBalanceParams) -> Self {
        (value.0, value.1)
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(into = "(Slot, Option<GetBlockConfig>)")]
pub struct GetBlockParams(Slot, Option<GetBlockConfig>);

impl GetBlockParams {
    pub fn get_transaction_details(&self) -> Option<&TransactionDetails> {
        self.1
            .as_ref()
            .and_then(|config| config.transaction_details.as_ref())
    }
}

impl From<sol_rpc_types::GetBlockParams> for GetBlockParams {
    fn from(params: sol_rpc_types::GetBlockParams) -> Self {
        // TODO XC-342: Check if all config fields are null, and if so, serialize it as null.
        //  Currently, we do not want it to be null since rewards=false is not the default value.
        let config = Some(GetBlockConfig {
            encoding: None,
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
#[derive(Serialize, Clone, Debug)]
#[serde(into = "(Vec<Pubkey>,)")]
pub struct GetRecentPrioritizationFeesParams(Vec<Pubkey>);

impl From<GetRecentPrioritizationFeesParams> for (Vec<Pubkey>,) {
    fn from(value: GetRecentPrioritizationFeesParams) -> Self {
        (value.0,)
    }
}

impl From<sol_rpc_types::GetRecentPrioritizationFeesParams> for GetRecentPrioritizationFeesParams {
    fn from(value: sol_rpc_types::GetRecentPrioritizationFeesParams) -> Self {
        Self(value.into())
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(into = "(Vec<Signature>, Option<GetSignatureStatusesConfig>)")]
pub struct GetSignatureStatusesParams(Vec<Signature>, Option<GetSignatureStatusesConfig>);

impl GetSignatureStatusesParams {
    pub fn num_signatures(&self) -> usize {
        self.0.len()
    }
}

impl From<sol_rpc_types::GetSignatureStatusesParams> for GetSignatureStatusesParams {
    fn from(params: sol_rpc_types::GetSignatureStatusesParams) -> Self {
        Self(
            params.signatures.into(),
            params
                .search_transaction_history
                .map(GetSignatureStatusesConfig::from),
        )
    }
}

impl From<GetSignatureStatusesParams> for (Vec<Signature>, Option<GetSignatureStatusesConfig>) {
    fn from(params: GetSignatureStatusesParams) -> Self {
        (params.0, params.1)
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, From)]
pub struct GetSignatureStatusesConfig {
    #[serde(rename = "searchTransactionHistory")]
    pub search_transaction_history: bool,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(into = "(String, Option<GetTokenAccountBalanceConfig>)")]
pub struct GetTokenAccountBalanceParams(String, Option<GetTokenAccountBalanceConfig>);

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetTokenAccountBalanceConfig {
    pub commitment: Option<CommitmentLevel>,
}

impl From<sol_rpc_types::GetTokenAccountBalanceParams> for GetTokenAccountBalanceParams {
    fn from(params: sol_rpc_types::GetTokenAccountBalanceParams) -> Self {
        Self(
            params.pubkey.to_string(),
            params
                .commitment
                .map(|commitment| GetTokenAccountBalanceConfig {
                    commitment: Some(commitment),
                }),
        )
    }
}

impl From<GetTokenAccountBalanceParams> for (String, Option<GetTokenAccountBalanceConfig>) {
    fn from(value: GetTokenAccountBalanceParams) -> Self {
        (value.0, value.1)
    }
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
        Self(params.signature.to_string(), config)
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
