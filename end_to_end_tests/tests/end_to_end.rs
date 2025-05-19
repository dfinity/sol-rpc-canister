use candid::Principal;
use ic_cdk::api::management_canister::schnorr::{
    SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgument, SchnorrPublicKeyResponse,
};
use sol_rpc_client::sign_transaction;
use sol_rpc_e2e_tests::{env, Setup};
use sol_rpc_types::{DerivationPath, Ed25519KeyId, SignTransactionRequestParams};
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_hash::Hash;
use solana_message::Message;
use solana_program::system_instruction;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use std::str::FromStr;

#[tokio::test(flavor = "multi_thread")]
async fn should_send_transaction() {
    let setup = Setup::new();
    let client = setup.client();

    fn load_derivation_path(key: &str) -> DerivationPath {
        let bytes: Vec<u8> = serde_json::from_str(&env(key)).unwrap_or_else(|e| {
            panic!("Failed to read bytes stored in environment variable '{key}': {e}")
        });
        DerivationPath::from(bytes.as_ref())
    }
    let sender_derivation_path = load_derivation_path("SOLANA_SENDER_DERIVATION_PATH_BYTES");
    let sender_pubkey =
        get_threshold_eddsa_key(setup.get_wallet_canister_id(), sender_derivation_path).await;

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
    let mut transaction = Transaction::new_unsigned(message);

    // Sign transaction with tEdDSA
    let signature = sign_transaction(
        client.runtime(),
        SignTransactionRequestParams {
            transaction: transaction.clone(),
            key_id: Ed25519KeyId::TestKey1,
            derivation_path: None,
        },
    )
    .await
    .expect("Failed to sign transaction");
    transaction.signatures = vec![signature];

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

async fn get_threshold_eddsa_key(
    canister_id: Principal,
    derivation_path: DerivationPath,
) -> Pubkey {
    let (SchnorrPublicKeyResponse {
        public_key: bytes, ..
    },) = ic_cdk::api::management_canister::schnorr::schnorr_public_key(SchnorrPublicKeyArgument {
        canister_id: Some(canister_id),
        derivation_path: derivation_path.into(),
        key_id: SchnorrKeyId {
            algorithm: SchnorrAlgorithm::Ed25519,
            name: "Test1".to_string(),
        },
    })
    .await
    .expect("Failed to fetch EdDSA public key");
    solana_pubkey::Pubkey::try_from(bytes.as_slice()).expect("Failed to parse bytes as public key")
}
