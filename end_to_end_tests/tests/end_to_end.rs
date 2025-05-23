use sol_rpc_client::ed25519::{get_pubkey, sign_message, Ed25519KeyId};
use sol_rpc_e2e_tests::{env, Setup};
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_hash::Hash;
use solana_message::Message;
use solana_program::system_instruction;
use solana_pubkey::{pubkey, Pubkey};
use solana_transaction::Transaction;
use std::str::FromStr;

#[tokio::test(flavor = "multi_thread")]
async fn should_send_transaction_with_recent_blockhash() {
    let setup = Setup::new();
    let client = setup.client();

    // Get the pubkey of the sender, which is derived from the given root Ed25519 key, and the
    // canister ID of the wallet canister (through which all calls are forwarded).
    let (sender_pubkey, _) =
        get_pubkey(client.runtime(), None, None, Ed25519KeyId::MainnetTestKey1)
            .await
            .unwrap_or_else(|e| panic!("Failed to get Ed25519 public key: {e:?}"));
    assert_eq!(
        sender_pubkey,
        pubkey!("2qL8z3PZS3tr8GV2x3z6mntNjNfLyh1VYcybfAENFSAn")
    );

    fn load_pubkey(key: &str) -> Pubkey {
        Pubkey::from_str(&env(key)).unwrap_or_else(|e| {
            panic!("Failed to parse environment variable '{key}' as a valid pubkey: {e}")
        })
    }
    let recipient_pubkey = load_pubkey("SOLANA_RECEIVER_PUBLIC_KEY");

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
    let blockhash = Hash::from_str(&block.blockhash).expect("Failed to parse blockhash");

    let message = Message::new_with_blockhash(
        &[set_cu_limit_ix, add_priority_fee_ix, transfer_ix],
        Some(&sender_pubkey),
        &blockhash,
    );

    // Sign transaction with t-EdDSA
    let signature = sign_message(
        client.runtime(),
        &message,
        Ed25519KeyId::MainnetTestKey1,
        None,
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

#[tokio::test(flavor = "multi_thread")]
async fn should_send_transaction_with_durable_nonce() {
    // TODO XC-347: Same as `should_send_transaction_with_recent_blockhash` but with a durable nonce
}
