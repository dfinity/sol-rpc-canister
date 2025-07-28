#[cfg(test)]
mod tests;

use crate::{constants::API_KEY_REPLACE_STRING, validate::validate_api_key};
#[cfg(test)]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use sol_rpc_types::{RegexSubstitution, RpcEndpoint};
use std::{fmt, fmt::Debug};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, PartialEq, Zeroize, ZeroizeOnDrop, Deserialize, Serialize)]
#[cfg_attr(test, derive(Arbitrary))]
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
