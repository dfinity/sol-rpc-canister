use candid::{encode_one, Encode, Principal};
use canlog::{Log, LogEntry};
use ic_canister_runtime::{CyclesWalletRuntime, Runtime};
use ic_http_types::{HttpRequest, HttpResponse};
use ic_management_canister_types::{CanisterId, CanisterSettings};
use ic_metrics_assert::{MetricsAssert, PocketIcAsyncHttpQuery};
use ic_pocket_canister_runtime::{MockHttpOutcalls, PocketIcRuntime};
use num_traits::ToPrimitive;
use pocket_ic::{nonblocking::PocketIc, PocketIcBuilder};
use sol_rpc_canister::logs::Priority;
use sol_rpc_client::{ClientBuilder, SolRpcClient};
use sol_rpc_types::{InstallArgs, RpcAccess, SupportedRpcProviderId};
use std::{
    env::{set_var, var},
    path::PathBuf,
    time::Duration,
};

pub const DEFAULT_CALLER_TEST_ID: Principal =
    Principal::from_slice(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x31, 0x1, 0x8, 0x1, 0x1]);
pub const DEFAULT_CONTROLLER_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x02]);
const MOCK_API_KEY: &str = "mock-api-key";

pub struct Setup {
    env: PocketIc,
    controller: Principal,
    sol_rpc_canister_id: CanisterId,
    wallet_canister_id: CanisterId,
}

impl Setup {
    pub async fn new() -> Self {
        Self::with_args(InstallArgs::default()).await
    }

    pub async fn with_args(args: InstallArgs) -> Self {
        Self::with_pocket_ic_and_args(
            PocketIcBuilder::new()
                .with_fiduciary_subnet()
                .build_async()
                .await,
            args,
        )
        .await
    }

    pub async fn with_pocket_ic_and_args(env: PocketIc, args: InstallArgs) -> Self {
        let controller = DEFAULT_CONTROLLER_TEST_ID;
        let wallet = DEFAULT_CALLER_TEST_ID;

        let sol_rpc_canister_id = env
            .create_canister_with_settings(
                None,
                Some(CanisterSettings {
                    controllers: Some(vec![controller]),
                    ..CanisterSettings::default()
                }),
            )
            .await;
        env.add_cycles(sol_rpc_canister_id, u64::MAX as u128).await;
        env.install_canister(
            sol_rpc_canister_id,
            sol_rpc_wasm(),
            Encode!(&args).unwrap(),
            Some(controller),
        )
        .await;

        let wallet_canister_id = env
            .create_canister_with_id(
                None,
                Some(CanisterSettings {
                    controllers: Some(vec![controller]),
                    ..CanisterSettings::default()
                }),
                wallet,
            )
            .await
            .unwrap();
        env.add_cycles(wallet_canister_id, u64::MAX as u128).await;
        env.install_canister(wallet_canister_id, wallet_wasm(), vec![], Some(controller))
            .await;

        Self {
            env,
            controller,
            sol_rpc_canister_id,
            wallet_canister_id,
        }
    }

    pub async fn upgrade_canister(&self, args: InstallArgs) {
        self.env.tick().await;
        // Avoid `CanisterInstallCodeRateLimited` error
        self.env.advance_time(Duration::from_secs(600)).await;
        self.env.tick().await;
        self.env
            .upgrade_canister(
                self.sol_rpc_canister_id,
                sol_rpc_wasm(),
                Encode!(&args).unwrap(),
                Some(self.controller),
            )
            .await
            .unwrap_or_else(|err| panic!("Upgrade canister failed: {:?}", err));
    }

    pub async fn get_canister_cycle_balance(&self) -> u128 {
        self.env.cycle_balance(self.sol_rpc_canister_id).await
    }

    pub async fn with_mock_api_keys(self) -> Self {
        let client = self.client(MockHttpOutcalls::never()).build();
        let providers = client.get_providers().await;
        let mut api_keys = Vec::new();
        for (id, provider) in providers {
            match provider.access {
                RpcAccess::Authenticated { .. } => {
                    api_keys.push((id, Some(MOCK_API_KEY.to_string())));
                }
                RpcAccess::Unauthenticated { .. } => {}
            }
        }
        self.env
            .update_call(
                self.sol_rpc_canister_id,
                self.controller,
                "updateApiKeys",
                encode_one(api_keys).expect("Failed to encode arguments."),
            )
            .await
            .expect("BUG: Failed to call updateApiKeys");
        self
    }

    pub async fn verify_api_key(&self, api_key: (SupportedRpcProviderId, Option<String>)) {
        let runtime = self.new_pocket_ic_runtime();
        runtime
            .query_call(self.sol_rpc_canister_id, "verifyApiKey", (api_key,))
            .await
            .unwrap()
    }

    pub async fn retrieve_logs(&self, priority: &str) -> Vec<LogEntry<Priority>> {
        let request = HttpRequest {
            method: "POST".to_string(),
            url: format!("/logs?priority={priority}"),
            headers: vec![],
            body: serde_bytes::ByteBuf::new(),
        };
        let runtime = self.new_pocket_ic_runtime();
        let response: HttpResponse = runtime
            .query_call(self.sol_rpc_canister_id, "http_request", (request,))
            .await
            .unwrap();
        serde_json::from_slice::<Log<Priority>>(&response.body)
            .expect("failed to parse SOL RPC canister log")
            .entries
    }

    pub fn client(
        &self,
        mocks: impl Into<MockHttpOutcalls>,
    ) -> ClientBuilder<CyclesWalletRuntime<PocketIcRuntime<'_>>> {
        self.client_without_mocks().with_runtime(|wallet_runtime| {
            wallet_runtime
                .with_runtime(|pocket_ic_runtime| pocket_ic_runtime.with_http_mocks(mocks.into()))
        })
    }

    pub fn client_without_mocks(&self) -> ClientBuilder<CyclesWalletRuntime<PocketIcRuntime<'_>>> {
        SolRpcClient::builder(self.new_pocket_ic_runtime(), self.sol_rpc_canister_id)
    }

    pub async fn sol_rpc_canister_cycles_balance(&self) -> u128 {
        self.env
            .canister_status(self.sol_rpc_canister_id, Some(self.controller))
            .await
            .unwrap()
            .cycles
            .0
            .to_u128()
            .unwrap()
    }

    fn new_pocket_ic_runtime(&self) -> CyclesWalletRuntime<PocketIcRuntime<'_>> {
        CyclesWalletRuntime::new(
            PocketIcRuntime::new(&self.env, self.controller),
            self.wallet_canister_id,
        )
    }

    pub async fn drop(self) {
        self.env.drop().await
    }

    pub fn controller(&self) -> Principal {
        self.controller
    }

    pub fn sol_rpc_canister_id(&self) -> CanisterId {
        self.sol_rpc_canister_id
    }

    pub async fn check_metrics(self) -> MetricsAssert<Self> {
        MetricsAssert::from_async_http_query(self).await
    }
}

impl PocketIcAsyncHttpQuery for Setup {
    fn get_pocket_ic(&self) -> &PocketIc {
        &self.env
    }

    fn get_canister_id(&self) -> CanisterId {
        self.sol_rpc_canister_id
    }
}

impl AsRef<PocketIc> for Setup {
    fn as_ref(&self) -> &PocketIc {
        &self.env
    }
}

fn sol_rpc_wasm() -> Vec<u8> {
    ic_test_utilities_load_wasm::load_wasm(
        PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap()).join("../canister"),
        "sol_rpc_canister",
        &[],
    )
}

fn wallet_wasm() -> Vec<u8> {
    if var("WALLET_WASM_PATH").is_err() {
        unsafe {
            set_var(
                "WALLET_WASM_PATH",
                PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap()).join("wallet.wasm.gz"),
            )
        }
    };
    ic_test_utilities_load_wasm::load_wasm(PathBuf::new(), "wallet", &[])
}
