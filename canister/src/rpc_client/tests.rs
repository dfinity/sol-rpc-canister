use crate::rpc_client::{
    GetAccountInfoRequest, GetBlockRequest, GetSignatureStatusesRequest,
    GetSignaturesForAddressRequest, GetSlotRequest, GetTransactionRequest, MultiRpcRequest,
    SendTransactionRequest,
};
use serde::Serialize;
use serde_json::json;
use sol_rpc_types::{
    CommitmentLevel, DataSlice, GetAccountInfoEncoding, GetAccountInfoParams, GetBalanceParams,
    GetBlockCommitmentLevel, GetBlockParams, GetRecentPrioritizationFeesParams,
    GetRecentPrioritizationFeesRpcConfig, GetSignatureStatusesParams,
    GetSignaturesForAddressParams, GetSlotParams, GetSlotRpcConfig, GetTokenAccountBalanceParams,
    GetTransactionEncoding, GetTransactionParams, Pubkey, RpcConfig, RpcSources,
    SendTransactionEncoding, SendTransactionParams, Signature, SolanaCluster, TransactionDetails,
    VecWithMaxLen,
};
use solana_pubkey::pubkey;
use std::str::FromStr;

const SOME_SIGNATURE: &str =
    "5iBbqBJzgqafuQn93Np8ztWyXeYe2ReGPzUB1zXP2suZ8b5EaxSwe74ZUhg5pZQuDQkNGW7XApgfXX91YLYUuo5y";
const ANOTHER_SIGNATURE: &str =
    "FAAHyQpENs991w9BR7jpwzyXk74jhQWzbsSbjs4NJWkYeL6nggNfT5baWy6eBNLSuqfiiYRGfEC5bhwxUVBZamB";

mod request_serialization_tests {
    use super::*;

    #[test]
    fn should_serialize_get_account_info_request() {
        assert_params_eq(
            GetAccountInfoRequest::get_account_info(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetAccountInfoParams::from(solana_pubkey::Pubkey::default()),
            )
            .unwrap(),
            json!(["11111111111111111111111111111111", null]),
        );
        assert_params_eq(
            GetAccountInfoRequest::get_account_info(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetAccountInfoParams {
                    pubkey: pubkey!("11111111111111111111111111111111").into(),
                    commitment: Some(CommitmentLevel::Processed),
                    encoding: Some(GetAccountInfoEncoding::Base58),
                    data_slice: Some(DataSlice {
                        length: 123,
                        offset: 8,
                    }),
                    min_context_slot: Some(456),
                },
            )
            .unwrap(),
            json!([
            "11111111111111111111111111111111",
            {
                "commitment": "processed",
                "encoding": "base58",
                "dataSlice": { "length": 123, "offset": 8 },
                "minContextSlot": 456,
            }]),
        );
    }

    #[test]
    fn should_serialize_get_slot_request() {
        assert_params_eq(
            GetSlotRequest::get_slot(
                RpcSources::Default(SolanaCluster::Mainnet),
                GetSlotRpcConfig::default(),
                GetSlotParams::default(),
            )
            .unwrap(),
            json!([null]),
        );
        assert_params_eq(
            GetSlotRequest::get_slot(
                RpcSources::Default(SolanaCluster::Mainnet),
                GetSlotRpcConfig::default(),
                GetSlotParams {
                    commitment: Some(CommitmentLevel::Finalized),
                    min_context_slot: Some(123),
                },
            )
            .unwrap(),
            json!([
                {
                    "commitment": "finalized",
                    "minContextSlot": 123
                },
            ]),
        );
    }

    #[test]
    fn should_serialize_get_signatures_for_address_request() {
        assert_params_eq(
            GetSignaturesForAddressRequest::get_signatures_for_address(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetSignaturesForAddressParams {
                    pubkey: Pubkey::default(),
                    commitment: None,
                    min_context_slot: None,
                    limit: None,
                    before: None,
                    until: None,
                },
            )
            .unwrap(),
            json!(["11111111111111111111111111111111", null]),
        );
        assert_params_eq(
            GetSignaturesForAddressRequest::get_signatures_for_address(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetSignaturesForAddressParams {
                    pubkey: Pubkey::default(),
                    commitment: Some(CommitmentLevel::Processed),
                    min_context_slot: Some(123),
                    limit: Some(10.try_into().unwrap()),
                    before: Some(Signature::from_str(SOME_SIGNATURE).unwrap()),
                    until: Some(Signature::from_str(ANOTHER_SIGNATURE).unwrap()),
                },
            )
            .unwrap(),
            json!([
                "11111111111111111111111111111111",
                {
                    "commitment": "processed",
                    "minContextSlot": 123,
                    "limit": 10,
                    "before": SOME_SIGNATURE,
                    "until": ANOTHER_SIGNATURE,
                }
            ]),
        );
    }

    #[test]
    fn should_serialize_get_signature_statuses_request() {
        assert_params_eq(
            GetSignatureStatusesRequest::get_signature_statuses(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetSignatureStatusesParams {
                    signatures: VecWithMaxLen::new(),
                    search_transaction_history: None,
                },
            )
            .unwrap(),
            json!([[], null]),
        );
        assert_params_eq(
            GetSignatureStatusesRequest::get_signature_statuses(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetSignatureStatusesParams {
                    signatures: vec![
                        Signature::from_str(SOME_SIGNATURE).unwrap(),
                        Signature::from_str(ANOTHER_SIGNATURE).unwrap(),
                    ]
                    .try_into()
                    .unwrap(),
                    search_transaction_history: Some(true),
                },
            )
            .unwrap(),
            json!([
                [SOME_SIGNATURE, ANOTHER_SIGNATURE],
                {
                    "searchTransactionHistory": true,
                }
            ]),
        );
    }

    #[test]
    fn should_serialize_get_transaction_request() {
        let signature = solana_signature::Signature::default().to_string();
        assert_params_eq(
            GetTransactionRequest::get_transaction(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetTransactionParams::from(solana_signature::Signature::default()),
            )
            .unwrap(),
            json!([signature, null]),
        );
        assert_params_eq(
            GetTransactionRequest::get_transaction(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetTransactionParams {
                    signature: Signature::default(),
                    commitment: Some(CommitmentLevel::Confirmed),
                    max_supported_transaction_version: Some(2),
                    encoding: Some(GetTransactionEncoding::Base64),
                },
            )
            .unwrap(),
            json!([
                signature,
                {
                    "commitment": "confirmed",
                    "maxSupportedTransactionVersion": 2,
                    "encoding": "base64",
                }
            ]),
        );
    }

    #[test]
    fn should_serialize_get_balance_request() {
        let pubkey = solana_pubkey::Pubkey::default();
        assert_params_eq(
            MultiRpcRequest::get_balance(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetBalanceParams::from(pubkey),
            )
            .unwrap(),
            json!([pubkey.to_string(), null]),
        );

        assert_params_eq(
            MultiRpcRequest::get_balance(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetBalanceParams {
                    pubkey: pubkey.into(),
                    commitment: Some(CommitmentLevel::Confirmed),
                    min_context_slot: Some(42),
                },
            )
            .unwrap(),
            json!(
                [
                    pubkey.to_string(),
                    {
                        "commitment": "confirmed",
                        "minContextSlot": 42
                    }
                ]
            ),
        );
    }

    #[test]
    fn should_serialize_get_token_account_balance_request() {
        let pubkey = solana_pubkey::Pubkey::default();
        assert_params_eq(
            MultiRpcRequest::get_token_account_balance(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetTokenAccountBalanceParams::from(pubkey),
            )
            .unwrap(),
            json!([pubkey.to_string(), null]),
        );

        assert_params_eq(
            MultiRpcRequest::get_token_account_balance(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetTokenAccountBalanceParams {
                    pubkey: pubkey.into(),
                    commitment: Some(CommitmentLevel::Confirmed),
                },
            )
            .unwrap(),
            json!([
                pubkey.to_string(),
                {"commitment": "confirmed"}
            ]),
        );
    }

    #[test]
    fn should_serialize_get_block_request() {
        assert_params_eq(
            GetBlockRequest::get_block(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetBlockParams::from(123),
            )
            .unwrap(),
            json!([
                123,
                {"rewards": false, "transactionDetails": "none"}
            ]),
        );
        assert_params_eq(
            GetBlockRequest::get_block(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetBlockParams {
                    slot: 123,
                    commitment: Some(GetBlockCommitmentLevel::Finalized),
                    max_supported_transaction_version: Some(2u8),
                    transaction_details: Some(TransactionDetails::Signatures),
                },
            )
            .unwrap(),
            json!([
                123,
                {
                    "rewards": false,
                    "transactionDetails": "signatures",
                    "commitment": "finalized",
                    "maxSupportedTransactionVersion": 2
                },
            ]),
        );
    }

    #[test]
    fn should_serialize_get_recent_prioritization_fees_request() {
        assert_params_eq(
            MultiRpcRequest::get_recent_prioritization_fees(
                RpcSources::Default(SolanaCluster::Mainnet),
                GetRecentPrioritizationFeesRpcConfig::default(),
                GetRecentPrioritizationFeesParams::default(),
            )
            .unwrap(),
            json!([[]]),
        );

        assert_params_eq(
            MultiRpcRequest::get_recent_prioritization_fees(
                RpcSources::Default(SolanaCluster::Mainnet),
                GetRecentPrioritizationFeesRpcConfig::default(),
                GetRecentPrioritizationFeesParams::try_from(vec![
                    pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
                    pubkey!("3emsAVdmGKERbHjmGfQ6oZ1e35dkf5iYcS6U4CPKFVaa"),
                ])
                .unwrap(),
            )
            .unwrap(),
            json!([[
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "3emsAVdmGKERbHjmGfQ6oZ1e35dkf5iYcS6U4CPKFVaa"
            ]]),
        );
    }

    #[test]
    fn should_serialize_send_transaction_request() {
        let transaction = "4F9ksKhLSgn9e7ugVnAmRpRXL9kjke4TT96FNDxMiUNc5KVDz8p1yuv";
        assert_params_eq(
            SendTransactionRequest::send_transaction(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                SendTransactionParams::from_encoded_transaction(
                    transaction.to_string(),
                    SendTransactionEncoding::Base64,
                ),
            )
            .unwrap(),
            json!([transaction, { "encoding": "base64" }]),
        );
        let mut params = SendTransactionParams::from_encoded_transaction(
            transaction.to_string(),
            SendTransactionEncoding::Base58,
        );
        params.max_retries = Some(5);
        params.skip_preflight = Some(true);
        params.preflight_commitment = Some(CommitmentLevel::Processed);
        params.min_context_slot = Some(456);
        assert_params_eq(
            SendTransactionRequest::send_transaction(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                params,
            )
            .unwrap(),
            json!([
                transaction,
                {
                    "encoding": "base58",
                    "maxRetries": 5,
                    "skipPreflight": true,
                    "preflightCommitment": "processed",
                    "minContextSlot": 456,
                }
            ]),
        );
    }

    fn assert_params_eq<Params: Serialize, Output>(
        request: MultiRpcRequest<Params, Output>,
        serialized: serde_json::Value,
    ) {
        assert_eq!(
            serde_json::to_value(request.request.params()).unwrap(),
            serialized
        )
    }
}
