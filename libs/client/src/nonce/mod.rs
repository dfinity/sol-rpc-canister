//! Module for interacting with Solana [nonce accounts](https://solana.com/de/developers/guides/advanced/introduction-to-durable-nonces#nonce-account).

use derive_more::From;
use solana_account::Account;
use solana_account_decoder_client_types::UiAccount;
use solana_hash::Hash;
use solana_rpc_client_nonce_utils::data_from_account;
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
/// assert_matches!(durable_nonce, Err(ExtractNonceError::DurableNonceError(_)));
/// # Ok(())
/// # }
/// ```
pub fn extract_durable_nonce(account: &UiAccount) -> Result<Hash, ExtractNonceError> {
    let account_data = account
        .decode::<Account>()
        .as_ref()
        .map(data_from_account)
        .ok_or(ExtractNonceError::AccountDecodingError)??;
    Ok(account_data.blockhash())
}

/// Errors that might happen when calling the [`extract_durable_nonce`] method.
#[derive(Debug, PartialEq, Error, From)]
pub enum ExtractNonceError {
    /// An error occurred while decoding the account. This error might happen for example
    /// if the account data is encoded in a format that is not supported such as `json`.
    #[error("Error while decoding account data")]
    AccountDecodingError,
    /// An error occurred while trying to read the durable nonce value from the account.
    /// This can happen, for example, if the provided account is not a valid nonce account.
    /// Refer to [`solana_rpc_client_nonce_utils::Error`] for more details on possible
    /// errors.
    #[error("Error while extracting durable nonce from account data: {0}")]
    DurableNonceError(solana_rpc_client_nonce_utils::Error),
}
