use sol_rpc_e2e_tests::{env, Setup};
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_hash::Hash;
use solana_keypair::Keypair;
use solana_program::system_instruction;
use solana_signer::Signer;
use solana_transaction::Transaction;
use std::str::FromStr;

#[tokio::test(flavor = "multi_thread")]
async fn should_send_transaction() {
    let setup = Setup::new();

    fn load_keypair(key: &str) -> Keypair {
        fn try_load_keypair(key: &str) -> Result<Keypair, String> {
            let value = env(key);
            let bytes = serde_json::from_str::<Vec<u8>>(&value).map_err(|e| e.to_string())?;
            Keypair::from_bytes(bytes.as_ref()).map_err(|e| e.to_string())
        }
        try_load_keypair(key).unwrap_or_else(|e| panic!("Unable to parse bytes stored in environment variable '{key}' as a valid keypair: {e}"))
    }

    let sender = load_keypair("SOLANA_SENDER_PRIVATE_KEY_BYTES");
    let recipient = load_keypair("SOLANA_RECEIVER_PRIVATE_KEY_BYTES");

    let sender_balance_before = setup.fund_account(&sender.pubkey(), 1_000_000_000).await;
    let recipient_balance_before = setup.fund_account(&recipient.pubkey(), 1_000_000_000).await;

    // Set a compute unit (CU) price fee based on the recent prioritization fees
    let prioritization_fees: Vec<_> = setup
        .client()
        .get_recent_prioritization_fees(&[])
        .unwrap()
        .send()
        .await
        .expect_consistent()
        .expect("Call to `getRecentPrioritizationFees` failed")
        .into_iter()
        .map(|fee| fee.prioritization_fee)
        .collect();
    let priority_fee = prioritization_fees.into_iter().max().unwrap_or_default();
    let add_priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);

    // Set a CU limit based for a simple transfer
    let set_cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(150);

    // Send some SOL from sender to recipient
    let transaction_amount = 1_000;
    let transfer_ix =
        system_instruction::transfer(&sender.pubkey(), &recipient.pubkey(), transaction_amount);

    let slot = setup
        .client()
        .get_slot()
        .send()
        .await
        .expect_consistent()
        .expect("Call to get slot failed");
    let block = setup
        .client()
        .get_block(slot)
        .send()
        .await
        .expect_consistent()
        .expect("Call to `getBlock` failed")
        .expect("Block not found");
    let blockhash = Hash::from_str(&block.blockhash).expect("Failed to parse blockhash");

    let transaction = Transaction::new_signed_with_payer(
        &[set_cu_limit_ix, add_priority_fee_ix, transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        blockhash,
    );

    let transaction_id = setup
        .client()
        .send_transaction(transaction)
        .send()
        .await
        .expect_consistent()
        .unwrap();

    // Wait until the transaction is successfully executed and confirmed.
    setup.confirm_transaction(&transaction_id).await;

    // Make sure the funds were sent from the sender to the recipient
    let sender_balance_after = setup.get_account_balance(&sender.pubkey()).await;
    let recipient_balance_after = setup.get_account_balance(&recipient.pubkey()).await;

    assert_eq!(
        recipient_balance_after,
        recipient_balance_before + transaction_amount
    );
    assert!(sender_balance_after + transaction_amount <= sender_balance_before);
}
