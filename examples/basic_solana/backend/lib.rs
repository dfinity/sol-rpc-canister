mod ed25519;
pub mod solana_wallet;
pub mod spl;
pub mod state;

use crate::solana_wallet::SolanaWallet;
use crate::spl::transfer_instruction_with_program_id;
use crate::state::{init_state, read_state, State};
use candid::{CandidType, Nat, Principal};
use ic_canister_runtime::IcRuntime;
use ic_cdk::{init, post_upgrade, update};
use num::ToPrimitive;
use serde::Deserialize;
use sol_rpc_client::nonce::nonce_from_account;
use sol_rpc_client::{ed25519::Ed25519KeyId, SolRpcClient};
use sol_rpc_types::{
    CommitmentLevel, ConsensusStrategy, GetAccountInfoEncoding, GetAccountInfoParams, RpcEndpoint,
    RpcSource, RpcSources, SolanaCluster, TokenAmount,
};
use solana_hash::Hash;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_system_interface::instruction;
use solana_transaction::Transaction;
use spl_associated_token_account_interface::{
    address::get_associated_token_address_with_program_id,
    instruction::create_associated_token_account_idempotent,
};
use std::str::FromStr;

// The SOL RPC canister ID is injected as PUBLIC_CANISTER_ID:sol_rpc at deploy time:
//   local: auto-injected by icp-cli after deploying the pre-built sol_rpc canister
//   ic:    set explicitly in icp.yaml to tghme-zyaaa-aaaar-qarca-cai (shared mainnet SOL RPC)
//
// See icp.yaml for the environment configuration.
fn sol_rpc_id() -> Principal {
    let id = ic_cdk::api::env_var_value("PUBLIC_CANISTER_ID:sol_rpc");
    Principal::from_text(&id).expect("invalid PUBLIC_CANISTER_ID:sol_rpc")
}

pub fn client() -> SolRpcClient<IcRuntime> {
    let rpc_sources = read_state(|state| state.solana_network().clone()).into();
    let consensus_strategy = match rpc_sources {
        RpcSources::Custom(_) => ConsensusStrategy::Equality,
        RpcSources::Default(_) => ConsensusStrategy::Threshold {
            min: 2,
            total: Some(3),
        },
    };
    SolRpcClient::builder(IcRuntime::default(), sol_rpc_id())
        .with_rpc_sources(rpc_sources)
        .with_consensus_strategy(consensus_strategy)
        .with_default_commitment_level(read_state(State::solana_commitment_level))
        .build()
}

#[init]
pub fn init(maybe_init: Option<InitArg>) {
    if let Some(init_arg) = maybe_init {
        init_state(init_arg)
    }
}

#[post_upgrade]
fn post_upgrade(maybe_init: Option<InitArg>) {
    if let Some(init_arg) = maybe_init {
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
    get_associated_token_address_with_program_id(
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

    let instructions = instruction::create_nonce_account(
        payer.as_ref(),
        nonce_account.as_ref(),
        payer.as_ref(),
        1_500_000,
    );

    let message = Message::new_with_blockhash(
        instructions.as_slice(),
        Some(payer.as_ref()),
        &recent_blockhash(&client).await,
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

    let account_owner = get_account_owner(&mint).await;

    let instruction = create_associated_token_account_idempotent(
        payer.as_ref(),
        payer.as_ref(),
        &mint,
        &account_owner,
    );

    let message = Message::new_with_blockhash(
        &[instruction],
        Some(payer.as_ref()),
        &recent_blockhash(&client).await,
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

    get_associated_token_address_with_program_id(payer.as_ref(), &mint, &account_owner).to_string()
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
    let instruction = instruction::transfer(payer.as_ref(), &recipient, amount);

    let message = Message::new_with_blockhash(
        &[instruction],
        Some(payer.as_ref()),
        &recent_blockhash(&client).await,
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
        instruction::advance_nonce_account(nonce_account.as_ref(), payer.as_ref()),
        instruction::transfer(payer.as_ref(), &recipient, amount),
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

    let from = get_associated_token_address_with_program_id(payer.as_ref(), &mint, &token_program);
    let to = get_associated_token_address_with_program_id(&recipient, &mint, &token_program);

    let instruction =
        transfer_instruction_with_program_id(&from, &to, payer.as_ref(), amount, &token_program);

    let message = Message::new_with_blockhash(
        &[instruction],
        Some(payer.as_ref()),
        &recent_blockhash(&client).await,
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

/// Fetches a recent blockhash from the Solana network, used to build transactions.
async fn recent_blockhash(client: &SolRpcClient<IcRuntime>) -> Hash {
    let (_slot, block) = client
        .get_recent_block()
        .try_send()
        .await
        .expect("Call to `getRecentBlock` failed");
    block
        .blockhash
        .parse()
        .expect("Failed to parse recent blockhash")
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

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct InitArg {
    pub solana_network: Option<SolanaNetwork>,
    /// Threshold Ed25519 (Schnorr) key name as used by the IC management canister:
    /// `"test_key_1"` (ICP mainnet testing, also available on the local network) or
    /// `"key_1"` (ICP mainnet production). Defaults to `"test_key_1"`.
    pub ed25519_key_name: Option<String>,
    pub solana_commitment_level: Option<CommitmentLevel>,
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub enum SolanaNetwork {
    Mainnet,
    #[default]
    Devnet,
    Custom(RpcEndpoint),
}

impl From<SolanaNetwork> for RpcSources {
    fn from(network: SolanaNetwork) -> Self {
        match network {
            SolanaNetwork::Mainnet => Self::Default(SolanaCluster::Mainnet),
            SolanaNetwork::Devnet => Self::Default(SolanaCluster::Devnet),
            SolanaNetwork::Custom(endpoint) => Self::Custom(vec![RpcSource::Custom(endpoint)]),
        }
    }
}

/// Maps a threshold Ed25519 (Schnorr) key name to the corresponding [`Ed25519KeyId`].
///
/// Only the two Internet Computer key names supported by this example are accepted:
/// `"test_key_1"` (testing, available both on ICP mainnet and locally) and `"key_1"`
/// (ICP mainnet production). Any other value traps.
pub fn ed25519_key_id(key_name: &str) -> Ed25519KeyId {
    match key_name {
        "test_key_1" => Ed25519KeyId::MainnetTestKey1,
        "key_1" => Ed25519KeyId::MainnetProdKey1,
        other => ic_cdk::trap(&format!(
            "unsupported ed25519 key name {other:?}, expected \"test_key_1\" or \"key_1\""
        )),
    }
}

pub fn validate_caller_not_anonymous() -> Principal {
    let principal = ic_cdk::api::msg_caller();
    if principal == Principal::anonymous() {
        panic!("anonymous principal is not allowed");
    }
    principal
}

ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
    /// Guards against unintentional changes to the canister's public Candid interface: fails if
    /// the committed `backend.did` no longer matches the interface exported by the code. If you
    /// change the interface on purpose, regenerate it with
    /// `candid-extractor <backend.wasm> > backend.did`.
    #[test]
    fn candid_interface_matches_committed_did() {
        use candid_parser::utils::{service_equal, CandidSource};

        // `ic_cdk::export_candid!()` (invoked at the crate root) generates `__export_service()`
        // from the `#[update]` endpoints; reuse it rather than re-running `export_service!` here,
        // where the endpoints are not in scope.
        let exported = super::__export_service();

        let committed_did =
            std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("backend.did");
        service_equal(
            CandidSource::Text(&exported),
            CandidSource::File(committed_did.as_path()),
        )
        .unwrap_or_else(|e| {
            panic!(
                "backend.did is out of date with the canister interface: {e}\n\
                 Regenerate it with `candid-extractor <backend.wasm> > backend.did`."
            )
        });
    }
}
