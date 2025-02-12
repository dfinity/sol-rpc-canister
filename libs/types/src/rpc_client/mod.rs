#[cfg(test)]
mod tests;

pub use ic_cdk::api::management_canister::http_request::HttpHeader;
use std::fmt::Debug;

use candid::CandidType;
use serde::{Deserialize, Serialize};
use strum::VariantArray;

#[derive(Clone, Debug, PartialEq, Eq, Default, CandidType, Deserialize)]
pub struct RpcConfig {
    #[serde(rename = "responseSizeEstimate")]
    pub response_size_estimate: Option<u64>,

    #[serde(rename = "responseConsensus")]
    pub response_consensus: Option<ConsensusStrategy>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, CandidType, Deserialize)]
pub enum ConsensusStrategy {
    /// All providers must return the same non-error result.
    #[default]
    Equality,

    /// A subset of providers must return the same non-error result.
    Threshold {
        /// Total number of providers to be queried:
        /// * If `None`, will be set to the number of providers manually specified in `RpcServices`.
        /// * If `Some`, must correspond to the number of manually specified providers in `RpcServices`;
        ///   or if they are none indicating that default providers should be used, select the corresponding number of providers.
        total: Option<u8>,

        /// Minimum number of providers that must return the same (non-error) result.
        min: u8,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub enum RpcServices {
    Custom {
        #[serde(rename = "chainId")]
        chain_id: u64,
        services: Vec<RpcApi>,
    },
    SolMainnet(Option<Vec<SolMainnetService>>),
    SolDevnet(Option<Vec<SolDevnetService>>),
}

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize, CandidType)]
pub struct RpcApi {
    pub url: String,
    pub headers: Option<Vec<HttpHeader>>,
}

impl RpcApi {
    pub fn host_str(&self) -> Option<String> {
        url::Url::parse(&self.url)
            .ok()
            .and_then(|u| u.host_str().map(|host| host.to_string()))
    }
}

impl Debug for RpcApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let host = self.host_str().unwrap_or("N/A".to_string());
        write!(f, "RpcApi {{ host: {}, url/headers: *** }}", host) //URL or header value could contain API keys
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
pub enum SolMainnetService {
    Alchemy,
    Ankr,
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
    Custom(RpcApi),
    SolMainnet(SolMainnetService),
    SolDevnet(SolDevnetService),
}

impl Debug for RpcService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcService::Provider(provider_id) => write!(f, "Provider({})", provider_id),
            RpcService::Custom(_) => write!(f, "Custom(..)"), // Redact credentials
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
    Authenticated {
        auth: RpcAuth,
        /// Public URL to use when the API key is not available.
        #[serde(rename = "publicUrl")]
        public_url: Option<String>,
    },
    Unauthenticated {
        #[serde(rename = "publicUrl")]
        public_url: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub enum RpcAuth {
    /// API key will be used in an Authorization header as Bearer token, e.g.,
    /// `Authorization: Bearer API_KEY`
    BearerToken { url: String },
    UrlParameter {
        #[serde(rename = "urlPattern")]
        url_pattern: String,
    },
}
