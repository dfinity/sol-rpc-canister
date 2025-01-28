use candid::{Encode, Principal};
use pocket_ic::management_canister::{CanisterId, CanisterSettings};
use pocket_ic::{PocketIc, PocketIcBuilder};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct Setup {
    env: Arc<PocketIc>,
    caller: Principal,
    controller: Principal,
    canister_id: CanisterId,
}

impl Default for Setup {
    fn default() -> Self {
        Self::new()
    }
}

impl Setup {
    pub fn new() -> Self {
        const DEFAULT_CALLER_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x01]);
        const DEFAULT_CONTROLLER_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x02]);

        let env = Arc::new(PocketIcBuilder::new().with_fiduciary_subnet().build());
        let controller = DEFAULT_CONTROLLER_TEST_ID;
        let canister_id = env.create_canister_with_settings(
            None,
            Some(CanisterSettings {
                controllers: Some(vec![controller]),
                ..CanisterSettings::default()
            }),
        );
        env.add_cycles(canister_id, u128::MAX);
        env.install_canister(
            canister_id,
            sol_rpc_wasm(),
            Encode!().unwrap(),
            Some(controller),
        );
        let caller = DEFAULT_CALLER_TEST_ID;

        Self {
            env,
            caller,
            controller,
            canister_id,
        }
    }
}

fn sol_rpc_wasm() -> Vec<u8> {
    ic_test_utilities_load_wasm::load_wasm(
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../canister"),
        "sol_rpc_canister",
        &[],
    )
}
