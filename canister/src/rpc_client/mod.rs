use crate::{constants::API_KEY_REPLACE_STRING, state::read_state};
use ic_cdk::api::management_canister::http_request::HttpHeader;
use sol_rpc_types::{Provider, RpcAccess, RpcApi, RpcAuth};

pub fn get_api(provider: &Provider) -> RpcApi {
    match &provider.access {
        RpcAccess::Authenticated { auth, public_url } => {
            let api_key = read_state(|s| s.get_api_key(&provider.provider_id));
            match api_key {
                Some(api_key) => match auth {
                    RpcAuth::BearerToken { url } => RpcApi {
                        url: url.to_string(),
                        headers: Some(vec![HttpHeader {
                            name: "Authorization".to_string(),
                            value: format!("Bearer {}", api_key.read()),
                        }]),
                    },
                    RpcAuth::UrlParameter { url_pattern } => RpcApi {
                        url: url_pattern.replace(API_KEY_REPLACE_STRING, api_key.read()),
                        headers: None,
                    },
                },
                None => RpcApi {
                    url: public_url.clone().unwrap_or_else(|| {
                        panic!(
                            "API key not yet initialized for provider: {}",
                            provider.provider_id
                        )
                    }),
                    headers: None,
                },
            }
        }
        RpcAccess::Unauthenticated { public_url } => RpcApi {
            url: public_url.to_string(),
            headers: None,
        },
    }
}
