use candid::{candid_method, Nat};
use canlog::{log, Log, Sort};
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use ic_cdk::{
    api::is_controller,
    {query, update},
};
use serde_json::json;
use sol_rpc_canister::{
    http_types, lifecycle,
    logs::Priority,
    providers::{find_provider, PROVIDERS},
    state::{mutate_state, read_state},
};
use sol_rpc_types::{ProviderId, RpcAccess};
use std::str::FromStr;

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
    log!(
        Priority::Info,
        "[{}] Updating API keys for providers: {}",
        ic_cdk::caller(),
        api_keys
            .iter()
            .map(|(id, _)| id.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
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

#[update(name = "getSlot")]
#[candid_method(query, rename = "getSlot")]
async fn get_slot() -> u64 {
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": "getSlot" });
    let request = CanisterHttpRequestArgument {
        url: "http://localhost:8899".to_string(),
        max_response_bytes: Some(1_000),
        method: HttpMethod::POST,
        headers: vec![HttpHeader {
            name: "content-type".to_string(),
            value: "application/json".to_string(),
        }],
        body: Some(serde_json::to_vec(&body).unwrap()),
        transform: None,
    };
    let response =
        match ic_cdk::api::management_canister::http_request::http_request(request, 1_000_000_000)
            .await
        {
            Ok((response,)) => response,
            Err((code, message)) => panic!("Error {code:?}: {message}"),
        };
    assert_eq!(
        response.status,
        Nat::from(200_u8),
        "Non successful HTTP response"
    );
    let response_body: serde_json::Value =
        serde_json::from_slice(response.body.as_slice()).expect("Invalid JSON response");
    response_body["result"].as_u64().unwrap()
}

#[query(hidden = true)]
fn http_request(request: http_types::HttpRequest) -> http_types::HttpResponse {
    match request.path() {
        "/logs" => {
            let max_skip_timestamp = match request.raw_query_param("time") {
                Some(arg) => match u64::from_str(arg) {
                    Ok(value) => value,
                    Err(_) => {
                        return http_types::HttpResponseBuilder::bad_request()
                            .with_body_and_content_length("failed to parse the 'time' parameter")
                            .build()
                    }
                },
                None => 0,
            };

            let mut log: Log<Priority> = Default::default();

            match request.raw_query_param("priority").map(Priority::from_str) {
                Some(Ok(priority)) => match priority {
                    Priority::Info => log.push_logs(Priority::Info),
                    Priority::Debug => log.push_logs(Priority::Debug),
                    Priority::TraceHttp => {}
                },
                Some(Err(_)) | None => {
                    log.push_logs(Priority::Info);
                    log.push_logs(Priority::Debug);
                }
            }

            log.entries
                .retain(|entry| entry.timestamp >= max_skip_timestamp);

            fn ordering_from_query_params(sort: Option<&str>, max_skip_timestamp: u64) -> Sort {
                match sort.map(Sort::from_str) {
                    Some(Ok(order)) => order,
                    Some(Err(_)) | None => {
                        if max_skip_timestamp == 0 {
                            Sort::Ascending
                        } else {
                            Sort::Descending
                        }
                    }
                }
            }

            log.sort_logs(ordering_from_query_params(
                request.raw_query_param("sort"),
                max_skip_timestamp,
            ));

            const MAX_BODY_SIZE: usize = 2_000_000;
            http_types::HttpResponseBuilder::ok()
                .header("Content-Type", "application/json; charset=utf-8")
                .with_body_and_content_length(log.serialize_logs(MAX_BODY_SIZE))
                .build()
        }
        _ => http_types::HttpResponseBuilder::not_found().build(),
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
