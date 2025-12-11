//! Tests to compare behavior between the official Solana RPC client directly interacting with a local validator
//! and the SOL RPC client that uses the SOL RPC canister that uses the local validator as JSON RPC provider.
//! Excepted for timing differences, the same behavior should be observed.

use assert_matches::assert_matches;
use futures::future;
use ic_canister_runtime::CyclesWalletRuntime;
use ic_pocket_canister_runtime::PocketIcRuntime;
use pocket_ic::PocketIcBuilder;
use sol_rpc_client::SolRpcClient;
use sol_rpc_types::{
    CommitmentLevel, ConfirmedTransactionStatusWithSignature, GetAccountInfoEncoding,
    GetBlockCommitmentLevel, GetTransactionEncoding, InstallArgs, Lamport, OverrideProvider,
    PrioritizationFee, RegexSubstitution, TransactionDetails, TransactionStatus,
};
use solana_account_decoder_client_types::{token::UiTokenAmount, UiAccount};
use solana_client::rpc_client::{
    GetConfirmedSignaturesForAddress2Config, RpcClient as SolanaRpcClient,
};
use solana_commitment_config::CommitmentConfig;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_keypair::Keypair;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    sysvar,
};
use solana_pubkey::{pubkey, Pubkey};
use solana_rpc_client_api::{
    config::{RpcBlockConfig, RpcTransactionConfig},
    response::{RpcConfirmedTransactionStatusWithSignature, RpcPrioritizationFee},
};
use solana_sdk_ids::system_program;
use solana_signature::Signature;
use solana_signer::Signer;
use solana_system_interface::instruction;
use solana_transaction::Transaction;
use solana_transaction_status_client_types::UiTransactionEncoding;
use spl_associated_token_account_interface::{
    address::get_associated_token_address_with_program_id,
    instruction::create_associated_token_account,
};
use std::{
    future::Future,
    iter::zip,
    num::NonZeroU8,
    thread::sleep,
    time::{Duration, Instant},
};

pub const SPL_TOKEN_2022_ID: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

#[tokio::test(flavor = "multi_thread")]
async fn should_get_slot() {
    let setup = Setup::new().await;

    let (sol_res, ic_res) = setup
        .compare_client(
            |sol| sol.get_slot().expect("Failed to get slot"),
            |ic| async move {
                ic.get_slot()
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
            instruction::transfer(&sender.pubkey(), &recipient.pubkey(), transaction_amount);
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
        + NUM_TRANSACTIONS * (NUM_TRANSACTIONS + 1) / 2; //prioritization_fee = 1 µL * CUL / 1_000_000 + .. + NUM_TRANSACTIONS * CUL / 1_000_000 and compute unit limit was set to 1 million.
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
                    .with_max_length(NonZeroU8::new(150).unwrap())
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

    let (sol_res, ic_res) = setup
        .compare_client(
            |sol| {
                sol.get_account_with_commitment(
                    &system_program::id(),
                    CommitmentConfig::confirmed(),
                )
                .expect("Failed to get account")
                .value
            },
            |ic| async move {
                ic.get_account_info(system_program::id())
                    .with_encoding(GetAccountInfoEncoding::Base64)
                    .send()
                    .await
                    .expect_consistent()
                    .unwrap_or_else(|e| panic!("`getAccountInfo` call failed: {e}"))
                    .map(decode_ui_account)
            },
        )
        .await;

    assert_matches!(sol_res, Some(_));
    assert_matches!(ic_res, Some(_));
    assert_eq!(sol_res, ic_res);

    setup.setup.drop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn should_not_get_account_info() {
    let setup = Setup::new().await;
    let pubkey = Pubkey::new_unique();

    let (sol_res, ic_res) = setup
        .compare_client(
            |sol| {
                sol.get_account_with_commitment(&pubkey, CommitmentConfig::confirmed())
                    .expect("Failed to get account")
                    .value
            },
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

    assert_eq!(sol_res, None);
    assert_eq!(ic_res, None);

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
                            transaction_details: Some(solana_transaction_status_client_types::TransactionDetails::Signatures),
                            commitment: Some(commitment_config),
                            ..RpcBlockConfig::default()
                        },
                    )
                        .expect("Failed to get block")
                },
                |ic| async move {
                    ic.get_block(slot)
                        .with_transaction_details(TransactionDetails::Signatures)
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
                ic.get_transaction(signature)
                    .with_encoding(GetTransactionEncoding::Base64)
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

    let blockhash = setup
        .icp_client()
        .estimate_recent_blockhash()
        .send()
        .await
        .unwrap();

    let transaction_amount = 1_000;
    let instruction =
        instruction::transfer(&sender.pubkey(), &recipient.pubkey(), transaction_amount);
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&sender.pubkey()),
        &[&sender],
        blockhash,
    );

    // Don't compare the result to the Solana validator since a transaction can only be submitted once.
    let transaction_id = setup
        .icp_client()
        .send_transaction(transaction)
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

    setup.setup.drop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn should_get_token_account_balance() {
    async fn compare_balances(setup: &Setup, account: Pubkey) -> UiTokenAmount {
        let pubkey = account;
        let (sol_res, ic_res) = setup
            .compare_client(
                |sol| {
                    sol.get_token_account_balance(&account)
                        .expect("Failed to get token account balance")
                },
                |ic| async move {
                    ic.get_token_account_balance(pubkey)
                        .send()
                        .await
                        .expect_consistent()
                        .expect("Failed to get token account balance from SOL RPC")
                },
            )
            .await;
        assert_eq!(sol_res, ic_res);
        sol_res
    }

    let setup = Setup::new().await;
    let (user, _) = setup.generate_keypair_and_fund_account();
    let (mint_authority, mint_account) = setup.create_spl_token();
    let associated_token_account = setup.create_associated_token_account(&user, &mint_account);

    assert_eq!(
        compare_balances(&setup, associated_token_account).await,
        UiTokenAmount {
            ui_amount: Some(0.0),
            decimals: 9,
            amount: "0".to_string(),
            ui_amount_string: "0".to_string(),
        }
    );

    setup.mint_spl(
        &mint_authority,
        1_000,
        mint_account,
        associated_token_account,
    );

    assert_eq!(
        compare_balances(&setup, associated_token_account).await,
        UiTokenAmount {
            ui_amount: Some(1e-6),
            decimals: 9,
            amount: "1000".to_string(),
            ui_amount_string: "0.000001".to_string()
        }
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn should_get_signature_statuses() {
    let setup = Setup::new().await;

    let signatures = {
        // Generate a transaction and get the signature
        let sig_1 = setup
            .solana_client
            .request_airdrop(&Keypair::new().pubkey(), 10_000_000_000)
            .expect("Error while requesting airdrop");
        setup.confirm_transaction(&sig_1);
        // An arbitrary signature not corresponding to any transaction
        let sig_2 = Signature::from([57u8; 64]);
        &vec![sig_1, sig_2]
    };

    let (sol_res, ic_res) = setup
        .compare_client(
            |sol| {
                sol.get_signature_statuses(signatures)
                    .expect("Failed to get signature statuses")
                    .value
            },
            |ic| async move {
                ic.get_signature_statuses(signatures)
                    .unwrap()
                    .send()
                    .await
                    .expect_consistent()
                    .unwrap_or_else(|e| panic!("`getSignatureStatuses` call failed: {e}"))
            },
        )
        .await;

    // Convert to sol_rpc_type::TransactionStatus to avoid comparing TransactionStatus#confirmations
    // which changes fast and hence is usually different for both calls
    assert_eq!(
        sol_res
            .into_iter()
            .map(|maybe_status| maybe_status.map(TransactionStatus::from))
            .collect::<Vec<_>>(),
        ic_res
            .into_iter()
            .map(|maybe_status| maybe_status.map(TransactionStatus::from))
            .collect::<Vec<_>>()
    );

    setup.setup.drop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn should_get_signatures_for_address() {
    let setup = Setup::new().await;

    // Start searching backwards from this transaction. This ensures the two clients get the
    // same answer. Otherwise, they might get different results due to one of them including
    // transactions from more recent blocks than others.
    // Also wait until this transaction is finalized, so that all transactions before it should
    // be finalized as well. Otherwise, the confirmation status of some transactions might change
    // while fetching them and the two clients might get different results.
    let before = setup
        .solana_client
        .request_airdrop(&Keypair::new().pubkey(), 10_000_000_000)
        .expect("Error while requesting airdrop");
    setup.confirm_transaction_with_commitment(&before, CommitmentConfig::finalized());

    let (sol_res, ic_res) = setup
        .compare_client(
            |sol| {
                sol.get_signatures_for_address_with_config(
                    &system_program::id(),
                    GetConfirmedSignaturesForAddress2Config {
                        before: Some(before),
                        until: None,
                        limit: Some(10),
                        commitment: Some(setup.solana_client.commitment()),
                    },
                )
                .unwrap_or_else(|e| panic!("Failed to get signatures for address: {e}"))
            },
            |ic| async move {
                ic.get_signatures_for_address(system_program::id())
                    .with_limit(10.try_into().unwrap())
                    .with_before(before)
                    .send()
                    .await
                    .expect_consistent()
                    .unwrap_or_else(|e| panic!("`getSignaturesForAddress` call failed: {e}"))
                    .into_iter()
                    .map(from_confirmed_transaction_status_with_signature)
                    .collect::<Vec<_>>()
            },
        )
        .await;

    assert_eq!(sol_res, ic_res);

    setup.setup.drop().await;
}

fn from_confirmed_transaction_status_with_signature(
    status: ConfirmedTransactionStatusWithSignature,
) -> RpcConfirmedTransactionStatusWithSignature {
    let ConfirmedTransactionStatusWithSignature {
        signature,
        slot,
        err,
        memo,
        block_time,
        confirmation_status,
    } = status;
    RpcConfirmedTransactionStatusWithSignature {
        signature: signature.into(),
        slot,
        err: err.map(Into::into),
        memo,
        block_time,
        confirmation_status: confirmation_status.map(Into::into),
    }
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
            solana_client: SolanaRpcClient::new_with_commitment(
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

    fn icp_client(&self) -> SolRpcClient<CyclesWalletRuntime<PocketIcRuntime<'_>>> {
        self.setup
            .client()
            .with_default_commitment_level(CommitmentLevel::Confirmed)
            .build()
    }

    async fn compare_client<'a, Sol, SolOutput, Icp, IcpOutput, Fut>(
        &'a self,
        solana_call: Sol,
        icp_call: Icp,
    ) -> (SolOutput, IcpOutput)
    where
        Sol: FnOnce(&SolanaRpcClient) -> SolOutput,
        Icp: FnOnce(SolRpcClient<CyclesWalletRuntime<PocketIcRuntime<'a>>>) -> Fut,
        Fut: Future<Output = IcpOutput>,
    {
        let a = async { solana_call(&self.solana_client) };
        let b = async { icp_call(self.icp_client()).await };
        future::join(a, b).await
    }

    fn airdrop(&self, account: &Pubkey, amount: u64) {
        let balance_before = self.solana_client.get_balance(account).unwrap();
        let _airdrop_tx = self.solana_client.request_airdrop(account, amount).unwrap();
        let expected_balance = balance_before + amount;
        assert_eq!(
            self.solana_client.wait_for_balance_with_commitment(
                account,
                Some(expected_balance),
                self.solana_client.commitment()
            ),
            Some(expected_balance)
        );
    }

    fn generate_keypair_and_fund_account(&self) -> (Keypair, u64) {
        let keypair = Keypair::new();
        let amount = 10_000_000_000;
        self.airdrop(&keypair.pubkey(), amount);
        (keypair, amount)
    }

    fn get_account_balance(&self, pubkey: &Pubkey) -> u64 {
        self.solana_client
            .get_balance(pubkey)
            .expect("Error while getting account balance")
    }

    pub fn create_associated_token_account(&self, user: &Keypair, mint_account: &Pubkey) -> Pubkey {
        let instruction = create_associated_token_account(
            &user.pubkey(),
            &user.pubkey(),
            mint_account,
            &SPL_TOKEN_2022_ID,
        );

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&user.pubkey()),
            &[&user],
            self.solana_client
                .get_latest_blockhash()
                .expect("Unable to fetch latest blockhash"),
        );

        self.solana_client
            .send_transaction(&transaction)
            .expect("Unable to create associated token account");

        let associated_token_account = get_associated_token_address_with_program_id(
            &user.pubkey(),
            mint_account,
            &SPL_TOKEN_2022_ID,
        );

        self.wait_for_account_to_exist(&associated_token_account);

        associated_token_account
    }

    fn create_spl_token(&self) -> (Keypair, Pubkey) {
        const MIN_ACCOUNT_LEN: u8 = 82;

        let mint_authority = Keypair::new();
        self.airdrop(&mint_authority.pubkey(), 1_000_000_000);
        let mint_account = Keypair::new();
        let mint_rent = self
            .solana_client
            .get_minimum_balance_for_rent_exemption(MIN_ACCOUNT_LEN as usize)
            .unwrap();
        let create_mint_account_ix = solana_system_interface::instruction::create_account(
            &mint_authority.pubkey(),
            &mint_account.pubkey(),
            mint_rent,
            MIN_ACCOUNT_LEN as u64,
            &SPL_TOKEN_2022_ID,
        );
        // See https://github.com/solana-program/token-2022/blob/644f0b014cbdb25c11c20ccedfb6e412d399b6dc/program/src/instruction.rs#L1207
        let initialize_mint_ix = {
            let decimals: u8 = 9;
            let mut buf = Vec::with_capacity(35);
            buf.push(0);
            buf.push(decimals);
            buf.extend_from_slice(mint_authority.pubkey().as_ref());
            buf.push(0); //no freeze authority

            Instruction {
                program_id: SPL_TOKEN_2022_ID,
                accounts: vec![
                    AccountMeta::new(mint_account.pubkey(), false),
                    AccountMeta::new_readonly(sysvar::rent::id(), false),
                ],
                data: buf,
            }
        };
        let token_mint = Transaction::new_signed_with_payer(
            &[create_mint_account_ix, initialize_mint_ix],
            Some(&mint_authority.pubkey()),
            &[&mint_authority, &mint_account],
            self.solana_client.get_latest_blockhash().unwrap(),
        );
        self.solana_client
            .send_and_confirm_transaction(&token_mint)
            .unwrap();
        (mint_authority, mint_account.pubkey())
    }

    fn mint_spl(
        &self,
        mint_authority: &Keypair,
        amount: u64,
        mint_account: Pubkey,
        user_associated_token_account: Pubkey,
    ) {
        assert!(
            self.solana_client
                .get_token_account(&user_associated_token_account)
                .unwrap()
                .is_some(),
            "Associated token account {user_associated_token_account} not found"
        );

        let mint_ix = {
            let mut buf = Vec::with_capacity(9);
            buf.push(7);
            buf.extend_from_slice(&amount.to_le_bytes());
            Instruction {
                program_id: SPL_TOKEN_2022_ID,
                accounts: vec![
                    AccountMeta::new(mint_account, false),
                    AccountMeta::new(user_associated_token_account, false),
                    AccountMeta::new_readonly(mint_authority.pubkey(), true),
                ],
                data: buf,
            }
        };

        let mint_spl_tx = Transaction::new_signed_with_payer(
            &[mint_ix],
            Some(&mint_authority.pubkey()),
            &[mint_authority],
            self.solana_client.get_latest_blockhash().unwrap(),
        );
        self.solana_client
            .send_and_confirm_transaction(&mint_spl_tx)
            .unwrap();
    }

    fn wait_for_account_to_exist(&self, account: &Pubkey) {
        let commitment_level = self.solana_client.commitment();
        let mut num_trials = 0;
        loop {
            num_trials += 1;
            if num_trials > 20 {
                panic!(
                    "Account {account} does not have desired commitment level {commitment_level:?}",
                );
            }
            let result = self
                .solana_client
                .get_account_with_commitment(account, commitment_level)
                .unwrap_or_else(|e| panic!("Failed to retrieve account {account}: {e}"));
            match result.value {
                Some(found_account) if found_account.lamports > 0 => {
                    break;
                }
                _ => {
                    sleep(Duration::from_millis(400));
                    continue;
                }
            }
        }
    }

    fn confirm_transaction(&self, transaction_id: &Signature) {
        self.confirm_transaction_with_commitment(transaction_id, self.solana_client.commitment())
    }

    fn confirm_transaction_with_commitment(
        &self,
        transaction_id: &Signature,
        commitment: CommitmentConfig,
    ) {
        // Wait until the transaction is confirmed
        let max_wait = Duration::from_secs(30);
        let start = Instant::now();
        loop {
            let confirmed = self
                .solana_client
                .confirm_transaction_with_commitment(transaction_id, commitment)
                .expect("Error while getting transaction confirmation status")
                .value;
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
