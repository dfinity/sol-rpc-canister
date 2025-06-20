//! Candid types used by the candid interface of the SOL RPC canister.

#![forbid(unsafe_code)]
#![forbid(missing_docs)]

#[cfg(test)]
mod tests;

mod lifecycle;
mod response;
mod rpc_client;
mod solana;

use candid::{CandidType, Deserialize};
use derive_more::Into;

pub use lifecycle::{InstallArgs, Mode, NumSubnetNodes};
pub use response::MultiRpcResult;
pub use rpc_client::{
    ConsensusStrategy, GetRecentPrioritizationFeesRpcConfig, GetSlotRpcConfig, HttpHeader,
    HttpOutcallError, JsonRpcError, NonZeroU8, OverrideProvider, ProviderError, RegexString,
    RegexSubstitution, RoundingError, RpcAccess, RpcAuth, RpcConfig, RpcEndpoint, RpcError,
    RpcResult, RpcSource, RpcSources, SolanaCluster, SupportedRpcProvider, SupportedRpcProviderId,
};
use serde::{Serialize, Serializer};
pub use solana::{
    account::{AccountData, AccountEncoding, AccountInfo, ParsedAccount},
    request::{
        CommitmentLevel, DataSlice, GetAccountInfoEncoding, GetAccountInfoParams, GetBalanceParams,
        GetBlockCommitmentLevel, GetBlockParams, GetRecentPrioritizationFeesParams,
        GetSignatureStatusesParams, GetSignaturesForAddressLimit, GetSignaturesForAddressParams,
        GetSlotParams, GetTokenAccountBalanceParams, GetTransactionEncoding, GetTransactionParams,
        SendTransactionEncoding, SendTransactionParams, TransactionDetails,
    },
    transaction::{
        error::{InstructionError, TransactionError},
        instruction::{CompiledInstruction, InnerInstructions, Instruction},
        reward::{Reward, RewardType},
        ConfirmedTransactionStatusWithSignature, EncodedConfirmedTransactionWithStatusMeta,
        EncodedTransaction, EncodedTransactionWithStatusMeta, LoadedAddresses, TokenAmount,
        TransactionBinaryEncoding, TransactionConfirmationStatus, TransactionReturnData,
        TransactionStatus, TransactionStatusMeta, TransactionTokenBalance, TransactionVersion,
    },
    ConfirmedBlock, Hash, Lamport, MicroLamport, PrioritizationFee, Pubkey, Signature, Slot,
    Timestamp,
};

/// A vector with a maximum capacity.
#[derive(Clone, Debug, Default, PartialEq, CandidType, Deserialize, Into)]
#[serde(try_from = "Vec<T>")]
pub struct VecWithMaxLen<T, const CAPACITY: usize>(Vec<T>);

impl<T, const CAPACITY: usize> VecWithMaxLen<T, CAPACITY> {
    /// Constructs a new, empty `VecWithMaxLen<T, CAPACITY>`.
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl<S: Into<T>, T, const CAPACITY: usize> TryFrom<Vec<S>> for VecWithMaxLen<T, CAPACITY> {
    type Error = RpcError;

    fn try_from(value: Vec<S>) -> Result<Self, Self::Error> {
        if value.len() > CAPACITY {
            return Err(RpcError::ValidationError(format!(
                "Expected at most {CAPACITY} items, but got {}",
                value.len()
            )));
        }
        Ok(Self(value.into_iter().map(Into::into).collect()))
    }
}

impl<T: Serialize, const CAPACITY: usize> Serialize for VecWithMaxLen<T, CAPACITY> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}
