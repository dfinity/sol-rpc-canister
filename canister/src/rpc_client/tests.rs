use crate::rpc_client::{
    GetAccountInfoRequest, GetBlockRequest, GetSlotRequest, GetTransactionRequest, MultiRpcRequest,
    SendTransactionRequest,
};
use serde::Serialize;
use serde_json::json;
use sol_rpc_types::{
    CommitmentLevel, DataSlice, GetAccountInfoEncoding, GetAccountInfoParams,
    GetBlockCommitmentLevel, GetBlockParams, GetSlotParams, GetSlotRpcConfig,
    GetTransactionEncoding, GetTransactionParams, RpcConfig, RpcSources, SendTransactionEncoding,
    SendTransactionParams, SolanaCluster, TransactionDetails,
};

mod request_serialization_tests {
    use super::*;

    #[test]
    fn should_serialize_get_account_info_request() {
        assert_serialized(
            GetAccountInfoRequest::get_account_info(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetAccountInfoParams::from(solana_pubkey::Pubkey::default()),
            )
            .unwrap(),
            json!(["11111111111111111111111111111111", null]),
        );
        assert_serialized(
            GetAccountInfoRequest::get_account_info(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetAccountInfoParams {
                    pubkey: "11111111111111111111111111111111".to_string(),
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
        assert_serialized(
            GetSlotRequest::get_slot(
                RpcSources::Default(SolanaCluster::Mainnet),
                GetSlotRpcConfig::default(),
                GetSlotParams::default(),
            )
            .unwrap(),
            json!([null]),
        );
        assert_serialized(
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
    fn should_serialize_get_transaction_request() {
        assert_serialized(
            GetTransactionRequest::get_transaction(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetTransactionParams::from(solana_signature::Signature::default()),
            )
            .unwrap(),
            json!([
                "1111111111111111111111111111111111111111111111111111111111111111",
                null
            ]),
        );
        assert_serialized(
            GetTransactionRequest::get_transaction(
                RpcSources::Default(SolanaCluster::Mainnet),
                RpcConfig::default(),
                GetTransactionParams {
                    signature: solana_signature::Signature::default().to_string(),
                    commitment: Some(CommitmentLevel::Confirmed),
                    max_supported_transaction_version: Some(2),
                    encoding: Some(GetTransactionEncoding::Base64),
                },
            )
            .unwrap(),
            json!([
                "1111111111111111111111111111111111111111111111111111111111111111",
                {
                    "commitment": "confirmed",
                    "maxSupportedTransactionVersion": 2,
                    "encoding": "base64",
                }
            ]),
        );
    }

    #[test]
    fn should_serialize_get_block_request() {
        assert_serialized(
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
        assert_serialized(
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
    fn should_serialize_send_transaction_request() {
        let transaction = "4F9ksKhLSgn9e7ugVnAmRpRXL9kjke4TT96FNDxMiUNc5KVDz8p1yuv";
        assert_serialized(
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
        assert_serialized(
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

    fn assert_serialized<Params: Serialize, Output>(
        request: MultiRpcRequest<Params, Output>,
        serialized: serde_json::Value,
    ) {
        assert_eq!(
            serde_json::to_value(request.request.params()).unwrap(),
            serialized
        )
    }
}
