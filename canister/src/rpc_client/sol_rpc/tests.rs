use super::*;
use canhttp::http::json::Id;
use proptest::proptest;
use serde_json::json;
use solana_account_decoder_client_types::{UiAccountData, UiAccountEncoding};

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
        assert_normalized(
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
            Some(UiAccount {
                lamports: 88849814690250,
                data: UiAccountData::Binary("1234".to_string(), UiAccountEncoding::Base58),
                owner: "11111111111111111111111111111111".to_string(),
                executable: false,
                rent_epoch: 18446744073709551615,
                space: Some(0),
            }),
        );
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
            None::<UiAccount>,
        );
    }

    proptest! {
        #[test]
        fn should_normalize_send_transaction_response(transaction_id in "[1-9A-HJ-NP-Za-km-z]+") {
            assert_normalized(
                &ResponseTransform::SendTransaction,
                &format!("\"{transaction_id}\""),
                transaction_id.to_string(),
            );
        }
    }

    #[test]
    fn should_normalize_get_block_response() {
        assert_normalized(
            &ResponseTransform::GetBlock,
            r#"{
                "previousBlockhash": "4Pcj2yJkCYyhnWe8Ze3uK2D2EtesBxhAevweDoTcxXf3",
                "blockhash": "8QeCusqSTKeC23NwjTKRBDcPuEfVLtszkxbpL6mXQEp4",
                "parentSlot": 372877611,
                "blockTime": 1744122369,
                "blockHeight": 360854634
            }"#,
            Some(UiConfirmedBlock {
                previous_blockhash: "4Pcj2yJkCYyhnWe8Ze3uK2D2EtesBxhAevweDoTcxXf3".to_string(),
                blockhash: "8QeCusqSTKeC23NwjTKRBDcPuEfVLtszkxbpL6mXQEp4".to_string(),
                parent_slot: 372877611,
                transactions: None,
                signatures: None,
                rewards: None,
                num_reward_partitions: None,
                block_time: Some(1744122369),
                block_height: Some(360854634),
            }),
        );
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
        assert_normalized(
            &ResponseTransform::GetBlock,
            "null",
            None::<UiConfirmedBlock>,
        );
    }

    fn assert_normalized<T>(transform: &ResponseTransform, result: &str, expected: T)
    where
        T: Debug + Serialize + DeserializeOwned,
    {
        let expected_response = to_vec(&JsonRpcResponse::from_ok(Id::Number(1), expected)).unwrap();
        let normalized_response = normalize_result(transform, result);
        assert_eq!(
            expected_response,
            normalized_response,
            "expected {:?}, actual: {:?}",
            from_slice::<Value>(&expected_response),
            from_slice::<Value>(&normalized_response)
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
