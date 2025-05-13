//! Candid types used by the candid interface of the SOL RPC canister.

#![forbid(unsafe_code)]
#![forbid(missing_docs)]

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
        CommitmentLevel, DataSlice, GetAccountInfoEncoding, GetAccountInfoParams, GetBalanceParams,
        GetBlockCommitmentLevel, GetBlockParams, GetSlotParams, GetTokenAccountBalanceParams,
        GetTransactionEncoding, GetTransactionParams, SendTransactionEncoding,
        SendTransactionParams, TransactionDetails,
    },
    transaction::{
        error::{InstructionError, TransactionError},
        instruction::{CompiledInstruction, InnerInstructions, Instruction},
        reward::{Reward, RewardType},
        EncodedTransaction, LoadedAddresses, TokenAmount, TransactionBinaryEncoding,
        TransactionInfo, TransactionReturnData, TransactionStatusMeta, TransactionTokenBalance,
        TransactionVersion,
    },
    ConfirmedBlock, Hash, Lamport, Pubkey, Signature, Slot, Timestamp,
};
