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
//! #   .with_mocked_responses()
//! #   .with_response_for_method("getSlotCyclesCost", Ok::<u128, RpcError>(100_000_000_000))
//! #   .with_response_for_method("getSlot", MultiRpcResult::Consistent(Ok(332_577_897_u64)))
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
//! #   .with_default_mocked_response(MultiRpcResult::Consistent(Ok(332_577_897_u64)))
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
//!
//! ⚠️ **Build Requirements**
//!
//! If you are using the `sol_rpc_client` crate inside a canister, make sure to follow the steps
//! outlined [here](https://github.com/dfinity/sol-rpc-canister/blob/main/libs/client/README.md#build-requirements)
//! to ensure your code compiles.

#![forbid(unsafe_code)]
#![forbid(missing_docs)]

#[cfg(feature = "ed25519")]
pub mod ed25519;
#[cfg(not(target_arch = "wasm32"))]
pub mod fixtures;
pub mod nonce;
mod request;

use crate::request::{
    GetAccountInfoRequest, GetAccountInfoRequestBuilder, GetBalanceRequest,
    GetBalanceRequestBuilder, GetBlockRequest, GetBlockRequestBuilder,
    GetRecentPrioritizationFeesRequest, GetRecentPrioritizationFeesRequestBuilder,
    GetSignatureStatusesRequest, GetSignatureStatusesRequestBuilder,
    GetSignaturesForAddressRequest, GetSignaturesForAddressRequestBuilder, GetSlotRequest,
    GetSlotRequestBuilder, GetTokenAccountBalanceRequest, GetTokenAccountBalanceRequestBuilder,
    GetTransactionRequest, GetTransactionRequestBuilder, JsonRequest, JsonRequestBuilder,
    SendTransactionRequest, SendTransactionRequestBuilder,
};
use async_trait::async_trait;
use candid::{utils::ArgumentEncoder, CandidType, Principal};
use ic_cdk::api::call::RejectionCode;
pub use request::{
    EstimateBlockhashRequestBuilder, EstimateRecentBlockhashError, Request, RequestBuilder,
    SolRpcEndpoint, SolRpcRequest,
};
use serde::de::DeserializeOwned;
use sol_rpc_types::{
    CommitmentLevel, ConsensusStrategy, GetAccountInfoParams, GetBalanceParams, GetBlockParams,
    GetRecentPrioritizationFeesParams, GetSignatureStatusesParams, GetSignaturesForAddressParams,
    GetTokenAccountBalanceParams, GetTransactionParams, Pubkey, RpcConfig, RpcResult, RpcSources,
    SendTransactionParams, SolanaCluster, SupportedRpcProvider, SupportedRpcProviderId,
};
use std::{fmt::Debug, sync::Arc};
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

    /// Mutates the builder to use the given [`ConsensusStrategy`] in the [`RpcConfig`].
    pub fn with_consensus_strategy(mut self, consensus_strategy: ConsensusStrategy) -> Self {
        self.config.rpc_config = Some(RpcConfig {
            response_consensus: Some(consensus_strategy),
            ..self.config.rpc_config.unwrap_or_default()
        });
        self
    }

    /// Mutates the builder to use the given `response_size_estimate` in the [`RpcConfig`].
    pub fn with_response_size_estimate(mut self, response_size_estimate: u64) -> Self {
        self.config.rpc_config = Some(RpcConfig {
            response_size_estimate: Some(response_size_estimate),
            ..self.config.rpc_config.unwrap_or_default()
        });
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
    /// #   .with_default_mocked_response(MultiRpcResult::Consistent(Ok(Some(usdc_account()))))
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
    ) -> GetAccountInfoRequestBuilder<R> {
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
    /// #   .with_default_mocked_response(MultiRpcResult::Consistent(Ok(389_086_612_571_u64)))
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
    pub fn get_balance(&self, params: impl Into<GetBalanceParams>) -> GetBalanceRequestBuilder<R> {
        RequestBuilder::new(
            self.clone(),
            GetBalanceRequest::new(params.into()),
            10_000_000_000,
        )
    }

    /// Call `getBlock` on the SOL RPC canister.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sol_rpc_client::SolRpcClient;
    /// use sol_rpc_types::{ConfirmedBlock, MultiRpcResult, RpcSources, SolanaCluster};
    /// use solana_pubkey::pubkey;
    /// use solana_reward_info::RewardType;
    /// use solana_transaction_status_client_types::{Reward, UiConfirmedBlock};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use std::str::FromStr;
    /// # use sol_rpc_types::{Hash, Pubkey};
    /// let client = SolRpcClient::builder_for_ic()
    /// #   .with_default_mocked_response(MultiRpcResult::Consistent(Ok(ConfirmedBlock {
    /// #     previous_blockhash: Hash::from_str("4yeCoXK2Q4yXcunuLtF37yTE1wVD4x8313adneZDmi8w").unwrap(),
    /// #     blockhash: Hash::from_str("C6Cxgzq6yZWxjYnxwvxvP2dhWFeQSEVxRQbUXG2eMYsY").unwrap(),
    /// #     parent_slot: 343459193,
    /// #     block_time: Some(1748606929),
    /// #     block_height: Some(321673899),
    /// #     signatures: None,
    /// #     rewards: Some(vec![
    /// #         sol_rpc_types::Reward {
    /// #             pubkey: Pubkey::from_str("ChorusmmK7i1AxXeiTtQgQZhQNiXYU84ULeaYF1EH15n").unwrap(),
    /// #             lamports: 29444484,
    /// #             post_balance: 633539232581,
    /// #             reward_type: Some(sol_rpc_types::RewardType::Fee),
    /// #             commission: None,
    /// #         }
    /// #     ]),
    /// #     num_reward_partitions: None,
    /// #     transactions: None,
    /// # })))
    ///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    ///     .build();
    ///
    /// let block = client
    ///     .get_block(343_459_194)
    ///     .send()
    ///     .await
    ///     .expect_consistent();
    ///
    /// assert_eq!(block, Ok(Some(UiConfirmedBlock {
    ///       previous_blockhash: "4yeCoXK2Q4yXcunuLtF37yTE1wVD4x8313adneZDmi8w".to_string(),
    ///       blockhash: "C6Cxgzq6yZWxjYnxwvxvP2dhWFeQSEVxRQbUXG2eMYsY".to_string(),
    ///       parent_slot: 343459193,
    ///       block_time: Some(1748606929),
    ///       block_height: Some(321673899),
    ///       signatures: None,
    ///       rewards: Some(vec![
    ///           Reward {
    ///               pubkey: "ChorusmmK7i1AxXeiTtQgQZhQNiXYU84ULeaYF1EH15n".to_string(),
    ///               lamports: 29444484,
    ///               post_balance: 633539232581,
    ///               reward_type: Some(RewardType::Fee),
    ///               commission: None,
    ///           }
    ///       ]),
    ///       num_reward_partitions: None,
    ///       transactions: None,
    /// })));
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_block(&self, params: impl Into<GetBlockParams>) -> GetBlockRequestBuilder<R> {
        RequestBuilder::new(self.clone(), GetBlockRequest::new(params.into()), 0).update_cycles()
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
    /// #   .with_default_mocked_response(MultiRpcResult::Consistent(Ok(TokenAmount {
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
    ) -> GetTokenAccountBalanceRequestBuilder<R> {
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
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::num::NonZeroU8;
    /// use sol_rpc_types::{MultiRpcResult, PrioritizationFee, TokenAmount};
    /// let client = SolRpcClient::builder_for_ic()
    /// #   .with_default_mocked_response(MultiRpcResult::Consistent(Ok(vec![PrioritizationFee{slot: 338637772, prioritization_fee: 166667}])))
    ///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    ///     .build();
    ///
    /// let fees = client
    ///     .get_recent_prioritization_fees(&[pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")])
    ///     .unwrap()
    ///     .with_max_length(NonZeroU8::MIN)
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

    /// Call `getSignaturesForAddress` on the SOL RPC canister.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sol_rpc_client::SolRpcClient;
    /// use sol_rpc_types::{
    ///     ConfirmedTransactionStatusWithSignature, InstructionError, RpcSources, Signature,
    ///     SolanaCluster, TransactionConfirmationStatus, TransactionError,
    /// };
    /// use solana_pubkey::pubkey;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use std::str::FromStr;
    /// # use sol_rpc_types::MultiRpcResult;
    /// let client = SolRpcClient::builder_for_ic()
    /// #   .with_default_mocked_response(MultiRpcResult::Consistent(Ok(vec![
    /// #        ConfirmedTransactionStatusWithSignature {
    /// #            signature: Signature::from_str("3jPA8CnZb9sfs4zVAypa9KB7VAGwrTdXB6mg9H1H9XpATN6Y8iek4Y21Nb9LjbrpYACbF9USV8RBWvXFFhVoQUAs").unwrap(),
    /// #            confirmation_status: Some(TransactionConfirmationStatus::Finalized),
    /// #            memo: None,
    /// #            slot: 340_372_399,
    /// #            err: None,
    /// #            block_time: Some(1_747_389_084)
    /// #        },
    /// #        ConfirmedTransactionStatusWithSignature {
    /// #            signature: Signature::from_str("3jPA8CnZb9sfs4zVAypa9KB7VAGwrTdXB6mg9H1H9XpATN6Y8iek4Y21Nb9LjbrpYACbF9USV8RBWvXFFhVoQUAs").unwrap(),
    /// #            confirmation_status: Some(TransactionConfirmationStatus::Finalized),
    /// #            memo: None,
    /// #            slot: 340_372_399,
    /// #            err: Some(TransactionError::InstructionError(3, InstructionError::Custom(6_001))),
    /// #            block_time: Some(1_747_389_084)
    /// #        },
    /// #    ])))
    ///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    ///     .build();
    ///
    /// let statuses = client
    ///     .get_signatures_for_address(pubkey!("BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG"))
    ///     .send()
    ///     .await
    ///     .expect_consistent();
    ///
    /// assert_eq!(
    ///     statuses,
    ///     Ok(vec![
    ///         ConfirmedTransactionStatusWithSignature {
    ///             signature: Signature::from_str("3jPA8CnZb9sfs4zVAypa9KB7VAGwrTdXB6mg9H1H9XpATN6Y8iek4Y21Nb9LjbrpYACbF9USV8RBWvXFFhVoQUAs").unwrap(),
    ///             confirmation_status: Some(TransactionConfirmationStatus::Finalized.into()),
    ///             memo: None,
    ///             slot: 340_372_399,
    ///             err: None,
    ///             block_time: Some(1_747_389_084)
    ///         },
    ///         ConfirmedTransactionStatusWithSignature {
    ///             signature: Signature::from_str("3jPA8CnZb9sfs4zVAypa9KB7VAGwrTdXB6mg9H1H9XpATN6Y8iek4Y21Nb9LjbrpYACbF9USV8RBWvXFFhVoQUAs").unwrap(),
    ///             confirmation_status: Some(TransactionConfirmationStatus::Finalized.into()),
    ///             memo: None,
    ///             slot: 340_372_399,
    ///             err: Some(TransactionError::InstructionError(3, InstructionError::Custom(6_001))),
    ///             block_time: Some(1_747_389_084)
    ///         },
    ///     ])
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_signatures_for_address(
        &self,
        params: impl Into<GetSignaturesForAddressParams>,
    ) -> GetSignaturesForAddressRequestBuilder<R> {
        RequestBuilder::new(
            self.clone(),
            GetSignaturesForAddressRequest::from(params.into()),
            2_000_000_000, // TODO XC-338: Check heuristic
        )
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
    /// #   .with_default_mocked_response(MultiRpcResult::Consistent(Ok(vec![
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
    /// #   .with_default_mocked_response(MultiRpcResult::Consistent(Ok(332_577_897_u64)))
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
    pub fn get_slot(&self) -> GetSlotRequestBuilder<R> {
        RequestBuilder::new(self.clone(), GetSlotRequest::default(), 10_000_000_000)
    }

    /// Call `getTransaction` on the SOL RPC canister.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sol_rpc_client::SolRpcClient;
    /// use sol_rpc_types::{RpcSources, SolanaCluster};
    /// use solana_pubkey::pubkey;
    /// use solana_signature::Signature;
    /// use solana_transaction_status_client_types::{
    ///     EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
    ///     EncodedTransactionWithStatusMeta, TransactionBinaryEncoding, UiConfirmedBlock,
    ///     UiLoadedAddresses, UiTransactionStatusMeta, option_serializer::OptionSerializer
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use std::str::FromStr;
    /// # use sol_rpc_types::{ConfirmedBlock, GetTransactionEncoding, Hash, MultiRpcResult, Pubkey};
    /// let client = SolRpcClient::builder_for_ic()
    /// #   .with_default_mocked_response(MultiRpcResult::Consistent(Ok(sol_rpc_types::EncodedConfirmedTransactionWithStatusMeta {
    /// #       slot: 344115445,
    /// #       block_time: Some(1748865196),
    /// #       transaction: sol_rpc_types::EncodedTransactionWithStatusMeta {
    /// #           transaction: sol_rpc_types::EncodedTransaction::Binary(
    /// #               "AezK+RzWcWWx92r0fdmhv7XPAaFQjkPd6MFbGVA7G48aioSd3xcYmwaPC2ih7PwypyeC/9to8mau9B\
    /// #                i7UnL51QUBAAEDCPqP+HgQC9XiKJ57C0YTNM3SFIvOA3aVl/IgkHIZDmuTFuOuQ+TscmAh2ImY30W1\
    /// #                llOzfsPudc98t1jqdNEmVQdhSB01dHS7fE12JOvTvbPYNV5z0RBD/A2jU4AAAAAA97B2Pa9+X8kE7k\
    /// #                E4774GwvI3QCvLgOTJRad8txcXNsUBAgIBAJQBDgAAANXIghQAAAAAHwEfAR4BHQEcARsBGgEZARgB\
    /// #                FwEWARUBFAETARIBEQEQAQ8BDgENAQwBCwEKAQkBCAEHAQYBBQEEAQMBAgEBiNvPO/moMFqBbr9xeM\
    /// #                JF4bBdB8XDJJ5LLsGewMTGlm8BrJA9aAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
    /// #                AA==".to_string(),
    /// #               sol_rpc_types::TransactionBinaryEncoding::Base64,
    /// #           ),
    /// #           version: None,
    /// #           meta: None,
    /// #       }
    /// #   })))
    ///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    ///     .build();
    ///
    /// let signature = Signature::from_str(
    ///     "5jb1Z64pwRu9vNsRrs36ydtYWzw3KtMYfLVkRz56DbBYjYzpfpMbPNtMS7adxGDmjaoDsmKE5MbQM14zjrG6VXVe"
    /// ).unwrap();
    /// let transaction = client
    ///     .get_transaction(signature)
    ///     .with_encoding(GetTransactionEncoding::Base64)
    ///     .send()
    ///     .await
    ///     .expect_consistent();
    ///
    /// match transaction {
    ///     Ok(Some(EncodedConfirmedTransactionWithStatusMeta { transaction, .. })) => {
    ///         assert_eq!(
    ///             transaction.transaction,
    ///             EncodedTransaction::Binary(
    ///                 "AezK+RzWcWWx92r0fdmhv7XPAaFQjkPd6MFbGVA7G48aioSd3xcYmwaPC2ih7PwypyeC/9to8mau9B\
    ///                 i7UnL51QUBAAEDCPqP+HgQC9XiKJ57C0YTNM3SFIvOA3aVl/IgkHIZDmuTFuOuQ+TscmAh2ImY30W1\
    ///                 llOzfsPudc98t1jqdNEmVQdhSB01dHS7fE12JOvTvbPYNV5z0RBD/A2jU4AAAAAA97B2Pa9+X8kE7k\
    ///                 E4774GwvI3QCvLgOTJRad8txcXNsUBAgIBAJQBDgAAANXIghQAAAAAHwEfAR4BHQEcARsBGgEZARgB\
    ///                 FwEWARUBFAETARIBEQEQAQ8BDgENAQwBCwEKAQkBCAEHAQYBBQEEAQMBAgEBiNvPO/moMFqBbr9xeM\
    ///                 JF4bBdB8XDJJ5LLsGewMTGlm8BrJA9aAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
    ///                 AA==".to_string(),
    ///                 TransactionBinaryEncoding::Base64,
    ///             ),
    ///         )
    ///     },
    ///     _ => panic!("Unable to get transaction for signature: `{:?}`", signature)
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_transaction(
        &self,
        params: impl Into<GetTransactionParams>,
    ) -> GetTransactionRequestBuilder<R> {
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
    /// #   .with_default_mocked_response(MultiRpcResult::Consistent(Ok("tspfR5p1PFphquz4WzDb7qM4UhJdgQXkEZtW88BykVEdX2zL2kBT9kidwQBviKwQuA3b6GMCR1gknHvzQ3r623T")))
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
    pub fn send_transaction<T>(&self, params: T) -> SendTransactionRequestBuilder<R>
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
    /// #    .with_default_mocked_response(MultiRpcResult::Consistent(Ok(json!({
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
    pub fn json_request(&self, json_request: serde_json::Value) -> JsonRequestBuilder<R> {
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

    /// Estimate a recent blockhash based on successive calls to `getSlot` and `getBlock`.
    ///
    /// Due to Solana's fast block time, the [`getLatestBlockhash`](https://solana.com/de/docs/rpc/http/getlatestblockhash)
    /// RPC method cannot directly be called by a canister running on the Internet Computer.
    /// Instead, to fetch a recent blockhash (e.g. to build a transaction), one must instead first
    /// get the current slot with `getSlot`, then get the corresponding block with `getBlock`, and
    /// finally extract the blockhash from the resulting block.
    ///
    /// Since `getSlot` can fail due to consensus errors (despite rounding) and `getBlock` can fail
    /// due to not every slot having a block, this method allows retrying to fetch a recent
    /// blockhash until either a blockhash is successfully retrieved, or a maximum number of
    /// attempts is performed. By default, 3 attempts are performed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use solana_hash::Hash;
    /// use sol_rpc_client::{SolRpcClient};
    /// use std::str::FromStr;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use sol_rpc_types::{ConfirmedBlock, MultiRpcResult};
    /// let client = SolRpcClient::builder_for_ic()
    /// #   .with_mocked_responses()
    /// #   .with_response_for_method("getSlot", MultiRpcResult::Consistent(Ok(332_577_897_u64)))
    /// #   .with_response_for_method("getBlock", MultiRpcResult::Consistent(Ok(ConfirmedBlock {
    /// #       previous_blockhash: Default::default(),
    /// #       blockhash: Hash::from_str("C6Cxgzq6yZWxjYnxwvxvP2dhWFeQSEVxRQbUXG2eMYsY").unwrap().into(),
    /// #       parent_slot: 0,
    /// #       block_time: None,
    /// #       block_height: None,
    /// #       signatures: None,
    /// #       rewards: None,
    /// #       num_reward_partitions: None,
    /// #       transactions: None,
    /// #   })))
    ///     .build();
    ///
    /// // Try to fetch a slot and the corresponding block up to 3 times
    /// let blockhash = client
    ///     .estimate_recent_blockhash()
    ///     .send()
    ///     .await;
    ///
    /// assert_eq!(blockhash, Ok(Hash::from_str("C6Cxgzq6yZWxjYnxwvxvP2dhWFeQSEVxRQbUXG2eMYsY").unwrap()));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// ```rust
    /// use solana_hash::Hash;
    /// use sol_rpc_client::{EstimateRecentBlockhashError, SolRpcClient};
    /// use std::num::NonZeroUsize;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use sol_rpc_types::{ConfirmedBlock, MultiRpcResult};
    /// let client = SolRpcClient::builder_for_ic()
    /// #   .with_mocked_responses()
    /// #   .with_response_for_method("getSlot", MultiRpcResult::Consistent(Ok(332_577_897_u64)))
    /// #   .with_response_for_method("getBlock", MultiRpcResult::Consistent(Ok(None::<ConfirmedBlock>)))
    ///     .build();
    ///
    /// let blockhash = client
    ///     .estimate_recent_blockhash()
    ///     .with_num_tries(NonZeroUsize::MIN)
    ///     .send()
    ///     .await;
    ///
    /// // Only one attempt was performed and there was no block for the fetched slot
    /// assert_eq!(blockhash, Err(vec![EstimateRecentBlockhashError::MissingBlock(332_577_897_u64)]));
    /// # Ok(())
    /// # }
    /// ```
    pub fn estimate_recent_blockhash(&self) -> EstimateBlockhashRequestBuilder<R> {
        EstimateBlockhashRequestBuilder::new(self.clone())
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
