pub mod json;
mod sol_rpc;
#[cfg(test)]
mod tests;

use crate::{
    http::{
        errors::HttpClientError, http_client, service_request_builder, ChargingPolicyWithCollateral,
    },
    memory::{read_state, State},
    metrics::MetricRpcMethod,
    providers::{request_builder, resolve_rpc_provider, Providers},
    rpc_client::sol_rpc::ResponseTransform,
    types::RoundingError,
};
use canhttp::{
    http::json::JsonRpcRequest,
    multi::{MultiResults, Reduce, ReduceWithEquality, ReduceWithThreshold},
    CyclesChargingPolicy, CyclesCostEstimator, MaxResponseBytesRequestExtension,
    TransformContextRequestExtension,
};
use http::{Request, Response};
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument as IcHttpRequest, TransformContext,
};
use serde::{de::DeserializeOwned, Serialize};
use sol_rpc_types::{
    ConsensusStrategy, GetSlotRpcConfig, ProviderError, RpcConfig, RpcError, RpcResult, RpcSource,
    RpcSources, TransactionDetails, Signature,
};
use solana_clock::Slot;
use std::{fmt::Debug, marker::PhantomData};
use tower::ServiceExt;

// This constant is our approximation of the expected header size.
// The HTTP standard doesn't define any limit, and many implementations limit
// the headers size to 8 KiB. We chose a lower limit because headers observed on most providers
// fit in the constant defined below, and if there is a spike, then the payload size adjustment
// should take care of that.
pub const HEADER_SIZE_LIMIT: u64 = 2 * 1024;

pub struct MultiRpcRequest<Params, Output> {
    providers: Providers,
    request: JsonRpcRequest<Params>,
    max_response_bytes: u64,
    transform: ResponseTransform,
    reduction_strategy: ReductionStrategy,
    _marker: PhantomData<Output>,
}

impl<Params, Output> MultiRpcRequest<Params, Output> {
    fn new(
        providers: Providers,
        request: JsonRpcRequest<Params>,
        max_response_bytes: u64,
        transform: ResponseTransform,
        reduction_strategy: ReductionStrategy,
    ) -> Self {
        Self {
            providers,
            request,
            max_response_bytes,
            transform,
            reduction_strategy,
            _marker: PhantomData,
        }
    }
    pub fn method(&self) -> &str {
        self.request.method()
    }
}

impl<Params: Clone, Output> Clone for MultiRpcRequest<Params, Output> {
    fn clone(&self) -> Self {
        Self {
            providers: self.providers.clone(),
            request: self.request.clone(),
            max_response_bytes: self.max_response_bytes,
            transform: self.transform.clone(),
            reduction_strategy: self.reduction_strategy.clone(),
            _marker: self._marker,
        }
    }
}

pub type GetAccountInfoRequest = MultiRpcRequest<
    json::GetAccountInfoParams,
    Option<solana_account_decoder_client_types::UiAccount>,
>;

impl GetAccountInfoRequest {
    pub fn get_account_info<Params: Into<json::GetAccountInfoParams>>(
        rpc_sources: RpcSources,
        config: RpcConfig,
        params: Params,
    ) -> Result<Self, ProviderError> {
        let consensus_strategy = config.response_consensus.unwrap_or_default();
        let providers = Providers::new(rpc_sources, consensus_strategy.clone())?;
        let max_response_bytes = config
            .response_size_estimate
            .unwrap_or(1024 + HEADER_SIZE_LIMIT);

        Ok(MultiRpcRequest::new(
            providers,
            JsonRpcRequest::new("getAccountInfo", params.into()),
            max_response_bytes,
            ResponseTransform::GetAccountInfo,
            ReductionStrategy::from(consensus_strategy),
        ))
    }
}

pub type GetBlockRequest = MultiRpcRequest<
    json::GetBlockParams,
    Option<solana_transaction_status_client_types::UiConfirmedBlock>,
>;

impl GetBlockRequest {
    pub fn get_block<Params: Into<json::GetBlockParams>>(
        rpc_sources: RpcSources,
        config: RpcConfig,
        params: Params,
    ) -> Result<Self, ProviderError> {
        let params = params.into();
        let consensus_strategy = config.response_consensus.unwrap_or_default();
        let providers = Providers::new(rpc_sources, consensus_strategy.clone())?;
        let max_response_bytes = config.response_size_estimate.unwrap_or(
            match params.get_transaction_details() {
                None | Some(TransactionDetails::None) => 1024,
                Some(TransactionDetails::Signatures) => 512 * 1024,
            } + HEADER_SIZE_LIMIT,
        );

        Ok(MultiRpcRequest::new(
            providers,
            JsonRpcRequest::new("getBlock", params),
            max_response_bytes,
            ResponseTransform::GetBlock,
            ReductionStrategy::from(consensus_strategy),
        ))
    }
}

pub type GetSlotRequest = MultiRpcRequest<json::GetSlotParams, Slot>;

impl GetSlotRequest {
    pub fn get_slot<Params: Into<json::GetSlotParams>>(
        rpc_sources: RpcSources,
        config: GetSlotRpcConfig,
        params: Params,
    ) -> Result<Self, ProviderError> {
        let consensus_strategy = config.response_consensus.unwrap_or_default();
        let providers = Providers::new(rpc_sources, consensus_strategy.clone())?;
        let max_response_bytes = config
            .response_size_estimate
            .unwrap_or(1024 + HEADER_SIZE_LIMIT);
        let rounding_error = config
            .rounding_error
            .map(RoundingError::from)
            .unwrap_or_default();

        Ok(MultiRpcRequest::new(
            providers,
            JsonRpcRequest::new("getSlot", params.into()),
            max_response_bytes,
            ResponseTransform::GetSlot(rounding_error),
            ReductionStrategy::from(consensus_strategy),
        ))
    }
}

pub type GetTransactionRequest = MultiRpcRequest<
    json::GetTransactionParams,
    Option<solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta>,
>;

impl GetTransactionRequest {
    pub fn get_transaction<Params: Into<json::GetTransactionParams>>(
        rpc_sources: RpcSources,
        config: RpcConfig,
        params: Params,
    ) -> Result<Self, ProviderError> {
        let consensus_strategy = config.response_consensus.unwrap_or_default();
        let providers = Providers::new(rpc_sources, consensus_strategy.clone())?;
        let max_response_bytes = config
            .response_size_estimate
            // TODO XC-343: Revisit this when we add support for more values of `encoding`
            .unwrap_or(10 * 1024 + HEADER_SIZE_LIMIT);

        Ok(MultiRpcRequest::new(
            providers,
            JsonRpcRequest::new("getTransaction", params.into()),
            max_response_bytes,
            ResponseTransform::GetBlock,
            ReductionStrategy::from(consensus_strategy),
        ))
    }
}

pub type SendTransactionRequest = MultiRpcRequest<json::SendTransactionParams, Signature>;

impl SendTransactionRequest {
    pub fn send_transaction<Params: Into<json::SendTransactionParams>>(
        rpc_sources: RpcSources,
        config: RpcConfig,
        params: Params,
    ) -> Result<Self, ProviderError> {
        let consensus_strategy = config.response_consensus.unwrap_or_default();
        let providers = Providers::new(rpc_sources, consensus_strategy.clone())?;
        let max_response_bytes = config
            .response_size_estimate
            .unwrap_or(128 + HEADER_SIZE_LIMIT);

        Ok(MultiRpcRequest::new(
            providers,
            JsonRpcRequest::new("sendTransaction", params.into()),
            max_response_bytes,
            ResponseTransform::SendTransaction,
            ReductionStrategy::from(consensus_strategy),
        ))
    }
}

pub type JsonRequest = MultiRpcRequest<serde_json::Value, serde_json::Value>;

impl JsonRequest {
    pub fn json_request(
        rpc_sources: RpcSources,
        config: RpcConfig,
        json_rpc_payload: String,
    ) -> RpcResult<Self> {
        let request: JsonRpcRequest<serde_json::Value> =
            match serde_json::from_str(&json_rpc_payload) {
                Ok(req) => req,
                Err(e) => {
                    return Err(RpcError::ValidationError(format!(
                        "Invalid JSON RPC request: {e}"
                    )))
                }
            };
        let consensus_strategy = config.response_consensus.unwrap_or_default();
        let providers = Providers::new(rpc_sources, consensus_strategy.clone())?;
        let max_response_bytes = config
            .response_size_estimate
            .unwrap_or(1024 + HEADER_SIZE_LIMIT);

        Ok(MultiRpcRequest::new(
            providers,
            request,
            max_response_bytes,
            ResponseTransform::Raw,
            ReductionStrategy::from(consensus_strategy),
        ))
    }
}

impl<Params, Output> MultiRpcRequest<Params, Output> {
    pub async fn send_and_reduce(self) -> ReducedResult<Output>
    where
        Params: Serialize + Clone + Debug,
        Output: Debug + DeserializeOwned + PartialEq + Serialize,
    {
        let strategy = self.reduction_strategy.clone();
        self.parallel_call().await.reduce(strategy)
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
    async fn parallel_call(self) -> MultiCallResults<Output>
    where
        Params: Serialize + Clone + Debug,
        Output: Debug + DeserializeOwned,
    {
        let num_providers = self.providers.sources.len();
        let rpc_method = MetricRpcMethod::from(self.request.method().to_string());
        let requests = self.create_json_rpc_requests();

        let client = http_client(rpc_method, true);

        let (requests, errors) = requests.into_inner();
        let (_client, mut results) = canhttp::multi::parallel_call(client, requests).await;
        results.add_errors(errors);
        assert_eq!(
            results.len(),
            num_providers,
            "BUG: expected 1 result per provider"
        );
        results
    }

    /// Estimate the exact cycles cost for the given request.
    ///
    /// *IMPORTANT*: the method is *synchronous* in a canister environment.
    pub async fn cycles_cost(self) -> RpcResult<u128>
    where
        Params: Serialize + Clone + Debug,
    {
        async fn extract_request(
            request: IcHttpRequest,
        ) -> Result<http::Response<IcHttpRequest>, HttpClientError> {
            Ok(http::Response::new(request))
        }

        let num_providers = self.providers.sources.len();
        let requests = self.create_json_rpc_requests();

        let client = service_request_builder()
            .service_fn(extract_request)
            .map_err(RpcError::from)
            .map_response(Response::into_body);

        let (requests, errors) = requests.into_inner();
        if let Some(error) = errors.into_values().next() {
            return Err(error);
        }

        let (_client, results) = canhttp::multi::parallel_call(client, requests).await;
        let (requests, errors) = results.into_inner();
        if !errors.is_empty() {
            return Err(errors
                .into_values()
                .next()
                .expect("BUG: errors is not empty"));
        }
        assert_eq!(
            requests.len(),
            num_providers,
            "BUG: expected 1 result per provider"
        );

        let mut cycles_to_attach = 0_u128;
        let estimator = CyclesCostEstimator::new(read_state(State::get_num_subnet_nodes));
        let policy = ChargingPolicyWithCollateral::default();
        for request in requests.into_values() {
            cycles_to_attach +=
                policy.cycles_to_charge(&request, estimator.cost_of_http_request(&request));
        }
        Ok(cycles_to_attach)
    }

    fn create_json_rpc_requests(self) -> MultiCallResults<Request<JsonRpcRequest<Params>>>
    where
        Params: Clone,
    {
        let transform_op = {
            let mut buf = vec![];
            minicbor::encode(&self.transform, &mut buf).unwrap();
            buf
        };
        let mut requests = MultiResults::default();
        for provider in self.providers.sources {
            let request = request_builder(
                resolve_rpc_provider(provider.clone()),
                &read_state(|state| state.get_override_provider()),
            )
            .map(|builder| {
                builder
                    .max_response_bytes(self.max_response_bytes)
                    .transform_context(TransformContext::from_name(
                        "cleanup_response".to_owned(),
                        transform_op.clone(),
                    ))
                    .body(self.request.clone())
                    .expect("BUG: invalid request")
            });
            requests.insert_once(provider.clone(), request);
        }
        requests
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
