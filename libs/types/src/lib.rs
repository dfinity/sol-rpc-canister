//! Candid types used by the candid interface of the SOL RPC canister.

#![forbid(unsafe_code)]
// TODO XC-293: Add documentation for all new types
// #![forbid(missing_docs)]

mod lifecycle;
mod response;
mod rpc_client;
mod solana;

pub use lifecycle::{InstallArgs, Mode, NumSubnetNodes};
pub use response::MultiRpcResult;
pub use rpc_client::{
    ConsensusStrategy, GetSlotRpcConfig, HttpHeader, HttpOutcallError, JsonRpcError,
    OverrideProvider, ProviderError, RegexString, RegexSubstitution, RpcAccess, RpcAuth, RpcConfig,
    RpcEndpoint, RpcError, RpcResult, RpcSource, RpcSources, SolanaCluster, SupportedRpcProvider,
    SupportedRpcProviderId,
};
pub use solana::{
    account::{AccountData, AccountEncoding, AccountInfo, ParsedAccount},
    request::{
        CommitmentLevel, DataSlice, GetAccountInfoEncoding, GetAccountInfoParams,
        GetBlockCommitmentLevel, GetBlockParams, GetSlotParams, GetTransactionEncoding,
        GetTransactionParams, SendTransactionEncoding, SendTransactionParams, TransactionDetails,
    },
    transaction::{
        error::{InstructionError, TransactionError},
        instruction::{CompiledInstruction, InnerInstructions, Instruction},
        reward::{Reward, RewardType},
        EncodedTransaction, LoadedAddresses, TransactionBinaryEncoding, TransactionInfo,
        TransactionReturnData, TransactionStatusMeta, TransactionTokenBalance, TransactionVersion,
    },
    Blockhash, ConfirmedBlock, Pubkey, Signature, Slot, Timestamp,
};
