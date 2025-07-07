pub mod error;
pub mod instruction;
pub mod reward;

use crate::{
    solana::{parse_opt, parse_vec, try_from_vec},
    Pubkey, RpcError, Signature, Slot, Timestamp,
};
use candid::{CandidType, Deserialize};
use error::TransactionError;
use instruction::InnerInstructions;
use reward::Reward;
use serde::Serialize;
use solana_account_decoder_client_types::token::UiTokenAmount;
use solana_transaction_status_client_types::{
    option_serializer::OptionSerializer, UiReturnDataEncoding, UiTransactionReturnData,
    UiTransactionStatusMeta,
};

/// Solana [transaction](https://solana.com/docs/references/terminology#transaction) information
/// for a confirmed transaction.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct EncodedConfirmedTransactionWithStatusMeta {
    /// The slot this transaction was processed in.
    pub slot: Slot,
    /// Estimated production time of when the transaction was processed. [`None`] if not available
    #[serde(rename = "blockTime")]
    pub block_time: Option<Timestamp>,
    /// Transaction information including the metadata, version and encoded transaction.
    pub transaction: EncodedTransactionWithStatusMeta,
}

impl TryFrom<solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta>
    for EncodedConfirmedTransactionWithStatusMeta
{
    type Error = RpcError;

    fn try_from(
        transaction: solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            slot: transaction.slot,
            block_time: transaction.block_time,
            transaction: EncodedTransactionWithStatusMeta::try_from(transaction.transaction)?,
        })
    }
}

impl From<EncodedConfirmedTransactionWithStatusMeta>
    for solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta
{
    fn from(transaction: EncodedConfirmedTransactionWithStatusMeta) -> Self {
        Self {
            slot: transaction.slot,
            transaction:
                solana_transaction_status_client_types::EncodedTransactionWithStatusMeta::from(
                    transaction.transaction,
                ),
            block_time: transaction.block_time,
        }
    }
}

/// Solana [transaction](https://solana.com/docs/references/terminology#transaction) information.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct EncodedTransactionWithStatusMeta {
    /// Transaction status [metadata](https://solana.com/de/docs/rpc/json-structures#transaction-status-metadata)
    /// object or [`None`].
    pub meta: Option<TransactionStatusMeta>,
    /// [Transaction](https://solana.com/de/docs/rpc/json-structures#transactions) object, either
    /// in JSON format or encoded binary data, depending on encoding parameter.
    pub transaction: EncodedTransaction,
    /// Transaction version. [`None`] if `maxSupportedTransactionVersion` is not set in request params.
    pub version: Option<TransactionVersion>,
}

impl TryFrom<solana_transaction_status_client_types::EncodedTransactionWithStatusMeta>
    for EncodedTransactionWithStatusMeta
{
    type Error = RpcError;

    fn try_from(
        transaction: solana_transaction_status_client_types::EncodedTransactionWithStatusMeta,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            meta: transaction
                .meta
                .map(TransactionStatusMeta::try_from)
                .transpose()?,
            transaction: EncodedTransaction::try_from(transaction.transaction)?,
            version: transaction.version.map(TransactionVersion::from),
        })
    }
}

impl From<EncodedTransactionWithStatusMeta>
    for solana_transaction_status_client_types::EncodedTransactionWithStatusMeta
{
    fn from(transaction: EncodedTransactionWithStatusMeta) -> Self {
        Self {
            transaction: transaction.transaction.into(),
            meta: transaction.meta.map(Into::into),
            version: transaction.version.map(Into::into),
        }
    }
}

/// Solana transaction signature information as returned by the [`getSignaturesForAddress`](https://solana.com/de/docs/rpc/http/getsignaturestatuses)
/// RPC method.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct ConfirmedTransactionStatusWithSignature {
    /// Transaction signature.
    pub signature: Signature,
    /// The slot that contains the block with the transaction.
    pub slot: Slot,
    /// Error if transaction failed, [`None`] if transaction succeeded.
    pub err: Option<TransactionError>,
    /// Memo associated with the transaction, [`None`] if no memo is present.
    pub memo: Option<String>,
    /// Estimated production time of when transaction was processed, [`None`] if not available.
    #[serde(rename = "blockTime")]
    pub block_time: Option<Timestamp>,
    /// The transaction's cluster confirmation status; Either `processed`, `confirmed`, or `finalized`.
    /// See [Commitment](https://solana.com/docs/rpc#configuring-state-commitment) for more on
    /// optimistic confirmation.
    #[serde(rename = "confirmationStatus")]
    pub confirmation_status: Option<TransactionConfirmationStatus>,
}

impl From<ConfirmedTransactionStatusWithSignature>
    for solana_transaction_status_client_types::ConfirmedTransactionStatusWithSignature
{
    fn from(value: ConfirmedTransactionStatusWithSignature) -> Self {
        Self {
            signature: solana_signature::Signature::from(value.signature),
            slot: value.slot,
            err: value
                .err
                .map(solana_transaction_error::TransactionError::from),
            memo: value.memo,
            block_time: value.block_time,
        }
    }
}

/// Solana transaction status as returned by the [`getSignatureStatuses`](https://solana.com/de/docs/rpc/http/getsignaturestatuses)
/// RPC method.
///
/// *WARNING*: The optional `confirmations` field in the `getSignatureStatuses` response is not
///     included in this type since it is ignored when processing the RPC response.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub struct TransactionStatus {
    /// The slot the transaction was processed.
    pub slot: u64,
    /// *DEPRECATED*: Transaction status:
    ///  * [`Ok(())`] - Transaction was successful
    ///  * [`Err(err)`] - Transaction failed with [`TransactionError`] `err`
    pub status: Result<(), TransactionError>,
    /// Error if transaction failed, [`None`] if transaction succeeded.
    pub err: Option<TransactionError>,
    /// The transaction's cluster confirmation status; Either [`TransactionConfirmationStatus::Processed`],
    /// [`TransactionConfirmationStatus::Confirmed`], or [`TransactionConfirmationStatus::Finalized`].
    /// See [Commitment](https://solana.com/docs/rpc#configuring-state-commitment) for more on
    /// optimistic confirmation.
    #[serde(rename = "confirmationStatus")]
    pub confirmation_status: Option<TransactionConfirmationStatus>,
}

impl From<solana_transaction_status_client_types::TransactionStatus> for TransactionStatus {
    fn from(status: solana_transaction_status_client_types::TransactionStatus) -> Self {
        Self {
            slot: status.slot,
            status: status.status.map_err(TransactionError::from),
            err: status.err.map(TransactionError::from),
            confirmation_status: status
                .confirmation_status
                .map(TransactionConfirmationStatus::from),
        }
    }
}

impl From<TransactionStatus> for solana_transaction_status_client_types::TransactionStatus {
    fn from(status: TransactionStatus) -> Self {
        Self {
            slot: status.slot,
            confirmations: None,
            status: status
                .status
                .map_err(solana_transaction_error::TransactionError::from),
            err: status
                .err
                .map(solana_transaction_error::TransactionError::from),
            confirmation_status: status
                .confirmation_status
                .map(solana_transaction_status_client_types::TransactionConfirmationStatus::from),
        }
    }
}

/// A Solana transaction confirmation status. See [Commitment](https://solana.com/docs/rpc#configuring-state-commitment)
/// for more on optimistic confirmation.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum TransactionConfirmationStatus {
    /// See [`crate::CommitmentLevel::Processed`].
    #[serde(rename = "processed")]
    Processed,
    /// See [`crate::CommitmentLevel::Confirmed`].
    #[serde(rename = "confirmed")]
    Confirmed,
    /// See [`crate::CommitmentLevel::Finalized`].
    #[serde(rename = "finalized")]
    Finalized,
}

impl From<solana_transaction_status_client_types::TransactionConfirmationStatus>
    for TransactionConfirmationStatus
{
    fn from(status: solana_transaction_status_client_types::TransactionConfirmationStatus) -> Self {
        use solana_transaction_status_client_types::TransactionConfirmationStatus;
        match status {
            TransactionConfirmationStatus::Processed => Self::Processed,
            TransactionConfirmationStatus::Confirmed => Self::Confirmed,
            TransactionConfirmationStatus::Finalized => Self::Finalized,
        }
    }
}

impl From<TransactionConfirmationStatus>
    for solana_transaction_status_client_types::TransactionConfirmationStatus
{
    fn from(status: TransactionConfirmationStatus) -> Self {
        match status {
            TransactionConfirmationStatus::Processed => Self::Processed,
            TransactionConfirmationStatus::Confirmed => Self::Confirmed,
            TransactionConfirmationStatus::Finalized => Self::Finalized,
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
    /// Number of execution compute units consumed by the transaction, excluding overhead (such as
    /// account loading).
    #[serde(rename = "computeUnitsConsumed")]
    pub compute_units_consumed: Option<u64>,
    /// Total compute units consumed by the transaction, including both execution and overhead (such
    /// as account loading).
    pub cost_units: Option<u64>,
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
            cost_units: OptionSerializer::or_skip(meta.cost_units),
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
            inner_instructions: meta.inner_instructions.map(try_from_vec).transpose()?,
            log_messages: meta.log_messages.into(),
            pre_token_balances: meta.pre_token_balances.map(try_from_vec).transpose()?,
            post_token_balances: meta.post_token_balances.map(try_from_vec).transpose()?,
            rewards: meta.rewards.map(try_from_vec).transpose()?,
            loaded_addresses: meta
                .loaded_addresses
                .map(LoadedAddresses::try_from)
                .transpose()?,
            return_data: meta
                .return_data
                .map(TransactionReturnData::try_from)
                .transpose()?,
            compute_units_consumed: meta.compute_units_consumed.into(),
            cost_units: meta.cost_units.into(),
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
    #[serde(rename = "legacyBinary")]
    LegacyBinary(String),
    ///The transaction is encoded in one of the [`TransactionBinaryEncoding`] formats.
    #[serde(rename = "binary")]
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

impl TryFrom<solana_transaction_status_client_types::UiTransactionTokenBalance>
    for TransactionTokenBalance
{
    type Error = RpcError;
    fn try_from(
        balance: solana_transaction_status_client_types::UiTransactionTokenBalance,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            account_index: balance.account_index,
            mint: balance.mint,
            ui_token_amount: balance.ui_token_amount.into(),
            owner: parse_opt(balance.owner)?,
            program_id: parse_opt(balance.program_id)?,
        })
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
            owner: balance.owner.map(|v| v.to_string()).into(),
            program_id: balance.program_id.map(|v| v.to_string()).into(),
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

impl TryFrom<solana_transaction_status_client_types::UiLoadedAddresses> for LoadedAddresses {
    type Error = RpcError;
    fn try_from(
        addresses: solana_transaction_status_client_types::UiLoadedAddresses,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            writable: parse_vec(addresses.writable)?,
            readonly: parse_vec(addresses.readonly)?,
        })
    }
}

impl From<LoadedAddresses> for solana_transaction_status_client_types::UiLoadedAddresses {
    fn from(addresses: LoadedAddresses) -> Self {
        Self {
            writable: addresses
                .writable
                .into_iter()
                .map(|v| v.to_string())
                .collect(),
            readonly: addresses
                .readonly
                .into_iter()
                .map(|v| v.to_string())
                .collect(),
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

impl TryFrom<UiTransactionReturnData> for TransactionReturnData {
    type Error = RpcError;

    fn try_from(return_data: UiTransactionReturnData) -> Result<Self, Self::Error> {
        let (data, encoding) = return_data.data;
        Ok(Self {
            program_id: return_data.program_id.parse()?,
            data: match encoding {
                UiReturnDataEncoding::Base64 => data,
            },
        })
    }
}

impl From<TransactionReturnData> for UiTransactionReturnData {
    fn from(return_data: TransactionReturnData) -> Self {
        Self {
            program_id: return_data.program_id.to_string(),
            data: (return_data.data, UiReturnDataEncoding::Base64),
        }
    }
}

/// Enum representing the version of a Solana transaction.
#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq)]
pub enum TransactionVersion {
    /// Legacy transaction format, which does not explicitly include a version number.
    #[serde(rename = "legacy")]
    Legacy,
    /// Versioned transaction format.
    #[serde(rename = "number")]
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
