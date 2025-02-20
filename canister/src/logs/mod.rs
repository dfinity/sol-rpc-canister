use crate::state::read_state;
use candid::CandidType;
use ic_canister_log::{declare_log_buffer, GlobalBuffer};
use serde::Deserialize;
use sol_rpc_logs::{LogFilter, LogPriority, PrintProxySink};
use std::str::FromStr;
use strum::VariantArray;

// High-priority messages.
declare_log_buffer!(name = INFO_BUF, capacity = 1000);

// Low-priority info messages.
declare_log_buffer!(name = DEBUG_BUF, capacity = 1000);

// Trace of HTTP requests and responses.
declare_log_buffer!(name = TRACE_HTTP_BUF, capacity = 1000);

pub const INFO: PrintProxySink<Priority> = PrintProxySink(&Priority::Info, &INFO_BUF);
pub const DEBUG: PrintProxySink<Priority> = PrintProxySink(&Priority::Debug, &DEBUG_BUF);
pub const TRACE_HTTP: PrintProxySink<Priority> =
    PrintProxySink(&Priority::TraceHttp, &TRACE_HTTP_BUF);

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, CandidType, Deserialize, serde::Serialize, VariantArray,
)]
pub enum Priority {
    Info,
    Debug,
    TraceHttp,
}

impl LogPriority for Priority {
    fn get_buffer(&self) -> &'static GlobalBuffer {
        match self {
            Self::Info => &INFO_BUF,
            Self::Debug => &DEBUG_BUF,
            Self::TraceHttp => &TRACE_HTTP_BUF,
        }
    }

    fn as_str_uppercase(&self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::TraceHttp => "TRACE_HTTP",
            Self::Debug => "DEBUG",
        }
    }

    fn get_priorities() -> &'static [Priority] {
        Self::VARIANTS
    }

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
