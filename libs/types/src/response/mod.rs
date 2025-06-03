use crate::{
    solana::account::AccountInfo, ConfirmedBlock, EncodedConfirmedTransactionWithStatusMeta,
    RpcResult, RpcSource, Signature, TokenAmount, TransactionStatus,
};
use candid::CandidType;
use serde::Deserialize;
use solana_account_decoder_client_types::{token::UiTokenAmount, UiAccount};
use solana_transaction_status_client_types::UiConfirmedBlock;
use std::fmt::Debug;

/// Represents an aggregated result from multiple RPC calls to different RPC providers.
/// The results are aggregated using a [`crate::ConsensusStrategy`].
#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
pub enum MultiRpcResult<T> {
    /// The results from the different providers were consistent.
    Consistent(RpcResult<T>),
    /// The results from the different providers were not consistent.
    Inconsistent(Vec<(RpcSource, RpcResult<T>)>),
}

impl<T> From<RpcResult<T>> for MultiRpcResult<T> {
    fn from(result: RpcResult<T>) -> Self {
        MultiRpcResult::Consistent(result)
    }
}

impl<T> MultiRpcResult<T> {
    /// Maps a [`MultiRpcResult`] containing values of type `T` to a [`MultiRpcResult`] containing
    /// values of type `R` by an infallible map.
    pub fn map<R, F>(self, f: F) -> MultiRpcResult<R>
    where
        F: FnOnce(T) -> R + Clone,
    {
        match self {
            MultiRpcResult::Consistent(result) => MultiRpcResult::Consistent(result.map(f)),
            MultiRpcResult::Inconsistent(results) => MultiRpcResult::Inconsistent(
                results
                    .into_iter()
                    .map(|(source, result)| (source, result.map(f.clone())))
                    .collect(),
            ),
        }
    }

    /// Maps a [`MultiRpcResult`] containing values of type `T` to a [`MultiRpcResult`] containing
    /// values of type `R` by a fallible map.
    pub fn and_then<R, F>(self, f: F) -> MultiRpcResult<R>
    where
        F: FnOnce(T) -> RpcResult<R> + Clone,
    {
        match self {
            MultiRpcResult::Consistent(result) => MultiRpcResult::Consistent(result.and_then(f)),
            MultiRpcResult::Inconsistent(results) => MultiRpcResult::Inconsistent(
                results
                    .into_iter()
                    .map(|(source, result)| (source, result.and_then(f.clone())))
                    .collect(),
            ),
        }
    }
}

impl<T: Debug> MultiRpcResult<T> {
    /// Returns the contents of a [`MultiRpcResult`] if it is an instance of
    /// [`MultiRpcResult::Consistent`] and panics otherwise.
    pub fn expect_consistent(self) -> RpcResult<T> {
        match self {
            MultiRpcResult::Consistent(result) => result,
            MultiRpcResult::Inconsistent(inconsistent_result) => {
                panic!("Expected consistent, but got: {:?}", inconsistent_result)
            }
        }
    }

    /// Returns the contents of a [`MultiRpcResult`] if it is an instance of
    /// [`MultiRpcResult::Inconsistent`] and panics otherwise.
    pub fn expect_inconsistent(self) -> Vec<(RpcSource, RpcResult<T>)> {
        match self {
            MultiRpcResult::Consistent(consistent_result) => {
                panic!("Expected inconsistent:, but got: {:?}", consistent_result)
            }
            MultiRpcResult::Inconsistent(results) => results,
        }
    }
}

impl From<MultiRpcResult<Signature>> for MultiRpcResult<solana_signature::Signature> {
    fn from(result: MultiRpcResult<Signature>) -> Self {
        result.map(solana_signature::Signature::from)
    }
}

impl From<MultiRpcResult<Option<AccountInfo>>> for MultiRpcResult<Option<UiAccount>> {
    fn from(result: MultiRpcResult<Option<AccountInfo>>) -> Self {
        result.map(|maybe_account| maybe_account.map(|account| account.into()))
    }
}

impl From<MultiRpcResult<Option<UiAccount>>> for MultiRpcResult<Option<AccountInfo>> {
    fn from(result: MultiRpcResult<Option<UiAccount>>) -> Self {
        result.map(|maybe_account| maybe_account.map(|account| account.into()))
    }
}

impl From<MultiRpcResult<Option<ConfirmedBlock>>> for MultiRpcResult<Option<UiConfirmedBlock>> {
    fn from(result: MultiRpcResult<Option<ConfirmedBlock>>) -> Self {
        result.map(|maybe_block| maybe_block.map(|block| block.into()))
    }
}

impl From<MultiRpcResult<Option<UiConfirmedBlock>>> for MultiRpcResult<Option<ConfirmedBlock>> {
    fn from(result: MultiRpcResult<Option<UiConfirmedBlock>>) -> Self {
        result.and_then(|maybe_block| maybe_block.map(ConfirmedBlock::try_from).transpose())
    }
}

impl
    From<
        MultiRpcResult<
            Option<
                solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta,
            >,
        >,
    > for MultiRpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>
{
    fn from(
        result: MultiRpcResult<
            Option<
                solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta,
            >,
        >,
    ) -> Self {
        result.and_then(|maybe_transaction| {
            maybe_transaction
                .map(|transaction| transaction.try_into())
                .transpose()
        })
    }
}

impl From<MultiRpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>>
    for MultiRpcResult<
        Option<solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta>,
    >
{
    fn from(result: MultiRpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>) -> Self {
        result.map(|maybe_transaction| maybe_transaction.map(|transaction| transaction.into()))
    }
}

impl From<MultiRpcResult<TokenAmount>> for MultiRpcResult<UiTokenAmount> {
    fn from(result: MultiRpcResult<TokenAmount>) -> Self {
        result.map(UiTokenAmount::from)
    }
}

impl From<MultiRpcResult<UiTokenAmount>> for MultiRpcResult<TokenAmount> {
    fn from(result: MultiRpcResult<UiTokenAmount>) -> Self {
        result.map(TokenAmount::from)
    }
}

impl From<MultiRpcResult<Vec<Option<TransactionStatus>>>>
    for MultiRpcResult<Vec<Option<solana_transaction_status_client_types::TransactionStatus>>>
{
    fn from(result: MultiRpcResult<Vec<Option<TransactionStatus>>>) -> Self {
        result.map(|statuses| {
            statuses
                .into_iter()
                .map(|maybe_status| {
                    maybe_status
                        .map(solana_transaction_status_client_types::TransactionStatus::from)
                })
                .collect()
        })
    }
}

impl From<MultiRpcResult<Vec<Option<solana_transaction_status_client_types::TransactionStatus>>>>
    for MultiRpcResult<Vec<Option<TransactionStatus>>>
{
    fn from(
        result: MultiRpcResult<
            Vec<Option<solana_transaction_status_client_types::TransactionStatus>>,
        >,
    ) -> Self {
        result.map(|statuses| {
            statuses
                .into_iter()
                .map(|maybe_status| maybe_status.map(TransactionStatus::from))
                .collect()
        })
    }
}
