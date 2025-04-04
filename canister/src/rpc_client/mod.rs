mod sol_rpc;

use crate::http::errors::HttpClientError;
use crate::http::{service_request_builder, ChargingPolicyWithCollateral};
use crate::memory::State;
use crate::{
    http::http_client,
    memory::read_state,
    metrics::MetricRpcMethod,
    providers::{request_builder, resolve_rpc_provider, Providers},
    rpc_client::sol_rpc::{ResponseSizeEstimate, ResponseTransform, HEADER_SIZE_LIMIT},
    types::RoundingError,
};
use canhttp::{
    http::json::JsonRpcRequest,
    multi::{MultiResults, Reduce, ReduceWithEquality, ReduceWithThreshold},
    CyclesChargingPolicy, CyclesCostEstimator, MaxResponseBytesRequestExtension,
    TransformContextRequestExtension,
};
use http::Request;
use ic_cdk::api::management_canister::http_request::CanisterHttpRequestArgument as IcHttpRequest;
use ic_cdk::api::management_canister::http_request::TransformContext;
use serde::{de::DeserializeOwned, Serialize};
use sol_rpc_types::{
    ConsensusStrategy, GetSlotParams, GetSlotRpcConfig, ProviderError, RpcConfig, RpcError,
    RpcResult, RpcSource, RpcSources,
};
use solana_clock::Slot;
use std::marker::PhantomData;
use std::{collections::BTreeSet, fmt::Debug};
use tower::ServiceExt;

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

pub type GetSlotRequest = MultiRpcRequest<Vec<GetSlotParams>, Slot>;

impl GetSlotRequest {
    pub fn get_slot(
        rpc_sources: RpcSources,
        config: GetSlotRpcConfig,
        params: GetSlotParams,
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
            JsonRpcRequest::new("getSlot", vec![params]),
            max_response_bytes,
            ResponseTransform::GetSlot(rounding_error),
            ReductionStrategy::from(consensus_strategy),
        ))
    }
}

pub type RawRequest = MultiRpcRequest<serde_json::Value, serde_json::Value>;

impl RawRequest {
    pub fn raw_request(
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
            .map_err(|e: HttpClientError| RpcError::from(e))
            .map_response(|r| r.into_body());

        let (requests, errors) = requests.into_inner();
        if !errors.is_empty() {
            return Err(errors
                .into_values()
                .next()
                .expect("BUG: errors is not empty"));
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
pub struct SolRpcClient {
    providers: Providers,
    config: RpcConfig,
    rounding_error: RoundingError,
}

impl SolRpcClient {
    pub fn new(
        source: RpcSources,
        config: Option<RpcConfig>,
        rounding_error: Option<RoundingError>,
    ) -> Result<Self, ProviderError> {
        let config = config.unwrap_or_default();
        let rounding_error = rounding_error.unwrap_or_default();
        let strategy = config.response_consensus.clone().unwrap_or_default();
        Ok(Self {
            providers: Providers::new(source, strategy)?,
            config,
            rounding_error,
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
        let request_body = JsonRpcRequest::new(method, params);
        let rpc_method = MetricRpcMethod::from(request_body.method().to_string());
        let requests =
            self.create_json_rpc_requests(request_body, response_size_estimate, response_transform);

        let client = http_client(rpc_method, true);

        let (requests, errors) = requests.into_inner();
        let (_client, mut results) = canhttp::multi::parallel_call(client, requests).await;
        results.add_errors(errors);
        assert_eq!(
            results.len(),
            self.providers().len(),
            "BUG: expected 1 result per provider"
        );
        results
    }

    async fn create_requests<I>(
        &self,
        method: impl Into<String>,
        params: I,
        response_size_estimate: ResponseSizeEstimate,
        response_transform: &Option<ResponseTransform>,
    ) -> MultiCallResults<IcHttpRequest>
    where
        I: Serialize + Clone + Debug,
    {
        async fn extract_request(
            request: IcHttpRequest,
        ) -> Result<http::Response<IcHttpRequest>, HttpClientError> {
            Ok(http::Response::new(request))
        }

        let request_body = JsonRpcRequest::new(method, params);
        let requests =
            self.create_json_rpc_requests(request_body, response_size_estimate, response_transform);

        let client = service_request_builder()
            .service_fn(extract_request)
            .map_err(|e: HttpClientError| RpcError::from(e))
            .map_response(|r| r.into_body());

        let (requests, errors) = requests.into_inner();
        let (_client, mut results) = canhttp::multi::parallel_call(client, requests).await;
        results.add_errors(errors);
        assert_eq!(
            results.len(),
            self.providers().len(),
            "BUG: expected 1 result per provider"
        );
        results
    }

    fn create_json_rpc_requests<I>(
        &self,
        request_body: JsonRpcRequest<I>,
        response_size_estimate: ResponseSizeEstimate,
        response_transform: &Option<ResponseTransform>,
    ) -> MultiCallResults<Request<JsonRpcRequest<I>>>
    where
        I: Clone,
    {
        let providers = self.providers();
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
        requests
    }

    /// Query the Solana [`getSlot`](https://solana.com/docs/rpc/http/getslot) RPC method.
    pub async fn get_slot(&self, params: GetSlotParams) -> ReducedResult<Slot> {
        self.parallel_call(
            "getSlot",
            vec![params],
            self.response_size_estimate(1024 + HEADER_SIZE_LIMIT),
            &Some(ResponseTransform::GetSlot(self.rounding_error)),
        )
        .await
        .reduce(self.reduction_strategy())
    }

    pub async fn get_slot_request_cost(&self, params: GetSlotParams) -> RpcResult<u128> {
        self.cycles_cost(
            self.create_requests(
                "getSlot",
                vec![params],
                self.response_size_estimate(1024 + HEADER_SIZE_LIMIT),
                &Some(ResponseTransform::GetSlot(self.rounding_error)),
            )
            .await,
        )
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

    fn cycles_cost(&self, requests: MultiCallResults<IcHttpRequest>) -> RpcResult<u128> {
        let (requests, errors) = requests.into_inner();
        if !errors.is_empty() {
            return Err(errors
                .into_values()
                .next()
                .expect("BUG: errors is not empty"));
        }
        let mut cycles_to_attach = 0_u128;
        let estimator = CyclesCostEstimator::new(read_state(State::get_num_subnet_nodes));
        let policy = ChargingPolicyWithCollateral::default();
        for request in requests.into_values() {
            cycles_to_attach +=
                policy.cycles_to_charge(&request, estimator.cost_of_http_request(&request));
        }
        Ok(cycles_to_attach)
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
