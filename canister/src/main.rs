use candid::candid_method;
use ic_cdk::query;
use sol_rpc_canister::{
    providers::PROVIDERS,
    types::{Provider, RpcAccess, RpcAuth},
};

#[query(name = "getProviders")]
#[candid_method(query, rename = "getProviders")]
fn get_providers() -> Vec<sol_rpc_types::Provider> {
    fn into_provider(provider: Provider) -> sol_rpc_types::Provider {
        sol_rpc_types::Provider {
            provider_id: provider.provider_id,
            chain_id: provider.chain_id,
            access: match provider.access {
                RpcAccess::Authenticated { auth, public_url } => {
                    sol_rpc_types::RpcAccess::Authenticated {
                        auth: match auth {
                            RpcAuth::BearerToken { url } => sol_rpc_types::RpcAuth::BearerToken {
                                url: url.to_string(),
                            },
                            RpcAuth::UrlParameter { url_pattern } => {
                                sol_rpc_types::RpcAuth::UrlParameter {
                                    url_pattern: url_pattern.to_string(),
                                }
                            }
                        },
                        public_url: public_url.map(|s| s.to_string()),
                    }
                }
                RpcAccess::Unauthenticated { public_url } => {
                    sol_rpc_types::RpcAccess::Unauthenticated {
                        public_url: public_url.to_string(),
                    }
                }
            },
            alias: provider.alias,
        }
    }
    PROVIDERS.iter().cloned().map(into_provider).collect()
}

fn main() {}

#[test]
fn check_candid_interface_compatibility() {
    use candid_parser::utils::{service_equal, CandidSource};

    candid::export_service!();

    let new_interface = __export_service();

    // check the public interface against the actual one
    let old_interface = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("sol_rpc_canister.did");

    service_equal(
        CandidSource::Text(dbg!(&new_interface)),
        CandidSource::File(old_interface.as_path()),
    )
    .unwrap();
}
