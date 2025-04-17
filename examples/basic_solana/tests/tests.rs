use basic_solana::{Ed25519KeyName, SolanaNetwork};
use candid::{Decode, Encode, Principal};
use pocket_ic::management_canister::{CanisterId, CanisterSettings};
use pocket_ic::{PocketIc, PocketIcBuilder};
use sol_rpc_types::{OverrideProvider, RegexSubstitution};
use std::env::var;
use std::path::PathBuf;

pub const USER: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x42]);

#[test]
fn test_basic_solana() {
    let setup = Setup::new();

    let solana_account = setup
        .env
        .update_call(
            setup.basic_solana_canister_id,
            USER,
            "solana_account",
            Encode!(&()).unwrap(),
        )
        .expect("Failed to call solana_account");
    let solana_account = Decode!(&solana_account, String).expect("Failed to decode solana_account");
    assert_eq!(solana_account, "FufA3YFUgqDQNj4yKM2HUe9QrmPDwbuwEGEdZ3ueDggS");
}

pub struct Setup {
    env: PocketIc,
    sol_rpc_canister_id: CanisterId,
    basic_solana_canister_id: CanisterId,
}

impl Setup {
    pub const DEFAULT_CONTROLLER: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x02]);
    const SOLANA_VALIDATOR_URL: &'static str = "http://localhost:8899";

    pub fn new() -> Self {
        let env = PocketIcBuilder::new().with_fiduciary_subnet().build();

        let sol_rpc_canister_id = env.create_canister_with_settings(
            None,
            Some(CanisterSettings {
                controllers: Some(vec![Self::DEFAULT_CONTROLLER]),
                ..CanisterSettings::default()
            }),
        );
        env.add_cycles(sol_rpc_canister_id, u64::MAX as u128);
        let sol_rpc_install_args = sol_rpc_types::InstallArgs {
            override_provider: Some(OverrideProvider {
                override_url: Some(RegexSubstitution {
                    pattern: ".*".into(),
                    replacement: Self::SOLANA_VALIDATOR_URL.to_string(),
                }),
            }),
            ..Default::default()
        };
        env.install_canister(
            sol_rpc_canister_id,
            sol_rpc_wasm(),
            Encode!(&sol_rpc_install_args).unwrap(),
            Some(Self::DEFAULT_CONTROLLER),
        );

        let basic_solana_canister_id = env.create_canister();
        env.add_cycles(basic_solana_canister_id, u64::MAX as u128);
        let basic_solana_install_args = basic_solana::InitArg {
            sol_rpc_canister_id: Some(basic_solana_canister_id),
            solana_network: Some(SolanaNetwork::Mainnet),
            ed25519_key_name: Some(Ed25519KeyName::ProductionKey1),
        };
        env.install_canister(
            basic_solana_canister_id,
            basic_solana_wasm(),
            Encode!(&basic_solana_install_args).unwrap(),
            None,
        );

        Self {
            env,
            sol_rpc_canister_id,
            basic_solana_canister_id,
        }
    }
}

impl Default for Setup {
    fn default() -> Self {
        Self::new()
    }
}

fn sol_rpc_wasm() -> Vec<u8> {
    ic_test_utilities_load_wasm::load_wasm(
        PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap()).join("../../canister"),
        "sol_rpc_canister",
        &[],
    )
}

fn basic_solana_wasm() -> Vec<u8> {
    ic_test_utilities_load_wasm::load_wasm(
        PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap()).join("."),
        "basic_solana",
        &[],
    )
}
