#[cfg(test)]
mod tests;

use candid::{
    types::{Serializer, Type, TypeInner},
    CandidType,
};
use derive_more::{From, Into};
use ic_cdk::api::call::RejectionCode;
pub use ic_cdk::api::management_canister::http_request::HttpHeader;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, num::TryFromIntError};
use strum::{Display, EnumIter};
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

impl From<solana_pubkey::ParsePubkeyError> for RpcError {
    fn from(e: solana_pubkey::ParsePubkeyError) -> Self {
        RpcError::ValidationError(format!("Invalid public key: {e}"))
    }
}

impl From<solana_signature::ParseSignatureError> for RpcError {
    fn from(e: solana_signature::ParseSignatureError) -> Self {
        RpcError::ValidationError(format!("Invalid signature: {e}"))
    }
}

impl From<solana_hash::ParseHashError> for RpcError {
    fn from(e: solana_hash::ParseHashError) -> Self {
        RpcError::ValidationError(format!("Invalid hash: {e}"))
    }
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
    #[error("Invalid HTTP JSON-RPC response: status {status}, body: {body}, parsing error: {parsing_error:?}"
    )]
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

/// Configures how to perform HTTP calls for the Solana `getSlot` RPC method.
#[derive(Clone, Debug, PartialEq, Eq, Default, CandidType, Deserialize)]
pub struct GetSlotRpcConfig {
    /// Describes the expected (90th percentile) number of bytes in the HTTP response body.
    /// This number should be less than `MAX_PAYLOAD_SIZE`.
    #[serde(rename = "responseSizeEstimate")]
    pub response_size_estimate: Option<u64>,

    /// Specifies how the responses of the different RPC providers should be aggregated into
    /// a single response.
    #[serde(rename = "responseConsensus")]
    pub response_consensus: Option<ConsensusStrategy>,

    /// The result of the `getSlot` method will be rounded down to the nearest value within
    /// this error threshold. This is done to achieve consensus between nodes on the value
    /// of the latest slot despite the fast Solana block time.
    #[serde(rename = "roundingError")]
    pub rounding_error: Option<RoundingError>,
}

impl From<GetSlotRpcConfig> for RpcConfig {
    fn from(config: GetSlotRpcConfig) -> Self {
        RpcConfig {
            response_size_estimate: config.response_size_estimate,
            response_consensus: config.response_consensus,
        }
    }
}

impl From<RpcConfig> for GetSlotRpcConfig {
    fn from(value: RpcConfig) -> Self {
        GetSlotRpcConfig {
            response_size_estimate: value.response_size_estimate,
            response_consensus: value.response_consensus,
            ..Default::default()
        }
    }
}

/// Configures how to perform HTTP calls for the Solana `getRecentPrioritizationFees` RPC method.
///
/// The response to `getRecentPrioritizationFees` corresponds to a (non-necessarily continuous) range of slots associated
/// with the priority fee for that slot and may include `processed` slots (a new `processed` slot is produced every ca. 400ms); e.g.
/// ```json
/// ...
/// {
///     "prioritizationFee": 166667,
///     "slot": 338637772
///},
///{
///    "prioritizationFee": 0,
///    "slot": 338637773
///},
///{
///    "prioritizationFee": 0,
///    "slot": 338637774
///},
///{
///    "prioritizationFee": 50000,
///    "slot": 338637775
///},
/// ...
/// ```
/// Similarly to the necessary rounding used for `getSlot`,
/// achieving consensus for `getRecentPrioritizationFees` requires selecting a subset of those slots
/// that can be seen by a super-majority of the nodes, which is done as follows:
/// 1. `max_slot_rounding_error`: round down the slot with the maximum value.
///    The selected subset will only contain priority fees for slots that are smaller or equal to the rounded-down slot.
/// 2. `max_length`: limit the size of the selected subset by removing priority fees for the older slots.
#[derive(Clone, Debug, PartialEq, Eq, Default, CandidType, Deserialize)]
pub struct GetRecentPrioritizationFeesRpcConfig {
    /// Describes the expected (90th percentile) number of bytes in the HTTP response body.
    /// This number should be less than `MAX_PAYLOAD_SIZE`.
    #[serde(rename = "responseSizeEstimate")]
    pub response_size_estimate: Option<u64>,

    /// Round down the slot with the maximum value.
    /// Increasing that value will reduce the freshness of the returned prioritization fees
    /// but increase the likelihood of nodes reaching consensus.
    #[serde(rename = "responseConsensus")]
    pub response_consensus: Option<ConsensusStrategy>,

    /// Round down the slot with the maximum value.
    /// Increasing that value will reduce the freshness of the returned prioritization fees
    /// but increase the likelihood of nodes reaching consensus.
    #[serde(rename = "maxSlotRoundingError")]
    pub max_slot_rounding_error: Option<RoundingError>,

    #[serde(rename = "maxLength")]
    max_length: Option<NonZeroU8>,
}

impl GetRecentPrioritizationFeesRpcConfig {
    /// Default number of priority fees to return.
    pub const DEFAULT_MAX_LENGTH: NonZeroU8 =
        NonZeroU8::new(std::num::NonZeroU8::new(100_u8).unwrap());

    /// Number of priority fees to return.
    ///
    /// Returns the current value or the default [`Self::DEFAULT_MAX_LENGTH`].
    pub fn max_length(&self) -> NonZeroU8 {
        self.max_length.unwrap_or(Self::DEFAULT_MAX_LENGTH)
    }

    /// Change the number of priority fees to return.
    ///
    /// A Solana validator returns at most 150 entries, so that bigger values are possible but not useful.
    /// Increasing that value can help in estimating the current priority fee
    /// but will reduce the likelihood of nodes reaching consensus.
    pub fn set_max_length(&mut self, len: NonZeroU8) {
        self.max_length = Some(len)
    }
}

impl From<RpcConfig> for GetRecentPrioritizationFeesRpcConfig {
    fn from(value: RpcConfig) -> Self {
        GetRecentPrioritizationFeesRpcConfig {
            response_size_estimate: value.response_size_estimate,
            response_consensus: value.response_consensus,
            ..Default::default()
        }
    }
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
    EnumIter,
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
    /// [Chainstack](https://www.chainstack.com/) provider on [Solana Mainnet](https://solana.com/docs/references/clusters)
    ChainstackMainnet,
    /// [Chainstack](https://www.chainstack.com/) provider on [Solana Devnet](https://solana.com/docs/references/clusters)
    ChainstackDevnet,
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

impl RpcSource {
    /// Return the supported RPC provider ID if the source corresponds to a supported provider,
    /// `None` otherwise.
    pub fn rpc_provider_id(&self) -> Option<SupportedRpcProviderId> {
        match &self {
            RpcSource::Supported(id) => Some(*id),
            RpcSource::Custom(_) => None,
        }
    }
}

/// Defines a collection of Solana RPC sources.
#[derive(
    Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize, CandidType, Debug,
)]
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

/// This type defines a rounding error to use when fetching the current
/// [slot](https://solana.com/docs/references/terminology#slot) from Solana using the JSON-RPC
/// interface, meaning slots will be rounded down to the nearest multiple of this error when
/// being fetched.
///
/// This is done to achieve consensus on the HTTP outcalls whose responses contain Solana slots
/// despite Solana's fast blocktime and hence fast-changing slot value. However, this solution
/// does not guarantee consensus on the slot value across nodes and different consensus rates
/// will be achieved depending on the rounding error value used. A higher rounding error will
/// lead to a higher consensus rate, but also means the slot value may differ more from the actual
/// value on the Solana blockchain. This means, for example, that setting a large rounding error
/// and then fetching the corresponding block with the Solana
/// [`getBlock`](https://solana.com/docs/rpc/http/getblock) RPC method can result in obtaining a
/// block whose hash is too old to use in a valid Solana transaction (see more details about using
/// recent blockhashes [here](https://solana.com/developers/guides/advanced/confirmation#how-does-transaction-expiration-work).
///
/// The default value given by [`RoundingError::default`]
/// has been experimentally shown to achieve a high HTTP outcall consensus rate.
///
/// See the [`RoundingError::round`] method for more details and examples.
#[derive(
    Debug,
    Clone,
    Copy,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    CandidType,
    From,
    Into,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct RoundingError(u64);

impl Default for RoundingError {
    fn default() -> Self {
        Self(20)
    }
}

impl AsRef<u64> for RoundingError {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl RoundingError {
    /// Create a new instance of [`RoundingError`] with the given value.
    pub fn new(rounding_error: u64) -> Self {
        Self(rounding_error)
    }

    /// Round the given value down to the nearest multiple of the rounding error.
    /// A rounding error of 0 or 1 leads to this method returning the input unchanged.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sol_rpc_types::RoundingError;
    ///
    /// assert_eq!(RoundingError::new(0).round(19), 19);
    /// assert_eq!(RoundingError::new(1).round(19), 19);
    /// assert_eq!(RoundingError::new(10).round(19), 10);
    /// assert_eq!(RoundingError::new(20).round(19), 0);
    /// ```
    pub fn round(&self, slot: u64) -> u64 {
        match self.0 {
            0 | 1 => slot,
            n => (slot / n) * n,
        }
    }
}

/// A wrapper around the primitive [`std::num::NonZeroU8`] to implement [`candid::CandidType`].
///
/// From the point of view of Candid, this is like a [`u8`], except that a zero value will fail to be deserialized.
///
/// # Examples
///
/// ```rust
/// use candid::{Decode, Encode};
/// use sol_rpc_types::NonZeroU8;
///
/// let one = 1_u8;
/// let non_zero_one = NonZeroU8::try_from(one).unwrap();
/// let encoded_one = Encode!(&one).unwrap();
/// assert_eq!(encoded_one, Encode!(&non_zero_one).unwrap());
/// assert_eq!(non_zero_one, Decode!(&encoded_one, NonZeroU8).unwrap());
///
/// let encoded_zero = Encode!(&0_u8).unwrap();
/// assert!(Decode!(&encoded_zero, NonZeroU8).is_err());
/// ```
#[derive(
    Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, From, Into, Serialize, Deserialize,
)]
#[serde(try_from = "u8", into = "u8")]
pub struct NonZeroU8(std::num::NonZeroU8);

impl CandidType for NonZeroU8 {
    fn _ty() -> Type {
        Type(TypeInner::Nat8.into())
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_nat8(self.0.get())
    }
}

impl NonZeroU8 {
    /// Construct a new instance of [`NonZeroU8`].
    pub const fn new(value: std::num::NonZeroU8) -> Self {
        Self(value)
    }
}

impl From<NonZeroU8> for u8 {
    fn from(value: NonZeroU8) -> Self {
        value.0.get()
    }
}

impl TryFrom<u8> for NonZeroU8 {
    type Error = TryFromIntError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        std::num::NonZeroU8::try_from(value).map(Self)
    }
}
