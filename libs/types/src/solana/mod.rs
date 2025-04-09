use candid::CandidType;
use serde::{Deserialize, Serialize};
use solana_account_decoder_client_types::{UiAccountEncoding, UiDataSliceConfig};
use solana_commitment_config::CommitmentConfig;
use solana_rpc_client_api::config::{RpcAccountInfoConfig, RpcBlockConfig};
use solana_transaction_status_client_types::TransactionDetails;
use std::fmt::Debug;

/// A Solana [slot](https://solana.com/docs/references/terminology#slot).
pub type Slot = u64;

/// The parameters for a Solana [`getSlot`](https://solana.com/docs/rpc/http/getslot) RPC method call.
#[derive(Debug, Clone, Default, Deserialize, Serialize, CandidType)]
pub struct GetSlotParams {
    /// The request returns the slot that has reached this or the default commitment level.
    pub commitment: Option<CommitmentLevel>,
    /// The minimum slot that the request can be evaluated at.
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
}

/// The parameters for a Solana [`getAccountInfo`](https://solana.com/docs/rpc/http/getAccountInfo) RPC method call.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
#[serde(into = "(String, Option<RpcAccountInfoConfig>)")]
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
        self.commitment.is_none()
            && self.encoding.is_none()
            && self.data_slice.is_none()
            && self.min_context_slot.is_none()
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

impl From<GetAccountInfoParams> for (String, Option<RpcAccountInfoConfig>) {
    fn from(params: GetAccountInfoParams) -> Self {
        let config = if params.is_default_config() {
            None
        } else {
            Some(RpcAccountInfoConfig {
                encoding: params.encoding.map(Into::into),
                data_slice: params.data_slice.map(Into::into),
                commitment: params.commitment.map(Into::into),
                min_context_slot: params.min_context_slot,
            })
        };
        (params.pubkey, config)
    }
}

impl From<GetAccountInfoEncoding> for UiAccountEncoding {
    fn from(encoding: GetAccountInfoEncoding) -> Self {
        match encoding {
            GetAccountInfoEncoding::Base58 => Self::Base58,
            GetAccountInfoEncoding::Base64 => Self::Base64,
            GetAccountInfoEncoding::Base64ZStd => Self::Base64Zstd,
            GetAccountInfoEncoding::JsonParsed => Self::JsonParsed,
        }
    }
}

/// The parameters for a Solana [`getBlock`](https://solana.com/docs/rpc/http/getblock) RPC method call.
// TODO XC-289: Add `rewards`, `encoding` and `transactionDetails` fields.
#[derive(Debug, Clone, Default, Deserialize, Serialize, CandidType)]
#[serde(into = "(Slot, Option<RpcBlockConfig>)")]
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
}

impl GetBlockParams {
    /// Returns `true` if all of the optional config parameters are `None` and `false` otherwise.
    pub fn is_default_config(&self) -> bool {
        self.commitment.is_none() && self.max_supported_transaction_version.is_none()
    }
}

impl From<Slot> for GetBlockParams {
    fn from(slot: Slot) -> Self {
        Self {
            slot,
            commitment: None,
            max_supported_transaction_version: None,
        }
    }
}

impl From<GetBlockParams> for (Slot, Option<RpcBlockConfig>) {
    fn from(params: GetBlockParams) -> Self {
        let config = if params.is_default_config() {
            None
        } else {
            Some(RpcBlockConfig {
                encoding: None,
                transaction_details: Some(TransactionDetails::None),
                rewards: Some(false),
                commitment: params.commitment.map(Into::into),
                max_supported_transaction_version: params.max_supported_transaction_version,
            })
        };
        (params.slot, config)
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

impl From<CommitmentLevel> for CommitmentConfig {
    fn from(commitment_level: CommitmentLevel) -> Self {
        Self {
            commitment: match commitment_level {
                CommitmentLevel::Processed => solana_commitment_config::CommitmentLevel::Processed,
                CommitmentLevel::Confirmed => solana_commitment_config::CommitmentLevel::Confirmed,
                CommitmentLevel::Finalized => solana_commitment_config::CommitmentLevel::Finalized,
            },
        }
    }
}

/// Subset of [`CommitmentLevel`] whose variants are allowed values for the `encoding`
/// field of [`GetBlockParams`].
#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
pub enum GetBlockCommitmentLevel {
    /// See [`CommitmentLevel::Confirmed`].
    #[serde(rename = "confirmed")]
    Confirmed,
    /// See [`CommitmentLevel::Finalized`].
    #[serde(rename = "finalized")]
    Finalized,
}

impl From<GetBlockCommitmentLevel> for CommitmentConfig {
    fn from(commitment_level: GetBlockCommitmentLevel) -> Self {
        use solana_commitment_config::CommitmentLevel;
        Self {
            commitment: match commitment_level {
                GetBlockCommitmentLevel::Confirmed => CommitmentLevel::Confirmed,
                GetBlockCommitmentLevel::Finalized => CommitmentLevel::Finalized,
            },
        }
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
    length: u32,
    /// Byte offset from which to start reading.
    offset: u32,
}

impl From<DataSlice> for UiDataSliceConfig {
    fn from(data: DataSlice) -> Self {
        Self {
            offset: data.offset as usize,
            length: data.length as usize,
        }
    }
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

impl From<UiAccountEncoding> for AccountEncoding {
    fn from(encoding: UiAccountEncoding) -> Self {
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

impl From<AccountEncoding> for UiAccountEncoding {
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

/// The result of a Solana `getBlock` RPC method call.
// TODO XC-289: Add `transactions`, `signatures`, `rewards` and `num_reward_partitions` fields.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct ConfirmedBlock {
    /// The blockhash of this block's parent, as base-58 encoded string; if the parent block is not
    /// available due to ledger cleanup, this field will return "11111111111111111111111111111111".
    #[serde(rename = "previousBlockhash")]
    pub previous_blockhash: String,
    /// The blockhash of this block, as base-58 encoded string.
    pub blockhash: String,
    /// The slot index of this block's parent.
    #[serde(rename = "parentSlot")]
    pub parent_slot: u64,
    /// Estimated production time, as Unix timestamp (seconds since the Unix epoch).
    #[serde(rename = "blockTime")]
    pub block_time: Option<i64>,
    /// The number of blocks beneath this block.
    #[serde(rename = "blockHeight")]
    pub block_height: Option<u64>,
}

impl From<solana_transaction_status_client_types::UiConfirmedBlock> for ConfirmedBlock {
    fn from(block: solana_transaction_status_client_types::UiConfirmedBlock) -> Self {
        Self {
            previous_blockhash: block.previous_blockhash,
            blockhash: block.blockhash,
            parent_slot: block.parent_slot,
            block_time: block.block_time,
            block_height: block.block_height,
        }
    }
}

impl From<ConfirmedBlock> for solana_transaction_status_client_types::UiConfirmedBlock {
    fn from(block: ConfirmedBlock) -> Self {
        Self {
            previous_blockhash: block.previous_blockhash,
            blockhash: block.blockhash,
            parent_slot: block.parent_slot,
            transactions: None,
            signatures: None,
            rewards: None,
            num_reward_partitions: None,
            block_time: block.block_time,
            block_height: block.block_height,
        }
    }
}
