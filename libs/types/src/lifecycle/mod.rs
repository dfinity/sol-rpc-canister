use crate::{OverrideProvider, RegexString};
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

/// The installation args for the Solana RPC canister
#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct InstallArgs {
    /// Principals allowed to manage API keys.
    #[serde(rename = "manageApiKeys")]
    pub manage_api_keys: Option<Vec<Principal>>,
    /// Overrides the RPC providers' default URL and HTTP headers.
    #[serde(rename = "overrideProvider")]
    pub override_provider: Option<OverrideProvider>,
    /// Only log entries matching this filter will be recorded.
    #[serde(rename = "logFilter")]
    pub log_filter: Option<LogFilter>,
}

/// Only log entries matching this filter will be recorded.
#[derive(Clone, Debug, Default, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum LogFilter {
    /// All log entries are recorded.
    #[default]
    ShowAll,
    /// No log entries are recorded.
    HideAll,
    /// Only log entries matching this regular expression are recorded.
    ShowPattern(RegexString),
    /// Only log entries not matching this regular expression are recorded.
    HidePattern(RegexString),
}
