use crate::{
    GetRecentPrioritizationFeesParams, GetSignatureStatusesParams, GetSignaturesForAddressParams,
};
use proptest::proptest;
use serde::Deserialize;
use serde_json::json;

mod get_signature_statuses_params_tests {
    use super::*;

    proptest! {
        #[test]
        fn should_deserialize(size in 0..256) {
            let params = json!({
                "signatures": vec!["5iBbqBJzgqafuQn93Np8ztWyXeYe2ReGPzUB1zXP2suZ8b5EaxSwe74ZUhg5pZQuDQkNGW7XApgfXX91YLYUuo5y"; size as usize]
            });

            let result = GetSignatureStatusesParams::deserialize(&params);

            assert!(result.is_ok());
        }
    }

    #[test]
    fn should_not_deserialize() {
        let params = json!({
            "signatures": vec!["5iBbqBJzgqafuQn93Np8ztWyXeYe2ReGPzUB1zXP2suZ8b5EaxSwe74ZUhg5pZQuDQkNGW7XApgfXX91YLYUuo5y"; 256 + 1]
        });

        let result = GetSignatureStatusesParams::deserialize(&params);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string().as_str(),
            "Validation error: Expected at most 256 items, but got 257"
        );
    }
}

mod get_recent_prioritization_fees_params_tests {
    use super::*;

    proptest! {
        #[test]
        fn should_deserialize(size in 0..128) {
            let params = json!(vec!["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; size as usize]);

            let result = GetRecentPrioritizationFeesParams::deserialize(&params);

            assert!(result.is_ok());
        }
    }

    #[test]
    fn should_not_deserialize() {
        let params = json!(vec![
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
            128 + 1
        ]);

        let result = GetRecentPrioritizationFeesParams::deserialize(&params);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string().as_str(),
            "Validation error: Expected at most 128 items, but got 129"
        );
    }
}

mod get_signatures_for_address_params_tests {
    use super::*;

    proptest! {
        #[test]
        fn should_deserialize(limit in 1..1000) {
            let params = json!({
                "pubkey": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "limit": limit
            });

            let result = GetSignaturesForAddressParams::deserialize(&params);

            assert!(result.is_ok());
        }
    }

    #[test]
    fn should_not_deserialize() {
        for limit in [0, 1001, 1234] {
            let params = json!({
                "pubkey": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "limit": limit
            });

            let result = GetSignaturesForAddressParams::deserialize(&params);

            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                format!("Validation error: Expected a value between 1 and 1000, but got {limit}")
            );
        }
    }
}
