use crate::state::read_state;
use canlog::{GetLogFilter, LogFilter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(canlog_derive::LogPriorityLevels, Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub enum Priority {
    #[log_level(capacity = 1000, name = "INFO")]
    Info,
    #[log_level(capacity = 1000, name = "DEBUG")]
    Debug,
    #[log_level(capacity = 1000, name = "TRACE_HTTP")]
    TraceHttp,
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
