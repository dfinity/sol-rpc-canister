#[cfg(test)]
mod tests;

use crate::{solana::Pubkey, RpcError, Signature, Slot, VecWithMaxLen};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use candid::{CandidType, Deserialize};
use serde::Serialize;

/// The parameters for a Solana [`getAccountInfo`](https://solana.com/docs/rpc/http/getaccountinfo) RPC method call.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
pub struct GetAccountInfoParams {
    /// The public key of the account whose info to fetch formatted as a base-58 string.
    pub pubkey: Pubkey,
    /// The commitment describes how finalized a block is at that point in time.
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

impl GetAccountInfoParams {
    /// Returns `true` if all of the optional config parameters are `None` and `false` otherwise.
    pub fn is_default_config(&self) -> bool {
        let GetAccountInfoParams {
            pubkey: _,
            commitment,
            encoding,
            data_slice,
            min_context_slot,
        } = &self;
        commitment.is_none()
            && encoding.is_none()
            && data_slice.is_none()
            && min_context_slot.is_none()
    }

    /// Parameters for a `getAccountInfo` request with the given pubkey.
    pub fn from_pubkey<P: Into<Pubkey>>(pubkey: P) -> Self {
        Self {
            pubkey: pubkey.into(),
            commitment: None,
            encoding: None,
            data_slice: None,
            min_context_slot: None,
        }
    }
}

impl From<solana_pubkey::Pubkey> for GetAccountInfoParams {
    fn from(pubkey: solana_pubkey::Pubkey) -> Self {
        Self::from_pubkey(pubkey)
    }
}

/// Encoding for the return value of the Solana [`getAccountInfo`](https://solana.com/docs/rpc/http/getaccountinfo) RPC method.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
pub enum GetAccountInfoEncoding {
    /// The account data is base-58 encoded. Limited to less than 129 bytes of data.
    #[serde(rename = "base58")]
    Base58,
    /// The account data is base-64 encoded.
    #[serde(rename = "base64")]
    Base64,
    /// Account data is first compressed using [Zstandard](http://facebook.github.io/zstd/) and the
    /// result is then base-64 encoded.
    #[serde(rename = "base64+zstd")]
    Base64ZStd,
    /// The encoding attempts to use program-specific state parsers to return more human-readable
    /// and explicit account state data. If [`GetAccountInfoEncoding::JsonParsed`] is requested but
    /// a parser cannot be found, the fallback is [`GetAccountInfoEncoding::Base64`] encoding.
    #[serde(rename = "jsonParsed")]
    JsonParsed,
}

/// Represents a slice of the return value of the Solana [`getAccountInfo`](https://solana.com/docs/rpc/http/getAccountInfo) RPC method.
#[derive(Debug, Clone, Default, Deserialize, Serialize, CandidType)]
pub struct DataSlice {
    /// Number of bytes to return.
    pub length: u32,
    /// Byte offset from which to start reading.
    pub offset: u32,
}

/// The parameters for a Solana [`getBalance`](https://solana.com/docs/rpc/http/getbalance) RPC method call.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
pub struct GetBalanceParams {
    /// The public key of the account to query formatted as a base-58 string.
    pub pubkey: Pubkey,
    /// The request returns the slot that has reached this or the default commitment level.
    pub commitment: Option<CommitmentLevel>,
    /// The minimum slot that the request can be evaluated at.
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
}

impl GetBalanceParams {
    /// Parameters for a `getBalance` request with the given pubkey.
    pub fn from_pubkey<P: Into<Pubkey>>(pubkey: P) -> Self {
        Self {
            pubkey: pubkey.into(),
            commitment: None,
            min_context_slot: None,
        }
    }
}

impl From<solana_pubkey::Pubkey> for GetBalanceParams {
    fn from(pubkey: solana_pubkey::Pubkey) -> Self {
        Self::from_pubkey(pubkey)
    }
}

/// The parameters for a Solana [`getBlock`](https://solana.com/docs/rpc/http/getblock) RPC method call.
// TODO XC-342: Add `rewards`, `encoding` and `transactionDetails` fields.
#[derive(Debug, Clone, Default, Deserialize, Serialize, CandidType)]
pub struct GetBlockParams {
    /// Slot number of the block to fetch.
    pub slot: Slot,
    /// The commitment describes how finalized a block is at that point in time.
    pub commitment: Option<GetBlockCommitmentLevel>,
    /// The max transaction version to return in responses.
    /// * If the requested block contains a transaction with a higher version,
    ///   an error will be returned.
    /// * If this parameter is omitted, only legacy transactions will be returned, and a block
    ///   containing any versioned transaction will prompt the error.
    #[serde(rename = "maxSupportedTransactionVersion")]
    pub max_supported_transaction_version: Option<u8>,
    /// Specifies what transaction details to include in the response.
    ///
    /// *Warning:* If this value is not specified, the default value of [`TransactionDetails::None`]
    /// will be used, which is different from the default value in the Solana RPC API. This is
    /// because the default value of `full` for the Solana RPC API results in response sizes that
    /// are generally too large to be supported by the ICP.
    #[serde(rename = "transactionDetails")]
    pub transaction_details: Option<TransactionDetails>,
}

impl GetBlockParams {
    /// Returns `true` if all of the optional config parameters are `None` and `false` otherwise.
    pub fn is_default_config(&self) -> bool {
        let GetBlockParams {
            slot: _,
            commitment,
            max_supported_transaction_version,
            transaction_details,
        } = &self;
        commitment.is_none()
            && max_supported_transaction_version.is_none()
            && transaction_details.is_none()
    }
}

impl From<Slot> for GetBlockParams {
    fn from(slot: Slot) -> Self {
        Self {
            slot,
            commitment: None,
            max_supported_transaction_version: None,
            transaction_details: None,
        }
    }
}

/// Determines whether and how transactions are included in `getBlock` response.
///
/// *Warning:* If this value is not specified, the default value of [`TransactionDetails::None`]
/// will be used, which is different from the default value in the Solana RPC API. This is
/// because the default value of `full` for the Solana RPC API results in response sizes that
/// are generally too large to be supported by the ICP.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, CandidType)]
pub enum TransactionDetails {
    /// Includes transaction signatures (IDs) and block metadata only.
    #[serde(rename = "signatures")]
    Signatures,
    /// Omits all transaction data and signatures; returns only block metadata.
    #[default]
    #[serde(rename = "none")]
    None,
}

/// The parameters for a Solana [`getRecentPrioritizationFees`](https://solana.com/de/docs/rpc/http/getrecentprioritizationfees) RPC method call.
#[derive(Debug, Clone, Default, Deserialize, Serialize, CandidType)]
pub struct GetRecentPrioritizationFeesParams(VecWithMaxLen<Pubkey, 128>);

impl<P: Into<Pubkey>> TryFrom<Vec<P>> for GetRecentPrioritizationFeesParams {
    type Error = RpcError;

    fn try_from(pubkeys: Vec<P>) -> Result<Self, Self::Error> {
        Ok(Self(
            pubkeys
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .try_into()?,
        ))
    }
}

impl From<solana_pubkey::Pubkey> for GetRecentPrioritizationFeesParams {
    fn from(value: solana_pubkey::Pubkey) -> Self {
        Self(VecWithMaxLen::try_from(vec![Pubkey::from(value)]).unwrap())
    }
}

impl From<GetRecentPrioritizationFeesParams> for Vec<Pubkey> {
    fn from(value: GetRecentPrioritizationFeesParams) -> Self {
        value.0.into()
    }
}

/// The parameters for a Solana [`getSignatureStatuses`](https://solana.com/docs/rpc/http/getsignaturestatuses) RPC method call.
#[derive(Debug, Clone, Default, Deserialize, Serialize, CandidType)]
pub struct GetSignatureStatusesParams {
    /// An array of transaction signatures to confirm, as base-58 encoded strings (up to a maximum of 256)
    pub signatures: VecWithMaxLen<Signature, 256>,
    /// If set to true, a Solana node will search its ledger cache for any signatures not found in the recent status cache.
    #[serde(rename = "searchTransactionHistory")]
    pub search_transaction_history: Option<bool>,
}

impl<S: Into<Signature>> TryFrom<Vec<S>> for GetSignatureStatusesParams {
    type Error = RpcError;

    fn try_from(signatures: Vec<S>) -> Result<Self, Self::Error> {
        Ok(Self {
            signatures: signatures
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .try_into()?,
            search_transaction_history: None,
        })
    }
}

/// The parameters for a Solana [`getSlot`](https://solana.com/docs/rpc/http/getslot) RPC method call.
#[derive(Debug, Clone, Default, Deserialize, Serialize, CandidType)]
pub struct GetSlotParams {
    /// The request returns the slot that has reached this or the default commitment level.
    pub commitment: Option<CommitmentLevel>,
    /// The minimum slot that the request can be evaluated at.
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
}

impl GetSlotParams {
    /// Returns `true` if all of the optional config parameters are `None` and `false` otherwise.
    pub fn is_default_config(&self) -> bool {
        let GetSlotParams {
            commitment,
            min_context_slot,
        } = &self;
        commitment.is_none() && min_context_slot.is_none()
    }
}

/// The parameters for a Solana [`getTokenAccountBalance`](https://solana.com/docs/rpc/http/gettokenaccountbalance) RPC method call.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
pub struct GetTokenAccountBalanceParams {
    /// The public key of the token account to query formatted as a base-58 string.
    pub pubkey: Pubkey,
    /// The commitment describes how finalized a block is at that point in time.
    pub commitment: Option<CommitmentLevel>,
}

impl GetTokenAccountBalanceParams {
    /// Parameters for a `getTokenAccountBalance` request with the given pubkey.
    pub fn from_pubkey<P: Into<Pubkey>>(pubkey: P) -> Self {
        Self {
            pubkey: pubkey.into(),
            commitment: None,
        }
    }
}

impl From<solana_pubkey::Pubkey> for GetTokenAccountBalanceParams {
    fn from(pubkey: solana_pubkey::Pubkey) -> Self {
        Self::from_pubkey(pubkey)
    }
}

/// The parameters for a Solana [`getTransaction`](https://solana.com/docs/rpc/http/gettransaction) RPC method call.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
pub struct GetTransactionParams {
    /// Transaction signature.
    pub signature: Signature,
    /// Specifies the required finality of the transaction being queried.
    pub commitment: Option<CommitmentLevel>,
    /// Set the max transaction version to return in responses.
    ///
    /// If the requested transaction is a higher version, an error will be returned. If this
    /// parameter is omitted, only legacy transactions will be returned, and any versioned
    /// transaction will prompt the error.
    #[serde(rename = "maxSupportedTransactionVersion")]
    pub max_supported_transaction_version: Option<u8>,
    /// Encoding for the returned transaction
    // TODO XC-343: Add notes about `jsonParsed` from https://solana.com/de/docs/rpc/http/gettransaction
    pub encoding: Option<GetTransactionEncoding>,
}

impl GetTransactionParams {
    /// Returns `true` if all of the optional config parameters are `None` and `false` otherwise.
    pub fn is_default_config(&self) -> bool {
        let GetTransactionParams {
            signature: _,
            commitment,
            max_supported_transaction_version,
            encoding,
        } = &self;
        commitment.is_none() && max_supported_transaction_version.is_none() && encoding.is_none()
    }
}

impl From<solana_signature::Signature> for GetTransactionParams {
    fn from(signature: solana_signature::Signature) -> Self {
        Self {
            signature: signature.into(),
            commitment: None,
            max_supported_transaction_version: None,
            encoding: None,
        }
    }
}

/// Encoding format for the returned transaction from a [`getTransaction`](https://solana.com/docs/rpc/http/gettransaction)`
/// RPC method call.
// TODO XC-343: Add support for `json` and `jsonParsed` formats.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
pub enum GetTransactionEncoding {
    /// The transaction is base64-encoded.
    #[serde(rename = "base64")]
    Base64,
    /// The transaction is base58-encoded.
    #[serde(rename = "base58")]
    Base58,
}

/// The parameters for a Solana [`sendTransaction`](https://solana.com/docs/rpc/http/sendtransaction) RPC method call.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
pub struct SendTransactionParams {
    /// Fully-signed transaction, as encoded string.
    transaction: String,
    /// Encoding format for the transaction.
    encoding: Option<SendTransactionEncoding>,
    /// When true, skip the preflight transaction checks. Default: false.
    #[serde(rename = "skipPreflight")]
    pub skip_preflight: Option<bool>,
    /// Commitment level to use for preflight. See Configuring State Commitment. Default finalized.
    #[serde(rename = "preflightCommitment")]
    pub preflight_commitment: Option<CommitmentLevel>,
    /// Maximum number of times for the RPC node to retry sending the transaction to the leader.
    /// If this parameter not provided, the RPC node will retry the transaction until it is
    /// finalized or until the blockhash expires.
    #[serde(rename = "maxRetries")]
    pub max_retries: Option<u32>,
    /// Set the minimum slot at which to perform preflight transaction checks
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
}

impl SendTransactionParams {
    /// Parameters for a `sendTransaction` request with the given transaction already encoded with
    /// the given encoding.
    pub fn from_encoded_transaction(
        transaction: String,
        encoding: SendTransactionEncoding,
    ) -> Self {
        Self {
            transaction,
            encoding: Some(encoding),
            skip_preflight: None,
            preflight_commitment: None,
            max_retries: None,
            min_context_slot: None,
        }
    }

    /// Returns `true` if all of the optional config parameters are `None` and `false` otherwise.
    pub fn is_default_config(&self) -> bool {
        let SendTransactionParams {
            transaction: _,
            encoding,
            skip_preflight,
            preflight_commitment,
            max_retries,
            min_context_slot,
        } = &self;
        encoding.is_none()
            && skip_preflight.is_none()
            && preflight_commitment.is_none()
            && max_retries.is_none()
            && min_context_slot.is_none()
    }

    /// The transaction being sent as an encoded string.
    pub fn get_transaction(&self) -> &str {
        &self.transaction
    }

    /// The encoding format for the transaction in the `sendTransaction` request.
    pub fn get_encoding(&self) -> Option<&SendTransactionEncoding> {
        self.encoding.as_ref()
    }
}

impl TryFrom<solana_transaction::Transaction> for SendTransactionParams {
    type Error = RpcError;

    fn try_from(transaction: solana_transaction::Transaction) -> Result<Self, RpcError> {
        let serialized = bincode::serialize(&transaction).map_err(|e| {
            RpcError::ValidationError(format!("Transaction serialization failed: {e}"))
        })?;
        Ok(Self::from_encoded_transaction(
            BASE64_STANDARD.encode(serialized),
            SendTransactionEncoding::Base64,
        ))
    }
}

/// The encoding format for the transaction argument to the Solana
/// [`sendTransaction`](https://solana.com/docs/rpc/http/sendtransaction) RPC method call.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
pub enum SendTransactionEncoding {
    /// The transaction is base-58 encoded (slow, deprecated).
    #[serde(rename = "base58")]
    Base58,
    /// The transaction is base-64 encoded.
    #[serde(rename = "base64")]
    Base64,
}

/// [Commitment levels](https://solana.com/docs/rpc#configuring-state-commitment) in Solana,
/// representing finality guarantees of transactions and memory queries.
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize, CandidType)]
pub enum CommitmentLevel {
    /// The transaction is processed by a leader, but may be dropped.
    #[serde(rename = "processed")]
    Processed,
    /// The transaction has been included in a block that has reached 1 confirmation.
    #[serde(rename = "confirmed")]
    Confirmed,
    /// The transaction is finalized and cannot be rolled back.
    #[serde(rename = "finalized")]
    #[default]
    Finalized,
}

impl From<CommitmentLevel> for solana_commitment_config::CommitmentConfig {
    fn from(commitment_level: CommitmentLevel) -> Self {
        match commitment_level {
            CommitmentLevel::Processed => Self::processed(),
            CommitmentLevel::Confirmed => Self::confirmed(),
            CommitmentLevel::Finalized => Self::finalized(),
        }
    }
}

/// Subset of [`CommitmentLevel`] whose variants are allowed values for the `encoding`
/// field of [`GetBlockParams`].
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, CandidType)]
pub enum GetBlockCommitmentLevel {
    /// See [`CommitmentLevel::Confirmed`].
    #[serde(rename = "confirmed")]
    Confirmed,
    /// See [`CommitmentLevel::Finalized`].
    #[serde(rename = "finalized")]
    Finalized,
}

impl From<GetBlockCommitmentLevel> for solana_commitment_config::CommitmentConfig {
    fn from(commitment_level: GetBlockCommitmentLevel) -> Self {
        match commitment_level {
            GetBlockCommitmentLevel::Confirmed => Self::confirmed(),
            GetBlockCommitmentLevel::Finalized => Self::finalized(),
        }
    }
}
