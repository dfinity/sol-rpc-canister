use crate::{HttpHeader, RoundingError, RpcEndpoint};

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
            (10, 19, 10),
            (10, 10, 10),
        ] {
            assert_eq!(RoundingError::new(rounding_error).round(slot), rounded);
        }
    }
}
