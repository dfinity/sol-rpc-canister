#[cfg(test)]
mod tests;

use candid::CandidType;
pub use ic_cdk::api::management_canister::http_request::HttpHeader;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// An API defining how to make an HTTP RPC request.
#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize, CandidType)]
pub struct RpcApi {
    /// The request URL to use when accessing the API.
    pub url: String,
    /// The HTTP headers to include in the requests to the API.
    pub headers: Option<Vec<HttpHeader>>,
}

impl RpcApi {
    /// Returns the [`RpcApi::url`]'s host.
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

/// [Solana clusters](https://solana.com/docs/references/clusters).
#[derive(
    Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, CandidType, Deserialize, Serialize,
)]
pub enum SolanaCluster {
    /// Mainnet: live production environment for deployed applications.
    Mainnet,
    /// Devnet: Testing with public accessibility for developers experimenting with their applications.
    Devnet,
    /// Testnet: Stress-testing for network upgrades and validator performance.
    Testnet,
}

/// Unique identifier for a Solana RPC provider
#[derive(
    Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, CandidType, Deserialize, Serialize,
)]
pub enum ProviderId {
    /// [Alchemy](https://www.alchemy.com/)
    Alchemy,
    /// [Ankr](https://www.ankr.com/)
    Ankr,
    /// [PublicNode](https://www.publicnode.com/)
    PublicNode,
}

/// A Solana RPC provider for a specific Solana cluster.
pub type RpcProvider = (ProviderId, SolanaCluster);

/// Defines an RPC service for one of the Solana clusters.
#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize, CandidType)]
pub enum RpcSource {
    /// A registered RPC service.
    Registered(RpcProvider),
    /// A custom RPC service defined by an [`RpcApi`].
    Custom(RpcApi),
}

impl Debug for RpcSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcSource::Registered((provider_id, cluster)) => {
                write!(f, "Registered({:?}, {:?})", provider_id, cluster)
            }
            RpcSource::Custom(_) => write!(f, "Custom(..)"), // Redact credentials
        }
    }
}

/// Defines the access method for a registered [`RpcSource`].
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub enum RpcAccess {
    /// Access to the RPC provider requires authentication.
    Authenticated {
        /// The authentication method required for RPC access.
        auth: RpcAuth,
        /// Public URL to use when the API key is not available.
        #[serde(rename = "publicUrl")]
        public_url: Option<String>,
    },
    /// Access to the provider does not require authentication.
    Unauthenticated {
        /// Public URL to use.
        #[serde(rename = "publicUrl")]
        public_url: String,
    },
}

/// Defines the authentication method for access to a [`ProviderId`].
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub enum RpcAuth {
    /// API key will be used in an Authorization header as Bearer token, e.g.,
    /// `Authorization: Bearer API_KEY`
    BearerToken {
        /// Request URL for the provider.
        url: String,
    },
    /// API key will be inserted as a parameter into the request URL.
    UrlParameter {
        /// Request URL for the provider with the `{API_KEY}` placeholder where the
        /// API key should be inserted, e.g. `https://rpc.ankr.com/eth/{API_KEY}`.
        #[serde(rename = "urlPattern")]
        url_pattern: String,
    },
}

/// A string used as a regex pattern.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct RegexString(pub String);

impl From<&str> for RegexString {
    fn from(value: &str) -> Self {
        RegexString(value.to_string())
    }
}

impl RegexString {
    /// Compile the string into a regular expression.
    ///
    /// This is a relatively expensive operation that's currently not cached.
    pub fn compile(&self) -> Result<Regex, regex::Error> {
        Regex::new(&self.0)
    }

    /// Checks if the given string matches the compiled regex pattern.
    ///
    /// Returns `Ok(true)` if `value` matches, `Ok(false)` if not, or an error if the regex is invalid.
    pub fn try_is_valid(&self, value: &str) -> Result<bool, regex::Error> {
        Ok(self.compile()?.is_match(value))
    }
}

/// A regex-based substitution with a pattern and replacement string.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct RegexSubstitution {
    /// The pattern to be matched.
    pub pattern: RegexString,
    /// The string to replace occurrences [`pattern`](`RegexSubstitution::pattern`) with.
    pub replacement: String,
}

/// Allows modifying an [`RpcApi`]'s request URL and HTTP headers.
///
/// Currently, the request URL is modified using the [`OverrideProvider::override_url`] regular
/// expression and HTTP headers are reset.
#[derive(Clone, Debug, Default, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct OverrideProvider {
    /// The regular expression used to override the [`RpcApi`] in when the [`OverrideProvider`] is applied.
    #[serde(rename = "overrideUrl")]
    pub override_url: Option<RegexSubstitution>,
}
