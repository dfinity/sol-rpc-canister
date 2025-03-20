use async_trait::async_trait;
use candid::{decode_args, encode_args, utils::ArgumentEncoder, CandidType, Encode, Principal};
use canlog::{Log, LogEntry};
use ic_cdk::api::call::RejectionCode;
use pocket_ic::{
    management_canister::{CanisterId, CanisterSettings},
    nonblocking::PocketIc,
    PocketIcBuilder, RejectCode, RejectResponse,
};
use regex::Regex;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use sol_rpc_canister::{
    http_types::{HttpRequest, HttpResponse},
    logs::Priority,
};
use sol_rpc_client::{Runtime, SolRpcClient};
use sol_rpc_types::{InstallArgs, SupportedRpcProviderId};
use std::env::{remove_var, set_var};
use std::path::Path;
use std::{path::PathBuf, time::Duration};

pub const DEFAULT_CALLER_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x01]);
pub const DEFAULT_CONTROLLER_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x02]);
pub const ADDITIONAL_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x03]);

pub struct Setup {
    env: PocketIc,
    caller: Principal,
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
        let caller = DEFAULT_CALLER_TEST_ID;
        let sol_rpc_canister_id =
            Self::create_canister(&env, controller, args, sol_rpc_wasm()).await;
        let wallet_canister_id = Self::create_canister(&env, controller, (), wallet_wasm()).await;
        env.update_call(
            wallet_canister_id,
            controller,
            "authorize",
            Encode!(&caller).unwrap(),
        )
        .await
        .expect("Failed to add caller as custodian for wallet canister");
        Self {
            env,
            caller,
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

    pub fn client(&self) -> SolRpcClient<PocketIcRuntime> {
        SolRpcClient::new(self.new_pocket_ic(), self.sol_rpc_canister_id)
    }

    pub fn client_live_mode(&self) -> SolRpcClient<PocketIcLiveModeRuntime> {
        SolRpcClient::new(self.new_live_pocket_ic(), self.sol_rpc_canister_id)
    }

    fn new_pocket_ic(&self) -> PocketIcRuntime {
        PocketIcRuntime {
            env: &self.env,
            caller: self.caller,
        }
    }

    fn new_live_pocket_ic(&self) -> PocketIcLiveModeRuntime {
        PocketIcLiveModeRuntime {
            env: &self.env,
            caller: self.caller,
            wallet: self.wallet_canister_id,
        }
    }

    pub async fn drop(self) {
        self.env.drop().await
    }

    pub fn controller(&self) -> Principal {
        self.controller
    }

    async fn create_canister<T: candid::CandidType>(
        env: &PocketIc,
        controller: Principal,
        install_args: T,
        wasm: Vec<u8>,
    ) -> CanisterId {
        let canister_id = env
            .create_canister_with_settings(
                None,
                Some(CanisterSettings {
                    controllers: Some(vec![controller]),
                    ..CanisterSettings::default()
                }),
            )
            .await;
        // TODO XC-323: Change me! u128::MAX causes a "cycles amount exceeds 64-bit representation" error
        env.add_cycles(canister_id, u64::MAX as u128).await;
        env.install_canister(
            canister_id,
            wasm,
            Encode!(&install_args).unwrap(),
            Some(controller),
        )
        .await;
        canister_id
    }
}

fn sol_rpc_wasm() -> Vec<u8> {
    ic_test_utilities_load_wasm::load_wasm(
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../canister"),
        "sol_rpc_canister",
        &[],
    )
}

fn wallet_wasm() -> Vec<u8> {
    struct DummyPath;

    impl AsRef<Path> for DummyPath {
        fn as_ref(&self) -> &Path {
            panic!("Should not be called!");
        }
    }

    set_var(
        "WALLET_WASM_PATH",
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("wallet.wasm.gz"),
    );
    let wasm = ic_test_utilities_load_wasm::load_wasm(DummyPath, "wallet", &[]);
    remove_var("WALLET_WASM_PATH");
    wasm
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
        result: Result<Vec<u8>, RejectResponse>,
    ) -> Result<Out, (RejectionCode, String)>
    where
        Out: CandidType + DeserializeOwned + 'static,
    {
        match result {
            Ok(bytes) => Self::decode_call_response(bytes),
            Err(e) => {
                let rejection_code = match e.reject_code {
                    RejectCode::SysFatal => RejectionCode::SysFatal,
                    RejectCode::SysTransient => RejectionCode::SysTransient,
                    RejectCode::DestinationInvalid => RejectionCode::DestinationInvalid,
                    RejectCode::CanisterReject => RejectionCode::CanisterReject,
                    RejectCode::CanisterError => RejectionCode::CanisterError,
                    RejectCode::SysUnknown => RejectionCode::Unknown,
                };
                Err((rejection_code, e.reject_message))
            }
        }
    }

    fn decode_call_response<Out>(bytes: Vec<u8>) -> Result<Out, (RejectionCode, String)>
    where
        Out: CandidType + DeserializeOwned + 'static,
    {
        decode_args(&bytes).map(|(res,)| res).map_err(|e| {
            (
                RejectionCode::CanisterError,
                format!(
                    "failed to decode canister response as {}: {}",
                    std::any::type_name::<Out>(),
                    e
                ),
            )
        })
    }
}

/// Runtime for when Pocket IC is used in [live mode](https://github.com/dfinity/ic/blob/f0c82237ae16745ac54dd3838b3f91ce32a6bc52/packages/pocket-ic/HOWTO.md?plain=1#L43).
///
/// The pocket IC instance will automatically progress and execute HTTPs outcalls (without mocking).
/// This setting renders the tests non-deterministic, which is unavoidable since
/// the solana-test-validator also progresses automatically (and also acceptable for end-to-end tests).
#[derive(Clone)]
pub struct PocketIcLiveModeRuntime<'a> {
    env: &'a PocketIc,
    caller: Principal,
    wallet: Principal,
}

#[async_trait]
impl Runtime for PocketIcLiveModeRuntime<'_> {
    async fn update_call<In, Out>(
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
        let args = CallCanisterArgs {
            canister: id,
            method_name: method.to_string(),
            args: PocketIcRuntime::encode_args(args),
            cycles,
        };

        // Forward the call through the wallet canister to attach cycles
        let id = self
            .env
            .submit_call(
                self.wallet,
                self.caller,
                "wallet_call128",
                Encode!(&args).unwrap(),
            )
            .await
            .unwrap();
        let call_result = self.env.await_call_no_ticks(id).await;

        match PocketIcRuntime::decode_call_result::<Result<CallResult, String>>(call_result)? {
            Ok(CallResult { r#return }) => PocketIcRuntime::decode_call_response(r#return),
            Err(message) => {
                let (_, [code, message]) =
                    Regex::new(r"^An error happened during the call: (\d+): (.*)$")
                        .unwrap()
                        .captures(&message)
                        .unwrap_or_else(|| {
                            panic!("Failed to parse error from wallet canister: {}", message)
                        })
                        .extract();
                Err((code.parse::<u32>().unwrap().into(), message.to_string()))
            }
        }
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

#[async_trait]
pub trait SolRpcTestClient<R: Runtime> {
    async fn verify_api_key(&self, api_key: (SupportedRpcProviderId, Option<String>));
    async fn retrieve_logs(&self, priority: &str) -> Vec<LogEntry<Priority>>;
    fn with_caller<T: Into<Principal>>(self, id: T) -> Self;
}

#[async_trait]
impl SolRpcTestClient<PocketIcRuntime<'_>> for SolRpcClient<PocketIcRuntime<'_>> {
    async fn verify_api_key(&self, api_key: (SupportedRpcProviderId, Option<String>)) {
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

/// From the [cycles wallet repository](https://github.com/dfinity/cycles-wallet)
#[derive(CandidType, Deserialize)]
struct CallCanisterArgs<TCycles> {
    canister: Principal,
    method_name: String,
    #[serde(with = "serde_bytes")]
    args: Vec<u8>,
    cycles: TCycles,
}

/// From the [cycles wallet repository](https://github.com/dfinity/cycles-wallet)
#[derive(CandidType, Deserialize)]
struct CallResult {
    #[serde(with = "serde_bytes")]
    r#return: Vec<u8>,
}
