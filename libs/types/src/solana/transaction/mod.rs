pub mod error;
pub mod reward;

use crate::{Pubkey, RpcError, Slot, Timestamp};
use candid::{CandidType, Deserialize};
use error::TransactionError;
use reward::Reward;
use serde::Serialize;
use solana_transaction_status_client_types::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransactionWithStatusMeta,
    UiReturnDataEncoding, UiTransactionReturnData, UiTransactionStatusMeta,
};

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct TransactionInfo {
    pub slot: Slot,
    pub block_time: Option<Timestamp>,
    pub meta: Option<TransactionStatusMeta>,
    pub transaction: EncodedTransaction,
    pub version: Option<TransactionVersion>,
}

impl TryFrom<EncodedConfirmedTransactionWithStatusMeta> for TransactionInfo {
    type Error = RpcError;

    fn try_from(
        transaction: EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            slot: transaction.slot,
            block_time: transaction.block_time,
            meta: transaction.transaction.meta.map(Into::into),
            transaction: transaction.transaction.transaction.try_into()?,
            version: transaction.transaction.version.map(Into::into),
        })
    }
}

impl From<TransactionInfo> for EncodedConfirmedTransactionWithStatusMeta {
    fn from(transaction: TransactionInfo) -> Self {
        Self {
            slot: transaction.slot,
            transaction: EncodedTransactionWithStatusMeta {
                transaction: transaction.transaction.into(),
                meta: transaction.meta.map(Into::into),
                version: transaction.version.map(Into::into),
            },
            block_time: transaction.block_time,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct TransactionStatusMeta {
    pub status: Result<(), TransactionError>,
    pub fee: u64,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub inner_instructions: Option<Vec<Vec<InnerInstruction>>>,
    pub log_messages: Option<Vec<String>>,
    pub pre_token_balances: Option<Vec<TransactionTokenBalance>>,
    pub post_token_balances: Option<Vec<TransactionTokenBalance>>,
    pub rewards: Option<Vec<Reward>>,
    pub loaded_addresses: Option<LoadedAddresses>,
    pub return_data: Option<TransactionReturnData>,
    pub compute_units_consumed: Option<u64>,
}

impl From<TransactionStatusMeta> for UiTransactionStatusMeta {
    fn from(meta: TransactionStatusMeta) -> Self {
        let status = meta.status.map_err(Into::into);
        Self {
            err: status.clone().err(),
            status,
            fee: meta.fee,
            pre_balances: meta.pre_balances,
            post_balances: meta.post_balances,
            inner_instructions: meta.inner_instructions.map(Into::into),
            log_messages: meta.log_messages.into(),
            pre_token_balances: meta.pre_token_balances.map(Into::into),
            post_token_balances: meta.post_token_balances.map(Into::into),
            rewards: meta.rewards.map(Into::into),
            loaded_addresses: meta.loaded_addresses.map(Into::into).into(),
            return_data: meta.return_data.map(Into::into).into(),
            compute_units_consumed: meta.compute_units_consumed,
        }
    }
}

impl From<UiTransactionStatusMeta> for TransactionStatusMeta {
    fn from(meta: UiTransactionStatusMeta) -> Self {
        Self {
            status: meta.status.map_err(Into::into),
            fee: meta.fee,
            pre_balances: meta.pre_balances,
            post_balances: meta.post_balances,
            inner_instructions: meta.inner_instructions.map(Into::into),
            log_messages: meta.log_messages.into(),
            pre_token_balances: meta.pre_token_balances.map(Into::into),
            post_token_balances: meta.post_token_balances.map(Into::into),
            rewards: meta.rewards.map(Into::into),
            loaded_addresses: meta.loaded_addresses.map(Into::into).into(),
            return_data: meta.return_data.map(Into::into).into(),
            compute_units_consumed: meta.compute_units_consumed.into(),
        }
    }
}

// TODO XC-343: Add variants corresponding to `Json` and `Accounts` in
//  `solana_transaction_status_client_types::EncodedTransaction`.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum EncodedTransaction {
    LegacyBinary(String),
    Binary(String, TransactionBinaryEncoding),
}

impl TryFrom<solana_transaction_status_client_types::EncodedTransaction> for EncodedTransaction {
    type Error = RpcError;

    fn try_from(
        transaction: solana_transaction_status_client_types::EncodedTransaction,
    ) -> Result<Self, Self::Error> {
        use solana_transaction_status_client_types::EncodedTransaction;
        match transaction {
            EncodedTransaction::LegacyBinary(binary) => Ok(Self::LegacyBinary(binary)),
            EncodedTransaction::Binary(blob, encoding) => Ok(Self::Binary(blob, encoding.into())),
            EncodedTransaction::Json(_) | EncodedTransaction::Accounts(_) => Err(
                RpcError::ValidationError("Unknown transaction encoding".to_string()),
            ),
        }
    }
}

impl From<EncodedTransaction> for solana_transaction_status_client_types::EncodedTransaction {
    fn from(transaction: EncodedTransaction) -> Self {
        match transaction {
            EncodedTransaction::LegacyBinary(binary) => Self::LegacyBinary(binary),
            EncodedTransaction::Binary(blob, encoding) => Self::Binary(blob, encoding.into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum TransactionBinaryEncoding {
    #[serde(rename = "base64")]
    Base64,
    #[serde(rename = "base58")]
    Base58,
}

impl From<solana_transaction_status_client_types::TransactionBinaryEncoding>
    for TransactionBinaryEncoding
{
    fn from(encoding: solana_transaction_status_client_types::TransactionBinaryEncoding) -> Self {
        use solana_transaction_status_client_types::TransactionBinaryEncoding;
        match encoding {
            TransactionBinaryEncoding::Base64 => Self::Base64,
            TransactionBinaryEncoding::Base58 => Self::Base58,
        }
    }
}

impl From<TransactionBinaryEncoding>
    for solana_transaction_status_client_types::TransactionBinaryEncoding
{
    fn from(encoding: TransactionBinaryEncoding) -> Self {
        match encoding {
            TransactionBinaryEncoding::Base64 => Self::Base64,
            TransactionBinaryEncoding::Base58 => Self::Base58,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct InnerInstruction {
    pub instruction: CompiledInstruction,
    pub stack_height: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct CompiledInstruction {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct TransactionTokenBalance {
    pub account_index: u8,
    pub mint: String,
    pub ui_token_amount: TokenAmount,
    pub owner: String,
    pub program_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct TokenAmount {
    pub ui_amount: Option<f64>,
    pub decimals: u8,
    pub amount: String,
    pub ui_amount_string: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct LoadedAddresses {
    pub writable: Vec<String>,
    pub readonly: Vec<String>,
}

impl From<solana_transaction_status_client_types::UiLoadedAddresses> for LoadedAddresses {
    fn from(addresses: solana_transaction_status_client_types::UiLoadedAddresses) -> Self {
        Self {
            writable: addresses.writable,
            readonly: addresses.readonly,
        }
    }
}

impl From<LoadedAddresses> for solana_transaction_status_client_types::UiLoadedAddresses {
    fn from(addresses: LoadedAddresses) -> Self {
        Self {
            writable: addresses.writable,
            readonly: addresses.readonly,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct TransactionReturnData {
    pub program_id: String,
    pub data: String,
}

impl From<UiTransactionReturnData> for TransactionReturnData {
    fn from(return_data: UiTransactionReturnData) -> Self {
        let (data, encoding) = return_data.data;
        Self {
            program_id: return_data.program_id,
            data: match encoding {
                UiReturnDataEncoding::Base64 => data,
            },
        }
    }
}

impl From<TransactionReturnData> for UiTransactionReturnData {
    fn from(return_data: TransactionReturnData) -> Self {
        Self {
            program_id: return_data.program_id,
            data: (return_data.data, UiReturnDataEncoding::Base64),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum ReturnDataEncoding {
    Base64,
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum TransactionVersion {
    Legacy,
    Number(u8),
}

impl From<solana_transaction::versioned::TransactionVersion> for TransactionVersion {
    fn from(version: solana_transaction::versioned::TransactionVersion) -> Self {
        match version {
            solana_transaction::versioned::TransactionVersion::Legacy(_) => Self::Legacy,
            solana_transaction::versioned::TransactionVersion::Number(version) => {
                Self::Number(version)
            }
        }
    }
}

impl From<TransactionVersion> for solana_transaction::versioned::TransactionVersion {
    fn from(version: TransactionVersion) -> Self {
        match version {
            TransactionVersion::Legacy => Self::LEGACY,
            TransactionVersion::Number(version) => Self::Number(version),
        }
    }
}
