//! Client to interact with the SOL RPC canister

#![forbid(unsafe_code)]
#![forbid(missing_docs)]

use async_trait::async_trait;
use candid::{utils::ArgumentEncoder, CandidType, Principal};
use ic_cdk::api::call::RejectionCode;
use serde::de::DeserializeOwned;
use sol_rpc_types::{
    CommitmentLevel, GetSlotParams, RpcConfig, RpcResult, RpcSource, RpcSources, SolanaCluster,
    SupportedRpcProvider, SupportedRpcProviderId,
};
use solana_clock::Slot;

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
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SolRpcClient<R: Runtime> {
    /// This setup's canister [`Runtime`].
    pub runtime: R,
    /// The [`Principal`] of the SOL RPC canister.
    pub sol_rpc_canister: Principal,
}

impl SolRpcClient<IcRuntime> {
    /// Instantiate a new client to be used by a canister on the Internet Computer.
    ///
    /// To use another runtime, see [`Self::new`].
    pub fn new_for_ic(sol_rpc_canister: Principal) -> Self {
        Self {
            runtime: IcRuntime {},
            sol_rpc_canister,
        }
    }
}

impl<R: Runtime> SolRpcClient<R> {
    /// Instantiate a new client with a specific runtime.
    ///
    /// To use the client inside a canister, see [`SolRpcClient<IcRuntime>::new_for_ic`].
    pub fn new(runtime: R, sol_rpc_canister: Principal) -> Self {
        Self {
            runtime,
            sol_rpc_canister,
        }
    }

    /// Call `getProviders` on the SOL RPC canister.
    pub async fn get_providers(&self) -> Vec<(SupportedRpcProviderId, SupportedRpcProvider)> {
        self.runtime
            .query_call(self.sol_rpc_canister, "getProviders", ())
            .await
            .unwrap()
    }

    /// Call `updateApiKeys` on the SOL RPC canister.
    pub async fn update_api_keys(&self, api_keys: &[(SupportedRpcProviderId, Option<String>)]) {
        self.runtime
            .update_call(
                self.sol_rpc_canister,
                "updateApiKeys",
                (api_keys.to_vec(),),
                10_000,
            )
            .await
            .unwrap()
    }

    /// Call `getSlot` on the SOL RPC canister.
    pub async fn get_slot(&self) -> sol_rpc_types::MultiRpcResult<Slot> {
        self.runtime
            .update_call(
                self.sol_rpc_canister,
                "getSlot",
                (
                    RpcSources::Default(SolanaCluster::Devnet),
                    None::<RpcConfig>,
                    Some(GetSlotParams {
                        commitment: Option::from(CommitmentLevel::Finalized),
                        min_context_slot: None,
                    }),
                ),
                1_000_000_000,
            )
            .await
            .expect("Client error: failed to call getSlot")
    }

    /// Call `request` on the SOL RPC canister.
    pub async fn request(
        &self,
        service: RpcSource,
        json_rpc_payload: &str,
        max_response_bytes: u64,
        cycles: u128,
    ) -> RpcResult<String> {
        self.runtime
            .update_call(
                self.sol_rpc_canister,
                "request",
                (service, json_rpc_payload, max_response_bytes),
                cycles,
            )
            .await
            .unwrap()
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
