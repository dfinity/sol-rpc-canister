use crate::OverrideProvider;
use candid::{CandidType, Principal};
use canlog::LogFilter;
use serde::{Deserialize, Serialize};

/// The installation args for the Solana RPC canister.
#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct InstallArgs {
    /// Principals allowed to manage API keys.
    /// If not specified, only controllers may manage API keys.
    #[serde(rename = "manageApiKeys")]
    pub manage_api_keys: Option<Vec<Principal>>,
    /// Overrides the RPC providers' default URL and HTTP headers.
    /// If not specified, default URL and HTTP headers are not modified.
    #[serde(rename = "overrideProvider")]
    pub override_provider: Option<OverrideProvider>,
    /// Only log entries matching this filter will be recorded.
    /// Default is `LogFilter::ShowAll`.
    #[serde(rename = "logFilter")]
    pub log_filter: Option<LogFilter>,
    /// Number of subnet nodes.
    /// Default is `34` (i.e. the number of nodes in the fiduciary subnet).
    #[serde(rename = "numSubnetNodes")]
    pub num_subnet_nodes: Option<NumSubnetNodes>,
    /// Mode of operation.
    /// Default is `Mode::Normal`.
    pub mode: Option<Mode>,
}

/// Mode of operation
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub enum Mode {
    #[default]
    /// Normal mode, where cycle payment is required for certain operations.
    Normal,
    /// Demo mode, where cycle payment is not required.
    Demo,
}

/// Number of subnet nodes with a default value set to 34.
#[derive(Debug, Copy, Clone, CandidType, Deserialize, Serialize)]
pub struct NumSubnetNodes(u32);

impl Default for NumSubnetNodes {
    fn default() -> Self {
        NumSubnetNodes(34)
    }
}

impl From<NumSubnetNodes> for u32 {
    fn from(nodes: NumSubnetNodes) -> u32 {
        nodes.0
    }
}

impl From<u32> for NumSubnetNodes {
    fn from(nodes: u32) -> Self {
        NumSubnetNodes(nodes)
    }
}
