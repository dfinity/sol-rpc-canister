#[cfg(test)]
mod tests;

pub mod account;
pub mod request;
pub mod transaction;

use crate::{EncodedTransactionWithStatusMeta, Reward, RpcError};
use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, str::FromStr};

/// A Solana [slot](https://solana.com/docs/references/terminology#slot).
pub type Slot = u64;

/// A Solana [Lamport](https://solana.com/de/docs/references/terminology#lamport).
pub type Lamport = u64;

/// Within the compute budget, a quantity of micro-lamports is used in the calculation of prioritization fees.
/// `1_000_000 MicroLamport == 1 Lamport`
pub type MicroLamport = u64;

/// Unix timestamp (seconds since the Unix epoch).
///
/// This type is defined as an unsigned integer to align with the Solana JSON-RPC interface,
/// although in practice, an unsigned integer type would be functionally equivalent.
pub type Timestamp = i64;

/// The result of a Solana `getBlock` RPC method call.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct ConfirmedBlock {
    /// The blockhash of this block's parent, as base-58 encoded string; if the parent block is not
    /// available due to ledger cleanup, this field will return "11111111111111111111111111111111".
    #[serde(rename = "previousBlockhash")]
    pub previous_blockhash: Hash,
    /// The blockhash of this block, as base-58 encoded string.
    pub blockhash: Hash,
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
    /// Array of rewards distributed in this block.
    pub rewards: Option<Vec<Reward>>,
    /// The epoch rewards are distributed over `1 + num_reward_partitions` blocks. See the
    /// [Partitioned Inflationary Rewards Distribution](https://docs.anza.xyz/proposals/partitioned-inflationary-rewards-distribution/)
    /// feature for more details.
    #[serde(rename = "numRewardPartition")]
    pub num_reward_partitions: Option<u64>,
    /// Transaction details for the transactions included in this block. Included in the response
    /// when `transactionDetails` is set to `accounts`.
    pub transactions: Option<Vec<EncodedTransactionWithStatusMeta>>,
}

impl TryFrom<solana_transaction_status_client_types::UiConfirmedBlock> for ConfirmedBlock {
    type Error = RpcError;

    fn try_from(
        block: solana_transaction_status_client_types::UiConfirmedBlock,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            previous_blockhash: block.previous_blockhash.parse()?,
            blockhash: block.blockhash.parse()?,
            parent_slot: block.parent_slot,
            block_time: block.block_time,
            block_height: block.block_height,
            signatures: block.signatures.map(parse_vec).transpose()?,
            rewards: block
                .rewards
                .map(|rewards| rewards.into_iter().map(Reward::try_from).collect())
                .transpose()?,
            num_reward_partitions: block.num_reward_partitions,
            transactions: block
                .transactions
                .map(|transactions| {
                    transactions
                        .into_iter()
                        .map(EncodedTransactionWithStatusMeta::try_from)
                        .collect()
                })
                .transpose()?,
        })
    }
}

impl From<ConfirmedBlock> for solana_transaction_status_client_types::UiConfirmedBlock {
    fn from(block: ConfirmedBlock) -> Self {
        Self {
            previous_blockhash: block.previous_blockhash.to_string(),
            blockhash: block.blockhash.to_string(),
            parent_slot: block.parent_slot,
            transactions: block.transactions.map(|transactions| {
                transactions
                    .into_iter()
                    .map(solana_transaction_status_client_types::EncodedTransactionWithStatusMeta::from)
                    .collect()
            }),
            signatures: block
                .signatures
                .map(|sigs| sigs.into_iter().map(|sig| sig.to_string()).collect()),
            rewards: block.rewards.map(|rewards| {
                rewards
                    .into_iter()
                    .map(solana_transaction_status_client_types::Reward::from)
                    .collect()
            }),
            num_reward_partitions: block.num_reward_partitions,
            block_time: block.block_time,
            block_height: block.block_height,
        }
    }
}

/// An entry in the result of a Solana `getRecentPrioritizationFees` RPC method call.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct PrioritizationFee {
    /// Slot in which the fee was observed.
    pub slot: Slot,
    /// The per-compute-unit fee paid by at least one successfully landed transaction,
    /// specified in increments of micro-lamports (0.000001 lamports)
    #[serde(rename = "prioritizationFee")]
    pub prioritization_fee: MicroLamport,
}

macro_rules! impl_candid {
    ($name: ident($data: ty), $error: ty) => {
        #[doc = concat!("Candid wrapper around `", stringify!($data), "`. ")]
        #[derive(Clone, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
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

impl_candid!(Hash(solana_hash::Hash), solana_hash::ParseHashError);

fn parse_vec<T, E>(values: Vec<String>) -> Result<Vec<T>, E>
where
    T: FromStr<Err = E>,
{
    values.into_iter().map(|v| v.parse()).collect()
}

fn parse_opt<V, T, E>(value: V) -> Result<Option<T>, E>
where
    V: Into<Option<String>>,
    T: FromStr<Err = E>,
{
    value.into().map(|v| v.parse()).transpose()
}

fn try_from_vec<U, V, E>(values: Vec<U>) -> Result<Vec<V>, E>
where
    V: TryFrom<U, Error = E>,
{
    values.into_iter().map(V::try_from).collect()
}
