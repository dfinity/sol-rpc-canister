mod ed25519;
mod solana_wallet;
mod state;

use crate::{
    solana_wallet::SolanaWallet,
    state::{init_state, read_state},
};
use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::{
    api::management_canister::http_request::{
        CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
        TransformContext,
    },
    init, query, update,
};
use num::BigUint;
use serde_json::Value;
use std::fmt::Display;

const CONTENT_TYPE_HEADER_LOWERCASE: &str = "content-type";
const CONTENT_TYPE_VALUE: &str = "application/json";

#[init]
pub fn init(maybe_init: Option<InitArg>) {
    if let Some(init_arg) = maybe_init {
        init_state(init_arg)
    }
}

#[update]
pub async fn solana_account(owner: Option<Principal>) -> String {
    let caller = validate_caller_not_anonymous();
    let owner = owner.unwrap_or(caller);
    let wallet = SolanaWallet::new(owner).await;
    wallet.solana_account().to_string()
}

#[update]
pub async fn get_balance(account: Option<String>) -> Nat {
    let account = account.unwrap_or(solana_account(None).await);

    let json = format!(
        r#"{{ "jsonrpc": "2.0", "method": "getBalance", "params": ["{}"], "id": 1 }}"#,
        account
    );

    let solana_network = read_state(|s| s.solana_network());

    let max_response_size_bytes = 500_u64;
    let num_cycles = 1_000_000_000u128;

    // TODO: Call SOL RPC canister
    let response =
        json_rpc_request(json, solana_network, num_cycles, max_response_size_bytes).await;

    // The response to a successful `getBalance` call has the following format:
    // { "id": "[ID]", "jsonrpc": "2.0", "result": { "context": { "slot": [SLOT] }, "value": [BALANCE] },  }
    let balance = response["result"]["value"].as_u64().unwrap();

    Nat(BigUint::from(balance))
}

// TODO: Remove!
async fn json_rpc_request(
    json: String,
    solana_network: SolanaNetwork,
    num_cycles: u128,
    _max_response_size_bytes: u64,
) -> Value {
    use ic_cdk::api::management_canister::http_request::http_request;
    let url = match solana_network {
        SolanaNetwork::Devnet => "https://api.devnet.solana.com",
        _ => panic!("Unsupported Solana network: {:?}", solana_network),
    };
    let request = CanisterHttpRequestArgument {
        url: url.to_string(),
        max_response_bytes: None,
        method: HttpMethod::POST,
        headers: vec![HttpHeader {
            name: CONTENT_TYPE_HEADER_LOWERCASE.to_string(),
            value: CONTENT_TYPE_VALUE.to_string(),
        }],
        body: Some(json.as_bytes().to_vec()),
        transform: Some(TransformContext::from_name(
            "__transform_json_rpc".to_string(),
            vec![],
        )),
    };
    match http_request(request, num_cycles).await {
        Ok((response,)) => {
            serde_json::from_str(&String::from_utf8(response.body).unwrap()).unwrap()
        }
        Err((code, string)) => panic!(
            "Received an error response with code {:?}: {:?}",
            code, string
        ),
    }
}

// TODO: Remove!
#[query(name = "__transform_json_rpc", hidden = true)]
fn transform(args: TransformArgs) -> HttpResponse {
    fn canonicalize_json(text: &[u8]) -> Option<Vec<u8>> {
        let json = serde_json::from_slice::<Value>(text).ok()?;
        serde_json::to_vec(&json).ok()
    }

    fn transform_http_request(args: TransformArgs) -> HttpResponse {
        HttpResponse {
            status: args.response.status,
            body: canonicalize_json(&args.response.body).unwrap_or(args.response.body),
            // Remove headers (which may contain a timestamp) for consensus
            headers: vec![],
        }
    }

    transform_http_request(args)
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct InitArg {
    pub solana_network: Option<SolanaNetwork>,
    pub ed5519_key_name: Option<Ed25519KeyName>,
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum SolanaNetwork {
    Mainnet,
    #[default]
    Devnet,
    Testnet,
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub enum Ed25519KeyName {
    #[default]
    TestKeyLocalDevelopment,
    TestKey1,
    ProductionKey1,
}

impl Display for Ed25519KeyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Ed25519KeyName::TestKeyLocalDevelopment => "dfx_test_key",
            Ed25519KeyName::TestKey1 => "test_key_1",
            Ed25519KeyName::ProductionKey1 => "key_1",
        }
        .to_string();
        write!(f, "{}", str)
    }
}

pub fn validate_caller_not_anonymous() -> Principal {
    let principal = ic_cdk::caller();
    if principal == Principal::anonymous() {
        panic!("anonymous principal is not allowed");
    }
    principal
}
