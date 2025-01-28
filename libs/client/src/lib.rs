use async_trait::async_trait;
use candid::utils::ArgumentEncoder;
use candid::{CandidType, Principal};
use ic_cdk::api::call::RejectionCode;
use serde::de::DeserializeOwned;
use sol_rpc_types::{DummyRequest, DummyResponse};

#[async_trait]
pub trait Runtime {
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

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SolRpcClient<R: Runtime> {
    runtime: R,
    sol_rpc_canister: Principal,
}

impl SolRpcClient<IcRuntime> {
    pub fn new_for_ic(sol_rpc_canister: Principal) -> Self {
        Self {
            runtime: IcRuntime {},
            sol_rpc_canister,
        }
    }
}

impl<R: Runtime> SolRpcClient<R> {
    pub fn new(runtime: R, sol_rpc_canister: Principal) -> Self {
        Self {
            runtime,
            sol_rpc_canister,
        }
    }

    pub async fn greet(&self, request: DummyRequest) -> DummyResponse {
        self.runtime
            .call(self.sol_rpc_canister, "greet", (request,), 10_000)
            .await
            .map(|(res,)| res)
            .unwrap()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct IcRuntime {}

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
