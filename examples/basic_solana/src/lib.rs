mod ed25519;
mod solana_rpc_canister;
mod solana_wallet;
mod spl;
mod state;

use crate::{
    solana_rpc_canister::{transform_http_request, SolanaRpcCanister},
    solana_wallet::SolanaWallet,
    state::{init_state, read_state},
};
use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::{
    api::management_canister::http_request::{HttpResponse, TransformArgs},
    init, post_upgrade, query, update,
};
use num::{BigUint, ToPrimitive};
use solana_message::Message;
use solana_program::system_instruction;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use std::{fmt::Display, str::FromStr};

const SOL_RPC: SolanaRpcCanister = SolanaRpcCanister;

#[init]
pub fn init(init_arg: InitArg) {
    init_state(init_arg)
}

#[post_upgrade]
fn post_upgrade(init_arg: Option<InitArg>) {
    if let Some(init_arg) = init_arg {
        init_state(init_arg)
    }
}

#[update]
pub async fn solana_account(owner: Option<Principal>) -> String {
    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;
    wallet.solana_account().to_string()
}

#[update]
pub async fn nonce_account(owner: Option<Principal>) -> String {
    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;
    wallet.derived_nonce_account().to_string()
}

#[update]
pub async fn associated_token_account(owner: Option<Principal>, mint_account: String) -> String {
    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let mint = Pubkey::from_str(&mint_account).unwrap();
    let wallet = SolanaWallet::new(owner).await;
    spl::get_associated_token_address(wallet.solana_account().as_ref(), &mint).to_string()
}

#[update]
pub async fn get_balance(account: Option<String>) -> Nat {
    let solana_network = read_state(|s| s.solana_network());
    let max_response_size_bytes = 500_u64;
    let num_cycles = 1_000_000_000u128;

    let account = account.unwrap_or(solana_account(None).await);

    let json = format!(
        r#"{{ "jsonrpc": "2.0", "method": "getBalance", "params": ["{}"], "id": 1 }}"#,
        account
    );

    let response = SOL_RPC
        .json_rpc_request(solana_network, json, num_cycles, max_response_size_bytes)
        .await;

    // The response to a successful `getBalance` call has the following format:
    // { "id": "[ID]", "jsonrpc": "2.0", "result": { "context": { "slot": [SLOT] } }, "value": [BALANCE] }, }
    let balance = response["result"]["value"].as_u64().unwrap();

    Nat(BigUint::from(balance))
}

#[update]
pub async fn get_nonce(account: Option<String>) -> String {
    let solana_network = read_state(|s| s.solana_network());
    let max_response_size_bytes = 500_u64;
    let num_cycles = 1_000_000_000u128;

    let account = account.unwrap_or(nonce_account(None).await);

    let blockhash = SOL_RPC
        .get_nonce_account_blockhash(solana_network, num_cycles, max_response_size_bytes, account)
        .await;

    blockhash.to_string()
}

#[update]
pub async fn get_spl_token_balance(account: Option<String>, mint_account: String) -> String {
    let solana_network = read_state(|s| s.solana_network());
    let max_response_size_bytes = 500_u64;
    let num_cycles = 1_000_000_000u128;

    let account = account.unwrap_or(associated_token_account(None, mint_account).await);

    let json = format!(
        r#"{{ "jsonrpc": "2.0", "method": "getTokenAccountBalance", "params": ["{}"], "id": 1 }}"#,
        account
    );

    let response = SOL_RPC
        .json_rpc_request(solana_network, json, num_cycles, max_response_size_bytes)
        .await;

    // The response to a successful `getTokenAccountBalance` call has the following format:
    // { "id": "[ID]", "jsonrpc": "2.0", "result": { "context": { "slot": [SLOT] } }, "value": [ { "uiAmountString": "FORMATTED AMOUNT" } ] }, }
    response["result"]["value"]["uiAmountString"]
        .as_str()
        .unwrap()
        .to_string()
}

#[update]
pub async fn create_nonce_account(owner: Option<Principal>) -> String {
    let solana_network = read_state(|s| s.solana_network());
    let max_response_size_bytes = 500_u64;
    let num_cycles = 1_000_000_000u128;

    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;

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

#[update]
pub async fn create_associated_token_account(
    owner: Option<Principal>,
    mint_account: String,
) -> String {
    let solana_network = read_state(|s| s.solana_network());
    let max_response_size_bytes = 500_u64;
    let num_cycles = 1_000_000_000u128;

    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;

    let payer = wallet.solana_account();
    let mint = Pubkey::from_str(&mint_account).unwrap();

    let instruction =
        spl::create_associated_token_account_instruction(payer.as_ref(), payer.as_ref(), &mint);
    let blockhash = SOL_RPC
        .get_latest_blockhash(solana_network, num_cycles, max_response_size_bytes)
        .await;

    let message = Message::new_with_blockhash(&[instruction], Some(payer.as_ref()), &blockhash);

    let signatures = vec![wallet.sign_with_ed25519(&message, &payer).await];
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
pub async fn send_sol(owner: Option<Principal>, to: String, amount: Nat) -> String {
    let solana_network = read_state(|s| s.solana_network());
    let max_response_size_bytes = 500_u64;
    let num_cycles = 1_000_000_000u128;

    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;

    let recipient = Pubkey::from_str(&to).unwrap();
    let payer = wallet.solana_account();
    let amount = amount.0.to_u64().unwrap();

    let instruction = system_instruction::transfer(payer.as_ref(), &recipient, amount);
    let blockhash = SOL_RPC
        .get_latest_blockhash(solana_network, num_cycles, max_response_size_bytes)
        .await;

    let message = Message::new_with_blockhash(&[instruction], Some(payer.as_ref()), &blockhash);
    let signatures = vec![wallet.sign_with_ed25519(&message, &payer).await];
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
pub async fn send_sol_with_durable_nonce(
    owner: Option<Principal>,
    to: String,
    amount: Nat,
) -> String {
    let solana_network = read_state(|s| s.solana_network());
    let max_response_size_bytes = 500_u64;
    let num_cycles = 1_000_000_000u128;

    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;

    let recipient = Pubkey::from_str(&to).unwrap();
    let payer = wallet.solana_account();
    let amount = amount.0.to_u64().unwrap();
    let nonce_account = wallet.derived_nonce_account();

    let instructions = &[
        system_instruction::advance_nonce_account(nonce_account.as_ref(), payer.as_ref()),
        system_instruction::transfer(payer.as_ref(), &recipient, amount),
    ];
    let blockhash = SOL_RPC
        .get_nonce_account_blockhash(
            solana_network,
            num_cycles,
            max_response_size_bytes,
            nonce_account.to_string(),
        )
        .await;

    let message = Message::new_with_blockhash(instructions, Some(payer.as_ref()), &blockhash);
    let signatures = vec![wallet.sign_with_ed25519(&message, &payer).await];
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
pub async fn send_spl_token(
    owner: Option<Principal>,
    mint_account: String,
    to: String,
    amount: Nat,
) -> String {
    let solana_network = read_state(|s| s.solana_network());
    let max_response_size_bytes = 500_u64;
    let num_cycles = 1_000_000_000u128;

    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;

    let payer = wallet.solana_account();
    let recipient = Pubkey::from_str(&to).unwrap();
    let mint = Pubkey::from_str(&mint_account).unwrap();
    let amount = amount.0.to_u64().unwrap();

    let from = spl::get_associated_token_address(payer.as_ref(), &mint);
    let to = spl::get_associated_token_address(&recipient, &mint);

    let instruction = spl::transfer_instruction(&from, &to, payer.as_ref(), amount);
    let blockhash = SOL_RPC
        .get_latest_blockhash(solana_network, num_cycles, max_response_size_bytes)
        .await;

    let message = Message::new_with_blockhash(&[instruction], Some(payer.as_ref()), &blockhash);
    let signatures = vec![wallet.sign_with_ed25519(&message, &payer).await];
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
    pub ed25519_key_name: Option<Ed25519KeyName>,
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
