use derive_more::From;
use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};
use serde_with::skip_serializing_none;
use sol_rpc_types::{
    CommitmentLevel, DataSlice, GetAccountInfoEncoding, GetBlockCommitmentLevel,
    GetSignaturesForAddressLimit, GetTransactionEncoding, Pubkey, SendTransactionEncoding,
    Signature, Slot, TransactionDetails,
};
use solana_transaction_status_client_types::UiTransactionEncoding;

#[derive(Deserialize, Clone, Debug)]
pub struct GetSlotParams(Option<GetSlotConfig>);

impl From<sol_rpc_types::GetSlotParams> for GetSlotParams {
    fn from(params: sol_rpc_types::GetSlotParams) -> Self {
        let sol_rpc_types::GetSlotParams {
            commitment,
            min_context_slot,
        } = params;
        let config = if commitment.is_none() && min_context_slot.is_none() {
            None
        } else {
            Some(GetSlotConfig {
                commitment,
                min_context_slot,
            })
        };
        Self(config)
    }
}

impl Serialize for GetSlotParams {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_optional_config(serializer, &self.0)
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetSlotConfig {
    pub commitment: Option<CommitmentLevel>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GetAccountInfoParams(String, Option<GetAccountInfoConfig>);

impl From<sol_rpc_types::GetAccountInfoParams> for GetAccountInfoParams {
    fn from(params: sol_rpc_types::GetAccountInfoParams) -> Self {
        let sol_rpc_types::GetAccountInfoParams {
            pubkey,
            commitment,
            encoding,
            data_slice,
            min_context_slot,
        } = params;
        let config = if commitment.is_none()
            && encoding.is_none()
            && data_slice.is_none()
            && min_context_slot.is_none()
        {
            None
        } else {
            Some(GetAccountInfoConfig {
                commitment,
                encoding,
                data_slice,
                min_context_slot,
            })
        };
        Self(pubkey.to_string(), config)
    }
}

impl Serialize for GetAccountInfoParams {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_params_and_optional_config(serializer, &self.0, &self.1)
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

#[derive(Deserialize, Clone, Debug)]
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

impl Serialize for GetBalanceParams {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_params_and_optional_config(serializer, &self.0, &self.1)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct GetBlockParams(Slot, Option<GetBlockConfig>);

impl GetBlockParams {
    pub fn get_transaction_details(&self) -> Option<TransactionDetails> {
        self.1
            .as_ref()
            .and_then(|config| config.transaction_details)
    }

    pub fn include_rewards(&self) -> Option<bool> {
        self.1.as_ref().and_then(|config| config.rewards)
    }
}

impl From<sol_rpc_types::GetBlockParams> for GetBlockParams {
    fn from(params: sol_rpc_types::GetBlockParams) -> Self {
        // We always use a non-null config since the default value for `transaction_details` is
        // `none` which is different from the Solana RPC API default of `full`.
        let config = Some(GetBlockConfig {
            encoding: None,
            transaction_details: Some(params.transaction_details.unwrap_or_default()),
            rewards: params.rewards,
            commitment: params.commitment,
            max_supported_transaction_version: params.max_supported_transaction_version,
        });
        Self(params.slot, config)
    }
}

impl Serialize for GetBlockParams {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_params_and_optional_config(serializer, &self.0, &self.1)
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Deserialize, Clone, Debug)]
pub struct GetSignaturesForAddressParams(Pubkey, Option<GetSignaturesForAddressConfig>);

impl GetSignaturesForAddressParams {
    pub fn get_limit(&self) -> u32 {
        self.1
            .as_ref()
            .and_then(|c| c.limit)
            .unwrap_or_default()
            .into()
    }
}

impl From<sol_rpc_types::GetSignaturesForAddressParams> for GetSignaturesForAddressParams {
    fn from(params: sol_rpc_types::GetSignaturesForAddressParams) -> Self {
        let sol_rpc_types::GetSignaturesForAddressParams {
            pubkey,
            commitment,
            min_context_slot,
            limit,
            before,
            until,
        } = params;
        let config = if commitment.is_some()
            || min_context_slot.is_some()
            || limit.is_some()
            || before.is_some()
            || until.is_some()
        {
            Some(GetSignaturesForAddressConfig {
                commitment,
                min_context_slot,
                limit,
                before,
                until,
            })
        } else {
            None
        };
        Self(pubkey, config)
    }
}

impl Serialize for GetSignaturesForAddressParams {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_params_and_optional_config(serializer, &self.0, &self.1)
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, From)]
pub struct GetSignaturesForAddressConfig {
    pub commitment: Option<CommitmentLevel>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<Slot>,
    pub limit: Option<GetSignaturesForAddressLimit>,
    pub before: Option<Signature>,
    pub until: Option<Signature>,
}

#[derive(Deserialize, Clone, Debug)]
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

impl Serialize for GetSignatureStatusesParams {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_params_and_optional_config(serializer, &self.0, &self.1)
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, From)]
pub struct GetSignatureStatusesConfig {
    #[serde(rename = "searchTransactionHistory")]
    pub search_transaction_history: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GetTokenAccountBalanceParams(String, Option<GetTokenAccountBalanceConfig>);

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

impl Serialize for GetTokenAccountBalanceParams {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_params_and_optional_config(serializer, &self.0, &self.1)
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetTokenAccountBalanceConfig {
    pub commitment: Option<CommitmentLevel>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GetTransactionParams(String, Option<GetTransactionConfig>);

impl From<sol_rpc_types::GetTransactionParams> for GetTransactionParams {
    fn from(params: sol_rpc_types::GetTransactionParams) -> Self {
        let sol_rpc_types::GetTransactionParams {
            signature,
            commitment,
            max_supported_transaction_version,
            encoding,
        } = params;
        let config = if commitment.is_none()
            && max_supported_transaction_version.is_none()
            && encoding.is_none()
        {
            None
        } else {
            Some(GetTransactionConfig {
                commitment,
                max_supported_transaction_version,
                encoding,
            })
        };
        Self(signature.to_string(), config)
    }
}

impl Serialize for GetTransactionParams {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_params_and_optional_config(serializer, &self.0, &self.1)
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

#[derive(Deserialize, Clone, Debug)]
pub struct SendTransactionParams(String, Option<SendTransactionConfig>);

impl From<sol_rpc_types::SendTransactionParams> for SendTransactionParams {
    fn from(params: sol_rpc_types::SendTransactionParams) -> Self {
        let transaction = params.get_transaction().to_string();
        let encoding = params.get_encoding().cloned();
        let sol_rpc_types::SendTransactionParams {
            skip_preflight,
            preflight_commitment,
            max_retries,
            min_context_slot,
            ..
        } = params;
        let config = if encoding.is_none()
            && skip_preflight.is_none()
            && preflight_commitment.is_none()
            && max_retries.is_none()
            && min_context_slot.is_none()
        {
            None
        } else {
            Some(SendTransactionConfig {
                encoding,
                skip_preflight,
                preflight_commitment,
                max_retries,
                min_context_slot,
            })
        };
        Self(transaction, config)
    }
}

impl Serialize for SendTransactionParams {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_params_and_optional_config(serializer, &self.0, &self.1)
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

fn serialize_optional_config<S: Serializer, Config: Serialize>(
    serializer: S,
    maybe_config: &Option<Config>,
) -> Result<S::Ok, S::Error> {
    let length = match maybe_config {
        None => 0,
        Some(_) => 1,
    };
    let mut seq = serializer.serialize_seq(Some(length))?;
    serialize_if_not_none(&mut seq, maybe_config)?;
    seq.end()
}

fn serialize_params_and_optional_config<S: Serializer, Params: Serialize, Config: Serialize>(
    serializer: S,
    params: &Params,
    maybe_config: &Option<Config>,
) -> Result<S::Ok, S::Error> {
    let length = match maybe_config {
        None => 1,
        Some(_) => 2,
    };
    let mut seq = serializer.serialize_seq(Some(length))?;
    seq.serialize_element(params)?;
    serialize_if_not_none(&mut seq, maybe_config)?;
    seq.end()
}

fn serialize_if_not_none<S: SerializeSeq, T: Serialize>(
    seq: &mut S,
    maybe_value: &Option<T>,
) -> Result<(), <S as SerializeSeq>::Error> {
    if let Some(value) = &maybe_value {
        Ok(seq.serialize_element(value)?)
    } else {
        Ok(())
    }
}
