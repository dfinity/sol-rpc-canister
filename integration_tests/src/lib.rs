use async_trait::async_trait;
use candid::{decode_args, utils::ArgumentEncoder, CandidType, Encode, Principal};
use canhttp::http::json::ConstantSizeId;
use canlog::{Log, LogEntry};
use ic_cdk::api::call::RejectionCode;
use ic_http_types::{HttpRequest, HttpResponse};
use num_traits::ToPrimitive;
use pocket_ic::{
    common::rest::{
        CanisterHttpReject, CanisterHttpRequest, CanisterHttpResponse, MockCanisterHttpResponse,
    },
    management_canister::{CanisterId, CanisterSettings},
    nonblocking::PocketIc,
    PocketIcBuilder, RejectCode, RejectResponse,
};
use serde::de::DeserializeOwned;
use sol_rpc_canister::logs::Priority;
use sol_rpc_client::{ClientBuilder, Runtime, SolRpcClient};
use sol_rpc_types::{InstallArgs, RpcAccess, SupportedRpcProviderId};
use std::{
    env::{set_var, var},
    path::PathBuf,
    time::Duration,
};

pub mod mock;
pub mod spl;
pub mod wallet;

use mock::{MockOutcall, MockOutcallBuilder};
use wallet::CallCanisterArgs;

const DEFAULT_MAX_RESPONSE_BYTES: u64 = 2_000_000;
const MAX_TICKS: usize = 10;
pub const DEFAULT_CALLER_TEST_ID: Principal =
    Principal::from_slice(&[0x0, 0x0, 0x0, 0x0, 0x3, 0x31, 0x1, 0x8, 0x2, 0x2]);
pub const DEFAULT_CONTROLLER_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x02]);
const MOCK_API_KEY: &str = "mock-api-key";

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

    pub async fn with_mock_api_keys(self) -> Self {
        let client = self.client().build();
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
        let args = (api_keys,);
        self.env
            .update_call(
                self.sol_rpc_canister_id,
                self.controller,
                "updateApiKeys",
                encode_args(args),
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

    pub fn client(&self) -> ClientBuilder<PocketIcRuntime> {
        SolRpcClient::builder(self.new_pocket_ic_runtime(), self.sol_rpc_canister_id)
    }

    pub fn client_live_mode(&self) -> ClientBuilder<PocketIcLiveModeRuntime> {
        SolRpcClient::builder(self.new_live_pocket_ic_runtime(), self.sol_rpc_canister_id)
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

    fn new_pocket_ic_runtime(&self) -> PocketIcRuntime {
        PocketIcRuntime {
            env: &self.env,
            caller: self.caller,
            mock_strategy: None,
            controller: self.controller,
            wallet: self.wallet_canister_id,
        }
    }

    fn new_live_pocket_ic_runtime(&self) -> PocketIcLiveModeRuntime {
        PocketIcLiveModeRuntime {
            env: &self.env,
            caller: self.caller,
            controller: self.controller,
            wallet: self.wallet_canister_id,
        }
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
}

async fn tick_until_http_request(env: &PocketIc) -> Vec<CanisterHttpRequest> {
    let mut requests = Vec::new();
    for _ in 0..MAX_TICKS {
        requests = env.get_canister_http().await;
        if !requests.is_empty() {
            break;
        }
        env.tick().await;
        env.advance_time(Duration::from_nanos(1)).await;
    }
    requests
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
        set_var(
            "WALLET_WASM_PATH",
            PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap()).join("wallet.wasm.gz"),
        )
    };
    ic_test_utilities_load_wasm::load_wasm(PathBuf::new(), "wallet", &[])
}

impl AsRef<PocketIc> for Setup {
    fn as_ref(&self) -> &PocketIc {
        &self.env
    }
}

#[derive(Clone)]
pub struct PocketIcRuntime<'a> {
    env: &'a PocketIc,
    caller: Principal,
    mock_strategy: Option<MockStrategy>,
    wallet: Principal,
    controller: Principal,
}

#[async_trait]
impl Runtime for PocketIcRuntime<'_> {
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
        // Forward the call through the wallet canister to attach cycles
        let message_id = self
            .env
            .submit_call(
                self.wallet,
                self.controller,
                "wallet_call128",
                Encode!(&CallCanisterArgs::new(id, method, args, cycles)).unwrap(),
            )
            .await
            .unwrap();
        self.execute_mock().await;
        wallet::decode_cycles_wallet_response(
            self.env
                .await_call(message_id)
                .await
                .map_err(PocketIcRuntime::parse_reject_response)?,
        )
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
        let result = self
            .env
            .query_call(id, self.caller, method, encode_args(args))
            .await
            .map_err(PocketIcRuntime::parse_reject_response);
        decode_call_response(result?)
    }
}

impl PocketIcRuntime<'_> {
    fn with_strategy(self, strategy: MockStrategy) -> Self {
        Self {
            mock_strategy: Some(strategy),
            ..self
        }
    }

    async fn execute_mock(&self) {
        match &self.mock_strategy {
            None => (),
            Some(MockStrategy::Mock(mock)) => {
                self.mock_http_once_inner(mock).await;
                while self.try_mock_http_inner(mock).await {}
            }
            Some(MockStrategy::MockOnce(mock)) => {
                self.mock_http_once_inner(mock).await;
            }
            Some(MockStrategy::MockSequence(mocks)) => {
                for mock in mocks {
                    self.mock_http_once_inner(mock).await;
                }
            }
        }
    }

    async fn mock_http_once_inner(&self, mock: &MockOutcall) {
        if !self.try_mock_http_inner(mock).await {
            panic!("no pending HTTP request")
        }
    }

    async fn try_mock_http_inner(&self, mock: &MockOutcall) -> bool {
        let http_requests = tick_until_http_request(self.env).await;
        let request = match http_requests.first() {
            Some(request) => request,
            None => return false,
        };
        mock.assert_matches(request);

        let response = match mock.response.clone() {
            CanisterHttpResponse::CanisterHttpReply(reply) => {
                let max_response_bytes = request
                    .max_response_bytes
                    .unwrap_or(DEFAULT_MAX_RESPONSE_BYTES);
                if reply.body.len() as u64 > max_response_bytes {
                    //approximate replica behaviour since headers are not accounted for.
                    CanisterHttpResponse::CanisterHttpReject(CanisterHttpReject {
                        reject_code: 1, //SYS_FATAL
                        message: format!(
                            "Http body exceeds size limit of {} bytes.",
                            max_response_bytes
                        ),
                    })
                } else {
                    CanisterHttpResponse::CanisterHttpReply(reply)
                }
            }
            CanisterHttpResponse::CanisterHttpReject(reject) => {
                CanisterHttpResponse::CanisterHttpReject(reject)
            }
        };
        let mock_response = MockCanisterHttpResponse {
            subnet_id: request.subnet_id,
            request_id: request.request_id,
            response,
            additional_responses: vec![],
        };
        self.env.mock_canister_http_response(mock_response).await;
        true
    }

    fn parse_reject_response(response: RejectResponse) -> (RejectionCode, String) {
        let rejection_code = match response.reject_code {
            RejectCode::SysFatal => RejectionCode::SysFatal,
            RejectCode::SysTransient => RejectionCode::SysTransient,
            RejectCode::DestinationInvalid => RejectionCode::DestinationInvalid,
            RejectCode::CanisterReject => RejectionCode::CanisterReject,
            RejectCode::CanisterError => RejectionCode::CanisterError,
            RejectCode::SysUnknown => RejectionCode::Unknown,
        };
        (rejection_code, response.reject_message)
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
    controller: Principal,
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
        In: ArgumentEncoder + Send,
        Out: CandidType + DeserializeOwned,
    {
        // Forward the call through the wallet canister to attach cycles
        let message_id = self
            .env
            .submit_call(
                self.wallet,
                self.controller,
                "wallet_call128",
                Encode!(&CallCanisterArgs::new(id, method, args, cycles)).unwrap(),
            )
            .await
            .unwrap();
        wallet::decode_cycles_wallet_response(
            self.env
                .await_call_no_ticks(message_id)
                .await
                .map_err(PocketIcRuntime::parse_reject_response)?,
        )
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
        let result = self
            .env
            .query_call(id, self.caller, method, encode_args(args))
            .await
            .map_err(PocketIcRuntime::parse_reject_response);
        decode_call_response(result?)
    }
}

pub fn encode_args<In: ArgumentEncoder>(args: In) -> Vec<u8> {
    candid::encode_args(args).expect("Failed to encode arguments.")
}

pub fn decode_call_response<Out>(bytes: Vec<u8>) -> Result<Out, (RejectionCode, String)>
where
    Out: CandidType + DeserializeOwned,
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

#[async_trait]
pub trait SolRpcTestClient<R: Runtime> {
    fn mock_http(self, mock: impl Into<MockOutcall>) -> Self;
    fn mock_http_once(self, mock: impl Into<MockOutcall>) -> Self;
    fn mock_http_sequence(self, mocks: Vec<impl Into<MockOutcall>>) -> Self;
    fn mock_sequential_json_rpc_responses<const N: usize>(
        self,
        status: u16,
        body: serde_json::Value,
    ) -> Self;
}

#[async_trait]
impl SolRpcTestClient<PocketIcRuntime<'_>> for ClientBuilder<PocketIcRuntime<'_>> {
    fn mock_http(self, mock: impl Into<MockOutcall>) -> Self {
        self.with_runtime(|r| r.with_strategy(MockStrategy::Mock(mock.into())))
    }

    fn mock_http_once(self, mock: impl Into<MockOutcall>) -> Self {
        self.with_runtime(|r| r.with_strategy(MockStrategy::MockOnce(mock.into())))
    }

    fn mock_http_sequence(self, mocks: Vec<impl Into<MockOutcall>>) -> Self {
        self.with_runtime(|r| {
            r.with_strategy(MockStrategy::MockSequence(
                mocks.into_iter().map(|mock| mock.into()).collect(),
            ))
        })
    }

    fn mock_sequential_json_rpc_responses<const N: usize>(
        self,
        status: u16,
        body: serde_json::Value,
    ) -> Self {
        let mocks = json_rpc_sequential_id::<N>(body)
            .into_iter()
            .map(|response| MockOutcallBuilder::new(status, &response))
            .collect();
        self.mock_http_sequence(mocks)
    }
}

pub fn json_rpc_sequential_id<const N: usize>(
    response: serde_json::Value,
) -> [serde_json::Value; N] {
    let mut first_id: ConstantSizeId = response["id"]
        .as_str()
        .expect("missing request ID")
        .parse()
        .expect("invalid request ID");
    let mut requests = Vec::with_capacity(N);
    for _ in 0..N {
        let mut next_request = response.clone();
        let new_id = first_id.get_and_increment();
        *next_request.get_mut("id").unwrap() = serde_json::Value::String(new_id.to_string());
        requests.push(next_request);
    }
    requests.try_into().unwrap()
}

#[derive(Clone, Debug)]
enum MockStrategy {
    Mock(MockOutcall),
    MockOnce(MockOutcall),
    MockSequence(Vec<MockOutcall>),
}
