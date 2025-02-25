use crate::OverrideProvider;
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
    /// Number of subnet nodes.
    #[serde(rename = "numSubnetNodes")]
    pub num_subnet_nodes: Option<NumSubnetNodes>,
    /// Mode of operation. Default is `Mode::Normal`.
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
