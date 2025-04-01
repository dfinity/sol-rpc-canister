//! Client to interact with the SOL RPC canister

#![forbid(unsafe_code)]
// #![forbid(missing_docs)]

mod request;

use crate::request::{GetSlotRequest, Request, RequestBuilder, SolRpcEndpoint};
use async_trait::async_trait;
use candid::{utils::ArgumentEncoder, CandidType, Principal};
use ic_cdk::api::call::RejectionCode;
use serde::de::DeserializeOwned;
use sol_rpc_types::{
    GetSlotParams, RpcConfig, RpcSources, SolanaCluster, SupportedRpcProvider,
    SupportedRpcProviderId,
};
use std::sync::Arc;

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
    pub fn builder(runtime: R, sol_rpc_canister: Principal) -> ClientBuilder<R> {
        ClientBuilder {
            config: ClientConfig {
                runtime,
                sol_rpc_canister,
                rpc_config: None,
                rpc_sources: RpcSources::Default(SolanaCluster::Mainnet),
            },
        }
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

#[must_use]
pub struct ClientBuilder<R> {
    config: ClientConfig<R>,
}

impl<R> ClientBuilder<R> {
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

    pub fn build(self) -> SolRpcClient<R> {
        SolRpcClient {
            config: Arc::new(self.config),
        }
    }
}

impl<R> SolRpcClient<R> {
    /// Call `getSlot` on the SOL RPC canister.
    pub fn get_slot(&self, params: Option<GetSlotParams>) -> RequestBuilder<R, GetSlotRequest> {
        self.rpc_request(GetSlotRequest::from(params), 10_000_000_000)
    }

    fn rpc_request<E>(&self, endpoint: E, cycles: u128) -> RequestBuilder<R, E> {
        let request = Request {
            endpoint,
            rpc_sources: self.config.rpc_sources.clone(),
            rpc_config: self.config.rpc_config.clone(),
            cycles,
        };
        RequestBuilder::new(self.clone(), request)
    }
}

impl SolRpcClient<IcRuntime> {}

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

    /// Call `request` on the SOL RPC canister.
    pub async fn request(
        &self,
        json_rpc_payload: &str,
        cycles: u128,
    ) -> sol_rpc_types::MultiRpcResult<String> {
        self.config
            .runtime
            .update_call(
                self.config.sol_rpc_canister,
                "request",
                (
                    self.config.rpc_sources.clone(),
                    self.config.rpc_config.clone(),
                    json_rpc_payload,
                ),
                cycles,
            )
            .await
            .unwrap()
    }

    pub async fn execute_request<E>(&self, request: Request<E>) -> E::Output
    where
        E: SolRpcEndpoint,
        E::Params: CandidType + Send,
        E::Output: CandidType + DeserializeOwned,
    {
        let rpc_method = request.endpoint.rpc_method().to_string();
        self.config
            .runtime
            .update_call(
                self.config.sol_rpc_canister,
                &rpc_method,
                (
                    request.rpc_sources,
                    request.rpc_config,
                    request.endpoint.params(),
                ),
                request.cycles,
            )
            .await
            .unwrap_or_else(|e| panic!("Client error: failed to call `{rpc_method}`: {e:?}"))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct IcRuntime {}

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
