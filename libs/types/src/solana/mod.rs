use base64::{prelude::BASE64_STANDARD, Engine};
use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A Solana [slot](https://solana.com/docs/references/terminology#slot).
pub type Slot = u64;

/// A Solana base58-encoded [transaction ID](https://solana.com/docs/references/terminology#transaction-id).
pub type TransactionId = String;

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
    /// Parameters for a `sendTransaction` request with the given transaction already encoded wit
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
    pub fn get_transaction(&self) -> String {
        self.transaction.clone()
    }

    /// The encoding format for the transaction in the `sendTransaction` request.
    pub fn get_encoding(&self) -> Option<SendTransactionEncoding> {
        self.encoding.clone()
    }
}

impl From<solana_transaction::Transaction> for SendTransactionParams {
    fn from(transaction: solana_transaction::Transaction) -> Self {
        Self::from_encoded_transaction(
            BASE64_STANDARD
                .encode(bincode::serialize(&transaction).expect("Failed to serialize transaction")),
            SendTransactionEncoding::Base64,
        )
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

/// The parameters for a Solana [`getSlot`](https://solana.com/docs/rpc/http/getslot) RPC method call.
#[derive(Debug, Clone, Default, Deserialize, Serialize, CandidType)]
pub struct GetSlotParams {
    /// The request returns the slot that has reached this or the default commitment level.
    pub commitment: Option<CommitmentLevel>,
    /// The minimum slot that the request can be evaluated at.
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
}

/// The parameters for a Solana [`getAccountInfo`](https://solana.com/docs/rpc/http/getaccountinfo) RPC method call.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
pub struct GetAccountInfoParams {
    /// The public key of the account whose info to fetch formatted as a base-58 string.
    pub pubkey: String,
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
}

impl From<solana_pubkey::Pubkey> for GetAccountInfoParams {
    fn from(pubkey: solana_pubkey::Pubkey) -> Self {
        Self {
            pubkey: pubkey.to_string(),
            commitment: None,
            encoding: None,
            data_slice: None,
            min_context_slot: None,
        }
    }
}

/// [Commitment levels](https://solana.com/docs/rpc#configuring-state-commitment) in Solana,
/// representing finality guarantees of transactions and memory queries.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
pub enum CommitmentLevel {
    /// The transaction is processed by a leader, but may be dropped.
    #[serde(rename = "processed")]
    Processed,
    /// The transaction has been included in a block that has reached 1 confirmation.
    #[serde(rename = "confirmed")]
    Confirmed,
    /// The transaction is finalized and cannot be rolled back.
    #[serde(rename = "finalized")]
    Finalized,
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
    length: u32,
    /// Byte offset from which to start reading.
    offset: u32,
}

/// Solana Ed25519 [public key](`https://solana.com/docs/references/terminology#public-key-pubkey`).
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct Pubkey(pub [u8; 32]);

impl From<solana_pubkey::Pubkey> for Pubkey {
    fn from(pubkey: solana_pubkey::Pubkey) -> Self {
        Pubkey(pubkey.to_bytes())
    }
}

impl From<Pubkey> for solana_pubkey::Pubkey {
    fn from(pubkey: Pubkey) -> Self {
        solana_pubkey::Pubkey::from(pubkey.0)
    }
}

/// Solana [account](https://solana.com/docs/references/terminology#account) information.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct AccountInfo {
    /// Number of lamports assigned to this account.
    pub lamports: u64,
    /// Data associated with the account.
    pub data: AccountData,
    /// base-58 encoded Pubkey of the program this account has been assigned to.
    pub owner: String,
    /// Boolean indicating if the account contains a program (and is strictly read-only).
    pub executable: bool,
    /// The epoch at which this account will next owe rent.
    #[serde(rename = "rentEpoch")]
    pub rent_epoch: u64,
    /// The data size of the account.
    pub space: u64,
}

impl From<solana_account_decoder_client_types::UiAccount> for AccountInfo {
    fn from(account: solana_account_decoder_client_types::UiAccount) -> Self {
        AccountInfo {
            lamports: account.lamports,
            data: account.data.into(),
            owner: account.owner,
            executable: account.executable,
            rent_epoch: account.rent_epoch,
            // The `space` field is optional for backwards compatibility reasons, however it should
            // always contain a value.
            space: account.space.expect("'space' field should not be null"),
        }
    }
}

impl From<AccountInfo> for solana_account_decoder_client_types::UiAccount {
    fn from(account: AccountInfo) -> Self {
        solana_account_decoder_client_types::UiAccount {
            lamports: account.lamports,
            data: account.data.into(),
            owner: account.owner,
            executable: account.executable,
            rent_epoch: account.rent_epoch,
            space: Some(account.space),
        }
    }
}

/// Represents the data stored in a Solana [account](https://solana.com/docs/references/terminology#account).
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum AccountData {
    /// The data is formatted as a binary string. This is a legacy format retained for RPC backwards compatibility
    #[serde(rename = "legacyBinary")]
    LegacyBinary(String),
    /// The data is formatted as a JSON [`ParsedAccount`].
    #[serde(rename = "json")]
    Json(ParsedAccount),
    /// The data is formatted as a string containing the account data encoded according to one of
    /// the [`AccountEncoding`] formats.
    #[serde(rename = "binary")]
    Binary(String, AccountEncoding),
}

impl From<solana_account_decoder_client_types::UiAccountData> for AccountData {
    fn from(data: solana_account_decoder_client_types::UiAccountData) -> Self {
        use solana_account_decoder_client_types::UiAccountData;
        match data {
            UiAccountData::LegacyBinary(value) => Self::LegacyBinary(value),
            UiAccountData::Json(value) => Self::Json(value.into()),
            UiAccountData::Binary(value, encoding) => Self::Binary(value, encoding.into()),
        }
    }
}

impl From<AccountData> for solana_account_decoder_client_types::UiAccountData {
    fn from(data: AccountData) -> Self {
        use solana_account_decoder_client_types::UiAccountData;
        match data {
            AccountData::LegacyBinary(value) => UiAccountData::LegacyBinary(value),
            AccountData::Json(value) => UiAccountData::Json(value.into()),
            AccountData::Binary(value, encoding) => UiAccountData::Binary(value, encoding.into()),
        }
    }
}

/// Represents parsed Solana [account](https://solana.com/docs/references/terminology#account) data.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct ParsedAccount {
    /// The Solana [program](https://solana.com/docs/references/terminology#program) that interprets the data.
    pub program: String,
    /// The account data parsed a JSON and formatted as a string.
    pub parsed: String,
    /// The data size of the account.
    pub space: u64,
}

impl From<solana_account_decoder_client_types::ParsedAccount> for ParsedAccount {
    fn from(account: solana_account_decoder_client_types::ParsedAccount) -> Self {
        Self {
            program: account.program,
            parsed: serde_json::to_string(&account.parsed)
                .expect("Unable to convert JSON to string"),
            space: account.space,
        }
    }
}

impl From<ParsedAccount> for solana_account_decoder_client_types::ParsedAccount {
    fn from(account: ParsedAccount) -> Self {
        Self {
            program: account.program,
            parsed: serde_json::from_str(&account.parsed).expect("Unable to parse string as JSON"),
            space: account.space,
        }
    }
}

/// Represents an encoding format for Solana [account](https://solana.com/docs/references/terminology#account) data.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum AccountEncoding {
    /// The account data is formatted as a binary string.
    #[serde(rename = "binary")]
    Binary, // Legacy. Retained for RPC backwards compatibility
    /// The account data is formatted as a base-58 string.
    #[serde(rename = "base58")]
    Base58,
    /// The account data is formatted as a base-64 string.
    #[serde(rename = "base64")]
    Base64,
    /// The account data was first compressed using [Zstandard](http://facebook.github.io/zstd/) and the
    /// result was then formatted as a base-64 string.
    #[serde(rename = "base64+zstd")]
    Base64Zstd,
    /// The account data is formatted as a JSON string.
    #[serde(rename = "jsonParsed")]
    JsonParsed,
}

impl From<solana_account_decoder_client_types::UiAccountEncoding> for AccountEncoding {
    fn from(encoding: solana_account_decoder_client_types::UiAccountEncoding) -> Self {
        use solana_account_decoder_client_types::UiAccountEncoding;
        match encoding {
            UiAccountEncoding::Binary => Self::Binary,
            UiAccountEncoding::Base58 => Self::Base58,
            UiAccountEncoding::Base64 => Self::Base64,
            UiAccountEncoding::JsonParsed => Self::JsonParsed,
            UiAccountEncoding::Base64Zstd => Self::Base64Zstd,
        }
    }
}

impl From<AccountEncoding> for solana_account_decoder_client_types::UiAccountEncoding {
    fn from(encoding: AccountEncoding) -> Self {
        match encoding {
            AccountEncoding::Binary => Self::Binary,
            AccountEncoding::Base58 => Self::Base58,
            AccountEncoding::Base64 => Self::Base64,
            AccountEncoding::JsonParsed => Self::JsonParsed,
            AccountEncoding::Base64Zstd => Self::Base64Zstd,
        }
    }
}
