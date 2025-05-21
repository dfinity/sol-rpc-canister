//! Module for interacting with Solana [nonce accounts](https://solana.com/de/developers/guides/advanced/introduction-to-durable-nonces#nonce-account).

use solana_account_decoder_client_types::UiAccount;
use solana_nonce::{state::State, versions::Versions};
use thiserror::Error;

#[cfg(test)]
mod tests;

/// Extracts the durable nonce value from the response of a `getAccountInfo` RPC call.
///
/// # Examples
///
/// ```rust
/// use sol_rpc_client::{nonce::extract_durable_nonce, SolRpcClient};
/// use sol_rpc_types::{RpcSources, SolanaCluster};
/// use solana_hash::Hash;
/// use solana_pubkey::pubkey;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use std::str::FromStr;
/// # use sol_rpc_client::fixtures::initialized_nonce_account;
/// # use sol_rpc_types::{AccountData, AccountEncoding, AccountInfo, MultiRpcResult};
/// let client = SolRpcClient::builder_for_ic()
/// #   .with_mocked_response(MultiRpcResult::Consistent(Ok(Some(initialized_nonce_account()))))
///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Devnet))
///     .build();
///
/// let nonce_account = client
///     .get_account_info(pubkey!("8DedqKHx9ogFajbHtRnTM3pPr3MRyVKDtepEpUiaDXX"))
///     .send()
///     .await
///     .expect_consistent()
///     .unwrap()
///     .unwrap();
///
/// let durable_nonce = extract_durable_nonce(&nonce_account)
///     .unwrap();
///
/// assert_eq!(durable_nonce, Hash::from_str("6QK3LC8dsRtH2qVU47cSvgchPHNU72f1scvg2LuN2z7e").unwrap());
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// The method will return an instance of [`ExtractNonceError`] if the account data does not
/// correspond to a valid and properly encoded nonce account. See [`ExtractNonceError`] for
/// more details.
///
/// ```rust
/// use sol_rpc_client::{nonce::{ExtractNonceError, extract_durable_nonce}, SolRpcClient};
/// use sol_rpc_types::{RpcSources, SolanaCluster};
/// use solana_hash::Hash;
/// use solana_pubkey::pubkey;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use std::str::FromStr;
/// # use assert_matches::assert_matches;
/// # use sol_rpc_client::fixtures::usdc_account;
/// # use sol_rpc_types::{AccountData, AccountEncoding, AccountInfo, MultiRpcResult};
/// let client = SolRpcClient::builder_for_ic()
/// #   .with_mocked_response(MultiRpcResult::Consistent(Ok(Some(usdc_account()))))
///     .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
///     .build();
///
/// // Fetch the USDC account data on Mainnet
/// let usdc_account = client
///     .get_account_info(pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"))
///     .send()
///     .await
///     .expect_consistent()
///     .unwrap()
///     .unwrap();
///
/// let durable_nonce = extract_durable_nonce(&usdc_account);
///
/// assert_matches!(durable_nonce, Err(ExtractNonceError::InvalidAccountData(_)));
/// # Ok(())
/// # }
/// ```
pub fn extract_durable_nonce(account: &UiAccount) -> Result<solana_hash::Hash, ExtractNonceError> {
    let data = account
        .data
        .decode()
        .ok_or(ExtractNonceError::UnsupportedEncodingFormat)?;
    let versions = bincode::deserialize::<Versions>(data.as_slice())
        .map_err(|e| ExtractNonceError::InvalidAccountData(e.to_string()))?;
    match versions.state() {
        State::Uninitialized => Err(ExtractNonceError::Uninitialized),
        State::Initialized(data) => Ok(data.blockhash()),
    }
}

/// Errors that might happen when calling the [`extract_durable_nonce`] method.
#[derive(Clone, Debug, PartialEq, Error)]
pub enum ExtractNonceError {
    /// The account data does not represent a valid nonce account.
    ///
    /// This can happen for example when trying to extract a durable nonce from a Solana account
    /// that is not a nonce account.
    #[error("Invalid account data: {0}")]
    InvalidAccountData(String),
    /// The account data is encoded in a format that is not supported. Currently, this
    /// only applies to account data encoded in `jsonParsed` format.
    #[error("Unsupported encoding format")]
    UnsupportedEncodingFormat,
    /// The nonce account exists but is not initialized.
    ///
    /// This can happen if the account was created but the
    /// [`InitializeNonceAccount`](https://github.com/solana-program/system/blob/960949f72057fa15f0a1faef9be84569aebef37d/interface/src/instruction.rs#L171)
    /// instruction was not used to initialize a nonce within the account.
    #[error("Nonce account is not initialized")]
    Uninitialized,
}
