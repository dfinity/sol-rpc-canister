use async_trait::async_trait;
use candid::utils::ArgumentEncoder;
use candid::{decode_args, encode_args, CandidType, Decode, Encode, Principal};
use ic_cdk::api::call::RejectionCode;
use pocket_ic::common::rest::{
    CanisterHttpReject, CanisterHttpRequest, CanisterHttpResponse, MockCanisterHttpResponse,
    RawMessageId,
};
use pocket_ic::management_canister::{CanisterId, CanisterSettings};
use pocket_ic::{nonblocking::PocketIc, PocketIcBuilder, UserError, WasmResult};
use serde::de::DeserializeOwned;
use sol_rpc_client::{Runtime, SolRpcClient};
use sol_rpc_types::{InstallArgs, ProviderId, RpcResult, RpcService};
use std::marker::PhantomData;
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

#[derive(Clone)]
pub struct PocketIcRuntime<'a> {
    env: &'a PocketIc,
    caller: Principal,
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
    async fn request(
        &self,
        service: RpcService,
        json_rpc_payload: &str,
        max_response_bytes: u64,
    ) -> CallFlow<'_, RpcResult<String>>;
    async fn verify_api_key(&self, api_key: (ProviderId, Option<String>));
    fn with_caller<T: Into<Principal>>(self, id: T) -> Self;
}

#[async_trait]
impl SolRpcTestClient<PocketIcRuntime<'_>> for SolRpcClient<PocketIcRuntime<'_>> {
    async fn request(
        &self,
        service: RpcService,
        json_rpc_payload: &str,
        max_response_bytes: u64,
    ) -> CallFlow<'_, RpcResult<String>> {
        CallFlow::from_update(
            self.runtime.clone(),
            self.sol_rpc_canister,
            "request",
            Encode!(&service, &json_rpc_payload, &max_response_bytes).unwrap(),
        )
        .await
    }

    async fn verify_api_key(&self, api_key: (ProviderId, Option<String>)) {
        self.runtime
            .query_call(self.sol_rpc_canister, "verifyApiKey", (api_key,))
            .await
            .unwrap()
    }

    fn with_caller<T: Into<Principal>>(mut self, id: T) -> Self {
        self.runtime.caller = id.into();
        self
    }
}

pub struct CallFlow<'a, R> {
    runtime: PocketIcRuntime<'a>,
    method: String,
    message_id: RawMessageId,
    phantom: PhantomData<R>,
}

impl<'a, R: CandidType + DeserializeOwned> CallFlow<'a, R> {
    pub async fn from_update(
        runtime: PocketIcRuntime<'a>,
        canister_id: CanisterId,
        method: &str,
        input: Vec<u8>,
    ) -> Self {
        let message_id = runtime
            .env
            .submit_call(canister_id, runtime.caller, method, input)
            .await
            .expect("failed to submit call");
        CallFlow::new(runtime, method, message_id)
    }

    pub fn new(
        runtime: PocketIcRuntime<'a>,
        method: impl ToString,
        message_id: RawMessageId,
    ) -> Self {
        Self {
            runtime,
            method: method.to_string(),
            message_id,
            phantom: Default::default(),
        }
    }

    pub async fn mock_http(self, mock: impl Into<MockOutcall>) -> Self {
        let mock = mock.into();
        self.mock_http_once_inner(&mock).await;
        while self.try_mock_http_inner(&mock).await {}
        self
    }

    pub async fn mock_http_n_times(self, mock: impl Into<MockOutcall>, count: u32) -> Self {
        let mock = mock.into();
        for _ in 0..count {
            self.mock_http_once_inner(&mock).await;
        }
        self
    }

    pub async fn mock_http_once(self, mock: impl Into<MockOutcall>) -> Self {
        let mock = mock.into();
        self.mock_http_once_inner(&mock).await;
        self
    }

    async fn mock_http_once_inner(&self, mock: &MockOutcall) {
        if !self.try_mock_http_inner(mock).await {
            panic!("no pending HTTP request")
        }
    }

    async fn try_mock_http_inner(&self, mock: &MockOutcall) -> bool {
        let http_requests = tick_until_http_request(self.runtime.env).await;
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
        self.runtime
            .env
            .mock_canister_http_response(mock_response)
            .await;
        true
    }

    pub async fn wait(self) -> R {
        match self
            .runtime
            .env
            .await_call(self.message_id)
            .await
            .unwrap_or_else(|err| {
                panic!("error during update call to `{}()`: {}", self.method, err)
            }) {
            WasmResult::Reply(bytes) => {
                Decode!(&bytes, R).expect("error while decoding Candid response from update call")
            }
            result => {
                panic!("Expected a successful reply, got {:?}", result)
            }
        }
    }
}
