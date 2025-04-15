mod ed25519;
mod solana_wallet;
mod spl;
mod state;
use canhttp::http::json::{ConstantSizeId, Id};
use solana_nonce::{state::State, versions::Versions as NonceVersions};

use crate::{
    solana_wallet::SolanaWallet,
    state::{init_state, read_state},
};
use base64::{prelude::BASE64_STANDARD, Engine};
use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::{init, post_upgrade, update};
use num::{BigUint, ToPrimitive};
use serde_json::json;
use sol_rpc_client::{IcRuntime, SolRpcClient};
use sol_rpc_types::{GetAccountInfoEncoding, GetAccountInfoParams, RpcSources, SolanaCluster};
use solana_account_decoder_client_types::{UiAccountData, UiAccountEncoding};
use solana_hash::Hash;
use solana_message::Message;
use solana_program::system_instruction;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use std::{fmt::Display, str::FromStr};

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
    let account = account.unwrap_or(solana_account(None).await);

    let response = client()
        .json_request(json!({
            "jsonrpc": "2.0",
            "id": Id::from(ConstantSizeId::ZERO),
            "method": "getBalance",
            "params": [ account ]
        }))
        .send()
        .await
        .expect_consistent()
        .expect("Call to `getBalance` failed");

    // The response to a successful `getBalance` call has the following format:
    // { "id": "[ID]", "jsonrpc": "2.0", "result": { "context": { "slot": [SLOT] } }, "value": [BALANCE] }, }
    let balance = serde_json::to_value(response)
        .expect("`getBalance` response is not a valid JSON")["result"]["value"]
        .as_u64()
        .unwrap();

    Nat(BigUint::from(balance))
}

#[update]
pub async fn get_nonce(account: Option<String>) -> String {
    let account = account.unwrap_or(nonce_account(None).await);

    // Fetch the account info with the data encoded in base64 format
    let mut params = GetAccountInfoParams::from_encoded_pubkey(account);
    params.encoding = Some(GetAccountInfoEncoding::Base64);
    let account_data = client()
        .get_account_info(params)
        .send()
        .await
        .expect_consistent()
        .expect("Call to `getAccountInfo` failed")
        .expect("Account not found for given pubkey")
        .data;

    // Extract the nonce from the account data
    let account_data = if let UiAccountData::Binary(blob, UiAccountEncoding::Base64) = account_data
    {
        BASE64_STANDARD
            .decode(blob)
            .expect("Unable to base64 decode account data")
    } else {
        panic!("Invalid response format");
    };
    match bincode::deserialize::<NonceVersions>(account_data.as_slice())
        .expect("Failed to deserialize nonce account data")
        .state()
    {
        State::Uninitialized => panic!("Nonce account is uninitialized"),
        State::Initialized(data) => data.blockhash().to_string(),
    }
}

#[update]
pub async fn get_spl_token_balance(account: Option<String>, mint_account: String) -> String {
    let account = account.unwrap_or(associated_token_account(None, mint_account).await);

    let response = client()
        .json_request(json!({
            "jsonrpc": "2.0",
            "id": Id::from(ConstantSizeId::ZERO),
            "method": "getTokenAccountBalance",
            "params": [ account ]
        }))
        .send()
        .await
        .expect_consistent()
        .expect("Call to `getTokenAccountBalance` failed");

    // The response to a successful `getTokenAccountBalance` call has the following format:
    // { "id": "[ID]", "jsonrpc": "2.0", "result": { "context": { "slot": [SLOT] } }, "value": [ { "uiAmountString": "FORMATTED AMOUNT" } ] }, }
    serde_json::to_value(response).expect("`getTokenAccountBalance` response is not a valid JSON")
        ["result"]["value"]["uiAmountString"]
        .as_str()
        .unwrap()
        .to_string()
}

#[update]
pub async fn create_nonce_account(owner: Option<Principal>) -> String {
    let client = client();

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

    let message = Message::new_with_blockhash(
        instructions.as_slice(),
        Some(payer.as_ref()),
        &get_recent_blockhash(&client).await,
    );

    let signatures = vec![
        wallet.sign_with_ed25519(&message, &payer).await,
        wallet.sign_with_ed25519(&message, &nonce_account).await,
    ];
    let transaction = Transaction {
        message,
        signatures,
    };

    client
        .send_transaction(transaction)
        .send()
        .await
        .expect_consistent()
        .expect("Call to `sendTransaction` failed")
        .to_string()
}

#[update]
pub async fn create_associated_token_account(
    owner: Option<Principal>,
    mint_account: String,
) -> String {
    let client = client();

    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;

    let payer = wallet.solana_account();
    let mint = Pubkey::from_str(&mint_account).unwrap();

    let instruction =
        spl::create_associated_token_account_instruction(payer.as_ref(), payer.as_ref(), &mint);

    let message = Message::new_with_blockhash(
        &[instruction],
        Some(payer.as_ref()),
        &get_recent_blockhash(&client).await,
    );

    let signatures = vec![wallet.sign_with_ed25519(&message, &payer).await];
    let transaction = Transaction {
        message,
        signatures,
    };

    client
        .send_transaction(transaction)
        .send()
        .await
        .expect_consistent()
        .expect("Call to `sendTransaction` failed")
        .to_string()
}

#[update]
pub async fn send_sol(owner: Option<Principal>, to: String, amount: Nat) -> String {
    let client = client();

    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;

    let recipient = Pubkey::from_str(&to).unwrap();
    let payer = wallet.solana_account();
    let amount = amount.0.to_u64().unwrap();

    let instruction = system_instruction::transfer(payer.as_ref(), &recipient, amount);

    let message = Message::new_with_blockhash(
        &[instruction],
        Some(payer.as_ref()),
        &get_recent_blockhash(&client).await,
    );
    let signatures = vec![wallet.sign_with_ed25519(&message, &payer).await];
    let transaction = Transaction {
        message,
        signatures,
    };

    client
        .send_transaction(transaction)
        .send()
        .await
        .expect_consistent()
        .expect("Call to `sendTransaction` failed")
        .to_string()
}

#[update]
pub async fn send_sol_with_durable_nonce(
    owner: Option<Principal>,
    to: String,
    amount: Nat,
) -> String {
    let client = client();

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

    let blockhash = Hash::from_str(&get_nonce(Some(nonce_account.to_string())).await)
        .expect("Unable to parse nonce as blockhash");

    let message = Message::new_with_blockhash(instructions, Some(payer.as_ref()), &blockhash);
    let signatures = vec![wallet.sign_with_ed25519(&message, &payer).await];
    let transaction = Transaction {
        message,
        signatures,
    };

    client
        .send_transaction(transaction)
        .send()
        .await
        .expect_consistent()
        .expect("Call to `sendTransaction` failed")
        .to_string()
}

#[update]
pub async fn send_spl_token(
    owner: Option<Principal>,
    mint_account: String,
    to: String,
    amount: Nat,
) -> String {
    let client = client();

    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;

    let payer = wallet.solana_account();
    let recipient = Pubkey::from_str(&to).unwrap();
    let mint = Pubkey::from_str(&mint_account).unwrap();
    let amount = amount.0.to_u64().unwrap();

    let from = spl::get_associated_token_address(payer.as_ref(), &mint);
    let to = spl::get_associated_token_address(&recipient, &mint);

    let instruction = spl::transfer_instruction(&from, &to, payer.as_ref(), amount);

    let message = Message::new_with_blockhash(
        &[instruction],
        Some(payer.as_ref()),
        &get_recent_blockhash(&client).await,
    );
    let signatures = vec![wallet.sign_with_ed25519(&message, &payer).await];
    let transaction = Transaction {
        message,
        signatures,
    };

    client
        .send_transaction(transaction)
        .send()
        .await
        .expect_consistent()
        .expect("Call to `sendTransaction` failed")
        .to_string()
}

async fn get_recent_blockhash(rpc_client: &SolRpcClient<IcRuntime>) -> Hash {
    let slot = rpc_client
        .get_slot()
        .send()
        .await
        .expect_consistent()
        .expect("Call to `getSlot` failed");
    let blockhash = rpc_client
        .get_block(slot)
        .send()
        .await
        .expect_consistent()
        .expect("Call to `getBlock` failed")
        .unwrap_or_else(|| panic!("Block for slot {slot} not found"))
        .blockhash;
    Hash::from_str(&blockhash).expect("Unable to parse blockhash")
}

fn client() -> SolRpcClient<IcRuntime> {
    SolRpcClient::builder_for_ic()
        .with_rpc_sources(RpcSources::Default(
            read_state(|state| state.solana_network()).into(),
        ))
        .build()
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

impl From<SolanaNetwork> for SolanaCluster {
    fn from(network: SolanaNetwork) -> Self {
        match network {
            SolanaNetwork::Mainnet => Self::Mainnet,
            SolanaNetwork::Devnet => Self::Devnet,
            SolanaNetwork::Testnet => Self::Testnet,
        }
    }
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
