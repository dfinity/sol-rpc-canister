pub mod account;
pub mod request;

use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A Solana [slot](https://solana.com/docs/references/terminology#slot).
pub type Slot = u64;

/// A Solana base58-encoded [transaction ID](https://solana.com/docs/references/terminology#transaction-id).
pub type TransactionId = String;

/// The result of a Solana `getBlock` RPC method call.
// TODO XC-342: Add `transactions`, `signatures`, `rewards` and `num_reward_partitions` fields.
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

// TODO XC-342: Set `transactions`, `signatures`, `rewards` and `num_reward_partitions` fields.
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
