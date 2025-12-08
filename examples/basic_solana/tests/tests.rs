use basic_solana::{Ed25519KeyName, SolanaNetwork};
use candid::{
    decode_args, encode_args, utils::ArgumentEncoder, CandidType, Encode, Nat, Principal,
};
use ic_management_canister_types::{CanisterId, CanisterSettings};
use pocket_ic::{PocketIc, PocketIcBuilder};
use serde::de::DeserializeOwned;
use sol_rpc_types::{
    CommitmentLevel, OverrideProvider, RegexSubstitution, RpcAccess, SupportedRpcProvider,
    SupportedRpcProviderId, TokenAmount,
};
use solana_client::{rpc_client::RpcClient as SolanaRpcClient, rpc_config::RpcTransactionConfig};
use solana_commitment_config::CommitmentConfig;
use solana_hash::Hash;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_program::sysvar;
use solana_pubkey::{pubkey, Pubkey};
use solana_signature::Signature;
use solana_signer::Signer;
use solana_transaction::Transaction;
use std::{env::var, path::PathBuf, sync::Arc, thread, time::Duration};

pub const SENDER: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x42]);
pub const RECEIVER: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x43]);
pub const AIRDROP_AMOUNT: u64 = 1_000_000_000; // 1 SOL
pub const SPL_TOKEN_2022_ID: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

// *NOTE*: Update instructions in README.md if you change this test!
#[test]
fn test_basic_solana() {
    let setup = Setup::new().with_mock_api_keys();
    let basic_solana = setup.basic_solana();

    // ## Step 2: Generating a Solana account
    let sender_solana_account: Pubkey = basic_solana
        .update_call::<_, String>(SENDER, "solana_account", ())
        .parse()
        .expect("Failed to parse public key");

    // ## Step 3: Receiving SOL
    setup.airdrop(&sender_solana_account, AIRDROP_AMOUNT);
    println!("User solana account {sender_solana_account}");

    let receiver_solana_account = basic_solana
        .update_call::<_, String>(RECEIVER, "solana_account", ())
        .parse()
        .expect("Failed to parse public key");
    assert_ne!(sender_solana_account, receiver_solana_account);
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
            SENDER,
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
        .update_call::<_, String>(SENDER, "create_nonce_account", ())
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
            SENDER,
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

    // ## Step 6: Sending Solana Program Library (SPL) tokens
    let (mint_authority, mint_account) = setup.create_spl_token();
    println!("Created SPL token at {mint_account}");
    let sender_associated_token_account: Pubkey = basic_solana
        .update_call::<_, String>(
            SENDER,
            "create_associated_token_account",
            (None::<Principal>, mint_account.to_string()),
        )
        .parse()
        .unwrap();
    setup.wait_for_account_to_exist(&sender_associated_token_account, CommitmentLevel::Confirmed);
    println!("Sender's associated token account {sender_associated_token_account}");

    let receiver_associated_token_account: Pubkey = basic_solana
        .update_call::<_, String>(
            RECEIVER,
            "create_associated_token_account",
            (None::<Principal>, mint_account.to_string()),
        )
        .parse()
        .unwrap();
    setup.wait_for_account_to_exist(
        &receiver_associated_token_account,
        CommitmentLevel::Confirmed,
    );
    println!("Receiver's associated token account {receiver_associated_token_account}");

    for (associated_token_account, owner) in [
        (sender_associated_token_account, sender_solana_account),
        (receiver_associated_token_account, receiver_solana_account),
    ] {
        let token_account = setup
            .solana_client
            .get_token_account(&associated_token_account)
            .unwrap()
            .expect("Missing user's associated token account");
        assert_eq!(token_account.mint, mint_account.to_string());
        assert_eq!(token_account.owner, owner.to_string());
        assert_eq!(token_account.token_amount.amount, "0");
    }

    setup.mint_spl(
        &mint_authority,
        1_000_000_000,
        mint_account,
        sender_associated_token_account,
    );
    let token_account = setup
        .solana_client
        .get_token_account(&sender_associated_token_account)
        .unwrap()
        .expect("Missing user's associated token account");
    assert_eq!(token_account.token_amount.amount, "1000000000");

    let send_spl_tx: Signature = basic_solana
        .update_call::<_, String>(
            SENDER,
            "send_spl_token",
            (
                None::<Principal>,
                mint_account.to_string(),
                receiver_solana_account.to_string(),
                Nat::from(1_000_u16),
            ),
        )
        .parse()
        .unwrap();
    setup.wait_for_transaction_to_have_commitment(&send_spl_tx, CommitmentLevel::Confirmed);
    let token_account = setup
        .solana_client
        .get_token_account(&sender_associated_token_account)
        .unwrap()
        .expect("Missing user's associated token account");
    assert_eq!(token_account.token_amount.amount, "999999000");
    let sender_spl_balance: TokenAmount = basic_solana.update_call(
        SENDER,
        "get_spl_token_balance",
        (
            Some(sender_associated_token_account.to_string()),
            mint_account.to_string(),
        ),
    );
    assert_eq!(
        sender_spl_balance,
        TokenAmount {
            ui_amount: Some(0.999999),
            decimals: 9,
            amount: "999999000".to_string(),
            ui_amount_string: "0.999999".to_string(),
        }
    );
    let token_account = setup
        .solana_client
        .get_token_account(&receiver_associated_token_account)
        .unwrap()
        .expect("Missing receiver's associated token account");
    assert_eq!(token_account.token_amount.amount, "1000");
    let receiver_spl_balance: TokenAmount = basic_solana.update_call(
        RECEIVER,
        "get_spl_token_balance",
        (
            Some(receiver_associated_token_account.to_string()),
            mint_account.to_string(),
        ),
    );
    assert_eq!(
        receiver_spl_balance,
        TokenAmount {
            ui_amount: Some(0.000001),
            decimals: 9,
            amount: "1000".to_string(),
            ui_amount_string: "0.000001".to_string(),
        }
    );
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
            ed25519_key_name: Some(Ed25519KeyName::MainnetProdKey1),
            solana_commitment_level: Some(CommitmentLevel::Confirmed),
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
            .update_call::<_, String>(SENDER, "get_nonce", (Some(nonce_account.to_string()),))
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

    fn create_spl_token(&self) -> (Keypair, Pubkey) {
        const MIN_ACCOUNT_LEN: u8 = 82;

        let mint_authority = Keypair::new();
        self.airdrop(&mint_authority.pubkey(), 1_000_000_000);
        let mint_account = Keypair::new();
        let mint_rent = self
            .solana_client
            .get_minimum_balance_for_rent_exemption(MIN_ACCOUNT_LEN as usize)
            .unwrap();
        let create_mint_account_ix = solana_system_interface::instruction::create_account(
            &mint_authority.pubkey(),
            &mint_account.pubkey(),
            mint_rent,
            MIN_ACCOUNT_LEN as u64,
            &SPL_TOKEN_2022_ID,
        );
        // See https://github.com/solana-program/token-2022/blob/644f0b014cbdb25c11c20ccedfb6e412d399b6dc/program/src/instruction.rs#L1207
        let initialize_mint_ix = {
            let decimals: u8 = 9;
            let mut buf = Vec::with_capacity(35);
            buf.push(0);
            buf.push(decimals);
            buf.extend_from_slice(mint_authority.pubkey().as_ref());
            buf.push(0); //no freeze authority

            Instruction {
                program_id: SPL_TOKEN_2022_ID,
                accounts: vec![
                    AccountMeta::new(mint_account.pubkey(), false),
                    AccountMeta::new_readonly(sysvar::rent::id(), false),
                ],
                data: buf,
            }
        };
        let token_mint = Transaction::new_signed_with_payer(
            &[create_mint_account_ix, initialize_mint_ix],
            Some(&mint_authority.pubkey()),
            &[&mint_authority, &mint_account],
            self.solana_client.get_latest_blockhash().unwrap(),
        );
        self.solana_client
            .send_and_confirm_transaction(&token_mint)
            .unwrap();
        (mint_authority, mint_account.pubkey())
    }

    fn mint_spl(
        &self,
        mint_authority: &Keypair,
        amount: u64,
        mint_account: Pubkey,
        user_associated_token_account: Pubkey,
    ) {
        assert!(
            self.solana_client
                .get_token_account(&user_associated_token_account)
                .unwrap()
                .is_some(),
            "Associated token account {user_associated_token_account} not found"
        );

        let mint_ix = {
            let mut buf = Vec::with_capacity(9);
            buf.push(7);
            buf.extend_from_slice(&amount.to_le_bytes());
            Instruction {
                program_id: SPL_TOKEN_2022_ID,
                accounts: vec![
                    AccountMeta::new(mint_account, false),
                    AccountMeta::new(user_associated_token_account, false),
                    AccountMeta::new_readonly(mint_authority.pubkey(), true),
                ],
                data: buf,
            }
        };

        let mint_spl_tx = Transaction::new_signed_with_payer(
            &[mint_ix],
            Some(&mint_authority.pubkey()),
            &[mint_authority],
            self.solana_client.get_latest_blockhash().unwrap(),
        );
        self.solana_client
            .send_and_confirm_transaction(&mint_spl_tx)
            .unwrap();
    }

    fn wait_for_transaction_to_have_commitment(
        &self,
        transaction: &Signature,
        commitment_level: CommitmentLevel,
    ) {
        let mut num_trials = 0;
        loop {
            num_trials += 1;
            if num_trials > 20 {
                panic!(
                    "Transaction {transaction} does not have desired commitment level {commitment_level:?}",
                );
            }
            let tx = self.solana_client.get_transaction_with_config(
                transaction,
                RpcTransactionConfig {
                    commitment: Some(match commitment_level {
                        CommitmentLevel::Processed => CommitmentConfig::processed(),
                        CommitmentLevel::Confirmed => CommitmentConfig::confirmed(),
                        CommitmentLevel::Finalized => CommitmentConfig::finalized(),
                    }),
                    ..Default::default()
                },
            );
            if tx.is_ok() {
                break;
            }
            thread::sleep(Duration::from_millis(400))
        }
    }

    fn wait_for_account_to_exist(&self, account: &Pubkey, commitment_level: CommitmentLevel) {
        let mut num_trials = 0;
        loop {
            num_trials += 1;
            if num_trials > 20 {
                panic!(
                    "Account {account} does not have desired commitment level {commitment_level:?}",
                );
            }
            let result = self
                .solana_client
                .get_account_with_commitment(
                    account,
                    match commitment_level {
                        CommitmentLevel::Processed => CommitmentConfig::processed(),
                        CommitmentLevel::Confirmed => CommitmentConfig::confirmed(),
                        CommitmentLevel::Finalized => CommitmentConfig::finalized(),
                    },
                )
                .unwrap_or_else(|e| panic!("Failed to retrieve account {account}: {e}"));
            match result.value {
                Some(found_account) if found_account.lamports > 0 => {
                    break;
                }
                _ => {
                    thread::sleep(Duration::from_millis(400));
                    continue;
                }
            }
        }
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
