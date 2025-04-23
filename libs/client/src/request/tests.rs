use crate::SolRpcClient;
use sol_rpc_types::{CommitmentLevel, GetAccountInfoParams, GetBalanceParams};
use solana_pubkey::pubkey;

#[test]
fn should_override_commitment_level() {
    let pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    let client_with_commitment_level = SolRpcClient::builder_for_ic()
        .with_default_commitment_level(CommitmentLevel::Confirmed)
        .build();

    let builder = client_with_commitment_level.get_account_info(pubkey);
    assert_eq!(
        builder.request.params.commitment,
        Some(CommitmentLevel::Confirmed)
    );

    let builder = client_with_commitment_level.get_account_info(GetAccountInfoParams {
        pubkey: pubkey.to_string(),
        commitment: Some(CommitmentLevel::Processed),
        encoding: None,
        data_slice: None,
        min_context_slot: None,
    });
    assert_eq!(
        builder.request.params.commitment,
        Some(CommitmentLevel::Processed)
    );

    let builder = client_with_commitment_level.get_balance(pubkey);
    assert_eq!(
        builder.request.params.commitment,
        Some(CommitmentLevel::Confirmed)
    );

    let builder = client_with_commitment_level.get_balance(GetBalanceParams {
        pubkey: pubkey.to_string(),
        commitment: Some(CommitmentLevel::Processed),
        min_context_slot: None,
    });
    assert_eq!(
        builder.request.params.commitment,
        Some(CommitmentLevel::Processed)
    );
}
