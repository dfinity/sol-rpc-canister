pub mod account;
pub mod request;
pub mod transaction;

use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A Solana [slot](https://solana.com/docs/references/terminology#slot).
pub type Slot = u64;

/// A Solana [Lamport](https://solana.com/de/docs/references/terminology#lamport).
pub type Lamport = u64;

/// A Solana base58-encoded [blockhash](https://solana.com/de/docs/references/terminology#blockhash).
pub type Blockhash = String;

/// A Solana base58-encoded [pubkey](https://solana.com/de/docs/references/terminology#public-key-pubkey).
pub type Pubkey = String;

/// A Solana base58-encoded [signature](https://solana.com/docs/references/terminology#signature).
pub type Signature = String;

/// Unix timestamp (seconds since the Unix epoch).
///
/// This type is defined as an unsigned integer to align with the Solana JSON-RPC interface,
/// although in practice, an unsigned integer type would be functionally equivalent.
pub type Timestamp = i64;

/// The result of a Solana `getBlock` RPC method call.
// TODO XC-342: Add `transactions`, `signatures`, `rewards` and `num_reward_partitions` fields.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct ConfirmedBlock {
    /// The blockhash of this block's parent, as base-58 encoded string; if the parent block is not
    /// available due to ledger cleanup, this field will return "11111111111111111111111111111111".
    #[serde(rename = "previousBlockhash")]
    pub previous_blockhash: Blockhash,
    /// The blockhash of this block, as base-58 encoded string.
    pub blockhash: Blockhash,
    /// The slot index of this block's parent.
    #[serde(rename = "parentSlot")]
    pub parent_slot: u64,
    /// Estimated production time.
    #[serde(rename = "blockTime")]
    pub block_time: Option<Timestamp>,
    /// The number of blocks beneath this block.
    #[serde(rename = "blockHeight")]
    pub block_height: Option<u64>,
    /// Signatures of the transactions in the block. Included in the response whenever
    /// `transactionDetails` is not `none`.
    pub signatures: Option<Vec<Signature>>,
}

impl From<solana_transaction_status_client_types::UiConfirmedBlock> for ConfirmedBlock {
    fn from(block: solana_transaction_status_client_types::UiConfirmedBlock) -> Self {
        Self {
            previous_blockhash: block.previous_blockhash,
            blockhash: block.blockhash,
            parent_slot: block.parent_slot,
            block_time: block.block_time,
            block_height: block.block_height,
            signatures: block.signatures,
        }
    }
}

// TODO XC-342: Set `transactions`, `signatures`, `rewards` and `num_reward_partitions` fields.
impl From<ConfirmedBlock> for solana_transaction_status_client_types::UiConfirmedBlock {
    fn from(block: ConfirmedBlock) -> Self {
        Self {
            previous_blockhash: block.previous_blockhash,
            blockhash: block.blockhash,
            parent_slot: block.parent_slot,
            transactions: None,
            signatures: block.signatures,
            rewards: None,
            num_reward_partitions: None,
            block_time: block.block_time,
            block_height: block.block_height,
        }
    }
}

/// An entry in the result of a Solana `getRecentPrioritizationFees` RPC method call.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct PrioritizationFee {
    /// Slot in which the fee was observed.
    pub slot: u64,
    /// The per-compute-unit fee paid by at least one successfully landed transaction,
    /// specified in increments of micro-lamports (0.000001 lamports)
    #[serde(rename = "prioritizationFee")]
    pub prioritization_fee: u64,
}
