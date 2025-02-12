use crate::rpc_client::RpcApi;
use ic_cdk::api::management_canister::http_request::HttpHeader;

#[test]
fn should_contain_host_without_sensitive_information() {
    for provider in [
        RpcApi {
            url: "https://sol-mainnet.g.alchemy.com/v2".to_string(),
            headers: None,
        },
        RpcApi {
            url: "https://sol-mainnet.g.alchemy.com/v2/key".to_string(),
            headers: None,
        },
        RpcApi {
            url: "https://sol-mainnet.g.alchemy.com/v2".to_string(),
            headers: Some(vec![HttpHeader {
                name: "authorization".to_string(),
                value: "Bearer key".to_string(),
            }]),
        },
    ] {
        let debug = format!("{:?}", provider);
        assert_eq!(
            debug,
            "RpcApi { host: sol-mainnet.g.alchemy.com, url/headers: *** }"
        );
    }
}
