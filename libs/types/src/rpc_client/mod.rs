pub use ic_cdk::api::management_canister::http_request::HttpHeader;
use std::fmt::Debug;

use candid::CandidType;
use serde::{Deserialize, Serialize};
use strum::VariantArray;

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    Deserialize,
    CandidType,
    VariantArray,
)]
pub enum SolMainnetService {
    Alchemy,
    Ankr,
    PublicNode,
}

impl SolMainnetService {
    pub const fn all() -> &'static [Self] {
        SolMainnetService::VARIANTS
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    Deserialize,
    CandidType,
    VariantArray,
)]
pub enum SolDevnetService {
    Alchemy,
    Ankr,
}

impl SolDevnetService {
    pub const fn all() -> &'static [Self] {
        SolDevnetService::VARIANTS
    }
}

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize, CandidType)]
pub enum RpcService {
    Provider(u64),
    // TODO: Custom(RpcApi),
    SolMainnet(SolMainnetService),
    SolDevnet(SolDevnetService),
}

impl Debug for RpcService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcService::Provider(provider_id) => write!(f, "Provider({})", provider_id),
            // TODO: RpcService::Custom(_) => write!(f, "Custom(..)"), // Redact credentials
            RpcService::SolMainnet(service) => write!(f, "{:?}", service),
            RpcService::SolDevnet(service) => write!(f, "{:?}", service),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub struct Provider {
    #[serde(rename = "providerId")]
    pub provider_id: u64,
    #[serde(rename = "chainId")]
    pub chain_id: u64,
    pub access: RpcAccess,
    pub alias: Option<RpcService>,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub enum RpcAccess {
    /// RPC access requires authentication via one of the methods defined in [`RpcAuth`].
    Authenticated {
        /// The authentication method required for RPC access.
        auth: RpcAuth,
        /// Public URL to use when the API key is not available.
        #[serde(rename = "publicUrl")]
        public_url: Option<String>,
    },
    /// RPC access does not require authentication.
    Unauthenticated {
        /// Public URL to use.
        #[serde(rename = "publicUrl")]
        public_url: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub enum RpcAuth {
    /// API key will be used in an Authorization header as Bearer token, e.g.,
    /// `Authorization: Bearer API_KEY`
    BearerToken { url: String },
    /// API key will be inserted as a parameter into the request URL.
    UrlParameter {
        /// Request URL with the `{API_KEY}` placeholder where the API key should
        /// be inserted, e.g. `https://rpc.ankr.com/eth/{API_KEY}`.
        #[serde(rename = "urlPattern")]
        url_pattern: String,
    },
}
