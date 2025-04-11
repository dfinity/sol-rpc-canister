use crate::{Runtime, SolRpcClient};
use candid::CandidType;
use serde::de::DeserializeOwned;
use sol_rpc_types::{
    AccountInfo, GetAccountInfoParams, GetSlotParams, GetSlotRpcConfig, RpcConfig, RpcResult,
    RpcSources, SendTransactionParams, TransactionId,
};
use solana_clock::Slot;
use strum::EnumIter;

/// Solana RPC endpoint supported by the SOL RPC canister.
pub trait SolRpcRequest {
    /// Type of RPC config for that request.
    type Config;
    /// The type of parameters taken by this endpoint.
    type Params;
    /// The Candid type returned when executing this request which is then converted to [`Self::Output`].
    type CandidOutput;
    /// The type returned by this endpoint.
    type Output;

    /// The name of the endpoint on the SOL RPC canister.
    fn endpoint(&self) -> SolRpcEndpoint;

    /// Return the request parameters.
    fn params(self) -> Self::Params;
}

/// Endpoint on the SOL RPC canister triggering a call to Solana providers.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, EnumIter)]
pub enum SolRpcEndpoint {
    /// `getAccountInfo` endpoint.
    GetAccountInfo,
    /// `getSlot` endpoint.
    GetSlot,
    /// `jsonRequest` endpoint.
    JsonRequest,
    /// `sendTransaction` endpoint.
    SendTransaction,
}

impl SolRpcEndpoint {
    /// Method name on the SOL RPC canister
    pub fn rpc_method(&self) -> &'static str {
        match &self {
            SolRpcEndpoint::GetAccountInfo => "getAccountInfo",
            SolRpcEndpoint::GetSlot => "getSlot",
            SolRpcEndpoint::JsonRequest => "jsonRequest",
            SolRpcEndpoint::SendTransaction => "sendTransaction",
        }
    }

    /// Method name on the SOL RPC canister to estimate the amount of cycles for that request.
    pub fn cycles_cost_method(&self) -> &'static str {
        match &self {
            SolRpcEndpoint::GetAccountInfo => "getAccountInfoCyclesCost",
            SolRpcEndpoint::GetSlot => "getSlotCyclesCost",
            SolRpcEndpoint::JsonRequest => "jsonRequestCyclesCost",
            SolRpcEndpoint::SendTransaction => "sendTransactionCyclesCost",
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetAccountInfoRequest(GetAccountInfoParams);

impl GetAccountInfoRequest {
    pub fn new(params: GetAccountInfoParams) -> Self {
        Self(params)
    }
}

impl SolRpcRequest for GetAccountInfoRequest {
    type Config = RpcConfig;
    type Params = GetAccountInfoParams;
    type CandidOutput = sol_rpc_types::MultiRpcResult<Option<AccountInfo>>;
    type Output =
        sol_rpc_types::MultiRpcResult<Option<solana_account_decoder_client_types::UiAccount>>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::GetAccountInfo
    }

    fn params(self) -> Self::Params {
        self.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct GetSlotRequest(Option<GetSlotParams>);

impl SolRpcRequest for GetSlotRequest {
    type Config = GetSlotRpcConfig;
    type Params = Option<GetSlotParams>;
    type CandidOutput = Self::Output;
    type Output = sol_rpc_types::MultiRpcResult<Slot>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::GetSlot
    }

    fn params(self) -> Self::Params {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct SendTransactionRequest(SendTransactionParams);

impl SendTransactionRequest {
    pub fn new(params: SendTransactionParams) -> Self {
        Self(params)
    }
}

impl SolRpcRequest for SendTransactionRequest {
    type Config = RpcConfig;
    type Params = SendTransactionParams;
    type CandidOutput = sol_rpc_types::MultiRpcResult<TransactionId>;
    type Output = sol_rpc_types::MultiRpcResult<solana_signature::Signature>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::SendTransaction
    }

    fn params(self) -> Self::Params {
        self.0
    }
}

pub struct JsonRequest(String);

impl TryFrom<serde_json::Value> for JsonRequest {
    type Error = String;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::to_string(&value)
            .map(JsonRequest)
            .map_err(|e| e.to_string())
    }
}

impl SolRpcRequest for JsonRequest {
    type Config = RpcConfig;
    type Params = String;
    type CandidOutput = sol_rpc_types::MultiRpcResult<String>;
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
pub struct RequestBuilder<Runtime, Config, Params, CandidOutput, Output> {
    client: SolRpcClient<Runtime>,
    request: Request<Config, Params, CandidOutput, Output>,
}

impl<Runtime, Config: Clone, Params: Clone, CandidOutput, Output> Clone
    for RequestBuilder<Runtime, Config, Params, CandidOutput, Output>
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            request: self.request.clone(),
        }
    }
}

impl<Runtime, Config, Params, CandidOutput, Output>
    RequestBuilder<Runtime, Config, Params, CandidOutput, Output>
{
    pub(super) fn new<RpcRequest>(
        client: SolRpcClient<Runtime>,
        rpc_request: RpcRequest,
        cycles: u128,
    ) -> Self
    where
        RpcRequest: SolRpcRequest<
            Config = Config,
            Params = Params,
            CandidOutput = CandidOutput,
            Output = Output,
        >,
        Config: From<RpcConfig>,
    {
        let request = Request {
            endpoint: rpc_request.endpoint(),
            rpc_sources: client.config.rpc_sources.clone(),
            rpc_config: client.config.rpc_config.clone().map(Config::from),
            params: rpc_request.params(),
            cycles,
            _candid_marker: Default::default(),
            _output_marker: Default::default(),
        };
        RequestBuilder::<Runtime, Config, Params, CandidOutput, Output> { client, request }
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
                _candid_marker: Default::default(),
                _output_marker: Default::default(),
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

impl<R: Runtime, Config, Params, CandidOutput, Output>
    RequestBuilder<R, Config, Params, CandidOutput, Output>
{
    /// Constructs the [`Request`] and send it using the [`SolRpcClient`].
    pub async fn send(self) -> Output
    where
        Config: CandidType + Send,
        Params: CandidType + Send,
        CandidOutput: Into<Output> + CandidType + DeserializeOwned,
    {
        self.client
            .execute_request::<Config, Params, CandidOutput, Output>(self.request)
            .await
    }
}

impl<Runtime, Params, CandidOutput, Output>
    RequestBuilder<Runtime, GetSlotRpcConfig, Params, CandidOutput, Output>
{
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
pub struct Request<Config, Params, CandidOutput, Output> {
    pub(super) endpoint: SolRpcEndpoint,
    pub(super) rpc_sources: RpcSources,
    pub(super) rpc_config: Option<Config>,
    pub(super) params: Params,
    pub(super) cycles: u128,
    pub(super) _candid_marker: std::marker::PhantomData<CandidOutput>,
    pub(super) _output_marker: std::marker::PhantomData<Output>,
}

impl<Config: Clone, Params: Clone, CandidOutput, Output> Clone
    for Request<Config, Params, CandidOutput, Output>
{
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            rpc_sources: self.rpc_sources.clone(),
            rpc_config: self.rpc_config.clone(),
            params: self.params.clone(),
            cycles: self.cycles,
            _candid_marker: self._candid_marker,
            _output_marker: self._output_marker,
        }
    }
}

impl<Config, Params, CandidOutput, Output> Request<Config, Params, CandidOutput, Output> {
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

pub type RequestCost<Config, Params> = Request<Config, Params, RpcResult<u128>, RpcResult<u128>>;

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
