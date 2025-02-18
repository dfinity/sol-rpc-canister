use candid::candid_method;
use ic_cdk::api::is_controller;
use ic_cdk::{query, update};
use sol_rpc_canister::{
    lifecycle,
    providers::{find_provider, PROVIDERS},
    state::{mutate_state, read_state},
};
use sol_rpc_types::{ProviderId, RpcAccess};

pub fn require_api_key_principal_or_controller() -> Result<(), String> {
    let caller = ic_cdk::caller();
    if read_state(|state| state.is_api_key_principal(&caller)) || is_controller(&caller) {
        Ok(())
    } else {
        Err("You are not authorized".to_string())
    }
}

#[query(name = "getProviders")]
#[candid_method(query, rename = "getProviders")]
fn get_providers() -> Vec<sol_rpc_types::Provider> {
    PROVIDERS.with(|providers| providers.clone().into_iter().collect())
}

#[update(
    name = "updateApiKeys",
    guard = "require_api_key_principal_or_controller"
)]
#[candid_method(rename = "updateApiKeys")]
/// Inserts or removes RPC provider API keys.
///
/// For each element of `api_keys`, passing `(id, Some(key))` corresponds to inserting or updating
/// an API key, while passing `(id, None)` indicates that the key should be removed from the canister.
///
/// Panics if the list of provider IDs includes a nonexistent or "unauthenticated" (fully public) provider.
async fn update_api_keys(api_keys: Vec<(ProviderId, Option<String>)>) {
    // TODO XC-286: Add logs
    // log!(
    //     INFO,
    //     "[{}] Updating API keys for providers: {}",
    //     ic_cdk::caller(),
    //     api_keys
    //         .iter()
    //         .map(|(id, _)| id.to_string())
    //         .collect::<Vec<_>>()
    //         .join(", ")
    // );
    for (provider_id, api_key) in api_keys {
        let provider = find_provider(|provider| provider.provider_id == provider_id)
            .unwrap_or_else(|| panic!("Provider not found: {}", provider_id));
        if let RpcAccess::Unauthenticated { .. } = provider.access {
            panic!(
                "Trying to set API key for unauthenticated provider: {}",
                provider_id
            )
        }
        match api_key {
            Some(key) => mutate_state(|state| {
                state.insert_api_key(provider_id, key.try_into().expect("Invalid API key"))
            }),
            None => mutate_state(|state| state.remove_api_key(provider_id)),
        }
    }
}

#[query(
    guard = "require_api_key_principal_or_controller",
    name = "verifyApiKey",
    hidden = true
)]
async fn verify_api_key(api_key: (ProviderId, Option<String>)) {
    let (provider_id, api_key) = api_key;
    let api_key = api_key.map(|key| TryFrom::try_from(key).expect("Invalid API key"));
    if read_state(|state| state.get_api_key(&provider_id)) != api_key {
        panic!("API key does not match input")
    }
}

#[ic_cdk::init]
fn init(args: sol_rpc_types::InstallArgs) {
    lifecycle::init(args);
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: Option<sol_rpc_types::InstallArgs>) {
    lifecycle::post_upgrade(args);
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
