//! Simple types to create basic unit tests for the [`crate::SolRpcClient`].
//!
//! Types and methods for this module are only available for non-canister architecture (non `wasm32`).

use crate::{ClientBuilder, Runtime};
use async_trait::async_trait;
use candid::{utils::ArgumentEncoder, CandidType, Decode, Encode, Principal};
use ic_cdk::api::call::RejectionCode;
use serde::de::DeserializeOwned;
use sol_rpc_types::{AccountData, AccountEncoding, AccountInfo};
use std::collections::BTreeMap;

impl<R> ClientBuilder<R> {
    /// Set the runtime to a [`MockRuntime`].
    pub fn with_mocked_responses(self) -> ClientBuilder<MockRuntime> {
        self.with_runtime(|_runtime| MockRuntime::default())
    }

    /// Change the runtime to return the given mocked response for all calls.
    pub fn with_default_mocked_response<Out: CandidType>(
        self,
        mocked_response: Out,
    ) -> ClientBuilder<MockRuntime> {
        self.with_mocked_responses()
            .with_default_response(mocked_response)
    }
}

impl ClientBuilder<MockRuntime> {
    /// Change the runtime to return the given mocked response for all calls.
    pub fn with_default_response<Out: CandidType>(
        self,
        mocked_response: Out,
    ) -> ClientBuilder<MockRuntime> {
        self.with_runtime(|runtime| runtime.with_default_response(mocked_response))
    }

    /// Change the runtime to return the given mocked response for calls to the given method.
    pub fn with_response_for_method<Out: CandidType>(
        self,
        method_name: &str,
        mocked_response: Out,
    ) -> ClientBuilder<MockRuntime> {
        self.with_runtime(|runtime| runtime.with_response_for_method(method_name, mocked_response))
    }
}

/// A dummy implementation of [`Runtime`] that always returns the same candid-encoded response
/// for a given method.
///
/// Implement your own [`Runtime`] in case a more refined approach is needed.
pub struct MockRuntime {
    default_call_result: Option<Vec<u8>>,
    method_to_call_result_map: BTreeMap<String, Vec<u8>>,
}

impl MockRuntime {
    /// Create a new [`MockRuntime`] with the given default mocked response.
    pub fn new() -> Self {
        Self {
            default_call_result: None,
            method_to_call_result_map: BTreeMap::new(),
        }
    }

    /// Create a new [`MockRuntime`] with the given default mocked response.
    pub fn with_default_response<Out: CandidType>(mut self, mocked_response: Out) -> Self {
        let result = Encode!(&mocked_response).expect("Failed to encode Candid mocked response");
        self.default_call_result = Some(result);
        self
    }

    /// Modify a [`MockRuntime`] to return the given response for the given method
    pub fn with_response_for_method<Out: CandidType>(
        mut self,
        method: &str,
        mocked_response: Out,
    ) -> Self {
        self.method_to_call_result_map.insert(
            method.to_string(),
            Encode!(&mocked_response).expect("Failed to encode Candid mocked response"),
        );
        self
    }

    fn call<Out>(&self, method: &str) -> Result<Out, (RejectionCode, String)>
    where
        Out: CandidType + DeserializeOwned,
    {
        let bytes = self
            .method_to_call_result_map
            .get(method)
            .or(self.default_call_result.as_ref())
            .unwrap_or_else(|| panic!("No available call response value for method `{method}`"));
        Ok(Decode!(bytes, Out).expect("Failed to decode Candid mocked response"))
    }
}

impl Default for MockRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for MockRuntime {
    async fn update_call<In, Out>(
        &self,
        _id: Principal,
        method: &str,
        _args: In,
        _cycles: u128,
    ) -> Result<Out, (RejectionCode, String)>
    where
        In: ArgumentEncoder + Send,
        Out: CandidType + DeserializeOwned,
    {
        self.call(method)
    }

    async fn query_call<In, Out>(
        &self,
        _id: Principal,
        method: &str,
        _args: In,
    ) -> Result<Out, (RejectionCode, String)>
    where
        In: ArgumentEncoder + Send,
        Out: CandidType + DeserializeOwned,
    {
        self.call(method)
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

/// Nonce account [`8DedqKHx9ogFajbHtRnTM3pPr3MRyVKDtepEpUiaDXX`](https://explorer.solana.com/address/8DedqKHx9ogFajbHtRnTM3pPr3MRyVKDtepEpUiaDXX?cluster=devnet) on Solana Devnet.
pub fn nonce_account() -> AccountInfo {
    AccountInfo {
        lamports: 1_499_900,
        data: AccountData::Binary("AQAAAAEAAAA+ZK6at2Umwl1p39ifPkNAu66sw5w0AKkY72a19k0LVFBDMPwL0VO7EYlFDc0BAwVcV446FBr/cRWZCGdrPYW9iBMAAAAAAAA=".to_string(), AccountEncoding::Base64),
        owner: "11111111111111111111111111111111".to_string(),
        executable: false,
        rent_epoch: 18_446_744_073_709_551_615,
        space: 80,
    }
}
