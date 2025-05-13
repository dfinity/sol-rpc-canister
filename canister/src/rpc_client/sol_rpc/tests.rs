use crate::{rpc_client::sol_rpc::ResponseTransform, types::RoundingError};
use canhttp::http::json::{Id, JsonRpcResponse};
use proptest::proptest;
use serde_json::{from_slice, json, to_vec, Value};

mod normalization_tests {
    use super::*;
    use strum::IntoEnumIterator;

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

    #[test]
    fn should_normalize_json_rpc_error() {
        fn normalize_json(transform: &ResponseTransform, response: &str) -> Vec<u8> {
            let mut bytes = response.bytes().collect();
            transform.apply(&mut bytes);
            bytes
        }

        for transform in ResponseTransform::iter() {
            let left = r#"{ "jsonrpc": "2.0", "error": { "code": -32602, "message": "Invalid param: could not find account" }, "id": 1 }"#;
            let right = r#"{ "error": { "message": "Invalid param: could not find account", "code": -32602 }, "id": 1, "jsonrpc": "2.0" }"#;
            let normalized_left = normalize_json(&transform, left);
            let normalized_right = normalize_json(&transform, right);

            assert_eq!(normalized_left, normalized_right);
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&normalized_left).unwrap(),
                json!(
                    { "jsonrpc": "2.0", "error": { "code": -32602, "message": "Invalid param: could not find account" }, "id": 1 }
                )
            );
        }
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

mod get_recent_prioritization_fees {
    use crate::rpc_client::sol_rpc::ResponseTransform;
    use crate::types::RoundingError;
    use proptest::arbitrary::any;
    use proptest::array::uniform32;
    use proptest::prelude::{prop, Strategy};
    use proptest::{prop_assert_eq, proptest};
    use rand::prelude::SliceRandom;
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use serde::Serialize;
    use serde_json::json;
    use sol_rpc_types::{PrioritizationFee, Slot};
    use std::ops::RangeInclusive;

    #[test]
    fn should_normalize_response_with_less_than_150_entries() {
        fn prioritization_fees(slots: Vec<u64>) -> Vec<serde_json::Value> {
            slots
                .into_iter()
                .map(|slot| {
                    json!({
                        "prioritizationFee": slot,
                        "slot": slot
                    })
                })
                .collect()
        }
        let raw_response = json_response(&prioritization_fees(vec![1, 2, 3, 4, 5]));

        for (transform, expected_fees) in [
            (
                ResponseTransform::GetRecentPrioritizationFees {
                    max_slot_rounding_error: RoundingError::new(2),
                    max_length: 2,
                },
                prioritization_fees(vec![3, 4]),
            ),
            (
                ResponseTransform::GetRecentPrioritizationFees {
                    max_slot_rounding_error: RoundingError::new(2),
                    max_length: 0,
                },
                prioritization_fees(vec![]),
            ),
            (
                ResponseTransform::GetRecentPrioritizationFees {
                    max_slot_rounding_error: RoundingError::new(2),
                    max_length: u8::MAX,
                },
                prioritization_fees(vec![1, 2, 3, 4]),
            ),
            (
                ResponseTransform::GetRecentPrioritizationFees {
                    max_slot_rounding_error: RoundingError::new(10),
                    max_length: 2,
                },
                prioritization_fees(vec![]),
            ),
        ] {
            let mut raw_bytes = serde_json::to_vec(&raw_response).unwrap();
            transform.apply(&mut raw_bytes);
            let transformed_response: serde_json::Value =
                serde_json::from_slice(&raw_bytes).unwrap();

            assert_eq!(transformed_response, json_response(&expected_fees));
        }
    }

    #[test]
    fn should_normalize_response_with_no_fees() {
        let raw_response = json_response::<PrioritizationFee>(&[]);
        let transform = ResponseTransform::GetRecentPrioritizationFees {
            max_slot_rounding_error: RoundingError::new(2),
            max_length: 2,
        };
        let original_bytes = serde_json::to_vec(&raw_response).unwrap();
        let mut transformed_bytes = original_bytes.clone();
        transform.apply(&mut transformed_bytes);
        let transformed_response: serde_json::Value =
            serde_json::from_slice(&transformed_bytes).unwrap();

        assert_eq!(raw_response, transformed_response);
    }

    // The API of [getRecentPrioritizationFees](https://solana.com/de/docs/rpc/http/getrecentprioritizationfees)
    // does not specify whether the array of prioritization fees includes a range of continuous slots.
    // The following was observed:
    // 1) On mainnet: the range seems most of the time continuous (e.g., for slots 337346483..=337346632), also for not used addresses
    // 2) Locally with solana-test-validator, the range is often not continuous, e.g.
    // RpcPrioritizationFee { slot: 5183, prioritization_fee: 150 }, RpcPrioritizationFee { slot: 5321, prioritization_fee: 0 }
    //
    // The non-continuity is probably because 
    // [not all slots have a block](https://docs.chainstack.com/docs/understanding-the-difference-between-blocks-and-slots-on-solana)/
    #[test]
    fn should_normalize_response_with_non_contiguous_slots() {
        let range_1 = [PrioritizationFee {
            slot: 150,
            prioritization_fee: 150,
        }];
        let range_2 = [PrioritizationFee {
            slot: 500,
            prioritization_fee: 500,
        }];
        let fees = [&range_1[..], &range_2[..]].concat();

        let transform = ResponseTransform::GetRecentPrioritizationFees {
            max_slot_rounding_error: RoundingError::new(10),
            max_length: 100,
        };
        let mut raw_bytes = serde_json::to_vec(&json_response(&fees)).unwrap();
        transform.apply(&mut raw_bytes);
        let transformed_response: serde_json::Value = serde_json::from_slice(&raw_bytes).unwrap();

        assert_eq!(transformed_response, json_response(&fees));
    }

    #[test]
    fn should_normalize_response_when_rounded_slot_not_in_range() {
        let fees = [
            PrioritizationFee {
                slot: 100,
                prioritization_fee: 100,
            },
            PrioritizationFee {
                slot: 200,
                prioritization_fee: 200,
            },
            PrioritizationFee {
                slot: 301,
                prioritization_fee: 300,
            },
        ];

        let transform = ResponseTransform::GetRecentPrioritizationFees {
            max_slot_rounding_error: RoundingError::new(10),
            max_length: 100,
        };
        let mut raw_bytes = serde_json::to_vec(&json_response(&fees)).unwrap();
        transform.apply(&mut raw_bytes);
        let transformed_response: serde_json::Value = serde_json::from_slice(&raw_bytes).unwrap();

        assert_eq!(transformed_response, json_response(&fees[0..2]));
    }

    proptest! {
        #[test]
        fn should_be_nop_when_failed_to_deserialize(original_bytes in  prop::collection::vec(any::<u8>(), 0..1000)) {
            let transform = ResponseTransform::GetRecentPrioritizationFees {
                max_slot_rounding_error: RoundingError::new(2),
                max_length: 2,
            };
            let mut transformed_bytes = original_bytes.clone();
            transform.apply(&mut transformed_bytes);

            assert_eq!(original_bytes, transformed_bytes);
        }

        #[test]
        fn should_normalize_get_recent_prioritization_fees_response(fees in arb_prioritization_fees(337346483..=337346632)) {
            let raw_response = json_response(&fees);
            let transform = ResponseTransform::GetRecentPrioritizationFees {
                max_slot_rounding_error: RoundingError::new(20),
                max_length: 100,
            };
            let mut raw_bytes = serde_json::to_vec(&raw_response).unwrap();
            transform.apply(&mut raw_bytes);
            let transformed_response: serde_json::Value = serde_json::from_slice(&raw_bytes).unwrap();

            let mut expected_fees = fees;
            // last slot is 337346632 and has index 150.
            // Last slot rounded by 20 is 337346620, which has index 138.
            expected_fees.drain(138..);
            expected_fees.drain(..38);
            prop_assert_eq!(expected_fees.len(), 100);

            prop_assert_eq!(
                transformed_response,
                json_response(&expected_fees)
            )
        }

        #[test]
        fn should_normalize_unsorted_prioritization_fees(
            seed in uniform32(any::<u8>()),
            fees in arb_prioritization_fees(337346483..=337346632)
        ) {
            let mut rng = ChaCha20Rng::from_seed(seed);
            let shuffled_fees = {
                let mut f = fees.clone();
                f.shuffle(&mut rng);
                f
            };
            let transform = ResponseTransform::GetRecentPrioritizationFees {
                max_slot_rounding_error: RoundingError::new(20),
                max_length: 100,
            };

            let sorted_fees_bytes = {
                let raw_response = json_response(&fees);
                let mut raw_bytes = serde_json::to_vec(&raw_response).unwrap();
                transform.apply(&mut raw_bytes);
                raw_bytes
            };

            let shuffled_fees_bytes = {
                let raw_response = json_response(&shuffled_fees);
                let mut raw_bytes = serde_json::to_vec(&raw_response).unwrap();
                transform.apply(&mut raw_bytes);
                raw_bytes
            };

            assert_eq!(sorted_fees_bytes, shuffled_fees_bytes);
        }
    }

    fn arb_prioritization_fees(
        slots: RangeInclusive<Slot>,
    ) -> impl Strategy<Value = Vec<PrioritizationFee>> {
        let len = if slots.is_empty() {
            0
        } else {
            slots.end() - slots.start() + 1
        };
        prop::collection::vec(any::<u64>(), len as usize).prop_map(move |fees| {
            fees.into_iter()
                .enumerate()
                .map(|(index, prioritization_fee)| {
                    let slot = slots.start() + index as u64;
                    assert!(slots.contains(&slot));
                    PrioritizationFee {
                        slot,
                        prioritization_fee,
                    }
                })
                .collect::<Vec<_>>()
        })
    }

    fn json_response<T: Serialize>(fees: &[T]) -> serde_json::Value {
        json!({
            "jsonrpc": "2.0",
            "result": fees,
            "id": 1
        })
    }
}
