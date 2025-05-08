//! Tests to compare behavior between the official Solana RPC client directly interacting with a local validator
//! and the SOL RPC client that uses the SOL RPC canister that uses the local validator as JSON RPC provider.
//! Excepted for timing differences, the same behavior should be observed.

use futures::future;
use pocket_ic::PocketIcBuilder;
use sol_rpc_client::SolRpcClient;
use sol_rpc_int_tests::PocketIcLiveModeRuntime;
use sol_rpc_types::{
    CommitmentLevel, GetAccountInfoEncoding, GetAccountInfoParams, GetBlockCommitmentLevel,
    GetBlockParams, GetSlotParams, GetTransactionEncoding, GetTransactionParams, InstallArgs,
    Lamport, OverrideProvider, PrioritizationFee, RegexSubstitution, SendTransactionParams,
    TransactionDetails,
};
use solana_account_decoder_client_types::UiAccount;
use solana_client::rpc_client::{RpcClient as SolanaRpcClient, RpcClient};
use solana_commitment_config::CommitmentConfig;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_hash::Hash;
use solana_keypair::Keypair;
use solana_program::system_instruction;
use solana_pubkey::Pubkey;
use solana_rpc_client_api::config::{RpcBlockConfig, RpcTransactionConfig};
use solana_rpc_client_api::response::RpcPrioritizationFee;
use solana_signature::Signature;
use solana_signer::Signer;
use solana_transaction::Transaction;
use solana_transaction_status_client_types::UiTransactionEncoding;
use std::iter::zip;
use std::{
    future::Future,
    str::FromStr,
    thread::sleep,
    time::{Duration, Instant},
};

#[tokio::test(flavor = "multi_thread")]
async fn should_get_slot() {
    let setup = Setup::new().await;

    let (sol_res, ic_res) = setup
        .compare_client(
            |sol| sol.get_slot().expect("Failed to get slot"),
            |ic| async move {
                ic.get_slot()
                    .with_params(GetSlotParams {
                        commitment: Some(CommitmentLevel::Confirmed),
                        ..GetSlotParams::default()
                    })
                    .send()
                    .await
                    .expect_consistent()
                    .unwrap_or_else(|e| panic!("`getSlot` call failed: {e}"))
            },
        )
        .await;

    assert!(
        sol_res.abs_diff(ic_res) < 20,
        "Difference is too large between slot {sol_res} from Solana client and slot {ic_res} from the SOL RPC canister"
    );

    setup.setup.drop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn should_get_recent_prioritization_fees() {
    const BASE_FEE_PER_SIGNATURE_LAMPORTS: u64 = 5000;
    const NUM_TRANSACTIONS: u64 = 10;

    let setup = Setup::new().await;

    let (sender, sender_balance_before) = setup.generate_keypair_and_fund_account();
    let (recipient, _recipient_balance_before) = setup.generate_keypair_and_fund_account();

    // Generate some transactions with priority fees to ensure that
    // 1) There are some slots
    // 2) Priority fee is not always 0
    let mut transactions = Vec::with_capacity(NUM_TRANSACTIONS as usize);
    let transaction_amount = 1;
    for micro_lamports in 1..=NUM_TRANSACTIONS {
        let modify_cu_ix = ComputeBudgetInstruction::set_compute_unit_limit(1_000_000);
        let add_priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(micro_lamports);
        let transfer_ix =
            system_instruction::transfer(&sender.pubkey(), &recipient.pubkey(), transaction_amount);
        let blockhash = setup.solana_client.get_latest_blockhash().unwrap();
        let transaction = Transaction::new_signed_with_payer(
            &[modify_cu_ix, add_priority_fee_ix, transfer_ix],
            Some(&sender.pubkey()),
            &[&sender],
            blockhash,
        );
        let signature = setup
            .solana_client
            .send_and_confirm_transaction(&transaction)
            .unwrap();
        println!("Sent transaction {micro_lamports}: {signature}");
        transactions.push(signature);
    }

    let spent_lamports = NUM_TRANSACTIONS * transaction_amount //amount sent
            + NUM_TRANSACTIONS * BASE_FEE_PER_SIGNATURE_LAMPORTS //base fee
            + NUM_TRANSACTIONS * (NUM_TRANSACTIONS+1) / 2; //prioritization_fee = 1 µL * CUL / 1_000_000 + .. + NUM_TRANSACTIONS * CUL / 1_000_000 and compute unit limit was set to 1 million.
    assert_eq!(
        sender_balance_before - setup.solana_client.get_balance(&sender.pubkey()).unwrap(),
        spent_lamports
    );

    setup.confirm_transaction(transactions.last().unwrap());

    let account = sender.pubkey();
    let (sol_res, ic_res) = setup
        .compare_client(
            |sol| {
                sol.get_recent_prioritization_fees(&[account])
                    .expect("Failed to get recent prioritization fees")
            },
            |ic| async move {
                ic.get_recent_prioritization_fees(&[account])
                    .unwrap()
                    .with_max_length(150)
                    .with_max_slot_rounding_error(1)
                    .send()
                    .await
                    .expect_consistent()
                    .unwrap_or_else(|e| panic!("`getRecentPrioritizationFees` call failed: {e}"))
            },
        )
        .await;

    assert_eq!(
        sol_res.len(),
        ic_res.len(),
        "SOL results {:?}, ICP results {:?}",
        sol_res,
        ic_res
    );
    for (fees_sol, fees_ic) in zip(sol_res, ic_res) {
        let RpcPrioritizationFee {
            slot: slot_sol,
            prioritization_fee: prioritization_fee_sol,
        } = fees_sol;
        let PrioritizationFee {
            slot: slot_ic,
            prioritization_fee: prioritization_fee_ic,
        } = fees_ic;

        assert_eq!(slot_sol, slot_ic);
        assert_eq!(prioritization_fee_sol, prioritization_fee_ic)
    }

    setup.setup.drop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn should_get_account_info() {
    let setup = Setup::new().await;
    let pubkey = Pubkey::from_str("11111111111111111111111111111111").unwrap();
    let params = GetAccountInfoParams {
        pubkey: pubkey.to_string(),
        commitment: None,
        encoding: Some(GetAccountInfoEncoding::Base64),
        data_slice: None,
        min_context_slot: None,
    };

    let (sol_res, ic_res) = setup
        .compare_client(
            |sol| solana_rpc_client_get_account(&pubkey, sol, None),
            |ic| async move {
                ic.get_account_info(params)
                    .send()
                    .await
                    .expect_consistent()
                    .unwrap_or_else(|e| panic!("`getAccountInfo` call failed: {e}"))
                    .map(decode_ui_account)
            },
        )
        .await;

    assert_eq!(sol_res, ic_res);

    setup.setup.drop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn should_not_get_account_info() {
    let setup = Setup::new().await;
    let pubkey = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

    let (sol_res, ic_res) = setup
        .compare_client(
            |sol| solana_rpc_client_get_account(&pubkey, sol, None),
            |ic| async move {
                ic.get_account_info(pubkey)
                    .send()
                    .await
                    .expect_consistent()
                    .unwrap_or_else(|e| panic!("`getAccountInfo` call failed: {e}"))
                    .map(decode_ui_account)
            },
        )
        .await;

    assert_eq!(sol_res, ic_res);

    setup.setup.drop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn should_get_block() {
    let setup = Setup::new().await;

    for commitment in [
        GetBlockCommitmentLevel::Confirmed,
        GetBlockCommitmentLevel::Finalized,
    ] {
        let commitment_config: CommitmentConfig = commitment.clone().into();
        let slot = setup
            .solana_client
            .get_slot_with_commitment(commitment_config)
            .expect("Failed to get slot");

        let (sol_res, ic_res) = setup
            .compare_client(
                |sol| {
                    sol.get_block_with_config(
                        slot,
                        RpcBlockConfig {
                            encoding: None,
                            transaction_details: Some(solana_transaction_status_client_types::TransactionDetails::Signatures),
                            rewards: Some(false),
                            commitment: Some(commitment_config),
                            max_supported_transaction_version: None,
                        },
                    )
                        .expect("Failed to get block")
                },
                |ic| async move {
                    ic.get_block(GetBlockParams {
                        slot,
                        commitment: Some(commitment),
                        max_supported_transaction_version: None,
                        transaction_details: Some(TransactionDetails::Signatures),
                    })
                        .send()
                        .await
                        .expect_consistent()
                        .unwrap_or_else(|e| panic!("`getBlock` call failed: {e}"))
                        .unwrap_or_else(|| panic!("No block for slot {slot}"))
                },
            )
            .await;

        assert_eq!(sol_res, ic_res);
    }

    setup.setup.drop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn should_get_transaction() {
    let setup = Setup::new().await;

    // Generate a transaction and get the signature
    let signature = setup
        .solana_client
        .request_airdrop(&Keypair::new().pubkey(), 10_000_000_000)
        .expect("Error while requesting airdrop");

    setup.confirm_transaction(&signature);

    let (sol_res, ic_res) = setup
        .compare_client(
            |sol| {
                sol.get_transaction_with_config(
                    &signature,
                    RpcTransactionConfig {
                        encoding: Some(UiTransactionEncoding::Base64),
                        commitment: Some(CommitmentConfig::confirmed()),
                        max_supported_transaction_version: None,
                    },
                )
                .expect("Failed to get transaction")
            },
            |ic| async move {
                let mut params: GetTransactionParams = signature.into();
                params.encoding = Some(GetTransactionEncoding::Base64);
                params.commitment = Some(CommitmentLevel::Confirmed);
                ic.get_transaction(params)
                    .send()
                    .await
                    .expect_consistent()
                    .unwrap_or_else(|e| panic!("`getTransaction` call failed: {e}"))
                    .expect("Transaction not found")
            },
        )
        .await;

    assert_eq!(sol_res, ic_res);

    setup.setup.drop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn should_send_transaction() {
    let setup = Setup::new().await;

    let (sender, sender_balance_before) = setup.generate_keypair_and_fund_account();
    let (recipient, recipient_balance_before) = setup.generate_keypair_and_fund_account();

    let slot = setup
        .icp_client()
        .get_slot()
        .send()
        .await
        .expect_consistent()
        .expect("Call to get slot failed");
    let block = setup
        .icp_client()
        .get_block(slot)
        .send()
        .await
        .expect_consistent()
        .expect("Call to get block failed")
        .expect("Block not found");
    let blockhash = Hash::from_str(&block.blockhash).expect("Failed to parse blockhash");

    let transaction_amount = 1_000;
    let instruction =
        system_instruction::transfer(&sender.pubkey(), &recipient.pubkey(), transaction_amount);
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&sender.pubkey()),
        &[&sender],
        blockhash,
    );

    let mut params: SendTransactionParams = transaction.clone().try_into().unwrap();
    params.preflight_commitment = Some(CommitmentLevel::Confirmed);

    // Don't compare the result to the Solana validator since a transaction can only be submitted once.
    let transaction_id = setup
        .icp_client()
        .send_transaction(params)
        .send()
        .await
        .expect_consistent()
        .unwrap();

    // Wait until the transaction is confirmed.
    setup.confirm_transaction(&transaction_id);

    // Make sure the funds were sent from the sender to the recipient
    let sender_balance_after = setup.get_account_balance(&sender.pubkey());
    let recipient_balance_after = setup.get_account_balance(&recipient.pubkey());

    assert_eq!(
        recipient_balance_after,
        recipient_balance_before + transaction_amount
    );
    assert!(sender_balance_after + transaction_amount <= sender_balance_before);

    setup.setup.drop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn should_get_balance() {
    async fn compare_balances(setup: &Setup, account: Pubkey) -> Lamport {
        let pubkey = account;
        let (sol_res, ic_res) = setup
            .compare_client(
                |sol| sol.get_balance(&account).expect("Failed to get balance"),
                |ic| async move {
                    ic.get_balance(pubkey)
                        .modify_params(|params| {
                            params.commitment = Some(CommitmentLevel::Confirmed)
                        })
                        .send()
                        .await
                        .expect_consistent()
                        .expect("Failed to get balance from SOL RPC")
                },
            )
            .await;
        assert_eq!(sol_res, ic_res);
        sol_res
    }

    let setup = Setup::new().await;
    let user = Keypair::new();
    let publickey = user.pubkey();

    assert_eq!(compare_balances(&setup, publickey).await, 0);

    let tx = setup
        .solana_client
        .request_airdrop(&user.pubkey(), 10_000_000_000)
        .expect("Error while requesting airdrop");
    setup.confirm_transaction(&tx);

    assert_eq!(compare_balances(&setup, publickey).await, 10_000_000_000);
}

#[tokio::test(flavor = "multi_thread")]
async fn should_get_token_account_balance() {
    // TODO XC-325: Add test for `getTokenAccountBalance` (requires some SPL test infrastructure)
}

fn solana_rpc_client_get_account(
    pubkey: &Pubkey,
    sol: &RpcClient,
    config: Option<solana_rpc_client_api::config::RpcAccountInfoConfig>,
) -> Option<solana_account::Account> {
    sol.get_account_with_config(pubkey, config.unwrap_or_default())
        .expect("Failed to get account")
        .value
}

fn decode_ui_account(account: UiAccount) -> solana_account::Account {
    account
        .decode::<solana_account::Account>()
        .unwrap_or_else(|| panic!("Failed to decode account"))
}

pub struct Setup {
    solana_client: SolanaRpcClient,
    setup: sol_rpc_int_tests::Setup,
}

impl Setup {
    const SOLANA_VALIDATOR_URL: &'static str = "http://localhost:8899";

    pub async fn new() -> Self {
        let mut pic = PocketIcBuilder::new()
            .with_nns_subnet() //make_live requires NNS subnet.
            .with_fiduciary_subnet()
            .build_async()
            .await;
        let _endpoint = pic.make_live(None).await;
        Setup {
            solana_client: RpcClient::new_with_commitment(
                Self::SOLANA_VALIDATOR_URL,
                // Using confirmed commitment in tests provides faster execution while maintaining
                // sufficient reliability.
                CommitmentConfig::confirmed(),
            ),
            setup: sol_rpc_int_tests::Setup::with_pocket_ic_and_args(
                pic,
                InstallArgs {
                    override_provider: Some(OverrideProvider {
                        override_url: Some(RegexSubstitution {
                            pattern: ".*".into(),
                            replacement: Self::SOLANA_VALIDATOR_URL.to_string(),
                        }),
                    }),
                    ..Default::default()
                },
            )
            .await
            .with_mock_api_keys()
            .await,
        }
    }

    fn icp_client(&self) -> SolRpcClient<PocketIcLiveModeRuntime> {
        self.setup.client_live_mode().build()
    }

    async fn compare_client<'a, Sol, SolOutput, Icp, IcpOutput, Fut>(
        &'a self,
        solana_call: Sol,
        icp_call: Icp,
    ) -> (SolOutput, IcpOutput)
    where
        Sol: FnOnce(&SolanaRpcClient) -> SolOutput,
        Icp: FnOnce(SolRpcClient<PocketIcLiveModeRuntime<'a>>) -> Fut,
        Fut: Future<Output = IcpOutput>,
    {
        let a = async { solana_call(&self.solana_client) };
        let b = async { icp_call(self.icp_client()).await };
        future::join(a, b).await
    }

    fn generate_keypair_and_fund_account(&self) -> (Keypair, u64) {
        let keypair = Keypair::new();
        // Airdrop 10 SOL to the account
        self.solana_client
            .request_airdrop(&keypair.pubkey(), 10_000_000_000)
            .expect("Error while requesting airdrop");
        // Wait until the funds appear in the account
        let max_wait = Duration::from_secs(10);
        let start = Instant::now();
        loop {
            let account_balance = self.get_account_balance(&keypair.pubkey());
            if account_balance == 0 {
                if start.elapsed() > max_wait {
                    panic!("Timed out waiting for airdrop confirmation.");
                }
                sleep(Duration::from_millis(500));
            } else {
                return (keypair, account_balance);
            }
        }
    }

    fn get_account_balance(&self, pubkey: &Pubkey) -> u64 {
        self.solana_client
            .get_balance(pubkey)
            .expect("Error while getting account balance")
    }

    fn confirm_transaction(&self, transaction_id: &Signature) {
        // Wait until the transaction is confirmed
        let max_wait = Duration::from_secs(10);
        let start = Instant::now();
        loop {
            let confirmed = self
                .solana_client
                .confirm_transaction(transaction_id)
                .expect("Error while getting transaction confirmation status");
            if confirmed {
                return;
            } else {
                if start.elapsed() > max_wait {
                    panic!("Timed out waiting for transaction confirmation.");
                }
                sleep(Duration::from_millis(500));
            }
        }
    }
}
