use crate::{GetRecentPrioritizationFeesParams, GetSignatureStatusesParams};
use serde::Deserialize;
use serde_json::json;

mod get_signature_statuses_params_tests {
    use super::*;

    #[test]
    fn should_deserialize() {
        let params = json!({
            "signature": vec!["5iBbqBJzgqafuQn93Np8ztWyXeYe2ReGPzUB1zXP2suZ8b5EaxSwe74ZUhg5pZQuDQkNGW7XApgfXX91YLYUuo5y"; 256]
        });

        let result = GetSignatureStatusesParams::deserialize(&params);

        assert!(result.is_ok());
    }

    #[test]
    fn should_not_deserialize() {
        let params = json!({
            "signature": vec!["5iBbqBJzgqafuQn93Np8ztWyXeYe2ReGPzUB1zXP2suZ8b5EaxSwe74ZUhg5pZQuDQkNGW7XApgfXX91YLYUuo5y"; 256 + 1]
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

    #[test]
    fn should_deserialize() {
        let params = json!(vec!["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; 128]);

        let result = GetRecentPrioritizationFeesParams::deserialize(&params);

        assert!(result.is_ok());
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
