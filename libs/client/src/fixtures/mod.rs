//! Simple types to create basic unit tests for the [`crate::SolRpcClient`].
//!
//! Types and methods for this module are only available for non-canister architecture (non `wasm32`).

use crate::{ClientBuilder, Runtime};
use async_trait::async_trait;
use candid::utils::ArgumentEncoder;
use candid::{CandidType, Principal};
use ic_cdk::api::call::RejectionCode;
use serde::de::DeserializeOwned;
use sol_rpc_types::{AccountData, AccountEncoding, AccountInfo};

impl<R> ClientBuilder<R> {
    /// Change the runtime to return a mocked response.
    pub fn with_mocked_response<Out: CandidType>(
        self,
        mocked_response: Out,
    ) -> ClientBuilder<MockRuntime> {
        self.with_runtime(|_runtime| MockRuntime::new(mocked_response))
    }
}

/// A dummy implementation of [`Runtime`] that always return the same candid-encoded response.
///
/// Implement your own [`Runtime`] in case a more refined approach is needed.
pub struct MockRuntime(Vec<u8>);

impl MockRuntime {
    /// Create a new [`MockRuntime`] to always return the given parameter.
    pub fn new<Out: CandidType>(mocked_response: Out) -> Self {
        Self(
            candid::encode_args((&mocked_response,))
                .expect("Failed to encode Candid mocked response"),
        )
    }
}

#[async_trait]
impl Runtime for MockRuntime {
    async fn update_call<In, Out>(
        &self,
        _id: Principal,
        _method: &str,
        _args: In,
        _cycles: u128,
    ) -> Result<Out, (RejectionCode, String)>
    where
        In: ArgumentEncoder + Send,
        Out: CandidType + DeserializeOwned,
    {
        Ok(candid::decode_args(&self.0)
            .map(|(r,)| r)
            .expect("Failed to decode Candid mocked response"))
    }

    async fn query_call<In, Out>(
        &self,
        _id: Principal,
        _method: &str,
        _args: In,
    ) -> Result<Out, (RejectionCode, String)>
    where
        In: ArgumentEncoder + Send,
        Out: CandidType + DeserializeOwned,
    {
        Ok(candid::decode_args(&self.0)
            .map(|(r,)| r)
            .expect("Failed to decode Candid mocked response"))
    }
}

/// USDC token account [`EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`](https://solscan.io/token/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v) on Solana Mainnet.
pub fn usdc_account() -> AccountInfo {
    AccountInfo {
        lamports: 388_127_047_454,
        data: AccountData::Binary(
            "KLUv/QBYkQIAAQAAAJj+huiNm+Lqi8HMpIeLKYjCQPUrhCS/tA7Rot3LXhmbQLUAvmbxIwAGAQEAAABicKqKWcWUBbRShshncubNEm6bil06OFNtN/e0FOi2Zw==".to_string(),
            AccountEncoding::Base64Zstd,
        ),
        owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
        executable: false,
        rent_epoch: 18_446_744_073_709_551_615,
        space: 82,
    }
}
