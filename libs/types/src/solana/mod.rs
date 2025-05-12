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
            signatures: block.signatures.map(|sigs| {
                sigs.into_iter()
                    .map(|sig| sig.parse().expect("BUG: invalid signature"))
                    .collect()
            }),
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
            signatures: block
                .signatures
                .map(|sigs| sigs.into_iter().map(|sig| sig.to_string()).collect()),
            rewards: None,
            num_reward_partitions: None,
            block_time: block.block_time,
            block_height: block.block_height,
        }
    }
}

macro_rules! impl_candid {
    ($name: ident($data: ty), $error: ty) => {
        #[doc = concat!("Candid wrapper around `", stringify!($data), "`. ")]
        #[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name($data);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(<$data>::default())
            }
        }

        impl From<$data> for $name {
            fn from(value: $data) -> Self {
                Self(value)
            }
        }

        impl From<&$data> for $name {
            fn from(value: &$data) -> Self {
                Self(*value)
            }
        }

        impl From<$name> for $data {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                self.0.as_ref()
            }
        }

        impl CandidType for $name {
            fn _ty() -> candid::types::Type {
                String::_ty()
            }

            fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
            where
                S: candid::types::Serializer,
            {
                serializer.serialize_text(&self.to_string())
            }
        }

        impl std::str::FromStr for $name {
            type Err = $error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.parse::<$data>().map(Self)
            }
        }

        impl TryFrom<String> for $name {
            type Error = $error;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                value.parse()
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.to_string()
            }
        }
    };
}

impl_candid!(
    Pubkey(solana_pubkey::Pubkey),
    solana_pubkey::ParsePubkeyError
);

impl_candid!(
    Signature(solana_signature::Signature),
    solana_signature::ParseSignatureError
);
