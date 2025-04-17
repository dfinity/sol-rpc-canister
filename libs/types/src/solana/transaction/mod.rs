pub mod error;
pub mod instruction;
pub mod reward;

use crate::{Pubkey, RpcError, Slot, Timestamp};
use candid::{CandidType, Deserialize};
use error::TransactionError;
use instruction::InnerInstructions;
use reward::Reward;
use serde::Serialize;
use solana_account_decoder_client_types::token::UiTokenAmount;
use solana_transaction_status_client_types::{
    option_serializer::OptionSerializer, EncodedConfirmedTransactionWithStatusMeta,
    EncodedTransactionWithStatusMeta, UiReturnDataEncoding, UiTransactionReturnData,
    UiTransactionStatusMeta,
};

/// Solana [transaction](https://solana.com/docs/references/terminology#transaction) information
/// as returned by the [`getTransaction`](https://solana.com/de/docs/rpc/http/gettransaction) RPC
/// method.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct TransactionInfo {
    /// The slot this transaction was processed in.
    pub slot: Slot,
    /// Estimated production time of when the transaction was processed. [`None`] if not available
    #[serde(rename = "blockTime")]
    pub block_time: Option<Timestamp>,
    /// Transaction status [metadata](https://solana.com/de/docs/rpc/json-structures#transaction-status-metadata)
    /// object or [`None`].
    pub meta: Option<TransactionStatusMeta>,
    /// [Transaction](https://solana.com/de/docs/rpc/json-structures#transactions) object, either
    /// in JSON format or encoded binary data, depending on encoding parameter.
    pub transaction: EncodedTransaction,
    /// Transaction version. [`None`] if `maxSupportedTransactionVersion` is not set in request params.
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

/// Transaction status [metadata](https://solana.com/de/docs/rpc/json-structures#transaction-status-metadata) object.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct TransactionStatusMeta {
    /// [`Err`] if transaction failed, [`Ok`] if transaction succeeded.
    pub status: Result<(), TransactionError>,
    /// Fee this transaction was charged.
    pub fee: u64,
    /// Array of account balances from before the transaction was processed.
    #[serde(rename = "preBalances")]
    pub pre_balances: Vec<u64>,
    /// Array of account balances after the transaction was processed.
    #[serde(rename = "postBalances")]
    pub post_balances: Vec<u64>,
    /// List of [inner instructions](https://solana.com/de/docs/rpc/json-structures#inner-instructions)
    /// or [`None`] if inner instruction recording was not enabled during this transaction.
    #[serde(rename = "innerInstructions")]
    pub inner_instructions: Option<Vec<InnerInstructions>>,
    /// Array of log messages or [`None`] if log message recording was not enabled during this
    /// transaction.
    #[serde(rename = "logMessages")]
    pub log_messages: Option<Vec<String>>,
    /// List of [token balances](https://solana.com/de/docs/rpc/json-structures#token-balances) from
    /// before the transaction was processed or [`None`] if token balance recording was not yet
    /// enabled during this transaction.
    #[serde(rename = "preTokenBalances")]
    pub pre_token_balances: Option<Vec<TransactionTokenBalance>>,
    /// List of [token balances](https://solana.com/de/docs/rpc/json-structures#token-balances) from
    /// after the transaction was processed or [`None`] if token balance recording was not yet
    /// enabled during this transaction.
    #[serde(rename = "postTokenBalances")]
    pub post_token_balances: Option<Vec<TransactionTokenBalance>>,
    /// Array of transaction-level rewards.
    pub rewards: Option<Vec<Reward>>,
    /// Transaction addresses loaded from address lookup tables. Undefined if `maxSupportedTransactionVersion`
    /// is not set in request params, or if `jsonParsed` encoding is set in request params.
    #[serde(rename = "loadedAddresses")]
    pub loaded_addresses: Option<LoadedAddresses>,
    /// The most-recent return data generated by an instruction in the transaction.
    #[serde(rename = "returnData")]
    pub return_data: Option<TransactionReturnData>,
    /// Number of compute units consumed by the transaction
    #[serde(rename = "computeUnitsConsumed")]
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
            loaded_addresses: OptionSerializer::or_skip(meta.loaded_addresses.map(Into::into)),
            return_data: OptionSerializer::or_skip(meta.return_data.map(Into::into)),
            compute_units_consumed: OptionSerializer::or_skip(meta.compute_units_consumed),
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
                        .map(InnerInstructions::try_from)
                        .collect::<Result<Vec<InnerInstructions>, Self::Error>>()
                })
                .transpose()?,
            log_messages: meta.log_messages.into(),
            pre_token_balances: meta
                .pre_token_balances
                .map(|balances| balances.into_iter().map(Into::into).collect()),
            post_token_balances: meta
                .post_token_balances
                .map(|balances| balances.into_iter().map(Into::into).collect()),
            rewards: meta
                .rewards
                .map(|rewards| rewards.into_iter().map(Into::into).collect()),
            loaded_addresses: meta.loaded_addresses.map(Into::into),
            return_data: meta.return_data.map(Into::into),
            compute_units_consumed: meta.compute_units_consumed.into(),
        })
    }
}

/// [Transaction](https://solana.com/de/docs/rpc/json-structures#transactions) object, either in
/// JSON format or encoded binary data.
// TODO XC-343: Add variants corresponding to `Json` and `Accounts` in
//  `solana_transaction_status_client_types::EncodedTransaction`.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum EncodedTransaction {
    /// Legacy format kept for backwards compatibility. The transaction is base58-encoded.
    LegacyBinary(String),
    ///The transaction is encoded in one of the [`TransactionBinaryEncoding`] formats.
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

/// Binary encoding format for an [`EncodedTransaction`].
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum TransactionBinaryEncoding {
    /// The transaction is base64-encoded.
    #[serde(rename = "base64")]
    Base64,
    /// The transaction is base58-encoded.
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

/// Represents the balance of a specific SPL token account.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct TransactionTokenBalance {
    /// Index of the account in which the token balance is provided for.
    #[serde(rename = "accountIndex")]
    pub account_index: u8,
    /// Pubkey of the token's mint.
    pub mint: String,
    /// A human-readable representation of the token amount.
    #[serde(rename = "uiTokenAmount")]
    pub ui_token_amount: TokenAmount,
    /// Pubkey of token balance's owner.
    pub owner: Option<Pubkey>,
    /// Pubkey of the Token program that owns the account.
    #[serde(rename = "programId")]
    pub program_id: Option<Pubkey>,
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

/// A human-readable representation of a token amount.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct TokenAmount {
    /// DEPRECATED: Token amount as a float, accounting for decimals.
    #[serde(rename = "uiAmount")]
    pub ui_amount: Option<f64>,
    /// Number of decimals configured for token's mint.
    pub decimals: u8,
    /// Raw amount of tokens as a string, ignoring decimals.
    pub amount: String,
    /// Token amount as a string, accounting for decimals.
    #[serde(rename = "uiAmountString")]
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

/// Transaction addresses loaded from address lookup tables.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct LoadedAddresses {
    /// Ordered list of base-58 encoded addresses for writable loaded accounts.
    pub writable: Vec<Pubkey>,
    /// Ordered list of base-58 encoded addresses for readonly loaded accounts.
    pub readonly: Vec<Pubkey>,
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

/// Represents the return data emitted by a program during transaction execution.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct TransactionReturnData {
    /// The program that generated the return data.
    #[serde(rename = "programId")]
    pub program_id: Pubkey,
    /// The return data itself, as base-64 encoded binary data.
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

/// Enum representing the version of a Solana transaction.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum TransactionVersion {
    /// Legacy transaction format, which does not explicitly include a version number.
    Legacy,
    /// Versioned transaction format.
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
