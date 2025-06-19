use basic_solana::{
    client, solana_wallet::SolanaWallet, spl, state::init_state, validate_caller_not_anonymous,
    InitArg,
};
use candid::{Nat, Principal};
use ic_cdk::{init, post_upgrade, update};
use num::ToPrimitive;
use sol_rpc_client::nonce::nonce_from_account;
use sol_rpc_types::{GetAccountInfoEncoding, GetAccountInfoParams, TokenAmount};
use solana_hash::Hash;
use solana_message::Message;
use solana_program::system_instruction;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use std::str::FromStr;

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
pub async fn nonce_account(owner: Option<Principal>) -> sol_rpc_types::Pubkey {
    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;
    wallet.derived_nonce_account().as_ref().into()
}

#[update]
pub async fn associated_token_account(owner: Option<Principal>, mint_account: String) -> String {
    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;
    let mint = Pubkey::from_str(&mint_account).unwrap();
    spl::get_associated_token_address(
        wallet.solana_account().as_ref(),
        &mint,
        &get_account_owner(&mint).await,
    )
    .to_string()
}

#[update]
pub async fn get_balance(account: Option<String>) -> Nat {
    let account = account.unwrap_or(solana_account(None).await);
    let public_key = Pubkey::from_str(&account).unwrap();
    let balance = client()
        .get_balance(public_key)
        .send()
        .await
        .expect_consistent()
        .expect("Call to `getBalance` failed");
    Nat::from(balance)
}

#[update]
pub async fn get_nonce(account: Option<sol_rpc_types::Pubkey>) -> sol_rpc_types::Hash {
    let account = account.unwrap_or(nonce_account(None).await);

    // Fetch the account info with the data encoded in base64 format
    let mut params = GetAccountInfoParams::from_pubkey(account);
    params.encoding = Some(GetAccountInfoEncoding::Base64);
    let account = client()
        .get_account_info(params)
        .send()
        .await
        .expect_consistent()
        .expect("Call to `getAccountInfo` failed")
        .expect("Account not found for given pubkey");

    // Extract the nonce from the account data
    nonce_from_account(&account)
        .expect("Failed to extract durable nonce from account data")
        .into()
}

#[update]
pub async fn get_spl_token_balance(account: Option<String>, mint_account: String) -> TokenAmount {
    let account = account.unwrap_or(associated_token_account(None, mint_account).await);
    let public_key = Pubkey::from_str(&account).unwrap();
    client()
        .get_token_account_balance(public_key)
        .send()
        .await
        .expect_consistent()
        .expect("Call to `getTokenAccountBalance` failed")
        .into()
}

#[update]
pub async fn create_nonce_account(owner: Option<Principal>) -> String {
    let client = client();

    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;

    let payer = wallet.solana_account();
    let nonce_account = wallet.derived_nonce_account();

    if let Some(_account) = client
        .get_account_info(*nonce_account.as_ref())
        .send()
        .await
        .expect_consistent()
        .unwrap_or_else(|e| {
            panic!(
                "Call to `getAccountInfo` for {} failed: {e}",
                nonce_account.as_ref()
            )
        })
    {
        ic_cdk::println!(
            "[create_nonce_account]: Account {} already exists. Skipping creation of nonce account",
            nonce_account.as_ref()
        );
        return nonce_account.as_ref().to_string();
    }

    let instructions = system_instruction::create_nonce_account(
        payer.as_ref(),
        nonce_account.as_ref(),
        payer.as_ref(),
        1_500_000,
    );

    let message = Message::new_with_blockhash(
        instructions.as_slice(),
        Some(payer.as_ref()),
        &client.estimate_recent_blockhash().send().await.unwrap(),
    );

    let signatures = vec![
        payer.sign_message(&message).await,
        nonce_account.sign_message(&message).await,
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
        .expect("Call to `sendTransaction` failed");

    nonce_account.as_ref().to_string()
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

    let (associated_token_account, instruction) = spl::create_associated_token_account_instruction(
        payer.as_ref(),
        payer.as_ref(),
        &mint,
        &get_account_owner(&mint).await,
    );

    if let Some(_account) = client
        .get_account_info(associated_token_account)
        .with_encoding(GetAccountInfoEncoding::Base64)
        .send()
        .await
        .expect_consistent()
        .unwrap_or_else(|e| {
            panic!("Call to `getAccountInfo` for {associated_token_account} failed: {e}")
        })
    {
        ic_cdk::println!(
            "[create_associated_token_account]: Account {} already exists. Skipping creation of associated token account",
            associated_token_account
        );
        return associated_token_account.to_string();
    }

    let message = Message::new_with_blockhash(
        &[instruction],
        Some(payer.as_ref()),
        &client.estimate_recent_blockhash().send().await.unwrap(),
    );

    let signatures = vec![payer.sign_message(&message).await];
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
        .to_string();

    associated_token_account.to_string()
}

#[update]
pub async fn send_sol(owner: Option<Principal>, to: String, amount: Nat) -> String {
    let client = client();

    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;

    let recipient = Pubkey::from_str(&to).unwrap();
    let payer = wallet.solana_account();
    let amount = amount.0.to_u64().unwrap();

    ic_cdk::println!(
        "Instruction to transfer {amount} lamports from {} to {recipient}",
        payer.as_ref()
    );
    let instruction = system_instruction::transfer(payer.as_ref(), &recipient, amount);

    let message = Message::new_with_blockhash(
        &[instruction],
        Some(payer.as_ref()),
        &client.estimate_recent_blockhash().send().await.unwrap(),
    );
    let signatures = vec![payer.sign_message(&message).await];
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

    let blockhash = Hash::from(get_nonce(Some(nonce_account.as_ref().into())).await);

    let message = Message::new_with_blockhash(instructions, Some(payer.as_ref()), &blockhash);
    let signatures = vec![payer.sign_message(&message).await];
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

    let token_program = get_account_owner(&mint).await;

    let from = spl::get_associated_token_address(payer.as_ref(), &mint, &token_program);
    let to = spl::get_associated_token_address(&recipient, &mint, &token_program);

    let instruction = spl::transfer_instruction_with_program_id(
        &from,
        &to,
        payer.as_ref(),
        amount,
        &token_program,
    );

    let message = Message::new_with_blockhash(
        &[instruction],
        Some(payer.as_ref()),
        &client.estimate_recent_blockhash().send().await.unwrap(),
    );
    let signatures = vec![payer.sign_message(&message).await];
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

async fn get_account_owner(account: &Pubkey) -> Pubkey {
    let owner = client()
        .get_account_info(*account)
        .with_encoding(GetAccountInfoEncoding::Base64)
        .send()
        .await
        .expect_consistent()
        .expect("Call to `getAccountInfo` failed")
        .unwrap_or_else(|| panic!("Account not found for pubkey `{account}`"))
        .owner;
    Pubkey::from_str(&owner).unwrap()
}

fn main() {}

#[test]
fn check_candid_interface_compatibility() {
    use candid_parser::utils::{service_equal, CandidSource};

    candid::export_service!();

    let new_interface = __export_service();

    // check the public interface against the actual one
    let old_interface = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("basic_solana.did");

    service_equal(
        CandidSource::Text(dbg!(&new_interface)),
        CandidSource::File(old_interface.as_path()),
    )
    .unwrap();
}
