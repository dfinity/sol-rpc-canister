//! Tests to compare behavior between the official Solana RPC client directly interacting with a local validator
//! and the SOL RPC client that uses the SOL RPC canister that uses the local validator as JSON RPC provider.
//! Excepted for timing differences, the same behavior should be observed.

use futures::future;
use pocket_ic::PocketIcBuilder;
use sol_rpc_client::SolRpcClient;
use sol_rpc_int_tests::PocketIcLiveModeRuntime;
use sol_rpc_types::{
    CommitmentLevel, GetAccountInfoEncoding, GetAccountInfoParams, GetSlotParams, InstallArgs,
    OverrideProvider, RegexSubstitution, SendTransactionParams,
};
use solana_account_decoder_client_types::UiAccount;
use solana_client::rpc_client::{RpcClient as SolanaRpcClient, RpcClient};
use solana_commitment_config::CommitmentConfig;
use solana_hash::Hash;
use solana_keypair::Keypair;
use solana_program::system_instruction;
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use solana_signer::Signer;
use solana_transaction::Transaction;
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
async fn should_send_transaction() {
    let setup = Setup::new().await;

    let (sender, sender_balance_before) = setup.generate_keypair_and_fund_account();
    let (recipient, recipient_balance_before) = setup.generate_keypair_and_fund_account();

    let transaction_amount = 1_000;
    let instruction =
        system_instruction::transfer(&sender.pubkey(), &recipient.pubkey(), transaction_amount);
    // TODO XC-289: get the block hash via `getSlot` + `getBlock`
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&sender.pubkey()),
        &[&sender],
        setup.get_latest_blockhash(),
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

    // Make sure the funds were sent from the sender to the recipient
    let sender_balance_after = setup.get_account_balance(&sender.pubkey());
    let recipient_balance_after = setup.get_account_balance(&recipient.pubkey());

    assert_eq!(
        recipient_balance_after,
        recipient_balance_before + transaction_amount
    );
    assert!(sender_balance_after + transaction_amount <= sender_balance_before);

    // Make sure the transaction whose ID was returned is indeed confirmed
    assert!(setup.confirm_transaction(&transaction_id));

    setup.setup.drop().await;
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

    fn get_latest_blockhash(&self) -> Hash {
        self.solana_client
            .get_latest_blockhash()
            .expect("Error while getting latest blockhash")
    }

    fn confirm_transaction(&self, transaction_id: &Signature) -> bool {
        self.solana_client
            .confirm_transaction(transaction_id)
            .expect("Error while getting confirming transaction")
    }
}
