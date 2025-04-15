pub mod error;
pub mod reward;

use crate::{RpcError, Slot, Timestamp};
use candid::{CandidType, Deserialize};
use error::TransactionError;
use reward::Reward;
use serde::Serialize;
use solana_account_decoder_client_types::token::UiTokenAmount;
use solana_transaction_status_client_types::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransactionWithStatusMeta,
    UiCompiledInstruction, UiInnerInstructions, UiInstruction, UiReturnDataEncoding,
    UiTransactionReturnData, UiTransactionStatusMeta,
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
            meta: transaction
                .transaction
                .meta
                .map(TryInto::try_into)
                .transpose()?,
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
    pub inner_instructions: Option<Vec<InnerInstructions>>,
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
            inner_instructions: meta
                .inner_instructions
                .map(|instructions| {
                    instructions
                        .into_iter()
                        .map(|instruction| instruction.into())
                        .collect()
                })
                .into(),
            log_messages: meta.log_messages.into(),
            pre_token_balances: meta
                .pre_token_balances
                .map(|balances| balances.into_iter().map(Into::into).collect())
                .into(),
            post_token_balances: meta
                .post_token_balances
                .map(|balances| balances.into_iter().map(Into::into).collect())
                .into(),
            rewards: meta
                .rewards
                .map(|rewards| rewards.into_iter().map(Into::into).collect())
                .into(),
            loaded_addresses: meta.loaded_addresses.map(Into::into).into(),
            return_data: meta.return_data.map(Into::into).into(),
            compute_units_consumed: meta.compute_units_consumed.into(),
        }
    }
}

impl TryFrom<UiTransactionStatusMeta> for TransactionStatusMeta {
    type Error = RpcError;

    fn try_from(meta: UiTransactionStatusMeta) -> Result<Self, Self::Error> {
        Ok(Self {
            status: meta.status.map_err(Into::into),
            fee: meta.fee,
            pre_balances: meta.pre_balances,
            post_balances: meta.post_balances,
            inner_instructions: meta
                .inner_instructions
                .map(|instructions| {
                    instructions
                        .into_iter()
                        .map(|instruction| instruction.try_into())
                        .collect::<Result<Vec<InnerInstructions>, Self::Error>>()
                })
                .transpose()?,
            log_messages: meta.log_messages.into(),
            pre_token_balances: meta
                .pre_token_balances
                .map(|balances| balances.into_iter().map(Into::into).collect())
                .into(),
            post_token_balances: meta
                .post_token_balances
                .map(|balances| balances.into_iter().map(Into::into).collect())
                .into(),
            rewards: meta
                .rewards
                .map(|rewards| rewards.into_iter().map(Into::into).collect())
                .into(),
            loaded_addresses: meta.loaded_addresses.map(Into::into).into(),
            return_data: meta.return_data.map(Into::into).into(),
            compute_units_consumed: meta.compute_units_consumed.into(),
        })
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
pub struct InnerInstructions {
    pub index: u8,
    pub instructions: Vec<Instruction>,
}

impl TryFrom<UiInnerInstructions> for InnerInstructions {
    type Error = RpcError;

    fn try_from(instructions: UiInnerInstructions) -> Result<Self, Self::Error> {
        Ok(Self {
            index: instructions.index,
            instructions: instructions
                .instructions
                .into_iter()
                .map(TryInto::<Instruction>::try_into)
                .collect::<Result<Vec<Instruction>, Self::Error>>()?,
        })
    }
}

impl From<InnerInstructions> for UiInnerInstructions {
    fn from(instructions: InnerInstructions) -> Self {
        Self {
            index: instructions.index,
            instructions: instructions
                .instructions
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum Instruction {
    Compiled(CompiledInstruction),
}

impl TryFrom<UiInstruction> for Instruction {
    type Error = RpcError;

    fn try_from(instruction: UiInstruction) -> Result<Self, Self::Error> {
        match instruction {
            UiInstruction::Compiled(compiled) => Ok(Self::Compiled(compiled.into())),
            UiInstruction::Parsed(_) => Err(RpcError::ValidationError(
                "Parsed instructions are not supported".to_string(),
            )),
        }
    }
}

impl From<Instruction> for UiInstruction {
    fn from(instruction: Instruction) -> Self {
        match instruction {
            Instruction::Compiled(compiled) => Self::Compiled(compiled.into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct CompiledInstruction {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub data: String,
    pub stack_height: Option<u32>,
}

impl From<UiCompiledInstruction> for CompiledInstruction {
    fn from(instruction: UiCompiledInstruction) -> Self {
        Self {
            program_id_index: instruction.program_id_index,
            accounts: instruction.accounts,
            data: instruction.data,
            stack_height: instruction.stack_height,
        }
    }
}

impl From<CompiledInstruction> for UiCompiledInstruction {
    fn from(instruction: CompiledInstruction) -> Self {
        Self {
            program_id_index: instruction.program_id_index,
            accounts: instruction.accounts,
            data: instruction.data,
            stack_height: instruction.stack_height,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct TransactionTokenBalance {
    pub account_index: u8,
    pub mint: String,
    pub ui_token_amount: TokenAmount,
    pub owner: Option<String>,
    pub program_id: Option<String>,
}

impl From<solana_transaction_status_client_types::UiTransactionTokenBalance>
    for TransactionTokenBalance
{
    fn from(balance: solana_transaction_status_client_types::UiTransactionTokenBalance) -> Self {
        Self {
            account_index: balance.account_index,
            mint: balance.mint,
            ui_token_amount: balance.ui_token_amount.into(),
            owner: balance.owner.into(),
            program_id: balance.program_id.into(),
        }
    }
}

impl From<TransactionTokenBalance>
    for solana_transaction_status_client_types::UiTransactionTokenBalance
{
    fn from(balance: TransactionTokenBalance) -> Self {
        Self {
            account_index: balance.account_index,
            mint: balance.mint,
            ui_token_amount: balance.ui_token_amount.into(),
            owner: balance.owner.into(),
            program_id: balance.program_id.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct TokenAmount {
    pub ui_amount: Option<f64>,
    pub decimals: u8,
    pub amount: String,
    pub ui_amount_string: String,
}

impl From<TokenAmount> for UiTokenAmount {
    fn from(amount: TokenAmount) -> Self {
        Self {
            ui_amount: amount.ui_amount,
            decimals: amount.decimals,
            amount: amount.amount,
            ui_amount_string: amount.ui_amount_string,
        }
    }
}

impl From<UiTokenAmount> for TokenAmount {
    fn from(amount: UiTokenAmount) -> Self {
        Self {
            ui_amount: amount.ui_amount,
            decimals: amount.decimals,
            amount: amount.amount,
            ui_amount_string: amount.ui_amount_string,
        }
    }
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
