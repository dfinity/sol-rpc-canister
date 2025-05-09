use crate::{SolRpcClient, SolRpcEndpoint};
use serde_json::json;
use sol_rpc_types::{
    CommitmentLevel, GetBlockCommitmentLevel, SendTransactionEncoding, SendTransactionParams,
};
use solana_pubkey::pubkey;
use solana_signature::Signature;
use strum::IntoEnumIterator;

#[test]
fn should_set_correct_commitment_level() {
    let pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    let client_with_commitment_level = SolRpcClient::builder_for_ic()
        .with_default_commitment_level(CommitmentLevel::Confirmed)
        .build();
    let client_without_commitment_level = SolRpcClient::builder_for_ic().build();

    for endpoint in SolRpcEndpoint::iter() {
        match endpoint {
            SolRpcEndpoint::GetAccountInfo => {
                let builder = client_with_commitment_level.get_account_info(pubkey);
                assert_eq!(
                    builder.request.params.commitment,
                    Some(CommitmentLevel::Confirmed)
                );
            }
            SolRpcEndpoint::GetBalance => {
                let builder = client_with_commitment_level.get_balance(pubkey);
                assert_eq!(
                    builder.request.params.commitment,
                    Some(CommitmentLevel::Confirmed)
                );
            }
            SolRpcEndpoint::GetBlock => {
                let builder = client_with_commitment_level.get_block(1_u64);
                assert_eq!(
                    builder.request.params.commitment,
                    Some(GetBlockCommitmentLevel::Confirmed)
                );
            }
            SolRpcEndpoint::GetRecentPrioritizationFees => {
                //no op, GetRecentPrioritizationFees does not use commitment level
            }
            SolRpcEndpoint::GetSlot => {
                let builder = client_with_commitment_level.get_slot();
                assert_eq!(
                    builder.request.params.and_then(|p| p.commitment),
                    Some(CommitmentLevel::Confirmed)
                );
            }
            SolRpcEndpoint::GetTokenAccountBalance => {
                let builder = client_with_commitment_level.get_token_account_balance(pubkey);
                assert_eq!(
                    builder.request.params.commitment,
                    Some(CommitmentLevel::Confirmed)
                );
            }
            SolRpcEndpoint::GetTransaction => {
                let builder = client_with_commitment_level.get_transaction("tspfR5p1PFphquz4WzDb7qM4UhJdgQXkEZtW88BykVEdX2zL2kBT9kidwQBviKwQuA3b6GMCR1gknHvzQ3r623T".parse::<Signature>().unwrap());
                assert_eq!(
                    builder.request.params.commitment,
                    Some(CommitmentLevel::Confirmed)
                );
            }
            SolRpcEndpoint::JsonRequest => {
                let json_req = json!({ "jsonrpc": "2.0", "id": 1, "method": "getVersion" });
                let builder_with_level =
                    client_with_commitment_level.json_request(json_req.clone());
                let builder_without_level = client_without_commitment_level.json_request(json_req);
                assert_eq!(builder_with_level.request, builder_without_level.request);
            }
            SolRpcEndpoint::SendTransaction => {
                let builder = client_with_commitment_level.send_transaction(
                    SendTransactionParams::from_encoded_transaction(
                        "abcD".to_string(),
                        SendTransactionEncoding::Base64,
                    ),
                );
                assert_eq!(
                    builder.request.params.preflight_commitment,
                    Some(CommitmentLevel::Confirmed)
                );
            }
        }
    }
}
