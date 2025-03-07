use async_trait::async_trait;
use candid::utils::ArgumentEncoder;
use candid::{decode_args, encode_args, CandidType, Encode, Principal};
use canlog::{Log, LogEntry};
use ic_cdk::api::call::RejectionCode;
use pocket_ic::management_canister::{CanisterId, CanisterSettings};
use pocket_ic::{nonblocking::PocketIc, PocketIcBuilder, UserError, WasmResult};
use serde::de::DeserializeOwned;
use sol_rpc_canister::{
    http_types::{HttpRequest, HttpResponse},
    logs::Priority,
};
use sol_rpc_client::{Runtime, SolRpcClient};
use sol_rpc_types::{InstallArgs, ProviderId};
use std::path::PathBuf;
use std::time::Duration;

pub const DEFAULT_CALLER_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x01]);
pub const DEFAULT_CONTROLLER_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x02]);
pub const ADDITIONAL_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x03]);

pub struct Setup {
    env: PocketIc,
    caller: Principal,
    controller: Principal,
    canister_id: CanisterId,
}

impl Setup {
    pub async fn new() -> Self {
        Self::with_args(InstallArgs::default()).await
    }

    pub async fn with_args(args: InstallArgs) -> Self {
        let env = PocketIcBuilder::new()
            .with_fiduciary_subnet()
            .build_async()
            .await;
        let controller = DEFAULT_CONTROLLER_TEST_ID;
        let canister_id = env
            .create_canister_with_settings(
                None,
                Some(CanisterSettings {
                    controllers: Some(vec![controller]),
                    ..CanisterSettings::default()
                }),
            )
            .await;
        env.add_cycles(canister_id, u128::MAX).await;
        env.install_canister(
            canister_id,
            sol_rpc_wasm(),
            Encode!(&args).unwrap(),
            Some(controller),
        )
        .await;
        let caller = DEFAULT_CALLER_TEST_ID;

        Self {
            env,
            caller,
            controller,
            canister_id,
        }
    }

    pub async fn upgrade_canister(&self, args: InstallArgs) {
        self.env.tick().await;
        // Avoid `CanisterInstallCodeRateLimited` error
        self.env.advance_time(Duration::from_secs(600)).await;
        self.env.tick().await;
        self.env
            .upgrade_canister(
                self.canister_id,
                sol_rpc_wasm(),
                Encode!(&args).unwrap(),
                Some(self.controller),
            )
            .await
            .unwrap_or_else(|err| panic!("Upgrade canister failed: {:?}", err));
    }

    pub fn client(&self) -> SolRpcClient<PocketIcRuntime> {
        SolRpcClient::new(self.new_pocket_ic(), self.canister_id)
    }

    fn new_pocket_ic(&self) -> PocketIcRuntime {
        PocketIcRuntime {
            env: &self.env,
            caller: self.caller,
        }
    }

    pub async fn drop(self) {
        self.env.drop().await
    }

    pub fn controller(&self) -> Principal {
        self.controller
    }
}

fn sol_rpc_wasm() -> Vec<u8> {
    ic_test_utilities_load_wasm::load_wasm(
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../canister"),
        "sol_rpc_canister",
        &[],
    )
}

#[derive(Clone)]
pub struct PocketIcRuntime<'a> {
    env: &'a PocketIc,
    caller: Principal,
}

#[async_trait]
impl Runtime for PocketIcRuntime<'_> {
    async fn update_call<In, Out>(
        &self,
        id: Principal,
        method: &str,
        args: In,
        _cycles: u128,
    ) -> Result<Out, (RejectionCode, String)>
    where
        In: ArgumentEncoder + Send + 'static,
        Out: CandidType + DeserializeOwned + 'static,
    {
        PocketIcRuntime::decode_call_result(
            self.env
                .update_call(id, self.caller, method, PocketIcRuntime::encode_args(args))
                .await,
        )
    }

    async fn query_call<In, Out>(
        &self,
        id: Principal,
        method: &str,
        args: In,
    ) -> Result<Out, (RejectionCode, String)>
    where
        In: ArgumentEncoder + Send + 'static,
        Out: CandidType + DeserializeOwned + 'static,
    {
        PocketIcRuntime::decode_call_result(
            self.env
                .query_call(id, self.caller, method, PocketIcRuntime::encode_args(args))
                .await,
        )
    }
}

impl PocketIcRuntime<'_> {
    fn encode_args<In>(args: In) -> Vec<u8>
    where
        In: ArgumentEncoder,
    {
        encode_args(args).expect("Failed to encode arguments.")
    }

    fn decode_call_result<Out>(
        result: Result<WasmResult, UserError>,
    ) -> Result<Out, (RejectionCode, String)>
    where
        Out: CandidType + DeserializeOwned + 'static,
    {
        match result {
            Ok(WasmResult::Reply(bytes)) => decode_args(&bytes).map(|(res,)| res).map_err(|e| {
                (
                    RejectionCode::CanisterError,
                    format!(
                        "failed to decode canister response as {}: {}",
                        std::any::type_name::<Out>(),
                        e
                    ),
                )
            }),
            Ok(WasmResult::Reject(s)) => Err((RejectionCode::CanisterReject, s)),
            Err(e) => {
                let rejection_code = match e.code as u64 {
                    100..=199 => RejectionCode::SysFatal,
                    200..=299 => RejectionCode::SysTransient,
                    300..=399 => RejectionCode::DestinationInvalid,
                    400..=499 => RejectionCode::CanisterReject,
                    500..=599 => RejectionCode::CanisterError,
                    _ => RejectionCode::Unknown,
                };
                Err((rejection_code, e.description))
            }
        }
    }
}

#[async_trait]
pub trait SolRpcTestClient<R: Runtime> {
    async fn verify_api_key(&self, api_key: (ProviderId, Option<String>));
    async fn retrieve_logs(&self, priority: &str) -> Vec<LogEntry<Priority>>;
    fn with_caller<T: Into<Principal>>(self, id: T) -> Self;
}

#[async_trait]
impl SolRpcTestClient<PocketIcRuntime<'_>> for SolRpcClient<PocketIcRuntime<'_>> {
    async fn verify_api_key(&self, api_key: (ProviderId, Option<String>)) {
        self.runtime
            .query_call(self.sol_rpc_canister, "verifyApiKey", (api_key,))
            .await
            .unwrap()
    }

    async fn retrieve_logs(&self, priority: &str) -> Vec<LogEntry<Priority>> {
        let request = HttpRequest {
            method: "POST".to_string(),
            url: format!("/logs?priority={priority}"),
            headers: vec![],
            body: serde_bytes::ByteBuf::new(),
        };
        let response: HttpResponse = self
            .runtime
            .query_call(self.sol_rpc_canister, "http_request", (request,))
            .await
            .unwrap();
        serde_json::from_slice::<Log<Priority>>(&response.body)
            .expect("failed to parse SOL RPC canister log")
            .entries
    }

    fn with_caller<T: Into<Principal>>(mut self, id: T) -> Self {
        self.runtime.caller = id.into();
        self
    }
}
