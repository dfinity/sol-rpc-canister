use serde::{Deserialize, Serialize};
use sol_rpc_types::{CommitmentLevel, DataSlice, GetAccountInfoEncoding, SendTransactionEncoding};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(into = "(String, Option<SendTransactionConfig>)")]
pub struct SendTransactionParams(String, Option<SendTransactionConfig>);

impl From<sol_rpc_types::SendTransactionParams> for SendTransactionParams {
    fn from(params: sol_rpc_types::SendTransactionParams) -> Self {
        let config = if params.is_default_config() {
            None
        } else {
            Some(SendTransactionConfig {
                encoding: params.encoding,
                skip_preflight: params.skip_preflight,
                preflight_commitment: params.preflight_commitment,
                max_retries: params.max_retries,
                min_context_slot: params.min_context_slot,
            })
        };
        Self(params.transaction, config)
    }
}

impl From<SendTransactionParams> for (String, Option<SendTransactionConfig>) {
    fn from(params: SendTransactionParams) -> Self {
        (params.0.to_string(), params.1)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SendTransactionConfig {
    /// Encoding format for the transaction.
    pub encoding: Option<SendTransactionEncoding>,
    /// When true, skip the preflight transaction checks. Default: false.
    #[serde(rename = "skipPreflight")]
    pub skip_preflight: Option<bool>,
    /// Commitment level to use for preflight. See Configuring State Commitment. Default finalized.
    #[serde(rename = "preflightCommitment")]
    pub preflight_commitment: Option<String>,
    /// Maximum number of times for the RPC node to retry sending the transaction to the leader.
    /// If this parameter not provided, the RPC node will retry the transaction until it is
    /// finalized or until the blockhash expires.
    #[serde(rename = "maxRetries")]
    pub max_retries: Option<u32>,
    /// Set the minimum slot at which to perform preflight transaction checks
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
}
