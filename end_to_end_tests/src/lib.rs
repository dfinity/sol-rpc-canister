use async_trait::async_trait;
use candid::{utils::ArgumentEncoder, CandidType, Encode, Principal};
use ic_agent::{identity::Secp256k1Identity, Agent};
use ic_canister_runtime::IcError;
use ic_error_types::RejectCode;
use serde::de::DeserializeOwned;
use serde_json::json;
use sol_rpc_client::{ClientBuilder, Runtime, SolRpcClient};
use sol_rpc_int_tests::{
    decode_call_response, encode_args,
    wallet::{decode_cycles_wallet_response, CallCanisterArgs},
};
use sol_rpc_types::{
    CommitmentLevel, ConsensusStrategy, Lamport, MultiRpcResult, RpcSource, RpcSources,
    SupportedRpcProviderId,
};
use solana_client::rpc_client::RpcClient as SolanaRpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use solana_transaction_status_client_types::TransactionStatus;
use std::{env, time::Duration};

const DEFAULT_IC_GATEWAY: &str = "https://icp0.io";
const SOLANA_DEVNET_URL: &str = "https://api.devnet.solana.com";

pub struct Setup {
    agent: Agent,
    sol_rpc_canister_id: Principal,
    wallet_canister_id: Principal,
    solana_client: SolanaRpcClient,
}

impl Setup {
    pub fn new() -> Self {
        Self {
            agent: Agent::builder()
                .with_url(DEFAULT_IC_GATEWAY)
                .with_identity({
                    Secp256k1Identity::from_pem(env("DFX_DEPLOY_KEY").as_bytes())
                        .expect("Unable to import identity from PEM file")
                })
                .build()
                .expect("Could not build agent"),
            sol_rpc_canister_id: Principal::from_text(env("sol_rpc_canister_id")).unwrap(),
            wallet_canister_id: Principal::from_text(env("wallet_canister_id")).unwrap(),
            solana_client: SolanaRpcClient::new_with_commitment(
                SOLANA_DEVNET_URL,
                CommitmentConfig::confirmed(),
            ),
        }
    }

    pub fn new_ic_agent_runtime(&self) -> IcAgentRuntime<'_> {
        IcAgentRuntime {
            agent: &self.agent,
            wallet_canister_id: self.wallet_canister_id,
        }
    }

    pub fn client_builder(&self) -> ClientBuilder<IcAgentRuntime<'_>> {
        SolRpcClient::builder(self.new_ic_agent_runtime(), self.sol_rpc_canister_id)
    }

    pub fn client(&self) -> SolRpcClient<IcAgentRuntime<'_>> {
        self.client_builder()
            .with_rpc_sources(RpcSources::Custom(vec![
                RpcSource::Supported(SupportedRpcProviderId::AnkrDevnet),
                RpcSource::Supported(SupportedRpcProviderId::DrpcDevnet),
                RpcSource::Supported(SupportedRpcProviderId::HeliusDevnet),
            ]))
            .with_consensus_strategy(ConsensusStrategy::Threshold {
                min: 2,
                total: None,
            })
            .with_default_commitment_level(CommitmentLevel::Confirmed)
            .build()
    }

    pub async fn confirm_transaction(&self, transaction_id: &Signature) -> TransactionStatus {
        let mut num_trials = 0;
        loop {
            num_trials += 1;
            if num_trials > 20 {
                panic!("Failed to confirm transaction {transaction_id}");
            }
            let statuses = self
                .client()
                .get_signature_statuses([transaction_id])
                .unwrap()
                .send()
                .await;
            if let MultiRpcResult::Consistent(Ok(statuses)) = statuses {
                if let Some(Some(status)) = statuses.into_iter().next() {
                    if let Some(err) = &status.err {
                        panic!("Transaction failed with error {:?}", err);
                    }
                    if status.satisfies_commitment(CommitmentConfig::confirmed()) {
                        return status;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(400)).await;
        }
    }

    pub async fn airdrop(&self, account: &Pubkey, amount: Lamport) -> Lamport {
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

    // Fund account with the SolanaRpcClient to avoid hitting rate limits due to replicated calls.
    pub fn fund_account(&self, account: &Pubkey, amount: Lamport) {
        let balance = self
            .solana_client
            .get_balance(account)
            .expect("Failed to get account balance");
        if balance < amount {
            self.solana_client
                .request_airdrop(account, amount)
                .expect("Failed to request airdrop");
            self.solana_client.wait_for_balance_with_commitment(
                account,
                Some(balance + amount),
                CommitmentConfig::confirmed(),
            );
        }
    }

    pub async fn get_account_balance(&self, pubkey: &Pubkey) -> Lamport {
        self.client()
            .get_balance(*pubkey)
            .send()
            .await
            .expect_consistent()
            .unwrap_or_else(|_| panic!("Failed to fetch account balance for account {pubkey}"))
    }

    pub async fn get_median_recent_prioritization_fees(
        &self,
        sender_pubkey: &Pubkey,
        recipient_pubkey: &Pubkey,
    ) -> Lamport {
        let mut prioritization_fees: Vec<_> = self
            .client()
            .get_recent_prioritization_fees([sender_pubkey, recipient_pubkey])
            .unwrap()
            .send()
            .await
            .expect_consistent()
            .expect("Call to `getRecentPrioritizationFees` failed")
            .into_iter()
            .map(|fee| fee.prioritization_fee)
            .collect();
        prioritization_fees.sort();

        if prioritization_fees.is_empty() {
            0
        } else {
            prioritization_fees[prioritization_fees.len() / 2]
        }
    }
}

impl Default for Setup {
    fn default() -> Self {
        Self::new()
    }
}

pub fn env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Environment variable '{key}' is not set!"))
}

#[derive(Clone, Debug)]
pub struct IcAgentRuntime<'a> {
    pub agent: &'a Agent,
    pub wallet_canister_id: Principal,
}

impl<'a> IcAgentRuntime<'a> {
    pub fn new(agent: &'a Agent, wallet_canister_id: Principal) -> Self {
        Self {
            agent,
            wallet_canister_id,
        }
    }
}

#[async_trait]
impl Runtime for IcAgentRuntime<'_> {
    async fn update_call<In, Out>(
        &self,
        id: Principal,
        method: &str,
        args: In,
        cycles: u128,
    ) -> Result<Out, IcError>
    where
        In: ArgumentEncoder + Send,
        Out: CandidType + DeserializeOwned,
    {
        // Forward the call through the wallet canister
        let result = self
            .agent
            .update(&self.wallet_canister_id, "wallet_call128")
            .with_arg(Encode!(&CallCanisterArgs::new(id, method, args, cycles)).unwrap())
            .call_and_wait()
            .await
            .map_err(|e| IcError::CallRejected {
                code: RejectCode::SysFatal,
                message: e.to_string(),
            })?;
        decode_cycles_wallet_response(result)
    }

    async fn query_call<In, Out>(
        &self,
        id: Principal,
        method: &str,
        args: In,
    ) -> Result<Out, IcError>
    where
        In: ArgumentEncoder + Send,
        Out: CandidType + DeserializeOwned,
    {
        let result = self
            .agent
            .query(&id, method)
            .with_arg(encode_args(args))
            .call()
            .await
            .map_err(|e| IcError::CallRejected {
                code: RejectCode::SysFatal,
                message: e.to_string(),
            })?;
        decode_call_response(result)
    }
}
