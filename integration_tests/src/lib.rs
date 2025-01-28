use candid::{Encode, Principal};
use pocket_ic::management_canister::{CanisterId, CanisterSettings};
use pocket_ic::{nonblocking::PocketIc, PocketIcBuilder};
use sol_rpc_client::SolRpcClient;
use std::path::PathBuf;

pub struct Setup {
    env: PocketIc,
    caller: Principal,
    controller: Principal,
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
            controller,
            canister_id,
        }
    }

    pub async fn new_with_client() -> (Self, SolRpcClient) {
        let setup = Setup::new().await;
        let client = setup.client();
        (setup, client)
    }

    pub fn client(&self) -> SolRpcClient {
        SolRpcClient::new(self.canister_id)
    }
}

fn sol_rpc_wasm() -> Vec<u8> {
    ic_test_utilities_load_wasm::load_wasm(
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../canister"),
        "sol_rpc_canister",
        &[],
    )
}
