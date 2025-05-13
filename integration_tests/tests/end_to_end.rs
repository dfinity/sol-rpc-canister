use ic_agent::{identity::BasicIdentity, Agent};
use pocket_ic::management_canister::CanisterId;
use serde_json::json;
use sol_rpc_client::{ClientBuilder, SolRpcClient};
use sol_rpc_int_tests::IcAgentRuntime;
use sol_rpc_types::{CommitmentLevel, RpcSources, SolanaCluster};
use solana_commitment_config::CommitmentConfig;
use solana_hash::Hash;
use solana_keypair::Keypair;
use solana_program::system_instruction;
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use solana_signer::Signer;
use solana_transaction::Transaction;
use std::{env, str::FromStr, time::Duration};
use ic_agent::identity::Secp256k1Identity;

// This test should be run together with end-to-end tests, not other integration tests
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn should_send_transaction() {
    let setup = Setup::new();

    let sender = Keypair::from_bytes(env("SOLANA_SENDER_PRIVATE_KEY_BYTES").as_ref()).unwrap();
    let recipient = Keypair::from_bytes(env("SOLANA_RECEIVER_PRIVATE_KEY_BYTES").as_ref()).unwrap();

    let sender_balance_before = setup.fund_account(&sender.pubkey(), 1_000_000_000).await;
    let recipient_balance_before = setup.fund_account(&recipient.pubkey(), 1_000_000_000).await;

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
        .expect("Call to get block failed")
        .expect("Block not found");
    let blockhash = Hash::from_str(&block.blockhash).expect("Failed to parse blockhash");

    let transaction_amount = 1_000;
    let transaction = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &sender.pubkey(),
            &recipient.pubkey(),
            transaction_amount,
        )],
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

    // Wait until the transaction is confirmed.
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

pub struct Setup {
    agent: Agent,
    sol_rpc_canister_id: CanisterId,
    wallet_canister_id: CanisterId,
}

impl Setup {
    fn new() -> Self {
        Self {
            agent: Agent::builder()
                .with_identity({
                    Secp256k1Identity::from_pem(env("DFX_DEPLOY_KEY").as_bytes())
                        .expect("Unable to import identity from PEM file")
                })
                .build()
                .expect("Could not build agent"),
            sol_rpc_canister_id: CanisterId::from_text(env("sol_rpc_canister_id")).unwrap(),
            wallet_canister_id: CanisterId::from_text(env("wallet_canister_id")).unwrap(),
        }
    }

    fn new_ic_agent_runtime(&self) -> IcAgentRuntime {
        IcAgentRuntime {
            agent: &self.agent,
            wallet_canister_id: self.wallet_canister_id,
        }
    }

    pub fn client_builder(&self) -> ClientBuilder<IcAgentRuntime> {
        SolRpcClient::builder(self.new_ic_agent_runtime(), self.sol_rpc_canister_id)
    }

    fn client(&self) -> SolRpcClient<IcAgentRuntime> {
        self.client_builder()
            .with_rpc_sources(RpcSources::Default(SolanaCluster::Devnet))
            .with_default_commitment_level(CommitmentLevel::Confirmed)
            .build()
    }

    async fn confirm_transaction(&self, transaction_id: &Signature) {
        let mut num_trials = 0;
        loop {
            num_trials += 1;
            if num_trials > 20 {
                panic!("Failed to confirm transaction {transaction_id}");
            }
            let statuses = self
                .client()
                .get_signature_statuses(vec![transaction_id])
                .send()
                .await
                .expect_consistent()
                .unwrap_or_else(|_| {
                    panic!("Failed to get status for transaction {transaction_id}")
                });
            if statuses.is_empty() || statuses[0].is_none() {
                continue;
            }
            let status = statuses[0].as_ref().unwrap();
            if status.satisfies_commitment(CommitmentConfig::confirmed()) && status.status.is_ok() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(400)).await;
        }
    }

    async fn airdrop(&self, account: &Pubkey, amount: u64) -> u64 {
        let balance_before = self.get_account_balance(account).await;
        let _airdrop_tx = self
            .client()
            .json_request(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "requestAirdrop",
                "params": [account.to_string(), amount]
            }))
            .send()
            .await;
        let expected_balance = balance_before + amount;
        let mut num_trials = 0;
        loop {
            num_trials += 1;
            if num_trials > 20 {
                panic!("Failed to airdrop funds to account {account}");
            }
            let balance = self.get_account_balance(account).await;
            if balance >= expected_balance {
                return balance;
            };
            tokio::time::sleep(Duration::from_millis(400)).await;
        }
    }

    async fn fund_account(&self, account: &Pubkey, amount: u64) -> u64 {
        let balance = self.get_account_balance(account).await;
        if balance < amount {
            self.airdrop(account, amount).await
        } else {
            balance
        }
    }

    async fn get_account_balance(&self, pubkey: &Pubkey) -> u64 {
        self.client()
            .get_balance(*pubkey)
            .send()
            .await
            .expect_consistent()
            .unwrap_or_else(|_| panic!("Failed to fetch account balance for account {pubkey}"))
    }
}

fn env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Environment variable '{key}' is not set!"))
}
