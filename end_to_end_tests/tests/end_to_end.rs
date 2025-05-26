use sol_rpc_client::{
    ed25519::{get_pubkey, sign_message, DerivationPath, Ed25519KeyId},
    nonce::extract_durable_nonce,
    SolRpcClient,
};
use sol_rpc_e2e_tests::{IcAgentRuntime, Setup};
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_hash::Hash;
use solana_message::Message;
use solana_program::instruction::Instruction;
use solana_program::system_instruction;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use std::str::FromStr;

const KEY_ID: Ed25519KeyId = Ed25519KeyId::MainnetTestKey1;

// Pubkey `ACCOUNT_A` was obtained with the `schnorr_public_key` with the team wallet canister ID
// the derivation path `DERIVATION_PATH_A`, and the `KEY_ID` key ID
const DERIVATION_PATH_A: &[&[u8]] = &[&[1]];
const ACCOUNT_A: &str = "HNELCCu1459ANnRXrQuBmEhaVVJfCk9FFRDZHL5YBXzH";

// Pubkey `PUBKEY_B` was obtained with the `schnorr_public_key` with the team wallet canister ID
// the derivation path `DERIVATION_PATH_B`, and the `KEY_ID` key ID
const DERIVATION_PATH_B: &[&[u8]] = &[&[2]];
const PUBKEY_B: &str = "G7Ut56qgcEphHZmLhLimM2DfHVC7QwHfT18tvj8ntn9";

// `NONCE_ACCOUNT_B` is an initialized nonce account with nonce authority `PUBKEY_B`
const NONCE_ACCOUNT_B: &str = "876vg5npuF9LCfc2MVWZtewBUEfcgzdbahCK7gXn5MLh";

#[tokio::test(flavor = "multi_thread")]
async fn should_send_transaction_with_recent_blockhash() {
    let sender_pubkey = Pubkey::from_str(ACCOUNT_A).unwrap();
    let sender_derivation_path = DerivationPath::from(DERIVATION_PATH_A);
    verify_pubkey(&sender_derivation_path, &sender_pubkey).await;

    let recipient_pubkey = Pubkey::from_str(PUBKEY_B).unwrap();

    let get_blockhash = async |client: &SolRpcClient<IcAgentRuntime>| {
        // TODO XC-317: Use method to estimate recent blockhash
        let slot = client
            .get_slot()
            .send()
            .await
            .expect_consistent()
            .expect("Call to get slot failed");
        let block = client
            .get_block(slot)
            .send()
            .await
            .expect_consistent()
            .expect("Call to `getBlock` failed")
            .expect("Block not found");
        Hash::from_str(&block.blockhash).expect("Failed to parse blockhash")
    };

    let modify_instructions = |_instructions: &mut Vec<Instruction>| {};

    send_transaction_test(
        sender_pubkey,
        sender_derivation_path,
        recipient_pubkey,
        get_blockhash,
        modify_instructions,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn should_send_transaction_with_durable_nonce() {
    let sender_pubkey = Pubkey::from_str(PUBKEY_B).unwrap();
    let sender_derivation_path = DerivationPath::from(DERIVATION_PATH_B);
    verify_pubkey(&sender_derivation_path, &sender_pubkey).await;

    let sender_nonce_account = Pubkey::from_str(NONCE_ACCOUNT_B).unwrap();
    let recipient_pubkey = Pubkey::from_str(ACCOUNT_A).unwrap();

    let get_blockhash = async |client: &SolRpcClient<IcAgentRuntime>| {
        let account = client
            .get_account_info(Pubkey::from_str(NONCE_ACCOUNT_B).unwrap())
            .send()
            .await
            .expect_consistent()
            .expect("Call to `getAccountInfo` failed")
            .expect("Account not found");
        extract_durable_nonce(&account).expect("Failed to extract durable nonce from account")
    };

    let modify_instructions = |instructions: &mut Vec<Instruction>| {
        let advance_nonce_ix =
            system_instruction::advance_nonce_account(&sender_nonce_account, &sender_pubkey);
        instructions.insert(2, advance_nonce_ix);
    };

    send_transaction_test(
        sender_pubkey,
        sender_derivation_path,
        recipient_pubkey,
        get_blockhash,
        modify_instructions,
    )
    .await;
}

async fn send_transaction_test<F, S>(
    sender_pubkey: Pubkey,
    sender_derivation_path: DerivationPath,
    recipient_pubkey: Pubkey,
    get_blockhash: F,
    modify_instructions: S,
) where
    F: AsyncFnOnce(&SolRpcClient<IcAgentRuntime>) -> Hash,
    S: FnOnce(&mut Vec<Instruction>),
{
    let setup = Setup::new();
    let client = setup.client();

    let sender_balance_before = setup.fund_account(&sender_pubkey, 1_000_000_000).await;
    let recipient_balance_before = setup.fund_account(&recipient_pubkey, 1_000_000_000).await;

    let prioritization_fees: Vec<_> = client
        .get_recent_prioritization_fees(&[sender_pubkey, recipient_pubkey])
        .unwrap()
        .send()
        .await
        .expect_consistent()
        .expect("Call to `getRecentPrioritizationFees` failed")
        .into_iter()
        .map(|fee| fee.prioritization_fee)
        .collect();

    // Set the compute unit (CU) price to the median of the recent prioritization fees
    let priority_fee = if !prioritization_fees.is_empty() {
        prioritization_fees[prioritization_fees.len() / 2]
    } else {
        0
    };
    let add_priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);

    // Set a CU limit based for a simple SOL transfer + instructions to set the CU price and CU limit
    let set_cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(500);

    // Send some SOL from sender to recipient
    let transaction_amount = 1_000;
    let transfer_ix =
        system_instruction::transfer(&sender_pubkey, &recipient_pubkey, transaction_amount);

    let blockhash = get_blockhash(&client).await;

    let mut instructions = vec![set_cu_limit_ix, add_priority_fee_ix, transfer_ix];
    modify_instructions(&mut instructions);

    let message =
        Message::new_with_blockhash(instructions.as_slice(), Some(&sender_pubkey), &blockhash);

    // Sign transaction with t-EdDSA
    let signature = sign_message(
        client.runtime(),
        &message,
        KEY_ID,
        Some(&sender_derivation_path),
    )
    .await
    .expect("Failed to sign transaction");

    let transaction = Transaction {
        message,
        signatures: vec![signature],
    };

    let transaction_id = client
        .send_transaction(transaction)
        .send()
        .await
        .expect_consistent()
        .unwrap();

    // Wait until the transaction is successfully executed and confirmed.
    setup.confirm_transaction(&transaction_id).await;

    // Make sure the funds were sent from the sender to the recipient
    let sender_balance_after = setup.get_account_balance(&sender_pubkey).await;
    let recipient_balance_after = setup.get_account_balance(&recipient_pubkey).await;

    assert_eq!(
        recipient_balance_after,
        recipient_balance_before + transaction_amount
    );
    assert!(sender_balance_after + transaction_amount <= sender_balance_before);
}

async fn verify_pubkey(derivation_path: &DerivationPath, expected_pubkey: &Pubkey) {
    let (pubkey, _) = get_pubkey(
        Setup::new().client().runtime(),
        None,
        Some(derivation_path),
        KEY_ID,
    )
    .await
    .unwrap_or_else(|e| panic!("Failed to get Ed25519 public key: {e:?}"));
    assert_eq!(&pubkey, expected_pubkey);
}
