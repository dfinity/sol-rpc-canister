use derive_more::From;
use ic_cdk::api::call::RejectionCode;
use std::collections::BTreeMap;
use std::time::Duration;

pub const BUCKETS_DEFAULT_MS: [u64; 8] =
    [1_000, 2_000, 4_000, 6_000, 8_000, 12_000, 20_000, u64::MAX];

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

#[macro_export]
macro_rules! add_latency_metric {
    ($metric:ident, $key:expr, $start_ns:expr) => {{
        $crate::memory::UNSTABLE_METRICS.with_borrow_mut(|m| {
            let end_ns = ::ic_cdk::api::time();
            m.$metric
                .entry($key)
                .or_default()
                .observe_latency($start_ns, end_ns);
        });
    }};
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LatencyHistogram(pub Histogram<8>);

impl Default for LatencyHistogram {
    fn default() -> Self {
        Self(Histogram::new(&BUCKETS_DEFAULT_MS))
    }
}

impl LatencyHistogram {
    pub fn observe_latency(&mut self, start_ns: u64, end_ns: u64) {
        let duration = Duration::from_nanos(end_ns.saturating_sub(start_ns));
        self.0.observe_value(duration.as_millis() as u64)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Histogram<const NUM_BUCKETS: usize> {
    bucket_upper_bounds: &'static [u64; NUM_BUCKETS],
    bucket_counts: [u64; NUM_BUCKETS],
    value_sum: u64,
}

impl<const NUM_BUCKETS: usize> Histogram<NUM_BUCKETS> {
    pub fn new(bucket_upper_bounds: &'static [u64; NUM_BUCKETS]) -> Self {
        Histogram {
            bucket_upper_bounds,
            bucket_counts: [0; NUM_BUCKETS],
            value_sum: 0,
        }
    }

    pub fn observe_value(&mut self, value: u64) {
        let bucket_index = self
            .bucket_upper_bounds
            .iter()
            .enumerate()
            .find_map(|(bucket_index, bucket_upper_bound)| {
                if value <= *bucket_upper_bound {
                    Some(bucket_index)
                } else {
                    None
                }
            })
            .expect("BUG: all values should be less than or equal to the last bucket upper bound");
        self.bucket_counts[bucket_index] += 1;
        self.value_sum += value;
    }

    /// Returns an iterator over the histogram buckets as tuples containing the bucket upper bound
    /// (inclusive), and the count of observed values within the bucket.
    pub fn iter(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.bucket_upper_bounds
            .iter()
            .enumerate()
            .map(|(bucket_index, bucket_upper_bound)| {
                if bucket_index == (NUM_BUCKETS - 1) {
                    f64::INFINITY
                } else {
                    *bucket_upper_bound as f64
                }
            })
            .zip(self.bucket_counts.iter().cloned())
            .map(|(k, v)| (k, v as f64))
    }

    /// Returns the sum of all observed latencies in milliseconds.
    pub fn sum(&self) -> u64 {
        self.value_sum
    }
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

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, From)]
pub struct MetricRpcMethod(pub String);

impl MetricLabels for MetricRpcMethod {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        vec![("method", &self.0)]
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, From)]
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

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, From)]
pub struct MetricHttpStatusCode(pub String);

impl From<u16> for MetricHttpStatusCode {
    fn from(value: u16) -> Self {
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

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, From)]
pub struct MetricRpcErrorCode(pub String);

impl From<i64> for MetricRpcErrorCode {
    fn from(value: i64) -> Self {
        MetricRpcErrorCode(value.to_string())
    }
}

impl MetricLabels for MetricRpcErrorCode {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        vec![("code", &self.0)]
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MetricRpcCallResponse {
    Success,
    IcError(RejectionCode),
    HttpError(MetricHttpStatusCode),
    JsonRpcError,
}

impl MetricLabels for MetricRpcCallResponse {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        match self {
            MetricRpcCallResponse::Success => vec![],
            MetricRpcCallResponse::IcError(rejection_code) => [("error", "ic")]
                .into_iter()
                .chain(rejection_code.metric_labels())
                .collect(),
            MetricRpcCallResponse::HttpError(status) => [("error", "http")]
                .into_iter()
                .chain(status.metric_labels())
                .collect(),
            MetricRpcCallResponse::JsonRpcError => vec![("error", "json-rpc")],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metrics {
    pub requests: BTreeMap<(MetricRpcMethod, MetricRpcHost), u64>,
    pub responses: BTreeMap<(MetricRpcMethod, MetricRpcHost, MetricRpcCallResponse), u64>,
    pub inconsistent_responses: BTreeMap<(MetricRpcMethod, MetricRpcHost), u64>,
    pub latencies: BTreeMap<(MetricRpcMethod, MetricRpcHost), LatencyHistogram>,
}

trait EncoderExtensions {
    fn counter_entries<K: MetricLabels, V: MetricValue>(
        &mut self,
        name: &str,
        map: &BTreeMap<K, V>,
        help: &str,
    );
}

impl EncoderExtensions for ic_metrics_encoder::MetricsEncoder<Vec<u8>> {
    fn counter_entries<K: MetricLabels, V: MetricValue>(
        &mut self,
        name: &str,
        map: &BTreeMap<K, V>,
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
            "Number of inconsistent JSON-RPC responses",
        );

        let mut histogram_vec = w.histogram_vec(
            "solrpc_latencies",
            "The latency of JSON-RPC calls in milliseconds.",
        )?;
        for (label, histogram) in &m.latencies {
            histogram_vec = histogram_vec.histogram(
                label.metric_labels().as_slice(),
                histogram.0.iter(),
                histogram.0.sum() as f64,
            )?;
        }

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
