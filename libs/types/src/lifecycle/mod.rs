use crate::OverrideProvider;
use candid::{CandidType, Principal};
use serde::Deserialize;
use sol_rpc_logs::LogFilter;

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
