use async_trait::async_trait;
use candid::utils::ArgumentEncoder;
use candid::{decode_args, encode_args, CandidType, Encode, Principal};
use ic_cdk::api::call::RejectionCode;
use pocket_ic::management_canister::{CanisterId, CanisterSettings};
use pocket_ic::{nonblocking::PocketIc, PocketIcBuilder, UserError, WasmResult};
use serde::de::DeserializeOwned;
use sol_rpc_client::{Runtime, SolRpcClient};
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
            Encode!().unwrap(),
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
    async fn call_update<In, Out>(
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
        let args_raw = encode_args(args).expect("Failed to encode arguments.");
        self.env
            .update_call(id, self.caller, method, args_raw)
            .await
            .map(map_wasm_result)
            .map_err(map_user_error)
    }

    async fn call_query<In, Out>(
        &self,
        id: Principal,
        method: &str,
        args: In,
    ) -> Result<Out, (RejectionCode, String)>
    where
        In: ArgumentEncoder + Send + 'static,
        Out: CandidType + DeserializeOwned + 'static,
    {
        let args_raw = encode_args(args).expect("Failed to encode arguments.");
        self.env
            .update_call(id, self.caller, method, args_raw)
            .await
            .map(map_wasm_result)
            .map_err(map_user_error)
    }
}

fn map_user_error<Out>(error: UserError) -> (RejectionCode, String)
where
    Out: CandidType + DeserializeOwned + 'static,
{
    let rejection_code = match error.code as u64 {
        100..=199 => RejectionCode::SysFatal,
        200..=299 => RejectionCode::SysTransient,
        300..=399 => RejectionCode::DestinationInvalid,
        400..=499 => RejectionCode::CanisterReject,
        500..=599 => RejectionCode::CanisterError,
        _ => RejectionCode::Unknown,
    };
    (rejection_code, error.description)
}

fn map_wasm_result<Out>(result: WasmResult) -> Result<Out, (RejectionCode, String)>
where
    Out: CandidType + DeserializeOwned + 'static,
{
    match result {
        WasmResult::Reply(bytes) => decode_args(&bytes).map(|(res,)| res).map_err(|e| {
            (
                RejectionCode::CanisterError,
                format!(
                    "failed to decode canister response as {}: {}",
                    std::any::type_name::<Out>(),
                    e
                ),
            )
        }),
        WasmResult::Reject(s) => Err((RejectionCode::CanisterReject, s)),
    }
}
