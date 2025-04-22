use basic_solana::{Ed25519KeyName, SolanaNetwork};
use candid::utils::ArgumentEncoder;
use candid::{decode_args, encode_args, CandidType, Encode, Principal};
use pocket_ic::management_canister::{CanisterId, CanisterSettings};
use pocket_ic::{PocketIc, PocketIcBuilder};
use serde::de::DeserializeOwned;
use sol_rpc_types::{OverrideProvider, RegexSubstitution};
use solana_client::rpc_client::RpcClient as SolanaRpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_pubkey::{pubkey, Pubkey};
use std::env::var;
use std::path::PathBuf;
use std::sync::Arc;

pub const USER: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x42]);
pub const AIRDROP_AMOUNT: u64 = 1_000_000_000; // 1 SOL

// *NOTE*: Update instructions in README.md if you change this test!
#[test]
fn test_basic_solana() {
    let setup = Setup::new();
    let basic_solana = setup.basic_solana();

    let user_solana_account: Pubkey = basic_solana
        .update_call::<_, String>(USER, "solana_account", ())
        .parse()
        .expect("Failed to parse public key");
    assert_eq!(
        user_solana_account,
        pubkey!("FufA3YFUgqDQNj4yKM2HUe9QrmPDwbuwEGEdZ3ueDggS")
    );

    let _airdrop_tx = setup
        .solana_client
        .request_airdrop(&user_solana_account, AIRDROP_AMOUNT)
        .unwrap();
    assert_eq!(
        setup.solana_client.wait_for_balance_with_commitment(
            &user_solana_account,
            Some(AIRDROP_AMOUNT),
            CommitmentConfig::confirmed()
        ),
        Some(AIRDROP_AMOUNT)
    );

    let receiver_solana_account = pubkey!("8HNiduWaBanrBv8c2pgGXZWnpKBdEYuQNHnspqto4yyq");
    assert_ne!(user_solana_account, receiver_solana_account);
    assert_eq!(
        setup
            .solana_client
            .get_balance(&receiver_solana_account)
            .unwrap(),
        0
    );

    let _send_sol_tx: String = basic_solana.update_call(
        USER,
        "send_sol",
        (None::<Principal>, receiver_solana_account.to_string(), 1),
    );
    assert_eq!(
        setup.solana_client.wait_for_balance_with_commitment(
            &receiver_solana_account,
            Some(1),
            CommitmentConfig::confirmed()
        ),
        Some(1)
    );
}

pub struct Setup {
    env: Arc<PocketIc>,
    solana_client: SolanaRpcClient,
    _sol_rpc_canister_id: CanisterId,
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
            solana_network: Some(SolanaNetwork::Devnet),
            ed25519_key_name: Some(Ed25519KeyName::ProductionKey1),
        };
        env.install_canister(
            basic_solana_canister_id,
            basic_solana_wasm(),
            Encode!(&basic_solana_install_args).unwrap(),
            None,
        );

        Self {
            env: Arc::new(env),
            solana_client: SolanaRpcClient::new_with_commitment(
                Self::SOLANA_VALIDATOR_URL,
                // Using confirmed commitment in tests provides faster execution while maintaining
                // sufficient reliability.
                CommitmentConfig::confirmed(),
            ),
            _sol_rpc_canister_id: sol_rpc_canister_id,
            basic_solana_canister_id,
        }
    }

    fn basic_solana(&self) -> Canister {
        Canister {
            env: self.env.clone(),
            id: self.basic_solana_canister_id,
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

pub struct Canister {
    env: Arc<PocketIc>,
    id: CanisterId,
}

impl Canister {
    pub fn update_call<In, Out>(&self, sender: Principal, method: &str, args: In) -> Out
    where
        In: ArgumentEncoder + Send,
        Out: CandidType + DeserializeOwned,
    {
        let result = self
            .env
            .update_call(
                self.id,
                sender,
                method,
                encode_args(args).unwrap_or_else(|e| {
                    panic!("Failed to encode arguments for method {method}: {e}")
                }),
            )
            .unwrap_or_else(|e| panic!("Failed to call method {method}: {e}"));
        let (res,) = decode_args(&result).unwrap_or_else(|e| {
            panic!("Failed to decode canister response for method {method}: {e}")
        });
        res
    }
}
