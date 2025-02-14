use candid::candid_method;
use ic_cdk::query;
use sol_rpc_canister::providers::PROVIDERS;

#[query(name = "getProviders")]
#[candid_method(query, rename = "getProviders")]
fn get_providers() -> Vec<sol_rpc_types::Provider> {
    PROVIDERS.with(|providers| providers.clone())
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
