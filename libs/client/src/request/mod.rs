use crate::{Runtime, SolRpcClient};
use candid::CandidType;
use serde::de::DeserializeOwned;
use sol_rpc_types::{GetAccountInfoParams, GetSlotParams, GetSlotRpcConfig, RpcConfig, RpcSources};
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
    fn rpc_method(&self) -> &str;

    /// Return the request parameters.
    fn params(self) -> Self::Params;
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
    type Output = sol_rpc_types::MultiRpcResult<sol_rpc_types::AccountInfo>;

    fn rpc_method(&self) -> &str {
        "getAccountInfo"
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
    type Output = sol_rpc_types::MultiRpcResult<Slot>;

    fn rpc_method(&self) -> &str {
        "getSlot"
    }

    fn params(self) -> Self::Params {
        self.0
    }
}

pub struct RawRequest(String);

impl TryFrom<serde_json::Value> for RawRequest {
    type Error = String;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::to_string(&value)
            .map(RawRequest)
            .map_err(|e| e.to_string())
    }
}

impl SolRpcRequest for RawRequest {
    type Config = RpcConfig;
    type Params = String;
    type Output = sol_rpc_types::MultiRpcResult<String>;

    fn rpc_method(&self) -> &str {
        "request"
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
            rpc_method: rpc_request.rpc_method().to_string(),
            rpc_sources: client.config.rpc_sources.clone(),
            rpc_config: client.config.rpc_config.clone().map(Config::from),
            params: rpc_request.params(),
            cycles,
            _marker: Default::default(),
        };
        RequestBuilder { client, request }
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

/// A request which can be executed with `SolRpcClient::execute_request`.
pub struct Request<Config, Params, Output> {
    pub(super) rpc_method: String,
    pub(super) rpc_sources: RpcSources,
    pub(super) rpc_config: Option<Config>,
    pub(super) params: Params,
    pub(super) cycles: u128,
    pub(super) _marker: std::marker::PhantomData<Output>,
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
