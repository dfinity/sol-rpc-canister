//! Candid types used by the candid interface of the SOL RPC canister.
//!
//! ⚠️ **Build Requirements**
//!
//! If you are using the `sol_rpc_types` crate inside a canister, make sure to follow these steps to ensure your code compiles:
//!
//! 1. **Patch Solana SDK dependencies**
//!
//!    Copy the `[patch.crates-io]` section from the top-level [`Cargo.toml`](https://github.com/dfinity/sol-rpc-canister/blob/main/Cargo.toml)
//!    file in the [`dfinity/sol-rpc`](https://github.com/dfinity/sol-rpc-canister/) repository into your own `Cargo.toml`.
//!
//!    This is necessary because the Solana SDK’s `wasm32-unknown-unknown` target assumes a browser environment and depends on
//!    `wasm-bindgen`, which is incompatible with use inside a canister.
//!    See [this issue](https://github.com/anza-xyz/solana-sdk/issues/117) for more information.
//!
//! 2. **Configure the `getrandom` crate**
//!
//!    Add the following entry to your `Cargo.toml` file:
//!
//!    ```toml
//!    getrandom = { version = "*", default-features = false, features = ["custom"] }
//!    ```
//!
//!    This prevents the `js` feature of `getrandom` (a transitive dependency of the Solana SDK) from being enabled.
//!    The `js` feature assumes a browser environment and depends on `wasm-bindgen`, which is incompatible with canisters.
//!
//!    💡 You can also specify a particular version for `getrandom`, as long as the `default-features = false` and `features = ["custom"]` flags are set.
//!
//!    See [this forum post](https://forum.dfinity.org/t/module-imports-function-wbindgen-describe-from-wbindgen-placeholder-that-is-not-exported-by-the-runtime/11545/6) for more details.
//!
//! 3. **macOS-specific setup for `zstd` dependency**
//!
//!    On **macOS**, an `llvm` version that supports the `wasm32-unknown-unknown` target is required.
//!    This is because the  [`zstd`](https://docs.rs/zstd/latest/zstd/) crate (used, for example, to decode
//!    `base64+zstd`-encoded responses from Solana’s [`getAccountInfo`](https://solana.com/de/docs/rpc/http/getaccountinfo))
//!    relies on LLVM during compilation.
//!
//!    The default LLVM bundled with Xcode does not support `wasm32-unknown-unknown`. To fix this:
//!
//!    - Install the [Homebrew version](https://formulae.brew.sh/formula/llvm) of LLVM:
//!
//!      ```sh
//!      brew install llvm
//!      ```
//!
//!    - Create (or modify) your top-level `.cargo/config.toml` and add:
//!
//!      ```toml
//!      [env]
//!      AR = "<LLVM_PATH>/bin/llvm-ar"
//!      CC = "<LLVM_PATH>/bin/clang"
//!      ```
//!
//!      Replace `<LLVM_PATH>` with the output of:
//!
//!      ```sh
//!      brew --prefix llvm
//!      ```

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
