use crate::{HttpHeader, RpcEndpoint};

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
