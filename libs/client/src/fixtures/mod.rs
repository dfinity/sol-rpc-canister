//! Simple types to create basic unit tests for the [`crate::SolRpcClient`].

use crate::{ClientBuilder, Runtime};
use async_trait::async_trait;
use candid::utils::ArgumentEncoder;
use candid::{CandidType, Principal};
use ic_cdk::api::call::RejectionCode;
use serde::de::DeserializeOwned;

impl<R> ClientBuilder<R> {
    /// Change the runtime to return a mocked response.
    pub fn with_mocked_response<Out: CandidType>(
        self,
        mocked_response: Out,
    ) -> ClientBuilder<MockRuntime> {
        self.with_runtime(|_runtime| MockRuntime::new(mocked_response))
    }
}

/// A dummy implementation of [`Runtime`] that always return the same response.
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
