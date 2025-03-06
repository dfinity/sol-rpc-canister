use async_trait::async_trait;
use candid::utils::ArgumentEncoder;
use candid::{decode_args, encode_args, CandidType, Encode, Principal};
use canlog::{Log, LogEntry};
use ic_cdk::api::call::RejectionCode;
use pocket_ic::common::rest::{
    CanisterHttpReject, CanisterHttpRequest, CanisterHttpResponse, MockCanisterHttpResponse,
};
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

pub mod mock;
use mock::MockOutcall;

const DEFAULT_MAX_RESPONSE_BYTES: u64 = 2_000_000;

const MAX_TICKS: usize = 10;
pub const DEFAULT_CALLER_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x01]);
pub const DEFAULT_CONTROLLER_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x02]);
pub const ADDITIONAL_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x03]);

pub struct Setup {
    env: PocketIc,
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

        Self {
            env,
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
        SolRpcClient::new(
            PocketIcRuntime {
                env: &self.env,
                caller: DEFAULT_CALLER_TEST_ID,
                mock_strategy: None,
            },
            self.canister_id,
        )
    }

    pub async fn drop(self) {
        self.env.drop().await
    }

    pub fn controller(&self) -> Principal {
        self.controller
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
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../canister"),
        "sol_rpc_canister",
        &[],
    )
}

pub struct PocketIcRuntime<'a> {
    env: &'a PocketIc,
    caller: Principal,
    mock_strategy: Option<MockStrategy>,
}

#[async_trait]
impl<'a> Runtime for PocketIcRuntime<'a> {
    async fn update_call<In, Out>(
        &self,
        id: Principal,
        method: &str,
        args: In,
        _cycles: u128,
    ) -> Result<Out, (RejectionCode, String)>
    where
        In: ArgumentEncoder + Send,
        Out: CandidType + DeserializeOwned,
    {
        let message_id = self
            .env
            .submit_call(id, self.caller, method, PocketIcRuntime::encode_args(args))
            .await
            .expect("failed to submit call");
        self.execute_mock().await;
        let result: Result<WasmResult, UserError> = self.env.await_call(message_id).await;
        PocketIcRuntime::decode_call_result(result)
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
        Out: CandidType + DeserializeOwned,
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
            Some(MockStrategy::MockNTimes(mock, count)) => {
                for _ in 0..*count {
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
}

#[async_trait]
pub trait SolRpcTestClient {
    async fn verify_api_key(&self, api_key: (ProviderId, Option<String>));
    async fn retrieve_logs(&self, priority: &str) -> Vec<LogEntry<Priority>>;
    fn with_caller<T: Into<Principal>>(self, id: T) -> Self;
    fn mock_http(self, mock: impl Into<MockOutcall>) -> Self;
    fn mock_http_once(self, mock: impl Into<MockOutcall>) -> Self;
    fn mock_http_n_times(self, mock: impl Into<MockOutcall>, count: u32) -> Self;
}

#[async_trait]
impl SolRpcTestClient for SolRpcClient<PocketIcRuntime<'_>> {
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

    fn mock_http(self, mock: impl Into<MockOutcall>) -> Self {
        Self {
            runtime: self.runtime.with_strategy(MockStrategy::Mock(mock.into())),
            ..self
        }
    }

    fn mock_http_once(self, mock: impl Into<MockOutcall>) -> Self {
        Self {
            runtime: self
                .runtime
                .with_strategy(MockStrategy::MockOnce(mock.into())),
            ..self
        }
    }

    fn mock_http_n_times(self, mock: impl Into<MockOutcall>, count: u32) -> Self {
        Self {
            runtime: self
                .runtime
                .with_strategy(MockStrategy::MockNTimes(mock.into(), count)),
            ..self
        }
    }
}

enum MockStrategy {
    Mock(MockOutcall),
    MockOnce(MockOutcall),
    MockNTimes(MockOutcall, u32),
}
