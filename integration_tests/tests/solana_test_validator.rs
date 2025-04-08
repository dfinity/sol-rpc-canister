//! Tests to compare behavior between the official Solana RPC client directly interacting with a local validator
//! and the SOL RPC client that uses the SOL RPC canister that uses the local validator as JSON RPC provider.
//! Excepted for timing differences, the same behavior should be observed.

use futures::future;
use pocket_ic::PocketIcBuilder;
use sol_rpc_client::SolRpcClient;
use sol_rpc_int_tests::PocketIcLiveModeRuntime;
use sol_rpc_types::{
    GetAccountInfoEncoding, GetAccountInfoParams, InstallArgs, OverrideProvider, RegexSubstitution,
};
use solana_account_decoder_client_types::UiAccount;
use solana_client::rpc_client::{RpcClient as SolanaRpcClient, RpcClient};
use solana_keypair::Keypair;
use solana_program::system_instruction;
use solana_pubkey::Pubkey;
use solana_signer::{EncodableKey, Signer};
use solana_transaction::Transaction;
use std::{env::var, future::Future, path::PathBuf, str::FromStr};

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

    let sender = Keypair::read_from_file(
        PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap()).join("keypair1.json"),
    )
    .unwrap();
    let recipient = Keypair::read_from_file(
        PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap()).join("keypair2.json"),
    )
    .unwrap();

    let blockhash = setup.solana_client.get_latest_blockhash().unwrap();
    let instruction = system_instruction::transfer(&sender.pubkey(), &recipient.pubkey(), 1_000);
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&sender.pubkey()),
        &[&sender],
        blockhash,
    );
    let transaction_clone = transaction.clone();

    let (sol_res, ic_res) = setup
        .compare_client(
            |sol| {
                sol.send_transaction(&transaction)
                    .expect("Failed to send transaction")
            },
            |ic| async move {
                ic.send_transaction(transaction_clone)
                    .send()
                    .await
                    .expect_consistent()
                    .unwrap_or_else(|e| panic!("`sendTransaction` call failed: {e}"))
            },
        )
        .await;

    assert_eq!(sol_res, ic_res);

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
    const SOLANA_VALIDATOR_URL: &str = "http://localhost:8899";

    pub async fn new() -> Self {
        let mut pic = PocketIcBuilder::new()
            .with_nns_subnet() //make_live requires NNS subnet.
            .with_fiduciary_subnet()
            .build_async()
            .await;
        let _endpoint = pic.make_live(None).await;
        Setup {
            solana_client: RpcClient::new(Self::SOLANA_VALIDATOR_URL),
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
}
