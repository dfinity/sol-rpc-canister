use candid::CandidType;
use derive_more::From;
use ic_cdk::api::call::RejectionCode;
use serde::Deserialize;
use std::collections::HashMap;

#[macro_export]
macro_rules! add_metric {
    ($metric:ident, $amount:expr) => {{
        $crate::memory::UNSTABLE_METRICS.with_borrow_mut(|m| m.$metric += $amount);
    }};
}

#[macro_export]
macro_rules! add_metric_entry {
    ($metric:ident, $key:expr, $amount:expr) => {{
        $crate::memory::UNSTABLE_METRICS.with_borrow_mut(|m| {
            let amount = $amount;
            if amount != 0 {
                m.$metric
                    .entry($key)
                    .and_modify(|counter| *counter += amount)
                    .or_insert(amount);
            }
        });
    }};
}

pub trait MetricValue {
    fn metric_value(&self) -> f64;
}

impl MetricValue for u32 {
    fn metric_value(&self) -> f64 {
        *self as f64
    }
}

impl MetricValue for u64 {
    fn metric_value(&self) -> f64 {
        *self as f64
    }
}

impl MetricValue for u128 {
    fn metric_value(&self) -> f64 {
        *self as f64
    }
}

pub trait MetricLabels {
    fn metric_labels(&self) -> Vec<(&str, &str)>;
}

impl<A: MetricLabels, B: MetricLabels> MetricLabels for (A, B) {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        [self.0.metric_labels(), self.1.metric_labels()].concat()
    }
}

impl<A: MetricLabels, B: MetricLabels, C: MetricLabels> MetricLabels for (A, B, C) {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        [
            self.0.metric_labels(),
            self.1.metric_labels(),
            self.2.metric_labels(),
        ]
        .concat()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, CandidType, Deserialize, From)]
pub struct MetricRpcMethod(pub String);

impl From<RpcMethod> for MetricRpcMethod {
    fn from(method: RpcMethod) -> Self {
        MetricRpcMethod(method.name().to_string())
    }
}

impl MetricLabels for MetricRpcMethod {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        vec![("method", &self.0)]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, CandidType, Deserialize, From)]
pub struct MetricRpcHost(pub String);

impl From<&str> for MetricRpcHost {
    fn from(hostname: &str) -> Self {
        MetricRpcHost(hostname.to_string())
    }
}

impl MetricLabels for MetricRpcHost {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        vec![("host", &self.0)]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, CandidType, Deserialize)]
pub struct MetricHttpStatusCode(pub String);

impl From<u32> for MetricHttpStatusCode {
    fn from(value: u32) -> Self {
        MetricHttpStatusCode(value.to_string())
    }
}

impl MetricLabels for MetricHttpStatusCode {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        vec![("status", &self.0)]
    }
}

impl MetricLabels for RejectionCode {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        let code = match self {
            RejectionCode::NoError => "NO_ERROR",
            RejectionCode::SysFatal => "SYS_FATAL",
            RejectionCode::SysTransient => "SYS_TRANSIENT",
            RejectionCode::DestinationInvalid => "DESTINATION_INVALID",
            RejectionCode::CanisterReject => "CANISTER_REJECT",
            RejectionCode::CanisterError => "CANISTER_ERROR",
            RejectionCode::Unknown => "UNKNOWN",
        };
        vec![("code", code)]
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, CandidType, Deserialize)]
pub struct Metrics {
    pub requests: HashMap<(MetricRpcMethod, MetricRpcHost), u64>,
    pub responses: HashMap<(MetricRpcMethod, MetricRpcHost, MetricHttpStatusCode), u64>,
    #[serde(rename = "inconsistentResponses")]
    pub inconsistent_responses: HashMap<(MetricRpcMethod, MetricRpcHost), u64>,
    #[serde(rename = "errHttpOutcall")]
    pub err_http_outcall: HashMap<(MetricRpcMethod, MetricRpcHost, RejectionCode), u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RpcMethod {
    GetAccountInfo,
    GetSlot,
    Generic,
    SendTransaction,
}

impl RpcMethod {
    fn name(self) -> &'static str {
        match self {
            RpcMethod::GetAccountInfo => "getAccountInfo",
            RpcMethod::GetSlot => "getSlot",
            RpcMethod::Generic => "generic",
            RpcMethod::SendTransaction => "sendTransaction",
        }
    }
}

trait EncoderExtensions {
    fn counter_entries<K: MetricLabels, V: MetricValue>(
        &mut self,
        name: &str,
        map: &HashMap<K, V>,
        help: &str,
    );
}

impl EncoderExtensions for ic_metrics_encoder::MetricsEncoder<Vec<u8>> {
    fn counter_entries<K: MetricLabels, V: MetricValue>(
        &mut self,
        name: &str,
        map: &HashMap<K, V>,
        help: &str,
    ) {
        map.iter().for_each(|(k, v)| {
            self.counter_vec(name, help)
                .and_then(|m| {
                    m.value(&k.metric_labels(), v.metric_value())?;
                    Ok(())
                })
                .unwrap_or(());
        })
    }
}

pub fn encode_metrics(w: &mut ic_metrics_encoder::MetricsEncoder<Vec<u8>>) -> std::io::Result<()> {
    const WASM_PAGE_SIZE_IN_BYTES: f64 = 65536.0;

    crate::memory::UNSTABLE_METRICS.with(|m| {
        let m = m.borrow();

        w.gauge_vec("cycle_balance", "Cycle balance of this canister")?
            .value(
                &[("canister", "solrpc")],
                ic_cdk::api::canister_balance128().metric_value(),
            )?;
        w.encode_gauge(
            "solrpc_canister_version",
            ic_cdk::api::canister_version().metric_value(),
            "Canister version",
        )?;
        w.encode_gauge(
            "stable_memory_bytes",
            ic_cdk::api::stable::stable_size() as f64 * WASM_PAGE_SIZE_IN_BYTES,
            "Size of the stable memory allocated by this canister.",
        )?;

        w.encode_gauge(
            "heap_memory_bytes",
            heap_memory_size_bytes() as f64,
            "Size of the heap memory allocated by this canister.",
        )?;

        w.counter_entries(
            "solrpc_requests",
            &m.requests,
            "Number of JSON-RPC requests",
        );
        w.counter_entries(
            "solrpc_responses",
            &m.responses,
            "Number of JSON-RPC responses",
        );
        w.counter_entries(
            "solrpc_inconsistent_responses",
            &m.inconsistent_responses,
            "Number of inconsistent RPC responses",
        );
        w.counter_entries(
            "solrpc_err_http_outcall",
            &m.err_http_outcall,
            "Number of unsuccessful HTTP outcalls",
        );

        Ok(())
    })
}

/// Returns the amount of heap memory in bytes that has been allocated.
#[cfg(target_arch = "wasm32")]
pub fn heap_memory_size_bytes() -> usize {
    const WASM_PAGE_SIZE_BYTES: usize = 65536;
    core::arch::wasm32::memory_size(0) * WASM_PAGE_SIZE_BYTES
}

#[cfg(not(any(target_arch = "wasm32")))]
pub fn heap_memory_size_bytes() -> usize {
    0
}
