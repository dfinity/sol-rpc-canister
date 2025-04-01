#[cfg(test)]
mod tests;

use crate::{constants::API_KEY_REPLACE_STRING, validate::validate_api_key};
use derive_more::{From, Into};
use minicbor::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sol_rpc_types::{RegexSubstitution, RpcEndpoint};
use std::{fmt, fmt::Debug};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, PartialEq, Zeroize, ZeroizeOnDrop, Deserialize, Serialize)]
pub struct ApiKey(String);

impl ApiKey {
    /// Explicitly read API key (use sparingly)
    pub fn read(&self) -> &str {
        &self.0
    }
}

/// Enable printing data structures which include an API key
impl Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{API_KEY_REPLACE_STRING}")
    }
}

impl TryFrom<String> for ApiKey {
    type Error = String;
    fn try_from(key: String) -> Result<ApiKey, Self::Error> {
        validate_api_key(&key)?;
        Ok(ApiKey(key))
    }
}

/// Copy of [`sol_rpc_types::OverrideProvider`] to keep the implementation details out of the
/// [`sol_rpc_types`] crate.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OverrideProvider {
    pub override_url: Option<RegexSubstitution>,
}

impl From<sol_rpc_types::OverrideProvider> for OverrideProvider {
    fn from(value: sol_rpc_types::OverrideProvider) -> Self {
        Self {
            override_url: value.override_url,
        }
    }
}

impl OverrideProvider {
    /// Override the resolved provider API (url and headers).
    ///
    /// # Limitations
    ///
    /// Currently, only the url can be replaced by regular expression. Headers will be reset.
    ///
    /// # Security considerations
    ///
    /// The resolved provider API may contain sensitive data (such as API keys) that may be extracted
    /// by using the override mechanism. Since only the controller of the canister can set the override parameters,
    /// upon canister initialization or upgrade, it's the controller's responsibility to ensure that this is not a problem
    /// (e.g., if only used for local development).
    pub fn apply(&self, api: RpcEndpoint) -> Result<RpcEndpoint, regex::Error> {
        match &self.override_url {
            None => Ok(api),
            Some(substitution) => {
                let regex = substitution.pattern.compile()?;
                let new_url = regex.replace_all(&api.url, &substitution.replacement);
                Ok(RpcEndpoint {
                    url: new_url.to_string(),
                    headers: None,
                })
            }
        }
    }
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
#[derive(Debug, Decode, Encode, Clone, Copy, Eq, PartialEq, From, Into)]
pub struct RoundingError(#[n(0)] u64);

impl Default for RoundingError {
    fn default() -> Self {
        Self(20)
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
    /// use sol_rpc_canister::types::RoundingError;
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
