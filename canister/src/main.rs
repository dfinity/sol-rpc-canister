use candid::candid_method;
use canhttp::multi::Timestamp;
use canlog::{log, Log, Sort};
use ic_cdk::{api::is_controller, query, update};
use ic_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_metrics_encoder::MetricsEncoder;
use sol_rpc_canister::{
    candid_rpc::send_multi,
    lifecycle,
    logs::Priority,
    memory::{mutate_state, read_state, State},
    metrics::encode_metrics,
    providers::{get_provider, PROVIDERS},
    rpc_client::MultiRpcRequest,
};
use sol_rpc_types::{
    AccountInfo, ConfirmedBlock, ConfirmedTransactionStatusWithSignature,
    EncodedConfirmedTransactionWithStatusMeta, GetAccountInfoParams, GetBalanceParams,
    GetBlockParams, GetRecentPrioritizationFeesParams, GetRecentPrioritizationFeesRpcConfig,
    GetSignatureStatusesParams, GetSignaturesForAddressParams, GetSlotParams, GetSlotRpcConfig,
    GetTokenAccountBalanceParams, GetTransactionParams, Lamport, MultiRpcResult, PrioritizationFee,
    RpcAccess, RpcConfig, RpcResult, RpcSources, SendTransactionParams, Signature, Slot,
    SupportedRpcProvider, SupportedRpcProviderId, TokenAmount, TransactionStatus,
};
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
fn get_providers() -> Vec<(SupportedRpcProviderId, SupportedRpcProvider)> {
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
async fn update_api_keys(api_keys: Vec<(SupportedRpcProviderId, Option<String>)>) {
    log!(
        Priority::Info,
        "[{}] Updating API keys for providers: {}",
        ic_cdk::caller(),
        api_keys
            .iter()
            .map(|(provider, _)| format!("{:?}", provider))
            .collect::<Vec<_>>()
            .join(", ")
    );
    for (provider, api_key) in api_keys {
        let access = get_provider(&provider)
            .map(|provider| provider.access)
            .unwrap_or_else(|| panic!("Provider not found: {:?}", provider));
        if let RpcAccess::Unauthenticated { .. } = access {
            panic!(
                "Trying to set API key for unauthenticated provider: {:?}",
                provider
            )
        }
        match api_key {
            Some(key) => mutate_state(|state| {
                state.insert_api_key(provider, key.try_into().expect("Invalid API key"))
            }),
            None => mutate_state(|state| state.remove_api_key(&provider)),
        }
    }
}

#[update(name = "getAccountInfo")]
#[candid_method(rename = "getAccountInfo")]
async fn get_account_info(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: GetAccountInfoParams,
) -> MultiRpcResult<Option<AccountInfo>> {
    let request =
        MultiRpcRequest::get_account_info(source, config.unwrap_or_default(), params, now());
    send_multi(request).await.into()
}

#[query(name = "getAccountInfoCyclesCost")]
#[candid_method(query, rename = "getAccountInfoCyclesCost")]
async fn get_account_info_cycles_cost(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: GetAccountInfoParams,
) -> RpcResult<u128> {
    if read_state(State::is_demo_mode_active) {
        return Ok(0);
    }
    MultiRpcRequest::get_account_info(source, config.unwrap_or_default(), params, now())?
        .cycles_cost()
        .await
}

#[update(name = "getBalance")]
#[candid_method(rename = "getBalance")]
async fn get_balance(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: GetBalanceParams,
) -> MultiRpcResult<Lamport> {
    let request = MultiRpcRequest::get_balance(source, config.unwrap_or_default(), params, now());
    send_multi(request).await
}

#[query(name = "getBalanceCyclesCost")]
#[candid_method(query, rename = "getBalanceCyclesCost")]
async fn get_balance_cycles_cost(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: GetBalanceParams,
) -> RpcResult<u128> {
    if read_state(State::is_demo_mode_active) {
        return Ok(0);
    }
    MultiRpcRequest::get_balance(source, config.unwrap_or_default(), params, now())?
        .cycles_cost()
        .await
}

#[update(name = "getBlock")]
#[candid_method(rename = "getBlock")]
async fn get_block(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: GetBlockParams,
) -> MultiRpcResult<Option<ConfirmedBlock>> {
    let request = MultiRpcRequest::get_block(source, config.unwrap_or_default(), params, now());
    send_multi(request).await.into()
}

#[query(name = "getBlockCyclesCost")]
#[candid_method(query, rename = "getBlockCyclesCost")]
async fn get_block_cycles_cost(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: GetBlockParams,
) -> RpcResult<u128> {
    if read_state(State::is_demo_mode_active) {
        return Ok(0);
    }
    MultiRpcRequest::get_block(source, config.unwrap_or_default(), params, now())?
        .cycles_cost()
        .await
}

#[update(name = "getRecentPrioritizationFees")]
#[candid_method(rename = "getRecentPrioritizationFees")]
async fn get_recent_prioritization_fees(
    source: RpcSources,
    config: Option<GetRecentPrioritizationFeesRpcConfig>,
    params: Option<GetRecentPrioritizationFeesParams>,
) -> MultiRpcResult<Vec<PrioritizationFee>> {
    let request = MultiRpcRequest::get_recent_prioritization_fees(
        source,
        config.unwrap_or_default(),
        params.unwrap_or_default(),
        now(),
    );
    send_multi(request).await
}

#[query(name = "getRecentPrioritizationFeesCyclesCost")]
#[candid_method(query, rename = "getRecentPrioritizationFeesCyclesCost")]
async fn get_recent_prioritization_fees_cycles_cost(
    source: RpcSources,
    config: Option<GetRecentPrioritizationFeesRpcConfig>,
    params: Option<GetRecentPrioritizationFeesParams>,
) -> RpcResult<u128> {
    if read_state(State::is_demo_mode_active) {
        return Ok(0);
    }
    MultiRpcRequest::get_recent_prioritization_fees(
        source,
        config.unwrap_or_default(),
        params.unwrap_or_default(),
        now(),
    )?
    .cycles_cost()
    .await
}

#[update(name = "getSignaturesForAddress")]
#[candid_method(rename = "getSignaturesForAddress")]
async fn get_signatures_for_address(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: GetSignaturesForAddressParams,
) -> MultiRpcResult<Vec<ConfirmedTransactionStatusWithSignature>> {
    let request = MultiRpcRequest::get_signatures_for_address(
        source,
        config.unwrap_or_default(),
        params,
        now(),
    );
    send_multi(request).await
}

#[query(name = "getSignaturesForAddressCyclesCost")]
#[candid_method(query, rename = "getSignaturesForAddressCyclesCost")]
async fn get_signatures_for_address_cycles_cost(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: GetSignaturesForAddressParams,
) -> RpcResult<u128> {
    if read_state(State::is_demo_mode_active) {
        return Ok(0);
    }
    MultiRpcRequest::get_signatures_for_address(source, config.unwrap_or_default(), params, now())?
        .cycles_cost()
        .await
}

#[update(name = "getSignatureStatuses")]
#[candid_method(rename = "getSignatureStatuses")]
async fn get_signature_statuses(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: GetSignatureStatusesParams,
) -> MultiRpcResult<Vec<Option<TransactionStatus>>> {
    let request =
        MultiRpcRequest::get_signature_statuses(source, config.unwrap_or_default(), params, now());
    send_multi(request).await.into()
}

#[query(name = "getSignatureStatusesCyclesCost")]
#[candid_method(query, rename = "getSignatureStatusesCyclesCost")]
async fn get_signature_statuses_cycles_cost(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: GetSignatureStatusesParams,
) -> RpcResult<u128> {
    if read_state(State::is_demo_mode_active) {
        return Ok(0);
    }
    MultiRpcRequest::get_signature_statuses(source, config.unwrap_or_default(), params, now())?
        .cycles_cost()
        .await
}

#[update(name = "getSlot")]
#[candid_method(rename = "getSlot")]
async fn get_slot(
    source: RpcSources,
    config: Option<GetSlotRpcConfig>,
    params: Option<GetSlotParams>,
) -> MultiRpcResult<Slot> {
    let request = MultiRpcRequest::get_slot(
        source,
        config.unwrap_or_default(),
        params.unwrap_or_default(),
        now(),
    );
    send_multi(request).await
}

#[query(name = "getSlotCyclesCost")]
#[candid_method(query, rename = "getSlotCyclesCost")]
async fn get_slot_cycles_cost(
    source: RpcSources,
    config: Option<GetSlotRpcConfig>,
    params: Option<GetSlotParams>,
) -> RpcResult<u128> {
    if read_state(State::is_demo_mode_active) {
        return Ok(0);
    }
    MultiRpcRequest::get_slot(
        source,
        config.unwrap_or_default(),
        params.unwrap_or_default(),
        now(),
    )?
    .cycles_cost()
    .await
}

#[update(name = "getTokenAccountBalance")]
#[candid_method(rename = "getTokenAccountBalance")]
async fn get_token_account_balance(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: GetTokenAccountBalanceParams,
) -> MultiRpcResult<TokenAmount> {
    let request = MultiRpcRequest::get_token_account_balance(
        source,
        config.unwrap_or_default(),
        params,
        now(),
    );
    send_multi(request).await.into()
}

#[query(name = "getTokenAccountBalanceCyclesCost")]
#[candid_method(query, rename = "getTokenAccountBalanceCyclesCost")]
async fn get_token_account_balance_cycles_cost(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: GetTokenAccountBalanceParams,
) -> RpcResult<u128> {
    if read_state(State::is_demo_mode_active) {
        return Ok(0);
    }
    MultiRpcRequest::get_token_account_balance(source, config.unwrap_or_default(), params, now())?
        .cycles_cost()
        .await
}

#[update(name = "getTransaction")]
#[candid_method(rename = "getTransaction")]
async fn get_transaction(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: GetTransactionParams,
) -> MultiRpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>> {
    let request =
        MultiRpcRequest::get_transaction(source, config.unwrap_or_default(), params, now());
    send_multi(request).await.into()
}

#[query(name = "getTransactionCyclesCost")]
#[candid_method(query, rename = "getTransactionCyclesCost")]
async fn get_transaction_cycles_cost(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: GetTransactionParams,
) -> RpcResult<u128> {
    if read_state(State::is_demo_mode_active) {
        return Ok(0);
    }
    MultiRpcRequest::get_transaction(source, config.unwrap_or_default(), params, now())?
        .cycles_cost()
        .await
}

#[update(name = "sendTransaction")]
#[candid_method(rename = "sendTransaction")]
async fn send_transaction(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: SendTransactionParams,
) -> MultiRpcResult<Signature> {
    let request =
        MultiRpcRequest::send_transaction(source, config.unwrap_or_default(), params, now());
    send_multi(request).await
}

#[query(name = "sendTransactionCyclesCost")]
#[candid_method(query, rename = "sendTransactionCyclesCost")]
async fn send_transaction_cycles_cost(
    source: RpcSources,
    config: Option<RpcConfig>,
    params: SendTransactionParams,
) -> RpcResult<u128> {
    if read_state(State::is_demo_mode_active) {
        return Ok(0);
    }
    MultiRpcRequest::send_transaction(source, config.unwrap_or_default(), params, now())?
        .cycles_cost()
        .await
}

#[update(name = "jsonRequest")]
#[candid_method(rename = "jsonRequest")]
async fn json_request(
    source: RpcSources,
    config: Option<RpcConfig>,
    json_rpc_payload: String,
) -> MultiRpcResult<String> {
    let request =
        MultiRpcRequest::json_request(source, config.unwrap_or_default(), json_rpc_payload, now());
    send_multi(request).await.map(|value| value.to_string())
}

#[query(name = "jsonRequestCyclesCost")]
#[candid_method(query, rename = "jsonRequestCyclesCost")]
async fn json_request_cycles_cost(
    source: RpcSources,
    config: Option<RpcConfig>,
    json_rpc_payload: String,
) -> RpcResult<u128> {
    if read_state(State::is_demo_mode_active) {
        return Ok(0);
    }
    MultiRpcRequest::json_request(source, config.unwrap_or_default(), json_rpc_payload, now())?
        .cycles_cost()
        .await
}

#[query(hidden = true)]
fn http_request(request: HttpRequest) -> HttpResponse {
    match request.path() {
        "/metrics" => {
            let mut writer = MetricsEncoder::new(vec![], ic_cdk::api::time() as i64 / 1_000_000);

            match encode_metrics(&mut writer) {
                Ok(()) => HttpResponseBuilder::ok()
                    .header("Content-Type", "text/plain; version=0.0.4")
                    .with_body_and_content_length(writer.into_inner())
                    .build(),
                Err(err) => {
                    HttpResponseBuilder::server_error(format!("Failed to encode metrics: {}", err))
                        .build()
                }
            }
        }
        "/logs" => {
            let max_skip_timestamp = match request.raw_query_param("time") {
                Some(arg) => match u64::from_str(arg) {
                    Ok(value) => value,
                    Err(_) => {
                        return HttpResponseBuilder::bad_request()
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
                    Priority::TraceHttp => log.push_logs(Priority::TraceHttp),
                },
                Some(Err(_)) | None => {
                    log.push_logs(Priority::Info);
                    log.push_logs(Priority::Debug);
                    log.push_logs(Priority::TraceHttp);
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
            HttpResponseBuilder::ok()
                .header("Content-Type", "application/json; charset=utf-8")
                .with_body_and_content_length(log.serialize_logs(MAX_BODY_SIZE))
                .build()
        }
        _ => HttpResponseBuilder::not_found().build(),
    }
}

#[query(
    guard = "require_api_key_principal_or_controller",
    name = "verifyApiKey",
    hidden = true
)]
async fn verify_api_key((provider, api_key): (SupportedRpcProviderId, Option<String>)) {
    let api_key = api_key.map(|key| TryFrom::try_from(key).expect("Invalid API key"));
    if read_state(|state| state.get_api_key(&provider)) != api_key {
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

fn now() -> Timestamp {
    Timestamp::from_nanos_since_unix_epoch(ic_cdk::api::time())
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
