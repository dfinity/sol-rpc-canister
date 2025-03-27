use candid::CandidType;
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, Default, Deserialize, Serialize, CandidType)]
pub struct GetAccountInfoParams {
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
    // TODO XC-288
    // /// Account data is first compressed using [Zstandard](http://facebook.github.io/zstd/) and the
    // /// result is then base-64 encoded.
    // #[serde(rename = "base64+zstd")]
    // Base64ZStd,
    /// The encoding attempts to use program-specific state parsers to return more human-readable
    /// and explicit account state data. If [`JsonParsed`] is requested but a parser cannot be
    /// found, the fallback is [`Base64`] encoding.
    #[serde(rename = "jsonParsed")]
    JsonParsed,
}

/// Represents a slice of the return value of the Solana [`getAccountInfo`](https://solana.com/docs/rpc/http/getAccountInfo) RPC method.
#[derive(Debug, Clone, Default, Deserialize, Serialize, CandidType)]
pub struct DataSlice {
    /// Number of bytes to return.
    length: u64, // TODO XC-288: Is this correct for usize?
    /// Byte offset from which to start reading.
    offset: u64, // TODO XC-288: Is this correct for usize?
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
pub struct Account {
    /// The lamports in the account.  Modifiable by programs.
    pub lamports: u64,
    /// The data held in this account.  Modifiable by programs.
    pub data: Vec<u8>,
    /// Program that owns this account
    pub owner: Pubkey,
    /// This account's data contains a loaded program (and is now read-only)
    pub executable: bool,
    /// The epoch at which this account will next owe rent
    pub rent_epoch: u64,
}

impl From<solana_account::Account> for Account {
    fn from(account: solana_account::Account) -> Self {
        Account {
            lamports: account.lamports,
            data: account.data,
            owner: account.owner.into(),
            executable: account.executable,
            rent_epoch: account.rent_epoch,
        }
    }
}

impl From<Account> for solana_account::Account {
    fn from(account: Account) -> Self {
        solana_account::Account {
            lamports: account.lamports,
            data: account.data,
            owner: account.owner.into(),
            executable: account.executable,
            rent_epoch: account.rent_epoch,
        }
    }
}
