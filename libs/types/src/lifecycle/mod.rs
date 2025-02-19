use candid::{CandidType, Principal};
use serde::Deserialize;

/// The installation args for the Solana RPC canister
#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct InstallArgs {
    /// Principals allowed to manage API keys.
    #[serde(rename = "manageApiKeys")]
    pub manage_api_keys: Option<Vec<Principal>>,
}
