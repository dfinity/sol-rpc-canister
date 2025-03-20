#[cfg(test)]
mod tests;

use crate::metrics::Metrics;
use canhttp::http::json::Id;
use std::cell::RefCell;

thread_local! {
    // Unstable static data: these are reset when the canister is upgraded.
    pub static UNSTABLE_METRICS: RefCell<Metrics> = RefCell::new(Metrics::default());
    static UNSTABLE_HTTP_REQUEST_COUNTER: RefCell<u64> = const {RefCell::new(0)};
}

pub fn next_request_id() -> Id {
    UNSTABLE_HTTP_REQUEST_COUNTER.with_borrow_mut(|counter| {
        let current_request_id = *counter;
        // overflow is not an issue here because we only use `next_request_id` to correlate
        // requests and responses in logs.
        *counter = counter.wrapping_add(1);
        Id::from(current_request_id)
    })
}
