use crate::{HttpHeader, RoundingError, RpcEndpoint};
use proptest::proptest;

#[test]
fn should_contain_host_without_sensitive_information() {
    for provider in [
        RpcEndpoint {
            url: "https://solana-mainnet.g.alchemy.com/v2".to_string(),
            headers: None,
        },
        RpcEndpoint {
            url: "https://solana-mainnet.g.alchemy.com/v2/key".to_string(),
            headers: None,
        },
        RpcEndpoint {
            url: "https://solana-mainnet.g.alchemy.com/v2".to_string(),
            headers: Some(vec![HttpHeader {
                name: "authorization".to_string(),
                value: "Bearer key".to_string(),
            }]),
        },
    ] {
        let debug = format!("{:?}", provider);
        assert_eq!(
            debug,
            "RpcApi { host: solana-mainnet.g.alchemy.com, url/headers: *** }"
        );
    }
}

mod rounding_error_tests {
    use super::*;

    #[test]
    fn should_round_slot() {
        for (rounding_error, slot, rounded) in [
            (0, 0, 0),
            (0, 13, 13),
            (1, 13, 13),
            (10, 13, 10),
            (10, 100, 100),
            (10, 101, 100),
            (10, 102, 100),
            (10, 103, 100),
            (10, 104, 100),
            (10, 105, 100),
            (10, 106, 100),
            (10, 107, 100),
            (10, 108, 100),
            (10, 109, 100),
            (10, 110, 110),
        ] {
            assert_eq!(RoundingError::new(rounding_error).round(slot), rounded);
        }
    }

    proptest! {
        #[test]
        fn should_not_panic (rounding_error: u64, slot: u64) {
            RoundingError::new(rounding_error).round(slot);
        }
    }
}
