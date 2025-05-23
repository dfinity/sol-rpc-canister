//! Simple types to create basic unit tests for the [`crate::SolRpcClient`].
//!
//! Types and methods for this module are only available for non-canister architecture (non `wasm32`).

use crate::{ClientBuilder, Runtime};
use async_trait::async_trait;
use candid::{utils::ArgumentEncoder, CandidType, Principal};
use ic_cdk::api::call::RejectionCode;
use serde::de::DeserializeOwned;
use sol_rpc_types::Account;
use solana_account_decoder_client_types::{UiAccountData, UiAccountEncoding};
use solana_pubkey::pubkey;

impl<R> ClientBuilder<R> {
    /// Change the runtime to return the same mocked response for both update and query calls.
    pub fn with_mocked_response<Out: CandidType>(
        self,
        mocked_response: Out,
    ) -> ClientBuilder<MockRuntime> {
        self.with_runtime(|_runtime| MockRuntime::same_response(mocked_response))
    }

    /// Change the runtime to return different mocked responses between update and query calls.
    pub fn with_mocked_responses<UpdateOut: CandidType, QueryOut: CandidType>(
        self,
        mocked_response_for_update_call: UpdateOut,
        mocked_response_for_query_call: QueryOut,
    ) -> ClientBuilder<MockRuntime> {
        self.with_runtime(|_runtime| {
            MockRuntime::new(
                mocked_response_for_update_call,
                mocked_response_for_query_call,
            )
        })
    }
}

/// A dummy implementation of [`Runtime`] that always return the same candid-encoded response.
///
/// Implement your own [`Runtime`] in case a more refined approach is needed.
pub struct MockRuntime {
    update_call_result: Vec<u8>,
    query_call_result: Vec<u8>,
}

impl MockRuntime {
    /// Create a new [`MockRuntime`] to always return the given parameter.
    pub fn same_response<Out: CandidType>(mocked_response: Out) -> Self {
        let result = candid::encode_args((&mocked_response,))
            .expect("Failed to encode Candid mocked response");
        Self {
            update_call_result: result.clone(),
            query_call_result: result,
        }
    }

    /// Create a new [`MockRuntime`] to always return the given parameters.
    pub fn new<UpdateOut: CandidType, QueryOut: CandidType>(
        mocked_update_result: UpdateOut,
        mocked_query_result: QueryOut,
    ) -> Self {
        let update_call_result = candid::encode_args((&mocked_update_result,))
            .expect("Failed to encode Candid mocked response");
        let query_call_result = candid::encode_args((&mocked_query_result,))
            .expect("Failed to encode Candid mocked response");
        Self {
            update_call_result,
            query_call_result,
        }
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
        Ok(candid::decode_args(&self.update_call_result)
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
        Ok(candid::decode_args(&self.query_call_result)
            .map(|(r,)| r)
            .expect("Failed to decode Candid mocked response"))
    }
}

/// USDC token account [`EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`](https://solscan.io/token/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v) on Solana Mainnet.
pub fn usdc_account() -> Account {
    Account {
        lamports: 388_127_047_454,
        data: UiAccountData::Binary(
            "KLUv/QBYkQIAAQAAAJj+huiNm+Lqi8HMpIeLKYjCQPUrhCS/tA7Rot3LXhmbQLUAvmbxIwAGAQEAAABicKqKWcWUBbRShshncubNEm6bil06OFNtN/e0FOi2Zw==".to_string(),
            UiAccountEncoding::Base64Zstd,
        ).decode().unwrap(),
        owner: pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").into(),
        executable: false,
        rent_epoch: 18_446_744_073_709_551_615,
    }
}
