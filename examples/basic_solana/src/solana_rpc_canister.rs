use crate::SolanaNetwork;
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};
use serde_json::Value;
use solana_hash::Hash;
use solana_nonce::{state::State, versions::Versions as NonceVersions};
use solana_transaction::Transaction;

const CONTENT_TYPE_HEADER_LOWERCASE: &str = "content-type";
const CONTENT_TYPE_VALUE: &str = "application/json";

pub struct SolanaRpcCanister;

impl SolanaRpcCanister {
    pub async fn get_nonce_account_blockhash(
        &self,
        solana_network: SolanaNetwork,
        num_cycles: u128,
        max_response_size_bytes: u64,
        account: String,
    ) -> Hash {
        let json = format!(
            r#"{{ "jsonrpc": "2.0", "method": "getAccountInfo", "params": ["{}", {{ "encoding": "base64" }}], "id": 1 }}"#,
            account
        );

        let response = self
            .json_rpc_request(solana_network, json, num_cycles, max_response_size_bytes)
            .await;

        // The response to a successful `getAccountInfo` call has the following format:
        // { "id": "[ID]", "jsonrpc": "2.0", "result": { ..., "value": { "data": [DATA, "base64"], ... } }, }
        let account_data = response["result"]["value"]["data"].as_array().unwrap()[0]
            .as_str()
            .unwrap();

        let account_data = bincode::deserialize::<NonceVersions>(
            base64::decode(account_data)
                .expect("Failed to decode account data")
                .as_slice(),
        )
        .expect("Failed to deserialize nonce account");

        match account_data.state() {
            State::Uninitialized => panic!("Nonce account is uninitialized"),
            State::Initialized(data) => data.blockhash(),
        }
    }

    pub async fn get_latest_blockhash(
        &self,
        solana_network: SolanaNetwork,
        num_cycles: u128,
        max_response_size_bytes: u64,
    ) -> Hash {
        let json = r#"{ "jsonrpc": "2.0", "method": "getLatestBlockhash", "params": [], "id": 1 }"#
            .to_string();
        let response = self
            .json_rpc_request(solana_network, json, num_cycles, max_response_size_bytes)
            .await;
        // The response to a successful `getLatestBlockHash` call has the following format:
        // { "id": "[ID]", "jsonrpc": "2.0", "result": { "context": { "slot": [SLOT] } }, "value": { "blockhash": [BLOCKHASH], "latestValidBlockHeight": [HEIGHT] }, }
        response["result"]["value"]["blockhash"]
            .as_str()
            .expect("Failed to extract blockhash")
            .to_string()
            .parse()
            .unwrap()
    }

    pub async fn send_transaction(
        &self,
        solana_network: SolanaNetwork,
        num_cycles: u128,
        max_response_size_bytes: u64,
        transaction: Transaction,
    ) -> String {
        let transaction =
            bincode::serialize(&transaction).expect("Failed to serialize transaction");
        let json = format!(
            r#"{{ "jsonrpc": "2.0", "method": "sendTransaction", "params": ["{}", {{ "encoding": "base64" }}], "id": 1 }}"#,
            base64::encode(transaction)
        );
        let response = self
            .json_rpc_request(solana_network, json, num_cycles, max_response_size_bytes)
            .await;
        // The response to a successful `sendTransaction` call has the following format:
        // { "id": "[ID]", "jsonrpc": "2.0", "result": [TXID], }
        response["result"]
            .as_str()
            .unwrap_or_else(|| panic!("Failed to extract transaction ID: {:?}", response))
            .to_string()
    }

    pub async fn json_rpc_request(
        &self,
        solana_network: SolanaNetwork,
        json: String,
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
            Ok((response,)) => serde_json::from_str(
                &String::from_utf8(response.body).expect("Failed to extract body"),
            )
            .expect("Failed to parse JSON"),
            Err((code, string)) => panic!(
                "Received an error response with code {:?}: {:?}",
                code, string
            ),
        }
    }
}

pub fn transform_http_request(args: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: args.response.status,
        body: canonicalize_json(&args.response.body).unwrap_or(args.response.body),
        // Remove headers (which may contain a timestamp) for consensus
        headers: vec![],
    }
}

fn canonicalize_json(text: &[u8]) -> Option<Vec<u8>> {
    let json = serde_json::from_slice::<Value>(text).ok()?;
    serde_json::to_vec(&json).ok()
}
