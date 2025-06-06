use async_trait::async_trait;
use sol_rpc_client::{
    ed25519::{get_pubkey, sign_message, DerivationPath, Ed25519KeyId},
    nonce::nonce_from_account,
};
use sol_rpc_e2e_tests::Setup;
use sol_rpc_types::Lamport;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_message::Message;
use solana_program::system_instruction;
use solana_pubkey::{pubkey, Pubkey};
use solana_transaction::Transaction;
use std::num::NonZeroUsize;

const FUNDING_AMOUNT: Lamport = 1_000_000_000;
const TRANSACTION_AMOUNT: Lamport = 100_000;
const KEY_ID: Ed25519KeyId = Ed25519KeyId::MainnetTestKey1;

// Pubkeys `ACCOUNT_A` and `ACCOUNT_B` were obtained through the `schnorr_public_key` management
// canister method. Since the `schnorr_public_key` method cannot be called via ingress message,
// it must be routed through a canister (e.g. a cycles wallet canister). The public keys may be
// obtained by calling `schnorr_public_key` with the following argument:
//     record {
//         canister_id: opt principal $WALLET_CANISTER_ID;
//         derivation_path: opt $DERIVATION_PATH;
//         key_id: record {
//             algorithm: variant { Ed25519 };
//             name: "test_key_1";
//         };
//     }
// Where `DERIVATION_PATH` is either `DERIVATION_PATH_A` or `DERIVATION_PATH_B` encoded as a
// vector of bytes and `WALLET_CANISTER_ID` is the principal of the team wallet canister.
//
// Also note that funds are sent in one test from `ACCOUNT_A` to `ACCOUNT_B` and in another test
// from `ACCOUNT_B` to `ACCOUNT_A` so that in normal operation the net flow of funds should be 0.
// The accounts still need to be occasionally topped-up to pay for transaction fees.
const DERIVATION_PATH_A: &[&[u8]] = &[&[1]];
const ACCOUNT_A: Pubkey = pubkey!("HNELCCu1459ANnRXrQuBmEhaVVJfCk9FFRDZHL5YBXzH");
const DERIVATION_PATH_B: &[&[u8]] = &[&[2]];
const PUBKEY_B: Pubkey = pubkey!("G7Ut56qgcEphHZmLhLimM2DfHVC7QwHfT18tvj8ntn9");

// `NONCE_ACCOUNT_B` is a nonce account with nonce authority `PUBKEY_B` which was created and
// initialized using the following Solana CLI commands:
//  solana-keygen new --outfile nonce-keypair.json
//  solana create-nonce-account nonce-keypair.json 0.01 --url devnet --nonce-authority $PUBKEY_B
const NONCE_ACCOUNT_B: Pubkey = pubkey!("876vg5npuF9LCfc2MVWZtewBUEfcgzdbahCK7gXn5MLh");

#[tokio::test(flavor = "multi_thread")]
async fn should_send_transaction_with_recent_blockhash() {
    let setup = &Setup::new();

    let sender_pubkey = ACCOUNT_A;
    let sender_derivation_path = DerivationPath::from(DERIVATION_PATH_A);
    verify_pubkey(&sender_derivation_path, &sender_pubkey).await;

    let recipient_pubkey = PUBKEY_B;

    fund_accounts(setup, &[sender_pubkey, recipient_pubkey]);

    let create_message = CreateMessageWithRecentBlockhash { setup };

    send_transaction_test(
        setup,
        sender_pubkey,
        sender_derivation_path,
        recipient_pubkey,
        create_message,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn should_send_transaction_with_durable_nonce() {
    let setup = &Setup::new();

    let sender_pubkey = PUBKEY_B;
    let sender_derivation_path = DerivationPath::from(DERIVATION_PATH_B);
    verify_pubkey(&sender_derivation_path, &sender_pubkey).await;

    let nonce_account = NONCE_ACCOUNT_B;
    let recipient_pubkey = ACCOUNT_A;

    fund_accounts(setup, &[sender_pubkey, recipient_pubkey]);

    let create_message = CreateMessageWithDurableNonce {
        setup,
        nonce_account,
    };

    send_transaction_test(
        setup,
        sender_pubkey,
        sender_derivation_path,
        recipient_pubkey,
        create_message,
    )
    .await;
}

async fn send_transaction_test<F: CreateSolanaMessage>(
    setup: &Setup,
    sender_pubkey: Pubkey,
    sender_derivation_path: DerivationPath,
    recipient_pubkey: Pubkey,
    create_message: F,
) {
    println!(
        "Sending transaction from sender account '{sender_pubkey:?}' to recipient account '{recipient_pubkey:?}'"
    );

    let client = setup.client();

    let sender_balance_before = setup.get_account_balance(&sender_pubkey).await;
    println!("Sender balance before sending transaction: {sender_balance_before:?} lamports");
    let recipient_balance_before = setup.get_account_balance(&recipient_pubkey).await;
    println!("Recipient balance before sending transaction: {recipient_balance_before:?} lamports");

    let message = create_message
        .create_message(sender_pubkey, recipient_pubkey)
        .await;

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
    println!("Sent transaction with ID '{transaction_id:?}'");

    // Wait until the transaction is successfully executed
    let status = setup.confirm_transaction(&transaction_id).await;
    println!(
        "Transaction was included in a block at slot {:?}",
        status.slot
    );

    // Make sure the funds were sent from the sender to the recipient
    let sender_balance_after = setup.get_account_balance(&sender_pubkey).await;
    println!("Sender balance after sending transaction: {sender_balance_before:?} lamports");
    let recipient_balance_after = setup.get_account_balance(&recipient_pubkey).await;
    println!("Recipient balance after sending transaction: {recipient_balance_after:?} lamports");

    assert_eq!(
        recipient_balance_after,
        recipient_balance_before + TRANSACTION_AMOUNT
    );
    assert!(sender_balance_after <= sender_balance_before - TRANSACTION_AMOUNT);
}

#[async_trait]
pub trait CreateSolanaMessage {
    async fn create_message(&self, sender_pubkey: Pubkey, recipient_pubkey: Pubkey) -> Message;
}

struct CreateMessageWithRecentBlockhash<'a> {
    setup: &'a Setup,
}

#[async_trait]
impl CreateSolanaMessage for CreateMessageWithRecentBlockhash<'_> {
    async fn create_message(&self, sender_pubkey: Pubkey, recipient_pubkey: Pubkey) -> Message {
        let client = self.setup.client();

        // Set a CU limit for instructions to: perform a SOL transfer, set the CU price, and set
        // the CU limit (150 CU x 3 = 450 CU)
        let set_cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(450);

        // Set the compute unit (CU) price to the median of the recent prioritization fees
        let priority_fee = self
            .setup
            .get_median_recent_prioritization_fees(&sender_pubkey, &recipient_pubkey)
            .await;
        let add_priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);

        // Send some SOL from sender to recipient
        let transfer_ix =
            system_instruction::transfer(&sender_pubkey, &recipient_pubkey, TRANSACTION_AMOUNT);

        // Fetch a recent blockhash
        let blockhash = client
            .estimate_recent_blockhash()
            .with_num_tries(NonZeroUsize::new(3).unwrap())
            .send()
            .await
            .expect("Failed to fetch recent blockhash");
        println!("Fetched recent blockhash: {blockhash}");

        Message::new_with_blockhash(
            &[set_cu_limit_ix, add_priority_fee_ix, transfer_ix],
            Some(&sender_pubkey),
            &blockhash,
        )
    }
}

struct CreateMessageWithDurableNonce<'a> {
    setup: &'a Setup,
    nonce_account: Pubkey,
}

#[async_trait]
impl CreateSolanaMessage for CreateMessageWithDurableNonce<'_> {
    async fn create_message(&self, sender_pubkey: Pubkey, recipient_pubkey: Pubkey) -> Message {
        let client = self.setup.client();

        // Instruction to advance nonce account; this instruction must be first.
        let advance_nonce_ix =
            system_instruction::advance_nonce_account(&self.nonce_account, &sender_pubkey);

        // Set a CU limit for instructions to: perform a SOL transfer, advance the nonce account,
        // and set the CU price, and set the CU limit (150 CU x 4 = 600 CU)
        let set_cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(600);

        // Set the compute unit (CU) price to the median of the recent prioritization fees
        let priority_fee = self
            .setup
            .get_median_recent_prioritization_fees(&sender_pubkey, &recipient_pubkey)
            .await;
        let add_priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);

        // Send some SOL from sender to recipient
        let transfer_ix =
            system_instruction::transfer(&sender_pubkey, &recipient_pubkey, TRANSACTION_AMOUNT);

        // Fetch the current durable nonce value
        let account = client
            .get_account_info(self.nonce_account)
            .send()
            .await
            .expect_consistent()
            .expect("Call to `getAccountInfo` failed")
            .expect("Account not found");
        let blockhash =
            nonce_from_account(&account).expect("Failed to extract durable nonce from account");
        println!("Fetched durable nonce: {:?}", blockhash);

        Message::new_with_blockhash(
            &[
                advance_nonce_ix,
                set_cu_limit_ix,
                add_priority_fee_ix,
                transfer_ix,
            ],
            Some(&sender_pubkey),
            &blockhash,
        )
    }
}

fn fund_accounts(setup: &Setup, accounts: &[Pubkey]) {
    for account in accounts {
        setup.fund_account(account, FUNDING_AMOUNT);
    }
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
