use crate::providers::PROVIDERS;
use crate::{constants::API_KEY_REPLACE_STRING, state::read_state};
use ic_cdk::api::management_canister::http_request::HttpHeader;
use sol_rpc_types::{ProviderId, RpcAccess, RpcApi, RpcAuth, RpcService, SolanaCluster};

pub(crate) fn from_rpc_provider(service: RpcService) -> RpcApi {
    match service {
        RpcService::Registered(provider, cluster) => PROVIDERS
            .with(|map| map.get(&(provider, cluster)).cloned())
            .map(|access| from_rpc_access(access, (provider, cluster)))
            .expect("Unknown provider"),
        RpcService::Custom(api) => api,
    }
}

fn from_rpc_access(access: RpcAccess, (provider, cluster): (ProviderId, SolanaCluster)) -> RpcApi {
    match &access {
        RpcAccess::Authenticated { auth, public_url } => {
            let api_key = read_state(|s| s.get_api_key((provider, cluster)));
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
                            "API key not yet initialized for provider: {:?}",
                            provider
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
