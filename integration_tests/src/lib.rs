use async_trait::async_trait;
use candid::utils::ArgumentEncoder;
use candid::{decode_args, encode_args, CandidType, Encode, Principal};
use ic_cdk::api::call::RejectionCode;
use pocket_ic::management_canister::{CanisterId, CanisterSettings};
use pocket_ic::{nonblocking::PocketIc, PocketIcBuilder, UserError, WasmResult};
use serde::de::DeserializeOwned;
use sol_rpc_client::{Runtime, SolRpcClient};
use sol_rpc_types::InstallArgs;
use std::path::PathBuf;

pub struct Setup {
    env: PocketIc,
    caller: Principal,
    _controller: Principal,
    canister_id: CanisterId,
}

impl Setup {
    pub async fn new() -> Self {
        const DEFAULT_CALLER_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x01]);
        const DEFAULT_CONTROLLER_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x02]);

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
            Encode!(&InstallArgs::default()).unwrap(),
            Some(controller),
        )
        .await;
        let caller = DEFAULT_CALLER_TEST_ID;

        Self {
            env,
            caller,
            _controller: controller,
            canister_id,
        }
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
