use basic_solana::{Ed25519KeyName, SolanaNetwork};
use candid::utils::ArgumentEncoder;
use candid::{decode_args, encode_args, CandidType, Encode, Nat, Principal};
use pocket_ic::management_canister::{CanisterId, CanisterSettings};
use pocket_ic::{PocketIc, PocketIcBuilder};
use serde::de::DeserializeOwned;
use sol_rpc_types::{
    OverrideProvider, RegexSubstitution, RpcAccess, SupportedRpcProvider, SupportedRpcProviderId,
};
use solana_client::rpc_client::RpcClient as SolanaRpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_hash::Hash;
use solana_pubkey::{pubkey, Pubkey};
use solana_signature::Signature;
use std::env::var;
use std::path::PathBuf;
use std::sync::Arc;

pub const USER: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x42]);
pub const AIRDROP_AMOUNT: u64 = 1_000_000_000; // 1 SOL

// *NOTE*: Update instructions in README.md if you change this test!
#[test]
fn test_basic_solana() {
    let setup = Setup::new().with_mock_api_keys();
    let basic_solana = setup.basic_solana();

    // ## Step 2: Generating a Solana account
    let user_solana_account: Pubkey = basic_solana
        .update_call::<_, String>(USER, "solana_account", ())
        .parse()
        .expect("Failed to parse public key");

    // ## Step 3: Receiving SOL
    setup.airdrop(&user_solana_account, AIRDROP_AMOUNT);
    println!("User solana account {user_solana_account}");

    let receiver_solana_account = pubkey!("8HNiduWaBanrBv8c2pgGXZWnpKBdEYuQNHnspqto4yyq");
    assert_ne!(user_solana_account, receiver_solana_account);
    // The receiver account must be Initialized before receiving SOL,
    // which will be done when requesting an airdrop
    setup.airdrop(&receiver_solana_account, AIRDROP_AMOUNT);
    println!("Receiver solana account {receiver_solana_account}");

    // ## Step 4: Sending SOL
    let receiver_balance_before = setup
        .solana_client
        .get_balance(&receiver_solana_account)
        .unwrap();
    let send_sol_tx: Signature = basic_solana
        .update_call::<_, String>(
            USER,
            "send_sol",
            (
                None::<Principal>,
                receiver_solana_account.to_string(),
                Nat::from(1_u8),
            ),
        )
        .parse()
        .unwrap();
    let expected_receiver_balance = receiver_balance_before + 1;
    assert_eq!(
        setup.solana_client.wait_for_balance_with_commitment(
            &receiver_solana_account,
            Some(expected_receiver_balance),
            CommitmentConfig::confirmed()
        ),
        Some(expected_receiver_balance)
    );
    assert!(setup
        .solana_client
        .confirm_transaction(&send_sol_tx)
        .unwrap());

    // ## Step 5: Sending SOL using durable nonces
    let nonce_account: Pubkey = basic_solana
        .update_call::<_, String>(USER, "create_nonce_account", ())
        .parse()
        .unwrap();
    setup.solana_client.wait_for_balance_with_commitment(
        &nonce_account,
        Some(1_500_000),
        CommitmentConfig::confirmed(),
    );
    let nonce_1 = setup.ensure_nonce_consistent(&nonce_account);

    let receiver_balance_before = setup
        .solana_client
        .get_balance(&receiver_solana_account)
        .unwrap();
    let send_sol_tx: Signature = basic_solana
        .update_call::<_, String>(
            USER,
            "send_sol_with_durable_nonce",
            (
                None::<Principal>,
                receiver_solana_account.to_string(),
                Nat::from(1_u8),
            ),
        )
        .parse()
        .unwrap();
    let expected_receiver_balance = receiver_balance_before + 1;
    assert_eq!(
        setup.solana_client.wait_for_balance_with_commitment(
            &receiver_solana_account,
            Some(expected_receiver_balance),
            CommitmentConfig::confirmed()
        ),
        Some(expected_receiver_balance)
    );
    assert!(setup
        .solana_client
        .confirm_transaction(&send_sol_tx)
        .unwrap());
    let nonce_2 = setup.ensure_nonce_consistent(&nonce_account);
    assert_ne!(nonce_1, nonce_2);
}

pub struct Setup {
    env: Arc<PocketIc>,
    solana_client: SolanaRpcClient,
    sol_rpc_canister_id: CanisterId,
    basic_solana_canister_id: CanisterId,
}

impl Setup {
    pub const DEFAULT_CONTROLLER: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x02]);
    const SOLANA_VALIDATOR_URL: &'static str = "http://localhost:8899";

    pub fn new() -> Self {
        let env = PocketIcBuilder::new()
            .with_nns_subnet() //make_live requires NNS subnet.
            .with_fiduciary_subnet()
            .build();

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
            sol_rpc_canister_id: Some(sol_rpc_canister_id),
            solana_network: Some(SolanaNetwork::Devnet),
            ed25519_key_name: Some(Ed25519KeyName::ProductionKey1),
        };
        env.install_canister(
            basic_solana_canister_id,
            basic_solana_wasm(),
            Encode!(&basic_solana_install_args).unwrap(),
            None,
        );

        println!("Basic solana canister ID {basic_solana_canister_id}");
        println!("SOL RPC canister id {sol_rpc_canister_id}");
        let mut env = env;
        let _endpoint = env.make_live(None);
        Self {
            env: Arc::new(env),
            solana_client: SolanaRpcClient::new_with_commitment(
                Self::SOLANA_VALIDATOR_URL,
                // Using confirmed commitment in tests provides faster execution while maintaining
                // sufficient reliability.
                CommitmentConfig::confirmed(),
            ),
            sol_rpc_canister_id,
            basic_solana_canister_id,
        }
    }

    fn with_mock_api_keys(self) -> Self {
        const MOCK_API_KEY: &str = "mock-api-key";
        let sol_rpc = self.sol_rpc();
        let providers: Vec<(SupportedRpcProviderId, SupportedRpcProvider)> =
            sol_rpc.update_call(Principal::anonymous(), "getProviders", ());
        let mut api_keys = Vec::new();
        for (id, provider) in providers {
            match provider.access {
                RpcAccess::Authenticated { .. } => {
                    api_keys.push((id, Some(MOCK_API_KEY.to_string())));
                }
                RpcAccess::Unauthenticated { .. } => {}
            }
        }
        let _res: () = sol_rpc.update_call(Self::DEFAULT_CONTROLLER, "updateApiKeys", (api_keys,));
        self
    }

    fn airdrop(&self, account: &Pubkey, amount: u64) {
        let balance_before = self.solana_client.get_balance(account).unwrap();
        let _airdrop_tx = self.solana_client.request_airdrop(account, amount).unwrap();
        let expected_balance = balance_before + amount;
        assert_eq!(
            self.solana_client.wait_for_balance_with_commitment(
                account,
                Some(expected_balance),
                CommitmentConfig::confirmed()
            ),
            Some(expected_balance)
        );
    }

    fn ensure_nonce_consistent(&self, nonce_account: &Pubkey) -> Hash {
        let expected_nonce: Hash = self
            .basic_solana()
            .update_call::<_, String>(USER, "get_nonce", (Some(nonce_account.to_string()),))
            .parse()
            .unwrap();
        let actual_nonce = solana_rpc_client_nonce_utils::data_from_account(
            &self.solana_client.get_account(nonce_account).unwrap(),
        )
        .unwrap()
        .blockhash();
        assert_eq!(expected_nonce, actual_nonce);
        expected_nonce
    }

    fn sol_rpc(&self) -> Canister {
        Canister {
            env: self.env.clone(),
            id: self.sol_rpc_canister_id,
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
        let message_id = self
            .env
            .submit_call(
                self.id,
                sender,
                method,
                encode_args(args).unwrap_or_else(|e| {
                    panic!("Failed to encode arguments for method {method}: {e}")
                }),
            )
            .unwrap_or_else(|e| panic!("Failed to call method {method}: {e}"));
        let response_bytes = self
            .env
            .await_call_no_ticks(message_id)
            .unwrap_or_else(|e| panic!("Failed to await call for method {method}: {e}"));
        let (res,) = decode_args(&response_bytes).unwrap_or_else(|e| {
            panic!("Failed to decode canister response for method {method}: {e}")
        });
        res
    }
}
