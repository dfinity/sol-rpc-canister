//! Client to interact with the SOL RPC canister

#![forbid(unsafe_code)]
#![forbid(missing_docs)]

use async_trait::async_trait;
use candid::utils::ArgumentEncoder;
use candid::{CandidType, Principal};
use ic_cdk::api::call::RejectionCode;
use serde::de::DeserializeOwned;
use sol_rpc_types::{DummyRequest, DummyResponse};

/// Abstract the canister runtime so that the client code can be reused:
/// * in production using `ic_cdk`,
/// * in unit tests by mocking this trait,
/// * in integration tests by implementing this trait for `PocketIc`.
#[async_trait]
pub trait Runtime {
    /// Defines how asynchronous inter-canister calls are made.
    async fn call<In, Out>(
        &self,
        id: Principal,
        method: &str,
        args: In,
        cycles: u128,
    ) -> Result<Out, (RejectionCode, String)>
    where
        In: ArgumentEncoder + Send + 'static,
        Out: CandidType + DeserializeOwned + 'static;
}

/// Client to interact with the SOL RPC canister.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SolRpcClient<R: Runtime> {
    runtime: R,
    sol_rpc_canister: Principal,
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

    /// Call `greet` on the SOL RPC canister.
    pub async fn greet(&self, request: DummyRequest) -> DummyResponse {
        self.runtime
            .call(self.sol_rpc_canister, "greet", (request,), 10_000)
            .await
            .unwrap()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct IcRuntime {}

#[async_trait]
impl Runtime for IcRuntime {
    async fn call<In, Out>(
        &self,
        id: Principal,
        method: &str,
        args: In,
        cycles: u128,
    ) -> Result<Out, (RejectionCode, String)>
    where
        In: ArgumentEncoder + Send + 'static,
        Out: CandidType + DeserializeOwned + 'static,
    {
        ic_cdk::api::call::call_with_payment128(id, method, args, cycles)
            .await
            .map(|(res,)| res)
    }
}
