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
//! ## Overriding client configuration for a specific call
//!
//! It is sometimes desirable to have a custom configuration for a specific call, e.g. to change the amount of cycles attached:
//!
//! ```rust
//! use sol_rpc_client::SolRpcClient;
//! let client = SolRpcClient::builder_for_ic().build();
//!
//! let slot = client.get_slot(None).with_cycles(42).send();
//! ```

#![forbid(unsafe_code)]
#![forbid(missing_docs)]

mod request;

pub use request::{Request, RequestBuilder, SolRpcRequest};
use std::marker::PhantomData;

use async_trait::async_trait;
use candid::{utils::ArgumentEncoder, CandidType, Principal};
use ic_cdk::api::call::RejectionCode;
use serde::de::DeserializeOwned;
use sol_rpc_types::{
    GetSlotParams, GetSlotRpcConfig, RpcConfig, RpcSources, SolanaCluster, SupportedRpcProvider,
    SupportedRpcProviderId,
};
use solana_clock::Slot;
use std::sync::Arc;

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
    /// This setup's canister [`Runtime`].
    pub runtime: R,
    /// The [`Principal`] of the SOL RPC canister.
    pub sol_rpc_canister: Principal,
    /// Configuration for how to perform RPC HTTP calls.
    pub rpc_config: Option<RpcConfig>,
    /// Defines what RPC sources to fetch from.
    pub rpc_sources: RpcSources,
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

    /// Creates a [`SolRpcClient`] from the configuration specified in the [`ClientBuilder`].
    pub fn build(self) -> SolRpcClient<R> {
        SolRpcClient {
            config: Arc::new(self.config),
        }
    }
}

impl<R> SolRpcClient<R> {
    /// Call `getSlot` on the SOL RPC canister.
    pub fn get_slot(
        &self,
        params: Option<GetSlotParams>,
    ) -> RequestBuilder<
        R,
        GetSlotRpcConfig,
        Option<GetSlotParams>,
        sol_rpc_types::MultiRpcResult<Slot>,
    > {
        self.rpc_request("getSlot", params, 10_000_000_000)
    }

    /// Call `request` on the SOL RPC canister.
    pub fn raw_request(
        &self,
        json_request: serde_json::Value,
    ) -> RequestBuilder<R, RpcConfig, String, sol_rpc_types::MultiRpcResult<String>> {
        self.rpc_request(
            "request",
            serde_json::to_string(&json_request).expect("Client error: invalid JSON request"),
            10_000_000_000,
        )
    }

    fn rpc_request<Config: From<RpcConfig>, Params, Output>(
        &self,
        rpc_method: impl Into<String>,
        params: Params,
        cycles: u128,
    ) -> RequestBuilder<R, Config, Params, Output> {
        let request = Request {
            rpc_method: rpc_method.into(),
            rpc_sources: self.config.rpc_sources.clone(),
            rpc_config: self.config.rpc_config.clone().map(Config::from),
            params,
            cycles,
            _marker: PhantomData,
        };
        RequestBuilder::new(self.clone(), request)
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

    async fn execute_request<Config, Params, Output>(
        &self,
        request: Request<Config, Params, Output>,
    ) -> Output
    where
        Config: CandidType + Send,
        Params: CandidType + Send,
        Output: CandidType + DeserializeOwned,
    {
        self.config
            .runtime
            .update_call(
                self.config.sol_rpc_canister,
                &request.rpc_method,
                (request.rpc_sources, request.rpc_config, request.params),
                request.cycles,
            )
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "Client error: failed to call `{}`: {e:?}",
                    request.rpc_method
                )
            })
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
