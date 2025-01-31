mod ed25519;
mod solana_rpc_canister;
mod solana_wallet;
mod state;

use crate::{
    solana_rpc_canister::{transform_http_request, SolanaRpcCanister},
    solana_wallet::SolanaWallet,
    state::{init_state, read_state},
};
use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::{
    api::management_canister::http_request::{HttpResponse, TransformArgs},
    init, query, update,
};
use num::{BigUint, ToPrimitive};
use solana_program::{message::Message, system_instruction};
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use std::{fmt::Display, str::FromStr};

const SOL_RPC: SolanaRpcCanister = SolanaRpcCanister;

#[init]
pub fn init(maybe_init: Option<InitArg>) {
    if let Some(init_arg) = maybe_init {
        init_state(init_arg)
    }
}

#[update]
pub async fn solana_account(owner: Option<Principal>) -> String {
    let caller = validate_caller_not_anonymous();
    let owner = owner.unwrap_or(caller);
    let wallet = SolanaWallet::new(owner).await;
    wallet.solana_account().to_string()
}

#[update]
pub async fn nonce_account(owner: Option<Principal>) -> String {
    let caller = validate_caller_not_anonymous();
    let owner = owner.unwrap_or(caller);
    let wallet = SolanaWallet::new(owner).await;
    wallet.derived_nonce_account().to_string()
}

#[update]
pub async fn get_balance(account: Option<String>) -> Nat {
    let account = account.unwrap_or(solana_account(None).await);

    let json = format!(
        r#"{{ "jsonrpc": "2.0", "method": "getBalance", "params": ["{}"], "id": 1 }}"#,
        account
    );

    let solana_network = read_state(|s| s.solana_network());

    let max_response_size_bytes = 500_u64;
    let num_cycles = 1_000_000_000u128;

    let response = SOL_RPC
        .json_rpc_request(solana_network, json, num_cycles, max_response_size_bytes)
        .await;

    // The response to a successful `getBalance` call has the following format:
    // { "id": "[ID]", "jsonrpc": "2.0", "result": { "context": { "slot": [SLOT] } }, "value": [BALANCE] }, }
    let balance = response["result"]["value"].as_u64().unwrap();

    Nat(BigUint::from(balance))
}

#[update]
pub async fn send_sol(to: String, amount: Nat) -> String {
    let solana_network = read_state(|s| s.solana_network());

    let max_response_size_bytes = 500_u64;
    let num_cycles = 1_000_000_000u128;

    let caller = validate_caller_not_anonymous();
    let wallet = SolanaWallet::new(caller).await;

    let to = Pubkey::from_str(to.as_str()).unwrap();
    let from = wallet.solana_account();
    let amount = amount.0.to_u64().unwrap();

    let instruction = system_instruction::transfer(from.as_ref(), &to, amount);
    let blockhash = SOL_RPC
        .get_latest_blockhash(solana_network, num_cycles, max_response_size_bytes)
        .await;

    let message = Message::new_with_blockhash(&[instruction], Some(from.as_ref()), &blockhash);
    let signatures = vec![wallet.sign_with_ed25519(&message, &from).await];
    let transaction = Transaction {
        message,
        signatures,
    };

    SOL_RPC
        .send_transaction(
            solana_network,
            num_cycles,
            max_response_size_bytes,
            transaction,
        )
        .await
}

#[update]
pub async fn create_nonce_account() -> String {
    let solana_network = read_state(|s| s.solana_network());

    let max_response_size_bytes = 500_u64;
    let num_cycles = 1_000_000_000u128;

    let caller = validate_caller_not_anonymous();
    let wallet = SolanaWallet::new(caller).await;

    let payer = wallet.solana_account();
    let nonce_account = wallet.derived_nonce_account();

    let instructions = system_instruction::create_nonce_account(
        payer.as_ref(),
        nonce_account.as_ref(),
        payer.as_ref(),
        1_500_000,
    );
    let blockhash = SOL_RPC
        .get_latest_blockhash(solana_network, num_cycles, max_response_size_bytes)
        .await;

    let instruction = instructions.as_slice();
    let message = Message::new_with_blockhash(instruction, Some(payer.as_ref()), &blockhash);

    let signatures = vec![
        wallet.sign_with_ed25519(&message, &payer).await,
        wallet.sign_with_ed25519(&message, &nonce_account).await,
    ];
    let transaction = Transaction {
        message,
        signatures,
    };

    SOL_RPC
        .send_transaction(
            solana_network,
            num_cycles,
            max_response_size_bytes,
            transaction,
        )
        .await
}

// TODO: Remove!
#[query(name = "__transform_json_rpc", hidden = true)]
fn transform(args: TransformArgs) -> HttpResponse {
    transform_http_request(args)
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct InitArg {
    pub solana_network: Option<SolanaNetwork>,
    pub ed5519_key_name: Option<Ed25519KeyName>,
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum SolanaNetwork {
    Mainnet,
    #[default]
    Devnet,
    Testnet,
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub enum Ed25519KeyName {
    #[default]
    TestKeyLocalDevelopment,
    TestKey1,
    ProductionKey1,
}

impl Display for Ed25519KeyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Ed25519KeyName::TestKeyLocalDevelopment => "dfx_test_key",
            Ed25519KeyName::TestKey1 => "test_key_1",
            Ed25519KeyName::ProductionKey1 => "key_1",
        }
        .to_string();
        write!(f, "{}", str)
    }
}

pub fn validate_caller_not_anonymous() -> Principal {
    let principal = ic_cdk::caller();
    if principal == Principal::anonymous() {
        panic!("anonymous principal is not allowed");
    }
    principal
}
