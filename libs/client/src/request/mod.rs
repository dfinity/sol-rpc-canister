use crate::{Runtime, SolRpcClient};
use candid::CandidType;
use serde::de::DeserializeOwned;
use sol_rpc_types::{GetSlotParams, GetSlotRpcConfig, RpcConfig, RpcResult, RpcSources};
use solana_clock::Slot;

/// Solana RPC endpoint supported by the SOL RPC canister.
pub trait SolRpcRequest {
    /// Type of RPC config for that request.
    type Config;
    /// The type of parameters taken by this endpoint.
    type Params;
    /// The type returned by this endpoint.
    type Output;

    /// The name of the endpoint on the SOL RPC canister.
    fn endpoint(&self) -> SolRpcEndpoint;

    /// Return the request parameters.
    fn params(self) -> Self::Params;
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum SolRpcEndpoint {
    GetSlot,
    JsonRequest,
}

impl SolRpcEndpoint {
    pub fn rpc_method(&self) -> &'static str {
        match &self {
            SolRpcEndpoint::GetSlot => "getSlot",
            SolRpcEndpoint::JsonRequest => "jsonRequest",
        }
    }

    pub fn cycles_cost_method(&self) -> &'static str {
        match &self {
            SolRpcEndpoint::GetSlot => "getSlotCyclesCost",
            SolRpcEndpoint::JsonRequest => "jsonRequestCyclesCost",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GetSlotRequest(Option<GetSlotParams>);

impl SolRpcRequest for GetSlotRequest {
    type Config = GetSlotRpcConfig;
    type Params = Option<GetSlotParams>;
    type Output = sol_rpc_types::MultiRpcResult<Slot>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::GetSlot
    }

    fn params(self) -> Self::Params {
        self.0
    }
}

pub struct RawJsonRequest(String);

impl TryFrom<serde_json::Value> for RawJsonRequest {
    type Error = String;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::to_string(&value)
            .map(RawJsonRequest)
            .map_err(|e| e.to_string())
    }
}

impl SolRpcRequest for RawJsonRequest {
    type Config = RpcConfig;
    type Params = String;
    type Output = sol_rpc_types::MultiRpcResult<String>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::JsonRequest
    }

    fn params(self) -> Self::Params {
        self.0
    }
}

/// A builder to construct a [`Request`].
///
/// To construct a [`RequestBuilder`], refer to the [`SolRpcClient`] documentation.
#[must_use = "RequestBuilder does nothing until you 'send' it"]
pub struct RequestBuilder<Runtime, Config, Params, Output> {
    client: SolRpcClient<Runtime>,
    request: Request<Config, Params, Output>,
}

impl<Runtime, Config: Clone, Params: Clone, Output> Clone
    for RequestBuilder<Runtime, Config, Params, Output>
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            request: self.request.clone(),
        }
    }
}

impl<Runtime, Config, Params, Output> RequestBuilder<Runtime, Config, Params, Output> {
    pub(super) fn new<RpcRequest>(
        client: SolRpcClient<Runtime>,
        rpc_request: RpcRequest,
        cycles: u128,
    ) -> Self
    where
        RpcRequest: SolRpcRequest<Config = Config, Params = Params, Output = Output>,
        Config: From<RpcConfig>,
    {
        let request = Request {
            endpoint: rpc_request.endpoint(),
            rpc_sources: client.config.rpc_sources.clone(),
            rpc_config: client.config.rpc_config.clone().map(Config::from),
            params: rpc_request.params(),
            cycles,
            _marker: Default::default(),
        };
        RequestBuilder { client, request }
    }

    /// Query the cycles cost for that request
    pub fn request_cost(self) -> RequestCostBuilder<Runtime, Config, Params> {
        RequestCostBuilder {
            client: self.client,
            request: RequestCost {
                endpoint: self.request.endpoint,
                rpc_sources: self.request.rpc_sources,
                rpc_config: self.request.rpc_config,
                params: self.request.params,
                cycles: 0,
                _marker: Default::default(),
            },
        }
    }

    /// Change the amount of cycles to send for that request.
    pub fn with_cycles(mut self, cycles: u128) -> Self {
        *self.request.cycles_mut() = cycles;
        self
    }

    /// Change the parameters to send for that request.
    pub fn with_params(mut self, params: impl Into<Params>) -> Self {
        *self.request.params_mut() = params.into();
        self
    }
}

impl<R: Runtime, Config, Params, Output> RequestBuilder<R, Config, Params, Output> {
    /// Constructs the [`Request`] and send it using the [`SolRpcClient`].
    pub async fn send(self) -> Output
    where
        Config: CandidType + Send,
        Params: CandidType + Send,
        Output: CandidType + DeserializeOwned,
    {
        self.client.execute_request(self.request).await
    }
}

impl<Runtime, Params, Output> RequestBuilder<Runtime, GetSlotRpcConfig, Params, Output> {
    /// Change the rounding error for `getSlot` request.
    pub fn with_rounding_error(mut self, rounding_error: u64) -> Self {
        if let Some(config) = self.request.rpc_config_mut() {
            config.rounding_error = Some(rounding_error);
            return self;
        }
        let config = GetSlotRpcConfig {
            rounding_error: Some(rounding_error),
            ..Default::default()
        };
        *self.request.rpc_config_mut() = Some(config);
        self
    }
}

/// A request which can be executed with `SolRpcClient::execute_request` or `SolRpcClient::execute_query_request`.
pub struct Request<Config, Params, Output> {
    pub(super) endpoint: SolRpcEndpoint,
    pub(super) rpc_sources: RpcSources,
    pub(super) rpc_config: Option<Config>,
    pub(super) params: Params,
    pub(super) cycles: u128,
    pub(super) _marker: std::marker::PhantomData<Output>,
}

impl<Config: Clone, Params: Clone, Output> Clone for Request<Config, Params, Output> {
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            rpc_sources: self.rpc_sources.clone(),
            rpc_config: self.rpc_config.clone(),
            params: self.params.clone(),
            cycles: self.cycles,
            _marker: self._marker,
        }
    }
}

impl<Config, Params, Output> Request<Config, Params, Output> {
    /// Get a mutable reference to the cycles.
    #[inline]
    pub fn cycles_mut(&mut self) -> &mut u128 {
        &mut self.cycles
    }

    /// Get a mutable reference to the RPC configuration.
    #[inline]
    pub fn rpc_config_mut(&mut self) -> &mut Option<Config> {
        &mut self.rpc_config
    }

    /// Get a mutable reference to the request parameters.
    #[inline]
    pub fn params_mut(&mut self) -> &mut Params {
        &mut self.params
    }
}

pub type RequestCost<Config, Params> = Request<Config, Params, RpcResult<u128>>;

#[must_use = "RequestCostBuilder does nothing until you 'send' it"]
pub struct RequestCostBuilder<Runtime, Config, Params> {
    client: SolRpcClient<Runtime>,
    request: RequestCost<Config, Params>,
}

impl<R: Runtime, Config, Params> RequestCostBuilder<R, Config, Params> {
    /// Constructs the [`Request`] and send it using the [`SolRpcClient`].
    pub async fn send(self) -> RpcResult<u128>
    where
        Config: CandidType + Send,
        Params: CandidType + Send,
    {
        self.client.execute_cycles_cost_request(self.request).await
    }
}
