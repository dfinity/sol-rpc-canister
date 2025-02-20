use crate::state::read_state;
use sol_rpc_logs::{declare_log_priorities, GetLogFilter, LogFilter};
use std::str::FromStr;

declare_log_priorities! {
    pub enum Priority {
        Info(capacity = 1000, buffer = INFO),
        Debug(capacity = 1000, buffer = DEBUG),
        TraceHttp(capacity = 1000, buffer = TRACE_HTTP)
    }
}

impl GetLogFilter for Priority {
    fn get_log_filter() -> LogFilter {
        read_state(|state| state.get_log_filter())
    }
}

impl FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(Priority::Info),
            "trace_http" => Ok(Priority::TraceHttp),
            "debug" => Ok(Priority::Debug),
            _ => Err("could not recognize priority".to_string()),
        }
    }
}
