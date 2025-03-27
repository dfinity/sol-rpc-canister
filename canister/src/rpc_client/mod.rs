mod sol_rpc;
#[cfg(test)]
mod tests;

use crate::{
    http::http_client,
    logs::Priority,
    memory::read_state,
    metrics::MetricRpcMethod,
    providers::{request_builder, resolve_rpc_provider, Providers},
    rpc_client::sol_rpc::{ResponseSizeEstimate, ResponseTransform, HEADER_SIZE_LIMIT},
};
use canhttp::{
    http::json::JsonRpcRequest,
    multi::{MultiResults, Reduce, ReduceWithEquality, ReduceWithThreshold},
    MaxResponseBytesRequestExtension, TransformContextRequestExtension,
};
use canlog::log;
use ic_cdk::api::management_canister::http_request::TransformContext;
use serde::{de::DeserializeOwned, Serialize};
use sol_rpc_types::{
    ConsensusStrategy, GetSlotParams, JsonRpcError, ProviderError, RpcConfig, RpcError, RpcSource,
    RpcSources,
};
use solana_clock::Slot;
use std::{collections::BTreeSet, fmt::Debug};
use tower::ServiceExt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SolRpcClient {
    providers: Providers,
    config: RpcConfig,
}

impl SolRpcClient {
    pub fn new(source: RpcSources, config: Option<RpcConfig>) -> Result<Self, ProviderError> {
        let config = config.unwrap_or_default();
        let strategy = config.response_consensus.clone().unwrap_or_default();
        Ok(Self {
            providers: Providers::new(source, strategy)?,
            config,
        })
    }

    fn providers(&self) -> &BTreeSet<RpcSource> {
        &self.providers.sources
    }

    fn response_size_estimate(&self, estimate: u64) -> ResponseSizeEstimate {
        ResponseSizeEstimate::new(self.config.response_size_estimate.unwrap_or(estimate))
    }

    fn reduction_strategy(&self) -> ReductionStrategy {
        ReductionStrategy::from(
            self.config
                .response_consensus
                .as_ref()
                .cloned()
                .unwrap_or_default(),
        )
    }

    /// Query all providers in parallel and return all results.
    /// It's up to the caller to decide how to handle the results, which could be inconsistent
    /// (e.g., if different providers gave different responses).
    /// This method is useful for querying data that is critical for the system to ensure that
    /// there is no single point of failure.
    /// Query all providers in parallel and return all results.
    /// It's up to the caller to decide how to handle the results, which could be inconsistent
    /// (e.g., if different providers gave different responses).
    /// This method is useful for querying data that is critical for the system to ensure that there is no single point of failure,
    /// e.g., ethereum logs upon which ckETH will be minted.
    async fn parallel_call<I, O>(
        &self,
        method: impl Into<String>,
        params: I,
        response_size_estimate: ResponseSizeEstimate,
        response_transform: &Option<ResponseTransform>,
    ) -> MultiCallResults<O>
    where
        I: Serialize + Clone + Debug,
        O: Debug + DeserializeOwned,
    {
        let providers = self.providers();
        let request_body = JsonRpcRequest::new(method, params);
        let effective_size_estimate = response_size_estimate.get();
        let transform_op = response_transform
            .as_ref()
            .map(|t| {
                let mut buf = vec![];
                minicbor::encode(t, &mut buf).unwrap();
                buf
            })
            .unwrap_or_default();
        let mut requests = MultiResults::default();
        for provider in providers {
            log!(
                Priority::Debug,
                "[parallel_call]: will call provider: {:?}",
                provider
            );
            let request = request_builder(
                resolve_rpc_provider(provider.clone()),
                &read_state(|state| state.get_override_provider()),
            )
            .map(|builder| {
                builder
                    .max_response_bytes(effective_size_estimate)
                    .transform_context(TransformContext::from_name(
                        "cleanup_response".to_owned(),
                        transform_op.clone(),
                    ))
                    .body(request_body.clone())
                    .expect("BUG: invalid request")
            });
            requests.insert_once(provider.clone(), request);
        }

        let rpc_method = MetricRpcMethod::from(request_body.method().to_string());
        let client =
            http_client(rpc_method, true).map_result(|r| match r?.into_body().into_result() {
                Ok(value) => Ok(value),
                Err(json_rpc_error) => Err(RpcError::JsonRpcError(JsonRpcError {
                    code: json_rpc_error.code,
                    message: json_rpc_error.message,
                })),
            });

        let (requests, errors) = requests.into_inner();
        let (_client, mut results) = canhttp::multi::parallel_call(client, requests).await;
        results.add_errors(errors);
        assert_eq!(
            results.len(),
            providers.len(),
            "BUG: expected 1 result per provider"
        );
        results
    }

    /// Query the Solana [`getSlot`](https://solana.com/docs/rpc/http/getslot) RPC method.
    pub async fn get_slot(&self, params: GetSlotParams) -> ReducedResult<Slot> {
        self.parallel_call(
            "getSlot",
            vec![params],
            self.response_size_estimate(1024 + HEADER_SIZE_LIMIT),
            &Some(ResponseTransform::GetSlot),
        )
        .await
        .reduce(self.reduction_strategy())
    }

    pub async fn raw_request<I>(
        &self,
        request: JsonRpcRequest<I>,
    ) -> ReducedResult<serde_json::Value>
    where
        I: Serialize + Clone + Debug,
    {
        self.parallel_call(
            request.method(),
            request.params(),
            self.response_size_estimate(1024 + HEADER_SIZE_LIMIT),
            &Some(ResponseTransform::Raw),
        )
        .await
        .reduce(self.reduction_strategy())
    }
}

pub enum ReductionStrategy {
    ByEquality(ReduceWithEquality),
    ByThreshold(ReduceWithThreshold),
}

impl From<ConsensusStrategy> for ReductionStrategy {
    fn from(value: ConsensusStrategy) -> Self {
        match value {
            ConsensusStrategy::Equality => ReductionStrategy::ByEquality(ReduceWithEquality),
            ConsensusStrategy::Threshold { total: _, min } => {
                ReductionStrategy::ByThreshold(ReduceWithThreshold::new(min))
            }
        }
    }
}

impl<T: PartialEq + Serialize> Reduce<RpcSource, T, RpcError> for ReductionStrategy {
    fn reduce(&self, results: MultiResults<RpcSource, T, RpcError>) -> ReducedResult<T> {
        match self {
            ReductionStrategy::ByEquality(r) => r.reduce(results),
            ReductionStrategy::ByThreshold(r) => r.reduce(results),
        }
    }
}

pub type MultiCallResults<T> = MultiResults<RpcSource, T, RpcError>;
pub type ReducedResult<T> = canhttp::multi::ReducedResult<RpcSource, T, RpcError>;
