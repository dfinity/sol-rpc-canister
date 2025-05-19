//! Client to interact with the SOL RPC canister
//!
//! # Examples
//!
//! ## Configuring the client
//!
//! By default, any RPC endpoint supported by the SOL RPC canister will call 3 providers and require equality between their results.
//! It is possible to customize the client so that another strategy, such as 3-out-of-2 in the example below, is used for all following calls.
//!
//! ```rust
//! use candid::Principal;
//! use sol_rpc_client::SolRpcClient;
//! use sol_rpc_types::{ConsensusStrategy, RpcConfig, RpcSources, SolanaCluster};
//!
//! let client = SolRpcClient::builder_for_ic()
//!     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
//!     .with_rpc_config(RpcConfig {
//!         response_consensus: Some(ConsensusStrategy::Threshold {
//!             total: Some(3),
//!             min: 2,
//!         }),
//!         ..Default::default()
//!     })
//!     .build();
//! ```
//!
//! ## Estimating the amount of cycles to send
//!
//! Every call made to the SOL RPC canister that triggers HTTPs outcalls (e.g., `getSlot`)
//! needs to attach some cycles to pay for the call.
//! By default, the client will attach some amount of cycles that should be sufficient for most cases.
//!
//! If this is not the case, the amount of cycles to be sent can be changed as follows:
//! 1. Determine the required amount of cycles to send for a particular request.
//!    The SOL RPC canister offers some query endpoints (e.g., `getSlotCyclesCost`) for that purpose.
//!    This could help establishing a baseline so that the estimated cycles cost for similar requests
//!    can be extrapolated from it instead of making additional queries to the SOL RPC canister.
//! 2. Override the amount of cycles to send for that particular request.
//!    It's advisable to actually send *more* cycles than required, since *unused cycles will be refunded*.
//!
//! ```rust
//! use sol_rpc_client::SolRpcClient;
//! use sol_rpc_types::MultiRpcResult;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use sol_rpc_types::RpcError;
//! let client = SolRpcClient::builder_for_ic()
//! #   .with_mocked_responses(
//! #        MultiRpcResult::Consistent(Ok(332_577_897_u64)),
//! #        Ok::<u128, RpcError>(100_000_000_000),
//! #    )
//!     .build();
//!
//! let request = client.get_slot();
//!
//! let minimum_required_cycles_amount = request.clone().request_cost().send().await.unwrap();
//!
//! let slot = request
//!     .with_cycles(minimum_required_cycles_amount)
//!     .send()
//!     .await
//!     .expect_consistent();
//!
//! assert_eq!(slot, Ok(332_577_897_u64));
//! # Ok(())
//! # }
//! ```
//!
//! ## Overriding client configuration for a specific call
//!
//! Besides changing the amount of cycles for a particular call as described above,
//! it is sometimes desirable to have a custom configuration for a specific
//! call that is different from the one used by the client for all the other calls.
//!
//! For example, maybe for most calls a 2 out-of 3 strategy is good enough, but for `getSlot`
//! your application requires a higher threshold and more robustness with a 3 out-of 5 :
//!
//! ```rust
//! use sol_rpc_client::SolRpcClient;
//! use sol_rpc_types::{
//!     ConsensusStrategy, GetSlotRpcConfig, MultiRpcResult, RpcConfig, RpcSources,
//!     SolanaCluster,
//! };
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = SolRpcClient::builder_for_ic()
//! #   .with_mocked_response(MultiRpcResult::Consistent(Ok(332_577_897_u64)))
//!     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
//!     .with_rpc_config(RpcConfig {
//!         response_consensus: Some(ConsensusStrategy::Threshold {
//!             total: Some(3),
//!             min: 2,
//!         }),
//!     ..Default::default()
//!     })
//!     .build();
//!
//! let slot = client
//!     .get_slot()
//!     .with_rpc_config(GetSlotRpcConfig {
//!         response_consensus: Some(ConsensusStrategy::Threshold {
//!             total: Some(5),
//!             min: 3,
//!         }),
//!         ..Default::default()
//!     })
//!     .send()
//!     .await
//!     .expect_consistent();
//!
//! assert_eq!(slot, Ok(332_577_897_u64));
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![forbid(missing_docs)]

#[cfg(not(target_arch = "wasm32"))]
pub mod fixtures;
mod request;

use crate::request::{
    GetAccountInfoRequest, GetBalanceRequest, GetBlockRequest, GetRecentPrioritizationFeesRequest,
    GetRecentPrioritizationFeesRequestBuilder, GetSignatureStatusesRequest,
    GetSignatureStatusesRequestBuilder, GetSlotRequest, GetTokenAccountBalanceRequest,
    GetTransactionRequest, JsonRequest, SendTransactionRequest,
};
use async_trait::async_trait;
use candid::{utils::ArgumentEncoder, CandidType, Principal};
use ic_cdk::api::{
    call::RejectionCode,
    management_canister::schnorr::{
        SchnorrAlgorithm, SchnorrKeyId, SignWithSchnorrArgument, SignWithSchnorrResponse,
    },
};
pub use request::{Request, RequestBuilder, SolRpcEndpoint, SolRpcRequest};
use serde::de::DeserializeOwned;
use sol_rpc_types::{
    CommitmentLevel, GetAccountInfoParams, GetBalanceParams, GetBlockParams,
    GetRecentPrioritizationFeesParams, GetSignatureStatusesParams, GetSlotParams, GetSlotRpcConfig,
    GetTokenAccountBalanceParams, GetTransactionParams, Lamport, Pubkey, RpcConfig, RpcError,
    RpcResult, RpcSources, SendTransactionParams, SignTransactionRequestParams, Signature, Slot,
    SolanaCluster, SupportedRpcProvider, SupportedRpcProviderId, TokenAmount, TransactionDetails,
    TransactionInfo,
};
use solana_account_decoder_client_types::token::UiTokenAmount;
use solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta;
use std::{fmt::Debug, sync::Arc};

// Source: https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#fees-for-the-t-schnorr-production-key
const SIGN_WITH_SCHNORR_FEE: u128 = 26_153_846_153;

/// The principal identifying the productive Solana RPC canister under NNS control.
///
/// ```rust
/// use candid::Principal;
/// use sol_rpc_client::SOL_RPC_CANISTER;
///
/// assert_eq!(SOL_RPC_CANISTER, Principal::from_text("tghme-zyaaa-aaaar-qarca-cai").unwrap())
/// ```
pub const SOL_RPC_CANISTER: Principal = Principal::from_slice(&[0, 0, 0, 0, 2, 48, 4, 68, 1, 1]);

/// Abstract the canister runtime so that the client code can be reused:
/// * in production using `ic_cdk`,
/// * in unit tests by mocking this trait,
/// * in integration tests by implementing this trait for `PocketIc`.
#[async_trait]
pub trait Runtime {
    /// Defines how asynchronous inter-canister update calls are made.
    async fn update_call<In, Out>(
        &self,
        id: Principal,
        method: &str,
        args: In,
        cycles: u128,
    ) -> Result<Out, (RejectionCode, String)>
    where
        In: ArgumentEncoder + Send,
        Out: CandidType + DeserializeOwned;

    /// Defines how asynchronous inter-canister query calls are made.
    async fn query_call<In, Out>(
        &self,
        id: Principal,
        method: &str,
        args: In,
    ) -> Result<Out, (RejectionCode, String)>
    where
        In: ArgumentEncoder + Send,
        Out: CandidType + DeserializeOwned;
}

/// Client to interact with the SOL RPC canister.
#[derive(Debug)]
pub struct SolRpcClient<R> {
    config: Arc<ClientConfig<R>>,
}

impl<R> Clone for SolRpcClient<R> {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
        }
    }
}

impl<R> SolRpcClient<R> {
    /// Creates a [`ClientBuilder`] to configure a [`SolRpcClient`].
    pub fn builder(runtime: R, sol_rpc_canister: Principal) -> ClientBuilder<R> {
        ClientBuilder::new(runtime, sol_rpc_canister)
    }

    /// Returns a reference to the client's runtime.
    pub fn runtime(&self) -> &R {
        &self.config.runtime
    }
}

impl SolRpcClient<IcRuntime> {
    /// Creates a [`ClientBuilder`] to configure a [`SolRpcClient`] targeting [`SOL_RPC_CANISTER`]
    /// running on the Internet Computer.
    pub fn builder_for_ic() -> ClientBuilder<IcRuntime> {
        ClientBuilder::new(IcRuntime, SOL_RPC_CANISTER)
    }
}

/// Client to interact with the SOL RPC canister.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ClientConfig<R> {
    runtime: R,
    sol_rpc_canister: Principal,
    rpc_config: Option<RpcConfig>,
    default_commitment_level: Option<CommitmentLevel>,
    rpc_sources: RpcSources,
}

/// A [`ClientBuilder`] to create a [`SolRpcClient`] with custom configuration.
#[must_use]
pub struct ClientBuilder<R> {
    config: ClientConfig<R>,
}

impl<R> ClientBuilder<R> {
    fn new(runtime: R, sol_rpc_canister: Principal) -> Self {
        Self {
            config: ClientConfig {
                runtime,
                sol_rpc_canister,
                rpc_config: None,
                default_commitment_level: None,
                rpc_sources: RpcSources::Default(SolanaCluster::Mainnet),
            },
        }
    }

    /// Modify the existing runtime by applying a transformation function.
    ///
    /// The transformation does not necessarily produce a runtime of the same type.
    pub fn with_runtime<S, F: FnOnce(R) -> S>(self, other_runtime: F) -> ClientBuilder<S> {
        ClientBuilder {
            config: ClientConfig {
                runtime: other_runtime(self.config.runtime),
                sol_rpc_canister: self.config.sol_rpc_canister,
                rpc_config: self.config.rpc_config,
                default_commitment_level: self.config.default_commitment_level,
                rpc_sources: self.config.rpc_sources,
            },
        }
    }

    /// Mutates the builder to use the given [`RpcSources`].
    pub fn with_rpc_sources(mut self, rpc_sources: RpcSources) -> Self {
        self.config.rpc_sources = rpc_sources;
        self
    }

    /// Mutates the builder to use the given [`RpcConfig`].
    pub fn with_rpc_config(mut self, rpc_config: RpcConfig) -> Self {
        self.config.rpc_config = Some(rpc_config);
        self
    }

    /// Mutates the builder to use the given [`CommitmentLevel`].
    ///
    /// All requests made by the built client will use that commitment level.
    /// This can be overridden by each  request.
    pub fn with_default_commitment_level(mut self, commitment_level: CommitmentLevel) -> Self {
        self.config.default_commitment_level = Some(commitment_level);
        self
    }

    /// Creates a [`SolRpcClient`] from the configuration specified in the [`ClientBuilder`].
    pub fn build(self) -> SolRpcClient<R> {
        SolRpcClient {
            config: Arc::new(self.config),
        }
    }
}

impl<R> SolRpcClient<R> {
    /// Call `getAccountInfo` on the SOL RPC canister.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sol_rpc_client::SolRpcClient;
    /// use sol_rpc_types::{RpcSources, SolanaCluster};
    /// use solana_pubkey::pubkey;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use sol_rpc_client::fixtures::usdc_account;
    /// # use sol_rpc_types::{AccountData, AccountEncoding, AccountInfo, MultiRpcResult};
    /// let client = SolRpcClient::builder_for_ic()
    ///     .with_mocked_response(MultiRpcResult::Consistent(Ok(Some(usdc_account()))))
    ///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    ///     .build();
    ///
    /// let usdc_account = client
    ///     .get_account_info(pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"))
    ///     .send()
    ///     .await
    ///     .expect_consistent()
    ///     .unwrap()
    ///     .unwrap();
    ///
    /// assert_eq!(usdc_account.owner, "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string());
    /// # Ok(())
    /// # }
    pub fn get_account_info(
        &self,
        params: impl Into<GetAccountInfoParams>,
    ) -> RequestBuilder<
        R,
        RpcConfig,
        GetAccountInfoParams,
        sol_rpc_types::MultiRpcResult<Option<sol_rpc_types::AccountInfo>>,
        sol_rpc_types::MultiRpcResult<Option<solana_account_decoder_client_types::UiAccount>>,
    > {
        RequestBuilder::new(
            self.clone(),
            GetAccountInfoRequest::new(params.into()),
            10_000_000_000,
        )
    }

    /// Call `getBalance` on the SOL RPC canister.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sol_rpc_client::SolRpcClient;
    /// use sol_rpc_types::{RpcSources, SolanaCluster};
    /// use solana_pubkey::pubkey;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use sol_rpc_types::MultiRpcResult;
    /// let client = SolRpcClient::builder_for_ic()
    /// #   .with_mocked_response(MultiRpcResult::Consistent(Ok(389_086_612_571_u64)))
    ///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    ///     .build();
    ///
    /// let balance = client
    ///     .get_balance(pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"))
    ///     .send()
    ///     .await
    ///     .expect_consistent();
    ///
    /// assert_eq!(balance, Ok(389_086_612_571_u64));
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_balance(
        &self,
        params: impl Into<GetBalanceParams>,
    ) -> RequestBuilder<
        R,
        RpcConfig,
        GetBalanceParams,
        sol_rpc_types::MultiRpcResult<Lamport>,
        sol_rpc_types::MultiRpcResult<Lamport>,
    > {
        RequestBuilder::new(
            self.clone(),
            GetBalanceRequest::new(params.into()),
            10_000_000_000,
        )
    }

    /// Call `getBlock` on the SOL RPC canister.
    pub fn get_block(
        &self,
        params: impl Into<GetBlockParams>,
    ) -> RequestBuilder<
        R,
        RpcConfig,
        GetBlockParams,
        sol_rpc_types::MultiRpcResult<Option<sol_rpc_types::ConfirmedBlock>>,
        sol_rpc_types::MultiRpcResult<
            Option<solana_transaction_status_client_types::UiConfirmedBlock>,
        >,
    > {
        let params = params.into();
        let cycles = match params.transaction_details.unwrap_or_default() {
            TransactionDetails::Signatures => 100_000_000_000,
            TransactionDetails::None => 10_000_000_000,
        };
        RequestBuilder::new(self.clone(), GetBlockRequest::new(params), cycles)
    }

    /// Call `getTokenAccountBalance` on the SOL RPC canister.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sol_rpc_client::SolRpcClient;
    /// use sol_rpc_types::{RpcSources, SolanaCluster};
    /// use solana_pubkey::pubkey;
    /// use solana_account_decoder_client_types::token::UiTokenAmount;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use sol_rpc_types::{MultiRpcResult, TokenAmount};
    /// let client = SolRpcClient::builder_for_ic()
    /// #   .with_mocked_response(MultiRpcResult::Consistent(Ok(TokenAmount {
    /// #       ui_amount: Some(251153323.575906),
    /// #       decimals: 6,
    /// #       amount: "251153323575906".to_string(),
    /// #       ui_amount_string: "251153323.575906".to_string(),
    /// #    })))
    ///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    ///     .build();
    ///
    /// let balance = client
    ///     .get_token_account_balance(pubkey!("3emsAVdmGKERbHjmGfQ6oZ1e35dkf5iYcS6U4CPKFVaa"))
    ///     .send()
    ///     .await
    ///     .expect_consistent();
    ///
    /// assert_eq!(balance, Ok(UiTokenAmount {
    ///         ui_amount: Some(251153323.575906),
    ///         decimals: 6,
    ///         amount: "251153323575906".to_string(),
    ///         ui_amount_string: "251153323.575906".to_string(),
    /// }));
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_token_account_balance(
        &self,
        params: impl Into<GetTokenAccountBalanceParams>,
    ) -> RequestBuilder<
        R,
        RpcConfig,
        GetTokenAccountBalanceParams,
        sol_rpc_types::MultiRpcResult<TokenAmount>,
        sol_rpc_types::MultiRpcResult<UiTokenAmount>,
    > {
        RequestBuilder::new(
            self.clone(),
            GetTokenAccountBalanceRequest::new(params.into()),
            10_000_000_000,
        )
    }

    /// Call `getRecentPrioritizationFees` on the SOL RPC canister.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sol_rpc_client::SolRpcClient;
    /// use sol_rpc_types::{RpcSources, SolanaCluster};
    /// use solana_pubkey::pubkey;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use sol_rpc_types::{MultiRpcResult, PrioritizationFee, TokenAmount};
    /// let client = SolRpcClient::builder_for_ic()
    /// #   .with_mocked_response(MultiRpcResult::Consistent(Ok(vec![PrioritizationFee{slot: 338637772, prioritization_fee: 166667}])))
    ///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    ///     .build();
    ///
    /// let fees = client
    ///     .get_recent_prioritization_fees(&[pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")])
    ///     .unwrap()
    ///     .with_max_length(1)
    ///     .send()
    ///     .await
    ///     .expect_consistent();
    ///
    /// assert_eq!
    ///     (fees,
    ///     Ok(vec![ PrioritizationFee {
    ///         slot: 338637772,
    ///         prioritization_fee: 166667
    ///     }]));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The number of account addresses that can be passed to
    /// [`getRecentPrioritizationFees`](https://solana.com/de/docs/rpc/http/getrecentprioritizationfees)
    /// is limited to 128. More accounts result in an error.
    ///
    /// ```rust
    /// use std::collections::BTreeSet;
    /// use assert_matches::assert_matches;
    /// use solana_pubkey::Pubkey;
    /// use sol_rpc_client::SolRpcClient;
    /// use sol_rpc_types::{RpcSources, SolanaCluster, RpcError};
    ///
    /// let client = SolRpcClient::builder_for_ic()
    ///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    ///     .build();
    ///
    /// let too_many_accounts: BTreeSet<Pubkey> = (0..129_u8)
    ///     .map(|i| Pubkey::from([i; 32]))
    ///     .collect();
    /// assert_eq!(too_many_accounts.len(), 129);
    ///
    /// let err = client.get_recent_prioritization_fees(&too_many_accounts).unwrap_err();
    /// assert_matches!(err, RpcError::ValidationError(_));
    /// ```
    pub fn get_recent_prioritization_fees<'a, I>(
        &self,
        addresses: I,
    ) -> RpcResult<GetRecentPrioritizationFeesRequestBuilder<R>>
    where
        I: IntoIterator<Item = &'a solana_pubkey::Pubkey>,
    {
        let params = GetRecentPrioritizationFeesParams::try_from(
            addresses.into_iter().map(Pubkey::from).collect::<Vec<_>>(),
        )?;
        Ok(RequestBuilder::new(
            self.clone(),
            GetRecentPrioritizationFeesRequest::from(params),
            10_000_000_000,
        ))
    }

    /// Call `getSignatureStatuses` on the SOL RPC canister.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sol_rpc_client::SolRpcClient;
    /// use sol_rpc_types::{RpcSources, SolanaCluster};
    /// use solana_instruction::error::InstructionError;
    /// use solana_signature::Signature;
    /// use solana_transaction_error::TransactionError;
    /// use solana_transaction_status_client_types::{TransactionConfirmationStatus, TransactionStatus};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use std::str::FromStr;
    /// # use sol_rpc_types::MultiRpcResult;
    /// let client = SolRpcClient::builder_for_ic()
    /// #   .with_mocked_response(MultiRpcResult::Consistent(Ok(vec![
    /// #        Some(sol_rpc_types::TransactionStatus {
    /// #            slot: 338837593,
    /// #            status: Ok(()),
    /// #            err: None,
    /// #            confirmation_status: Some(sol_rpc_types::TransactionConfirmationStatus::Finalized),
    /// #        }),
    /// #        Some(sol_rpc_types::TransactionStatus {
    /// #            slot: 338838881,
    /// #            status: Err(sol_rpc_types::TransactionError::InstructionError(2, sol_rpc_types::InstructionError::GenericError)),
    /// #            err: Some(sol_rpc_types::TransactionError::InstructionError(2, sol_rpc_types::InstructionError::GenericError)),
    /// #            confirmation_status: Some(sol_rpc_types::TransactionConfirmationStatus::Finalized),
    /// #        }),
    /// #    ])))
    ///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    ///     .build();
    ///
    /// let statuses = client
    ///     .get_signature_statuses(&[
    ///         Signature::from_str("5iBbqBJzgqafuQn93Np8ztWyXeYe2ReGPzUB1zXP2suZ8b5EaxSwe74ZUhg5pZQuDQkNGW7XApgfXX91YLYUuo5y").unwrap(),
    ///         Signature::from_str("FAAHyQpENs991w9BR7jpwzyXk74jhQWzbsSbjs4NJWkYeL6nggNfT5baWy6eBNLSuqfiiYRGfEC5bhwxUVBZamB").unwrap()
    ///     ])
    ///     .expect("Invalid `getSignatureStatuses` request parameters")
    ///     .send()
    ///     .await
    ///     .expect_consistent();
    ///
    /// assert_eq!(statuses, Ok(vec![
    ///     Some(TransactionStatus {
    ///         slot: 338837593,
    ///         confirmations: None,
    ///         status: Ok(()),
    ///         err: None,
    ///         confirmation_status: Some(TransactionConfirmationStatus::Finalized),
    ///     }),
    ///     Some(TransactionStatus {
    ///         slot: 338838881,
    ///         confirmations: None,
    ///         status: Err(TransactionError::InstructionError(2, InstructionError::GenericError)),
    ///         err: Some(TransactionError::InstructionError(2, InstructionError::GenericError)),
    ///         confirmation_status: Some(TransactionConfirmationStatus::Finalized),
    ///     }),
    /// ]));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The number of signatures that can be passed to
    /// [`getSignatureStatuses`](https://solana.com/de/docs/rpc/http/getsignaturestatuses)
    /// is limited to 256. More signatures result in an error.
    ///
    /// ```rust
    /// use std::{str::FromStr, collections::BTreeSet};
    /// use assert_matches::assert_matches;
    /// use solana_signature::Signature;
    /// use sol_rpc_client::SolRpcClient;
    /// use sol_rpc_types::{RpcSources, SolanaCluster, RpcError};
    ///
    /// let client = SolRpcClient::builder_for_ic()
    ///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    ///     .build();
    ///
    /// let too_many_signatures: BTreeSet<Signature> = (0..257_u16)
    ///     .map(|i| {
    ///         let mut bytes = [0; 64];
    ///         bytes[0] = (i >> 8) as u8;
    ///         bytes[1] = (i & 0xff) as u8;
    ///         Signature::from(bytes)
    ///     })
    ///     .collect();
    /// assert_eq!(too_many_signatures.len(), 257);
    ///
    /// let err = client.get_signature_statuses(&too_many_signatures).unwrap_err();
    /// assert_matches!(err, RpcError::ValidationError(_));
    /// ```
    pub fn get_signature_statuses<'a, I>(
        &self,
        signatures: I,
    ) -> RpcResult<GetSignatureStatusesRequestBuilder<R>>
    where
        I: IntoIterator<Item = &'a solana_signature::Signature>,
    {
        let signatures = signatures.into_iter().collect::<Vec<_>>();
        let num_signatures = signatures.len();
        Ok(RequestBuilder::new(
            self.clone(),
            GetSignatureStatusesRequest::from(GetSignatureStatusesParams::try_from(signatures)?),
            2_000_000_000 + num_signatures as u128 * 1_000_000, // TODO XC-338: Check heuristic
        ))
    }

    /// Call `getSlot` on the SOL RPC canister.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sol_rpc_client::SolRpcClient;
    /// use sol_rpc_types::{CommitmentLevel, GetSlotParams, MultiRpcResult, RpcSources, SolanaCluster};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = SolRpcClient::builder_for_ic()
    /// #   .with_mocked_response(MultiRpcResult::Consistent(Ok(332_577_897_u64)))
    ///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    ///     .build();
    ///
    /// let slot = client
    ///     .get_slot()
    ///     .with_params(GetSlotParams {
    ///         commitment: Some(CommitmentLevel::Finalized),
    ///         ..Default::default()
    ///     })
    ///     .send()
    ///     .await
    ///     .expect_consistent();
    ///
    /// assert_eq!(slot, Ok(332_577_897_u64));
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_slot(
        &self,
    ) -> RequestBuilder<
        R,
        GetSlotRpcConfig,
        Option<GetSlotParams>,
        sol_rpc_types::MultiRpcResult<Slot>,
        sol_rpc_types::MultiRpcResult<Slot>,
    > {
        RequestBuilder::new(self.clone(), GetSlotRequest::default(), 10_000_000_000)
    }

    /// Call `getTransaction` on the SOL RPC canister.
    pub fn get_transaction(
        &self,
        params: impl Into<GetTransactionParams>,
    ) -> RequestBuilder<
        R,
        RpcConfig,
        GetTransactionParams,
        sol_rpc_types::MultiRpcResult<Option<TransactionInfo>>,
        sol_rpc_types::MultiRpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>,
    > {
        RequestBuilder::new(
            self.clone(),
            GetTransactionRequest::new(params.into()),
            10_000_000_000,
        )
    }

    /// Call `sendTransaction` on the SOL RPC canister.
    ///
    /// # Examples
    ///
    /// See the [basic_solana](https://github.com/dfinity/sol-rpc-canister/tree/main/examples/basic_solana) example
    /// to know how to sign a Solana transaction using the [threshold Ed25519 API](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/signatures/signing-messages-t-schnorr).
    ///
    /// ```rust
    /// use sol_rpc_client::SolRpcClient;
    /// use sol_rpc_types::{CommitmentLevel, MultiRpcResult, RpcSources, SendTransactionEncoding, SendTransactionParams, SolanaCluster};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = SolRpcClient::builder_for_ic()
    /// #   .with_mocked_response(MultiRpcResult::Consistent(Ok("tspfR5p1PFphquz4WzDb7qM4UhJdgQXkEZtW88BykVEdX2zL2kBT9kidwQBviKwQuA3b6GMCR1gknHvzQ3r623T")))
    ///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    ///     .build();
    ///
    /// let transaction_id = client
    ///     .send_transaction(SendTransactionParams::from_encoded_transaction(
    ///         "ASy...pwEC".to_string(),
    ///         SendTransactionEncoding::Base64,
    ///     ))
    ///     .send()
    ///     .await
    ///     .expect_consistent();
    ///
    /// assert_eq!(
    ///     transaction_id,
    ///     Ok("tspfR5p1PFphquz4WzDb7qM4UhJdgQXkEZtW88BykVEdX2zL2kBT9kidwQBviKwQuA3b6GMCR1gknHvzQ3r623T".parse().unwrap())
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_transaction<T>(
        &self,
        params: T,
    ) -> RequestBuilder<
        R,
        RpcConfig,
        SendTransactionParams,
        sol_rpc_types::MultiRpcResult<Signature>,
        sol_rpc_types::MultiRpcResult<solana_signature::Signature>,
    >
    where
        T: TryInto<SendTransactionParams>,
        <T as TryInto<SendTransactionParams>>::Error: Debug,
    {
        let params = params
            .try_into()
            .expect("Unable to build request parameters");
        RequestBuilder::new(
            self.clone(),
            SendTransactionRequest::new(params),
            10_000_000_000,
        )
    }

    /// Call `jsonRequest` on the SOL RPC canister.
    ///
    /// This method is useful to send any JSON-RPC request in case the SOL RPC canister
    /// does not offer a Candid API for the requested JSON-RPC method.
    ///
    /// # Examples
    ///
    /// The following example calls `getVersion`:
    ///
    /// ```rust
    /// use sol_rpc_client::SolRpcClient;
    /// use serde_json::json;
    /// use sol_rpc_types::{MultiRpcResult, RpcSources, SolanaCluster};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = SolRpcClient::builder_for_ic()
    /// #    .with_mocked_response(MultiRpcResult::Consistent(Ok(json!({
    /// #            "jsonrpc": "2.0",
    /// #            "result": {
    /// #                "feature-set": 3271415109_u32,
    /// #                "solana-core": "2.1.16"
    /// #            },
    /// #            "id": 1
    /// #        })
    /// #    .to_string())))
    ///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    ///     .build();
    ///
    /// let version: serde_json::Value = client
    ///     .json_request(json!({
    ///             "jsonrpc": "2.0",
    ///             "id": 1,
    ///             "method": "getVersion"
    ///         }))
    ///     .send()
    ///     .await
    ///     .expect_consistent()
    ///     .map(|s| serde_json::from_str(&s).unwrap())
    ///     .unwrap();
    ///
    /// assert_eq!(
    ///     version,
    ///     json!({
    ///         "jsonrpc": "2.0",
    ///         "result": {
    ///             "feature-set": 3271415109_u32,
    ///             "solana-core": "2.1.16"
    ///         },
    ///         "id": 1
    ///     })
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn json_request(
        &self,
        json_request: serde_json::Value,
    ) -> RequestBuilder<
        R,
        RpcConfig,
        String,
        sol_rpc_types::MultiRpcResult<String>,
        sol_rpc_types::MultiRpcResult<String>,
    > {
        RequestBuilder::new(
            self.clone(),
            JsonRequest::try_from(json_request).expect("Client error: invalid JSON request"),
            10_000_000_000,
        )
    }
}

impl<R: Runtime> SolRpcClient<R> {
    /// Call `getProviders` on the SOL RPC canister.
    pub async fn get_providers(&self) -> Vec<(SupportedRpcProviderId, SupportedRpcProvider)> {
        self.config
            .runtime
            .query_call(self.config.sol_rpc_canister, "getProviders", ())
            .await
            .unwrap()
    }

    /// Call `updateApiKeys` on the SOL RPC canister.
    pub async fn update_api_keys(&self, api_keys: &[(SupportedRpcProviderId, Option<String>)]) {
        self.config
            .runtime
            .update_call(
                self.config.sol_rpc_canister,
                "updateApiKeys",
                (api_keys.to_vec(),),
                0,
            )
            .await
            .unwrap()
    }

    async fn execute_request<Config, Params, CandidOutput, Output>(
        &self,
        request: Request<Config, Params, CandidOutput, Output>,
    ) -> Output
    where
        Config: CandidType + Send,
        Params: CandidType + Send,
        CandidOutput: Into<Output> + CandidType + DeserializeOwned,
    {
        self.config
            .runtime
            .update_call::<(RpcSources, Option<Config>, Params), CandidOutput>(
                self.config.sol_rpc_canister,
                request.endpoint.rpc_method(),
                (request.rpc_sources, request.rpc_config, request.params),
                request.cycles,
            )
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "Client error: failed to call `{}`: {e:?}",
                    request.endpoint.rpc_method()
                )
            })
            .into()
    }

    async fn execute_cycles_cost_request<Config, Params, CandidOutput, Output>(
        &self,
        request: Request<Config, Params, CandidOutput, Output>,
    ) -> Output
    where
        Config: CandidType + Send,
        Params: CandidType + Send,
        CandidOutput: Into<Output> + CandidType + DeserializeOwned,
    {
        self.config
            .runtime
            .query_call::<(RpcSources, Option<Config>, Params), CandidOutput>(
                self.config.sol_rpc_canister,
                request.endpoint.cycles_cost_method(),
                (request.rpc_sources, request.rpc_config, request.params),
            )
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "Client error: failed to call `{}`: {e:?}",
                    request.endpoint.cycles_cost_method()
                )
            })
            .into()
    }
}

/// Sign an unsigned Solana transaction with threshold EdDSA, see threshold Schnorr documentation
/// [here](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-schnorr).
///
/// # Examples
///
/// ```rust
/// use solana_hash::Hash;
/// use solana_message::legacy::Message;
/// use solana_program::system_instruction::transfer;
/// use solana_pubkey::pubkey;
/// use solana_signature::Signature;
/// use solana_transaction::Transaction;
/// use sol_rpc_client::{IcRuntime, sign_transaction, SolRpcClient};
/// use sol_rpc_types::{DerivationPath, Ed25519KeyId, SignTransactionRequestParams};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use sol_rpc_client::fixtures::MockRuntime;
/// # use std::str::FromStr;
/// use candid::Principal;
/// # use ic_cdk::api::management_canister::schnorr::SignWithSchnorrResponse;
/// let runtime = IcRuntime;
/// # let runtime = MockRuntime::same_response((SignWithSchnorrResponse {
/// #     signature: "ityU6OGhNgvUXCL8gOy9p0LNThE8eKn4LUPNFwpeQVyXiUmNOzohl0VkcwEQnTqg".to_string().into_bytes(),
/// # },));
///
/// let key_id = Ed25519KeyId::TestKey1;
/// let derivation_path = None;
/// // This pubkey should be derived from the root key `key_id` with `derivation_path`, see:
/// // https://internetcomputer.org/docs/references/ic-interface-spec#ic-schnorr_public_key
/// let payer = pubkey!("3EdRSc7CnKUGxGUSZwJ58rd7haBM8CR2Xh87KheEX7iS");
///
/// let recipient = pubkey!("BPebStjcgCPnWTK3FXZJ8KhqwNYLk9aubC9b4Cgqb6oE");
///
/// // TODO XC-317: Use client method to fetch recent blockhash
/// let recent_blockhash = Hash::new_unique();
///
/// let message = Message::new_with_blockhash(
///     &[transfer(&payer, &recipient, 1_000_000)],
///     Some(&payer),
///     &recent_blockhash,
///  );
///
/// let mut transaction = Transaction::new_unsigned(message);
/// let signature = sign_transaction(
///     &runtime,
///     SignTransactionRequestParams {
///         transaction: transaction.clone(),
///         derivation_path,
///         key_id,
///     },
/// ).await;
///
/// assert_eq!(
///     signature,
///     Ok(Signature::from_str("37HbmunhjSC1xxnVsaFX2xaS8gYnb5JYiLy9B51Ky9Up69aF7Qra6dHSLMCaiurRYq3Y8ZxSVUwC5sntziWuhZee").unwrap())
/// );
///
/// // The transaction is now signed and can be submitted with the `sendTransaction` RPC method.
/// transaction.signatures = vec![signature.unwrap()];
/// # Ok(())
/// # }
/// ```
pub async fn sign_transaction<R: Runtime>(
    runtime: &R,
    params: SignTransactionRequestParams,
) -> RpcResult<solana_signature::Signature> {
    let arg = SignWithSchnorrArgument {
        message: params.transaction.message_data(),
        derivation_path: params.derivation_path.unwrap_or_default().into(),
        key_id: SchnorrKeyId {
            algorithm: SchnorrAlgorithm::Ed25519,
            name: params.key_id.to_string(),
        },
    };
    let (response,): (SignWithSchnorrResponse,) = R::update_call(
        runtime,
        Principal::management_canister(),
        "sign_with_schnorr",
        (arg,),
        SIGN_WITH_SCHNORR_FEE,
    )
    .await
    .map_err(|(rejection_code, message)| {
        RpcError::ValidationError(format!(
            "Failed to sign transaction, management canister returned code {rejection_code:?}: {message}")
        )
    })?;
    solana_signature::Signature::try_from(response.signature).map_err(|bytes| {
        RpcError::ValidationError(format!(
            "Expected signature to contain 64 bytes, got {} bytes",
            bytes.len()
        ))
    })
}

/// Runtime when interacting with a canister running on the Internet Computer.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct IcRuntime;

#[async_trait]
impl Runtime for IcRuntime {
    async fn update_call<In, Out>(
        &self,
        id: Principal,
        method: &str,
        args: In,
        cycles: u128,
    ) -> Result<Out, (RejectionCode, String)>
    where
        In: ArgumentEncoder + Send,
        Out: CandidType + DeserializeOwned,
    {
        ic_cdk::api::call::call_with_payment128(id, method, args, cycles)
            .await
            .map(|(res,)| res)
    }

    async fn query_call<In, Out>(
        &self,
        id: Principal,
        method: &str,
        args: In,
    ) -> Result<Out, (RejectionCode, String)>
    where
        In: ArgumentEncoder + Send,
        Out: CandidType + DeserializeOwned,
    {
        ic_cdk::api::call::call(id, method, args)
            .await
            .map(|(res,)| res)
    }
}
