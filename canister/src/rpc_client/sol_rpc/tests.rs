use crate::{rpc_client::sol_rpc::ResponseTransform, types::RoundingError};
use canhttp::http::json::{Id, JsonRpcResponse};
use proptest::proptest;
use serde_json::{from_slice, json, to_vec, Value};

mod normalization_tests {
    use super::*;

    #[test]
    fn should_normalize_raw_response() {
        assert_normalized_equal(
            &ResponseTransform::Raw,
            r#"{"k1":"v1","k2":"v2"}"#,
            r#"{"k1":"v1","k2":"v2"}"#,
        );
        assert_normalized_equal(
            &ResponseTransform::Raw,
            r#"{"k1":"v1","k2":"v2"}"#,
            r#"{"k2":"v2","k1":"v1"}"#,
        );
        assert_normalized_not_equal(
            &ResponseTransform::Raw,
            r#"{"k1":"v1","k2":"v2"}"#,
            r#"{"k1":"v1","k3":"v3"}"#,
        );
    }

    #[test]
    fn should_normalize_get_slot_response() {
        assert_normalized_equal(
            &ResponseTransform::GetSlot(RoundingError::default()),
            "329535108",
            "329535108",
        );
        assert_normalized_equal(
            &ResponseTransform::GetSlot(RoundingError::default()),
            "329535108",
            "329535116",
        );
        assert_normalized_not_equal(
            &ResponseTransform::GetSlot(RoundingError::default()),
            "329535108",
            "329535128",
        );
    }

    #[test]
    fn should_normalize_get_account_info_response() {
        assert_normalized_equal(
            &ResponseTransform::GetAccountInfo,
            r#"{
                "context": { "apiVersion": "2.0.15", "slot": 341197053 },
                "value": {
                    "data": ["1234", "base58"],
                    "executable": false,
                    "lamports": 88849814690250,
                    "owner": "11111111111111111111111111111111",
                    "rentEpoch": 18446744073709551615,
                    "space": 0
                }
            }"#,
            r#"{
                "context": { "apiVersion": "2.0.15", "slot": 341197053 },
                "value": {
                    "space": 0,
                    "rentEpoch": 18446744073709551615,
                    "executable": false,
                    "lamports": 88849814690250,
                    "data": ["1234", "base58"],
                    "owner": "11111111111111111111111111111111"
                }
            }"#,
        );
    }

    proptest! {
        #[test]
        fn should_ignore_get_account_info_response_context(slot1: u64, slot2: u64) {
            assert_normalized_equal(
                &ResponseTransform::GetAccountInfo,
                json!({
                    "context": { "apiVersion": "2.0.15", "slot": slot1 },
                    "value": {
                        "data": ["1234", "base58"],
                        "executable": false,
                        "lamports": 88849814690250u64,
                        "owner": "11111111111111111111111111111111",
                        "rentEpoch": 18446744073709551615u64,
                        "space": 0
                    }
                }).to_string(),
                json!({
                    "context": { "apiVersion": "2.0.15", "slot": slot2 },
                    "value": {
                        "data": ["1234", "base58"],
                        "executable": false,
                        "lamports": 88849814690250u64,
                        "owner": "11111111111111111111111111111111",
                        "rentEpoch": 18446744073709551615u64,
                        "space": 0
                    }
                }).to_string(),
            );
        }
    }

    #[test]
    fn should_normalize_empty_get_account_info_response() {
        assert_normalized(
            &ResponseTransform::GetAccountInfo,
            r#"{"context": { "apiVersion": "2.0.15", "slot": 341197053 }}"#,
            Value::Null,
        );
    }

    proptest! {
        #[test]
        fn should_normalize_send_transaction_response(transaction_id in "[1-9A-HJ-NP-Za-km-z]+") {
            assert_normalized(
                &ResponseTransform::SendTransaction,
                &format!("\"{transaction_id}\""),
                Value::String(transaction_id),
            );
        }
    }

    #[test]
    fn should_normalize_get_block_response() {
        assert_normalized_equal(
            &ResponseTransform::GetBlock,
            r#"{
                "previousBlockhash": "4Pcj2yJkCYyhnWe8Ze3uK2D2EtesBxhAevweDoTcxXf3",
                "blockhash": "8QeCusqSTKeC23NwjTKRBDcPuEfVLtszkxbpL6mXQEp4",
                "parentSlot": 372877611,
                "blockTime": 1744122369,
                "blockHeight": 360854634
            }"#,
            r#"{
                "blockHeight": 360854634,
                "blockTime": 1744122369,
                "blockhash": "8QeCusqSTKeC23NwjTKRBDcPuEfVLtszkxbpL6mXQEp4",
                "previousBlockhash": "4Pcj2yJkCYyhnWe8Ze3uK2D2EtesBxhAevweDoTcxXf3",
                "parentSlot": 372877611
            }"#,
        );
    }

    #[test]
    fn should_normalize_empty_get_block_response() {
        assert_normalized(&ResponseTransform::GetBlock, "null", Value::Null);
    }

    #[test]
    fn should_normalize_get_transaction_response() {
        assert_normalized_equal(
            &ResponseTransform::GetTransaction,
            r#"{
                  "slot": 120133,
                  "transaction": [
                    "Aeuy7wv/RoaKMYAjzzd16aEQi9elf/Kcpf1gNKTn2cnaQxIJ8KCzmPPljqp6VfeMKahWxPnF+ho82t46h7vQgQ0BAAEDWrC6Wz0HQvlvLX3yuJPFIs2A97rFB0Duo19vnKOAHdcPsWHHq0i1GkB9cmG/amgN4E4jafef5+WodPVJDQS/iAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAApMRQc5RO87aiC9YUMJlSr+njrNgBy9m5jJVApNSV5W8BAgIAAQwCAAAAAOQLVAIAAAA=",
                    "base64"
                  ],
                  "meta": {
                    "err": null,
                    "status": {
                      "Ok": null
                    },
                    "fee": 5000,
                    "preBalances": [
                      999409999660000,
                      0,
                      1
                    ],
                    "postBalances": [
                      999399999655000,
                      10000000000,
                      1
                    ],
                    "innerInstructions": [],
                    "logMessages": [
                      "Program 11111111111111111111111111111111 invoke [1]",
                      "Program 11111111111111111111111111111111 success"
                    ],
                    "preTokenBalances": [],
                    "postTokenBalances": [],
                    "rewards": [],
                    "loadedAddresses": {
                      "writable": [],
                      "readonly": []
                    },
                    "computeUnitsConsumed": 150
                  },
                  "blockTime": 1744486970
                }"#,
            r#"{
                  "transaction": [
                    "Aeuy7wv/RoaKMYAjzzd16aEQi9elf/Kcpf1gNKTn2cnaQxIJ8KCzmPPljqp6VfeMKahWxPnF+ho82t46h7vQgQ0BAAEDWrC6Wz0HQvlvLX3yuJPFIs2A97rFB0Duo19vnKOAHdcPsWHHq0i1GkB9cmG/amgN4E4jafef5+WodPVJDQS/iAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAApMRQc5RO87aiC9YUMJlSr+njrNgBy9m5jJVApNSV5W8BAgIAAQwCAAAAAOQLVAIAAAA=",
                    "base64"
                  ],
                  "slot": 120133,
                  "meta": {
                    "fee": 5000,
                    "err": null,
                    "status": {
                      "Ok": null
                    },
                    "loadedAddresses": {
                      "writable": [],
                      "readonly": []
                    },
                    "preBalances": [
                      999409999660000,
                      0,
                      1
                    ],
                    "postBalances": [
                      999399999655000,
                      10000000000,
                      1
                    ],
                    "logMessages": [
                      "Program 11111111111111111111111111111111 invoke [1]",
                      "Program 11111111111111111111111111111111 success"
                    ],
                    "innerInstructions": [],
                    "preTokenBalances": [],
                    "postTokenBalances": [],
                    "rewards": [],
                    "computeUnitsConsumed": 150
                  },
                  "blockTime": 1744486970
                }"#,
        );
    }

    #[test]
    fn should_normalize_empty_get_transaction_response() {
        assert_normalized(&ResponseTransform::GetTransaction, "null", Value::Null);
    }

    #[test]
    fn should_normalize_get_balance_response() {
        assert_normalized(
            &ResponseTransform::GetAccountInfo,
            r#"{ "context": { "slot": 334035824, "apiVersion": "2.1.9" }, "value": 0 }"#,
            json!(0),
        );

        assert_normalized(
            &ResponseTransform::GetAccountInfo,
            r#"{ "context": { "slot": 334035824, "apiVersion": "2.1.9" }, "value": 1000000 }"#,
            json!(1000000),
        );

        assert_normalized_equal(
            &ResponseTransform::GetBalance,
            r#"{
                    "context": {
                        "slot": 334036571,
                        "apiVersion": "2.1.9"
                    },
                    "value": 1000000
                }"#,
            r#"{
                    "context": {
                        "slot": 334036572,
                        "apiVersion": "2.1.9"
                    },
                    "value": 1000000
                }"#,
        );
    }

    fn assert_normalized(transform: &ResponseTransform, result: &str, expected: Value) {
        let expected_response = to_vec(&JsonRpcResponse::from_ok(Id::Number(1), expected)).unwrap();
        let normalized_response = normalize_result(transform, result);
        assert_eq!(
            expected_response,
            normalized_response,
            "expected {:?}, actual: {:?}",
            from_slice::<Value>(&expected_response),
            from_slice::<Value>(&normalized_response),
        );
    }

    fn normalize_result(transform: &ResponseTransform, result: &str) -> Vec<u8> {
        fn add_envelope(reply: &str) -> Vec<u8> {
            format!("{{\"jsonrpc\": \"2.0\", \"id\": 1, \"result\": {}}}", reply).into_bytes()
        }
        let mut response = add_envelope(result);
        transform.apply(&mut response);
        response
    }

    fn assert_normalized_equal(
        transform: &ResponseTransform,
        left: impl AsRef<str>,
        right: impl AsRef<str>,
    ) {
        assert_eq!(
            normalize_result(transform, left.as_ref()),
            normalize_result(transform, right.as_ref())
        );
    }

    fn assert_normalized_not_equal(transform: &ResponseTransform, left: &str, right: &str) {
        assert_ne!(
            normalize_result(transform, left),
            normalize_result(transform, right)
        );
    }
}
