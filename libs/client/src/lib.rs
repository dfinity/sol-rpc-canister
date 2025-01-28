use candid::Principal;
use sol_rpc_types::{DummyRequest, DummyResponse};

pub struct SolRpcClient {
    sol_rpc_canister: Principal,
}

impl SolRpcClient {
    pub fn new(sol_rpc_canister: Principal) -> Self {
        Self { sol_rpc_canister }
    }

    pub async fn greet(&self, request: DummyRequest) -> DummyResponse {
        ic_cdk::api::call::call_with_payment128(self.sol_rpc_canister, "greet", (request,), 10_000)
            .await
            .map(|(res,)| res)
            .unwrap()
    }
}
