use crate::providers::PROVIDERS;
use crate::{constants::API_KEY_REPLACE_STRING, state::read_state};
use ic_cdk::api::management_canister::http_request::HttpHeader;
use sol_rpc_types::{RpcAccess, RpcAuth, RpcEndpoint, RpcSource, SupportedProvider};

pub fn from_rpc_provider(service: RpcSource) -> RpcEndpoint {
    match service {
        RpcSource::Supported(provider_id) => PROVIDERS
            .with(|providers| providers.get(&provider_id).cloned())
            .map(|provider| from_rpc_access(provider.access, provider_id))
            .expect("Unknown provider"),
        RpcSource::Custom(api) => api,
    }
}

fn from_rpc_access(access: RpcAccess, provider: SupportedProvider) -> RpcEndpoint {
    match &access {
        RpcAccess::Authenticated { auth, public_url } => {
            let api_key = read_state(|s| s.get_api_key(&provider));
            match api_key {
                Some(api_key) => match auth {
                    RpcAuth::BearerToken { url } => RpcEndpoint {
                        url: url.to_string(),
                        headers: Some(vec![HttpHeader {
                            name: "Authorization".to_string(),
                            value: format!("Bearer {}", api_key.read()),
                        }]),
                    },
                    RpcAuth::UrlParameter { url_pattern } => RpcEndpoint {
                        url: url_pattern.replace(API_KEY_REPLACE_STRING, api_key.read()),
                        headers: None,
                    },
                },
                None => RpcEndpoint {
                    url: public_url.clone().unwrap_or_else(|| {
                        panic!("API key not yet initialized for provider: {:?}", provider)
                    }),
                    headers: None,
                },
            }
        }
        RpcAccess::Unauthenticated { public_url } => RpcEndpoint {
            url: public_url.to_string(),
            headers: None,
        },
    }
}
