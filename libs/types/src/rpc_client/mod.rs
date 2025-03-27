#[cfg(test)]
mod tests;

use candid::CandidType;
use derive_more::From;
use ic_cdk::api::call::RejectionCode;
pub use ic_cdk::api::management_canister::http_request::HttpHeader;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use strum::Display;
use thiserror::Error;

/// An RPC result type.
pub type RpcResult<T> = Result<T, RpcError>;

/// An RPC error.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, CandidType, Deserialize, Error, From)]
pub enum RpcError {
    /// An error occurred with the RPC provider.
    #[error("Provider error: {0}")]
    ProviderError(ProviderError),
    /// An error occurred with the HTTP outcall.
    #[error("HTTP outcall error: {0}")]
    HttpOutcallError(HttpOutcallError),
    /// A JSON-RPC error occurred.
    #[error("JSON-RPC error: {0}")]
    JsonRpcError(JsonRpcError),
    /// A validation error occurred.
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// An error with an RPC provider.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, CandidType, Deserialize, Error)]
pub enum ProviderError {
    /// Attempted to make an HTTP outcall with an insufficient amount of cycles.
    #[error("Not enough cycles, expected {expected}, received {received}")]
    TooFewCycles {
        /// Expected to receive this many cycles.
        expected: u128,
        /// Received this many cycles.
        received: u128,
    },
    /// The [`RpcConfig`] was invalid.
    #[error("Invalid RPC config: {0}")]
    InvalidRpcConfig(String),
    /// The [`SolanaCluster`] is not supported.
    #[error("Unsupported Solana cluster: {0}")]
    UnsupportedCluster(String),
}

/// An HTTP outcall error.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, CandidType, Deserialize, Error)]
pub enum HttpOutcallError {
    /// Error from the IC system API.
    #[error("IC error (code: {code:?}): {message}")]
    IcError {
        /// The error code.
        code: RejectionCode,
        /// The error message.
        message: String,
    },
    /// Response is not a valid JSON-RPC response,
    /// which means that the response was not successful (status other than 2xx)
    /// or that the response body could not be deserialized into a JSON-RPC response.
    #[error("Invalid HTTP JSON-RPC response: status {status}, body: {body}, parsing error: {parsing_error:?}")]
    InvalidHttpJsonRpcResponse {
        /// The HTTP status code returned.
        status: u16,
        /// The serialized response body.
        body: String,
        /// The parsing error message.
        #[serde(rename = "parsingError")]
        parsing_error: Option<String>,
    },
}

/// A JSON-RPC 2.0 error as per the [specifications](https://www.jsonrpc.org/specification#error_object).
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, CandidType, Deserialize, Error)]
#[error("JSON-RPC error (code: {code}): {message}")]
pub struct JsonRpcError {
    /// The error code. See the specifications for a detailed list of error codes.
    pub code: i64,
    /// The error message.
    pub message: String,
}

/// Configures how to perform RPC HTTP calls.
#[derive(Clone, Debug, PartialEq, Eq, Default, CandidType, Deserialize)]
pub struct RpcConfig {
    /// Describes the expected (90th percentile) number of bytes in the HTTP response body.
    /// This number should be less than `MAX_PAYLOAD_SIZE`.
    #[serde(rename = "responseSizeEstimate")]
    pub response_size_estimate: Option<u64>,

    /// Specifies how the responses of the different RPC providers should be aggregated into
    /// a single response.
    #[serde(rename = "responseConsensus")]
    pub response_consensus: Option<ConsensusStrategy>,
}

/// Defines a consensus strategy for combining responses from different providers.
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

/// An API defining how to make an HTTP RPC request.
#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize, CandidType)]
pub struct RpcEndpoint {
    /// The request URL to use when accessing the API.
    pub url: String,
    /// The HTTP headers to include in the requests to the API.
    pub headers: Option<Vec<HttpHeader>>,
}

impl RpcEndpoint {
    /// Returns the [`RpcEndpoint::url`]'s host.
    pub fn host_str(&self) -> Option<String> {
        url::Url::parse(&self.url)
            .ok()
            .and_then(|u| u.host_str().map(|host| host.to_string()))
    }
}

impl Debug for RpcEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let host = self.host_str().unwrap_or("N/A".to_string());
        write!(f, "RpcApi {{ host: {}, url/headers: *** }}", host) // URL or header value could contain API keys
    }
}

/// [Solana clusters](https://solana.com/docs/references/clusters).
#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    CandidType,
    Deserialize,
    Serialize,
    Display,
)]
pub enum SolanaCluster {
    /// Mainnet: live production environment for deployed applications.
    Mainnet,
    /// Devnet: Testing with public accessibility for developers experimenting with their applications.
    Devnet,
    /// Testnet: Stress-testing for network upgrades and validator performance.
    Testnet,
}

/// Uniquely identifies a supported RPC provider for a particular Solana cluster.
#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    CandidType,
    Deserialize,
    Serialize,
    Display,
)]
pub enum SupportedRpcProviderId {
    /// [Alchemy](https://www.alchemy.com/) provider for [Solana Mainnet](https://solana.com/docs/references/clusters)
    AlchemyMainnet,
    /// [Alchemy](https://www.alchemy.com/) provider on [Solana Devnet](https://solana.com/docs/references/clusters)
    AlchemyDevnet,
    /// [Ankr](https://www.ankr.com/) provider on [Solana Mainnet](https://solana.com/docs/references/clusters)
    AnkrMainnet,
    /// [Ankr](https://www.ankr.com/) provider on [Solana Devnet](https://solana.com/docs/references/clusters)
    AnkrDevnet,
    /// [dRPC](https://drpc.org/) provider on [Solana Mainnet](https://solana.com/docs/references/clusters)
    DrpcMainnet,
    /// [dRPC](https://drpc.org/) provider on [Solana Devnet](https://solana.com/docs/references/clusters)
    DrpcDevnet,
    /// [Helius](https://www.helius.dev/) provider on [Solana Mainnet](https://solana.com/docs/references/clusters)
    HeliusMainnet,
    /// [Helius](https://www.helius.dev/) provider on [Solana Devnet](https://solana.com/docs/references/clusters)
    HeliusDevnet,
    /// [PublicNode](https://www.publicnode.com/) provider on [Solana Mainnet](https://solana.com/docs/references/clusters)
    PublicNodeMainnet,
}

/// Defines a supported RPC provider for a particular Solana cluster.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, CandidType)]
pub struct SupportedRpcProvider {
    /// The Solana cluster that is accessed by this provider.
    pub cluster: SolanaCluster,
    /// The access method for this RPC provider.
    pub access: RpcAccess,
}

/// Defines a Solana RPC source.
#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize, CandidType)]
pub enum RpcSource {
    /// A supported RPC provider.
    Supported(SupportedRpcProviderId),
    /// A custom RPC service defined by an explicit [`RpcEndpoint`].
    Custom(RpcEndpoint),
}

/// Defines a collection of Solana RPC sources.
#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize, CandidType)]
pub enum RpcSources {
    /// A collection of [`RpcSource`] (either [`RpcSource::Supported`] or [`RpcSource::Custom`]).
    Custom(Vec<RpcSource>),
    /// Use the default supported providers for the given [`SolanaCluster`].
    Default(SolanaCluster),
}

impl Debug for RpcSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcSource::Supported(provider) => {
                write!(f, "Supported({:?})", provider)
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

/// Defines the authentication method for access to a [`SupportedRpcProviderId`].
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

/// Allows modifying an [`RpcEndpoint`]'s request URL and HTTP headers.
///
/// Currently, the request URL is modified using the [`OverrideProvider::override_url`] regular
/// expression and HTTP headers are reset.
#[derive(Clone, Debug, Default, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct OverrideProvider {
    /// The regular expression used to override the [`RpcEndpoint`] in when the [`OverrideProvider`] is applied.
    #[serde(rename = "overrideUrl")]
    pub override_url: Option<RegexSubstitution>,
}
