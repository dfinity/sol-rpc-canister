use crate::{GetRecentBlockError, RequestBuilder, SolRpcClient, SolRpcEndpoint};
use serde_json::json;
use sol_rpc_types::{
    CommitmentLevel, ConsensusStrategy, DataSlice, GetAccountInfoEncoding, GetAccountInfoParams,
    GetBalanceParams, GetBlockCommitmentLevel, GetBlockParams, GetSignatureStatusesParams,
    GetSignaturesForAddressParams, GetSlotParams, GetTokenAccountBalanceParams,
    GetTransactionEncoding, GetTransactionParams, RpcConfig, RpcSources, SendTransactionEncoding,
    SendTransactionParams, Slot, SupportedRpcProviderId, TransactionDetails,
};
use sol_rpc_types::{ConfirmedBlock, Hash, MultiRpcResult, RpcError, RpcSource};
use solana_pubkey::{pubkey, Pubkey};
use solana_signature::Signature;
use std::{fmt::Debug, num::NonZeroUsize, str::FromStr};
use strum::IntoEnumIterator;

const PUBKEY: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const BLOCKHASH: &str = "C6Cxgzq6yZWxjYnxwvxvP2dhWFeQSEVxRQbUXG2eMYsY";
const MIN_CONTEXT_SLOT: Slot = 1144441;
const SLOT: Slot = 332_577_897;

#[test]
fn should_set_correct_commitment_level() {
    let client_with_commitment_level = SolRpcClient::builder_for_ic()
        .with_default_commitment_level(CommitmentLevel::Confirmed)
        .build();
    let client_without_commitment_level = SolRpcClient::builder_for_ic().build();

    for endpoint in SolRpcEndpoint::iter() {
        match endpoint {
            SolRpcEndpoint::GetAccountInfo => {
                let builder = client_with_commitment_level.get_account_info(PUBKEY);
                assert_eq!(
                    builder.request.params.commitment,
                    Some(CommitmentLevel::Confirmed)
                );
            }
            SolRpcEndpoint::GetBalance => {
                let builder = client_with_commitment_level.get_balance(PUBKEY);
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
                // no op, GetRecentPrioritizationFees does not use commitment level
            }
            SolRpcEndpoint::GetSignaturesForAddress => {
                let builder = client_with_commitment_level.get_signatures_for_address(PUBKEY);
                assert_eq!(
                    builder.request.params.commitment,
                    Some(CommitmentLevel::Confirmed)
                );
            }
            SolRpcEndpoint::GetSignatureStatuses => {
                // no op, GetSignatureStatuses does not use commitment level
            }
            SolRpcEndpoint::GetSlot => {
                let builder = client_with_commitment_level.get_slot();
                assert_eq!(
                    builder.request.params.and_then(|p| p.commitment),
                    Some(CommitmentLevel::Confirmed)
                );
            }
            SolRpcEndpoint::GetTokenAccountBalance => {
                let builder = client_with_commitment_level.get_token_account_balance(PUBKEY);
                assert_eq!(
                    builder.request.params.commitment,
                    Some(CommitmentLevel::Confirmed)
                );
            }
            SolRpcEndpoint::GetTransaction => {
                let builder = client_with_commitment_level.get_transaction(signature());
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

#[test]
fn should_set_request_parameters() {
    let client = SolRpcClient::builder_for_ic().build();

    for endpoint in SolRpcEndpoint::iter() {
        match endpoint {
            SolRpcEndpoint::GetAccountInfo => assert_params_eq(
                client
                    .get_account_info(PUBKEY)
                    .with_commitment(CommitmentLevel::Confirmed)
                    .with_encoding(GetAccountInfoEncoding::Base64)
                    .with_data_slice(DataSlice {
                        length: 1,
                        offset: 2,
                    })
                    .with_min_context_slot(MIN_CONTEXT_SLOT),
                client
                    .get_account_info(PUBKEY)
                    .with_params(GetAccountInfoParams {
                        pubkey: PUBKEY.into(),
                        commitment: Some(CommitmentLevel::Confirmed),
                        encoding: Some(GetAccountInfoEncoding::Base64),
                        data_slice: Some(DataSlice {
                            length: 1,
                            offset: 2,
                        }),
                        min_context_slot: Some(MIN_CONTEXT_SLOT),
                    }),
            ),
            SolRpcEndpoint::GetBalance => assert_params_eq(
                client
                    .get_balance(PUBKEY)
                    .with_commitment(CommitmentLevel::Confirmed)
                    .with_min_context_slot(MIN_CONTEXT_SLOT),
                client.get_balance(PUBKEY).with_params(GetBalanceParams {
                    pubkey: PUBKEY.into(),
                    commitment: Some(CommitmentLevel::Confirmed),
                    min_context_slot: Some(MIN_CONTEXT_SLOT),
                }),
            ),
            SolRpcEndpoint::GetBlock => assert_params_eq(
                client
                    .get_block(123)
                    .with_commitment(GetBlockCommitmentLevel::Confirmed)
                    .with_max_supported_transaction_version(0)
                    .with_transaction_details(TransactionDetails::Signatures)
                    .without_rewards(),
                client.get_block(GetBlockParams {
                    slot: 123,
                    commitment: Some(GetBlockCommitmentLevel::Confirmed),
                    max_supported_transaction_version: Some(0),
                    transaction_details: Some(TransactionDetails::Signatures),
                    rewards: Some(false),
                }),
            ),
            SolRpcEndpoint::GetRecentPrioritizationFees => {
                // No optional request parameters
            }
            SolRpcEndpoint::GetSignaturesForAddress => assert_params_eq(
                client
                    .get_signatures_for_address(PUBKEY)
                    .with_commitment(CommitmentLevel::Confirmed)
                    .with_min_context_slot(MIN_CONTEXT_SLOT)
                    .with_limit(456.try_into().unwrap())
                    .with_before(signature())
                    .with_until(another_signature()),
                client.get_signatures_for_address(GetSignaturesForAddressParams {
                    pubkey: PUBKEY.into(),
                    commitment: Some(CommitmentLevel::Confirmed),
                    min_context_slot: Some(MIN_CONTEXT_SLOT),
                    limit: Some(456.try_into().unwrap()),
                    before: Some(signature().into()),
                    until: Some(another_signature().into()),
                }),
            ),
            SolRpcEndpoint::GetSignatureStatuses => assert_params_eq(
                client
                    .get_signature_statuses(&[signature()])
                    .unwrap()
                    .with_search_transaction_history(true),
                client
                    .get_signature_statuses(&[signature()])
                    .unwrap()
                    .with_params(GetSignatureStatusesParams {
                        signatures: vec![signature()].try_into().unwrap(),
                        search_transaction_history: Some(true),
                    }),
            ),
            SolRpcEndpoint::GetSlot => assert_params_eq(
                client
                    .get_slot()
                    .with_min_context_slot(MIN_CONTEXT_SLOT)
                    .with_commitment(CommitmentLevel::Confirmed),
                client.get_slot().with_params(Some(GetSlotParams {
                    commitment: Some(CommitmentLevel::Confirmed),
                    min_context_slot: Some(MIN_CONTEXT_SLOT),
                })),
            ),
            SolRpcEndpoint::GetTokenAccountBalance => assert_params_eq(
                client
                    .get_token_account_balance(PUBKEY)
                    .with_commitment(CommitmentLevel::Confirmed),
                client.get_token_account_balance(GetTokenAccountBalanceParams {
                    pubkey: PUBKEY.into(),
                    commitment: Some(CommitmentLevel::Confirmed),
                }),
            ),
            SolRpcEndpoint::GetTransaction => assert_params_eq(
                client
                    .get_transaction(signature())
                    .with_commitment(CommitmentLevel::Confirmed)
                    .with_max_supported_transaction_version(0)
                    .with_encoding(GetTransactionEncoding::Base64),
                client.get_transaction(GetTransactionParams {
                    signature: signature().into(),
                    commitment: Some(CommitmentLevel::Confirmed),
                    max_supported_transaction_version: Some(0),
                    encoding: Some(GetTransactionEncoding::Base64),
                }),
            ),
            SolRpcEndpoint::JsonRequest => {
                // No optional request parameters
            }
            SolRpcEndpoint::SendTransaction => assert_params_eq(
                client
                    .send_transaction(transaction())
                    .with_skip_preflight(true)
                    .with_preflight_commitment(CommitmentLevel::Confirmed)
                    .with_max_retries(10)
                    .with_min_context_slot(MIN_CONTEXT_SLOT),
                client
                    .send_transaction(transaction())
                    .modify_params(|params| {
                        params.skip_preflight = Some(true);
                        params.preflight_commitment = Some(CommitmentLevel::Confirmed);
                        params.max_retries = Some(10);
                        params.min_context_slot = Some(MIN_CONTEXT_SLOT);
                    }),
            ),
        }
    }
}

mod get_recent_block {
    use super::*;
    use ic_canister_runtime::IcError;

    #[tokio::test]
    async fn should_return_block_on_success() {
        let client = SolRpcClient::builder_for_ic()
            .with_stub_responses()
            .add_stub_response(MultiRpcResult::Consistent(Ok(SLOT)))
            .add_stub_response(MultiRpcResult::Consistent(Ok(Some(block()))))
            .build();

        let result = client
            .get_recent_block()
            .with_num_tries(NonZeroUsize::MIN)
            .try_send()
            .await;

        assert_eq!(
            result,
            Ok((
                SLOT,
                solana_transaction_status_client_types::UiConfirmedBlock::from(block())
            ))
        );
    }

    #[tokio::test]
    async fn should_return_missing_block_error() {
        let client = SolRpcClient::builder_for_ic()
            .with_stub_responses()
            .add_stub_response(MultiRpcResult::Consistent(Ok(SLOT)))
            .add_stub_response(MultiRpcResult::Consistent(Ok(None::<ConfirmedBlock>)))
            .build();

        let result = client
            .get_recent_block()
            .with_num_tries(NonZeroUsize::MIN)
            .try_send()
            .await;

        assert_eq!(result, Err(vec![GetRecentBlockError::MissingBlock(SLOT)]));
    }

    #[tokio::test]
    async fn should_return_get_slot_rpc_error() {
        let error = RpcError::ValidationError("getSlot error".to_string());
        let client = SolRpcClient::builder_for_ic()
            .with_stub_responses()
            .add_stub_response(MultiRpcResult::Consistent(Err::<Slot, _>(error.clone())))
            .build();

        let result = client
            .get_recent_block()
            .with_num_tries(NonZeroUsize::MIN)
            .try_send()
            .await;

        assert_eq!(
            result,
            Err(vec![GetRecentBlockError::GetSlotRpcError(error)])
        );
    }

    #[tokio::test]
    async fn should_return_get_block_rpc_error() {
        let error = RpcError::ValidationError("getBlock error".to_string());
        let client = SolRpcClient::builder_for_ic()
            .with_stub_responses()
            .add_stub_response(MultiRpcResult::Consistent(Ok(SLOT)))
            .add_stub_response(MultiRpcResult::Consistent(
                Err::<Option<ConfirmedBlock>, _>(error.clone()),
            ))
            .build();

        let result = client
            .get_recent_block()
            .with_num_tries(NonZeroUsize::MIN)
            .try_send()
            .await;

        assert_eq!(
            result,
            Err(vec![GetRecentBlockError::GetBlockRpcError(error)])
        );
    }

    #[tokio::test]
    async fn should_return_get_slot_consensus_error() {
        let inconsistent_results = vec![
            (
                RpcSource::Supported(SupportedRpcProviderId::AlchemyMainnet),
                Ok(SLOT),
            ),
            (
                RpcSource::Supported(SupportedRpcProviderId::AnkrMainnet),
                Ok(SLOT + 1),
            ),
        ];
        let client = SolRpcClient::builder_for_ic()
            .with_stub_responses()
            .add_stub_response(MultiRpcResult::Inconsistent(inconsistent_results.clone()))
            .build();

        let result = client
            .get_recent_block()
            .with_num_tries(NonZeroUsize::MIN)
            .try_send()
            .await;

        assert_eq!(
            result,
            Err(vec![GetRecentBlockError::GetSlotConsensusError(
                inconsistent_results
            )])
        );
    }

    #[tokio::test]
    async fn should_return_get_block_consensus_error() {
        let block = block();
        let inconsistent_results = vec![
            (
                RpcSource::Supported(SupportedRpcProviderId::AlchemyMainnet),
                Ok(Some(block.clone())),
            ),
            (
                RpcSource::Supported(SupportedRpcProviderId::AnkrMainnet),
                Ok(None),
            ),
        ];
        let client = SolRpcClient::builder_for_ic()
            .with_stub_responses()
            .add_stub_response(MultiRpcResult::Consistent(Ok(SLOT)))
            .add_stub_response(MultiRpcResult::Inconsistent(inconsistent_results.clone()))
            .build();

        let result = client
            .get_recent_block()
            .with_num_tries(NonZeroUsize::MIN)
            .try_send()
            .await;

        // Convert ConfirmedBlock to UiConfirmedBlock for comparison
        let expected_results: Vec<_> = inconsistent_results
            .into_iter()
            .map(|(source, r)| (source, r.map(|opt| opt.map(Into::into))))
            .collect();
        assert_eq!(
            result,
            Err(vec![GetRecentBlockError::GetBlockConsensusError(
                expected_results
            )])
        );
    }

    #[tokio::test]
    async fn should_return_get_slot_ic_error() {
        let error = IcError::CallPerformFailed;
        let client = SolRpcClient::builder_for_ic()
            .with_stub_responses()
            .add_stub_error(error.clone())
            .build();

        let result = client
            .get_recent_block()
            .with_num_tries(NonZeroUsize::MIN)
            .try_send()
            .await;

        assert_eq!(result, Err(vec![GetRecentBlockError::IcError(error)]));
    }

    #[tokio::test]
    async fn should_return_get_block_ic_error() {
        let error = IcError::CallPerformFailed;
        let client = SolRpcClient::builder_for_ic()
            .with_stub_responses()
            .add_stub_response(MultiRpcResult::Consistent(Ok(SLOT)))
            .add_stub_error(error.clone())
            .build();

        let result = client
            .get_recent_block()
            .with_num_tries(NonZeroUsize::MIN)
            .try_send()
            .await;

        assert_eq!(result, Err(vec![GetRecentBlockError::IcError(error)]));
    }

    #[tokio::test]
    async fn should_retry_on_error() {
        let error = RpcError::ValidationError("first attempt fails".to_string());
        let client = SolRpcClient::builder_for_ic()
            .with_stub_responses()
            // First attempt: getSlot fails
            .add_stub_response(MultiRpcResult::Consistent(Err::<Slot, _>(error.clone())))
            // Second attempt: getSlot succeeds, getBlock succeeds
            .add_stub_response(MultiRpcResult::Consistent(Ok(SLOT)))
            .add_stub_response(MultiRpcResult::Consistent(Ok(Some(block()))))
            .build();

        let result = client
            .get_recent_block()
            .with_num_tries(NonZeroUsize::new(2).unwrap())
            .try_send()
            .await;

        assert_eq!(
            result,
            Ok((
                SLOT,
                solana_transaction_status_client_types::UiConfirmedBlock::from(block())
            ))
        );
    }

    #[tokio::test]
    async fn should_collect_all_errors_on_exhausted_retries() {
        let error1 = RpcError::ValidationError("first error".to_string());
        let error2 = RpcError::ValidationError("second error".to_string());
        let client = SolRpcClient::builder_for_ic()
            .with_stub_responses()
            // First attempt fails
            .add_stub_response(MultiRpcResult::Consistent(Err::<Slot, _>(error1.clone())))
            // Second attempt fails
            .add_stub_response(MultiRpcResult::Consistent(Err::<Slot, _>(error2.clone())))
            .build();

        let result = client
            .get_recent_block()
            .with_num_tries(NonZeroUsize::new(2).unwrap())
            .try_send()
            .await;

        assert_eq!(
            result,
            Err(vec![
                GetRecentBlockError::GetSlotRpcError(error1),
                GetRecentBlockError::GetSlotRpcError(error2),
            ])
        );
    }
}

mod num_providers_tests {
    use super::*;

    #[test]
    fn should_default_to_3_for_default_sources() {
        let client = SolRpcClient::builder_for_ic().build();
        let builder = client.get_balance(PUBKEY);
        assert_eq!(builder.num_providers(), 3);
    }

    #[test]
    fn should_count_single_custom_provider() {
        let client = SolRpcClient::builder_for_ic()
            .with_rpc_sources(RpcSources::Custom(vec![RpcSource::Supported(
                SupportedRpcProviderId::AlchemyMainnet,
            )]))
            .build();
        let builder = client.get_balance(PUBKEY);
        assert_eq!(builder.num_providers(), 1);
    }

    #[test]
    fn should_use_custom_provider_count() {
        let providers = vec![
            RpcSource::Supported(SupportedRpcProviderId::AlchemyMainnet),
            RpcSource::Supported(SupportedRpcProviderId::HeliusMainnet),
            RpcSource::Supported(SupportedRpcProviderId::AnkrMainnet),
            RpcSource::Supported(SupportedRpcProviderId::DrpcMainnet),
            RpcSource::Supported(SupportedRpcProviderId::PublicNodeMainnet),
        ];
        let client = SolRpcClient::builder_for_ic()
            .with_rpc_sources(RpcSources::Custom(providers))
            .build();
        let builder = client.get_balance(PUBKEY);
        assert_eq!(builder.num_providers(), 5);
    }

    #[test]
    fn should_use_threshold_total() {
        let client = SolRpcClient::builder_for_ic()
            .with_rpc_config(RpcConfig {
                response_size_estimate: None,
                response_consensus: Some(ConsensusStrategy::Threshold {
                    total: Some(5),
                    min: 3,
                }),
            })
            .build();
        let builder = client.get_balance(PUBKEY);
        assert_eq!(builder.num_providers(), 5);
    }
}

fn assert_params_eq<Runtime, Config, Params, CandidOutput, Output>(
    left: RequestBuilder<Runtime, Config, Params, CandidOutput, Output>,
    right: RequestBuilder<Runtime, Config, Params, CandidOutput, Output>,
) where
    Params: Debug + PartialEq,
{
    assert_eq!(left.request.params, right.request.params);
}

fn signature() -> Signature {
    Signature::from_str(
        "tspfR5p1PFphquz4WzDb7qM4UhJdgQXkEZtW88BykVEdX2zL2kBT9kidwQBviKwQuA3b6GMCR1gknHvzQ3r623T",
    )
    .unwrap()
}

fn another_signature() -> Signature {
    Signature::from_str(
        "3WM42nYDQAHgBWFd6SbJ3pj1AGgiTJfxXJ2d5dHu49GgqSUui5qdh64S5yLCN1cMKcLMFVKKo776GrtVhfatLqP6",
    )
    .unwrap()
}

fn transaction() -> solana_transaction::Transaction {
    let keypair = solana_keypair::Keypair::from_base58_string(
        "3jipnj2WowKxqMaSoTj8v79kcSb5bbvJHomd5FwycLg1juPnWdhJBzszABAAxVEfRmsxdo2bnbi7hpag3CrLNU1c",
    );
    solana_transaction::Transaction::new_signed_with_payer(
        &[],
        Some(&pubkey!("3HwVowmCYKPWjRvkqfEfYFWetZLPmZW6LCnLEQDHqpJJ")),
        &[keypair],
        solana_hash::Hash::from_str("4Pcj2yJkCYyhnWe8Ze3uK2D2EtesBxhAevweDoTcxXf3").unwrap(),
    )
}

fn block() -> ConfirmedBlock {
    ConfirmedBlock {
        previous_blockhash: Hash::from_str("4yeCoXK2Q4yXcunuLtF37yTE1wVD4x8313adneZDmi8w").unwrap(),
        blockhash: Hash::from_str(BLOCKHASH).unwrap(),
        parent_slot: SLOT - 1,
        block_time: Some(1748606929),
        block_height: Some(321673899),
        signatures: None,
        rewards: None,
        num_reward_partitions: None,
        transactions: None,
    }
}
