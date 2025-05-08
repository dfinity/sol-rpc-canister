#[cfg(test)]
mod tests;

use crate::{Runtime, SolRpcClient};
use candid::CandidType;
use serde::de::DeserializeOwned;
use sol_rpc_types::{
    AccountInfo, CommitmentLevel, ConfirmedBlock, GetAccountInfoParams, GetBalanceParams,
    GetBlockCommitmentLevel, GetBlockParams, GetRecentPrioritizationFeesParams,
    GetRecentPrioritizationFeesRpcConfig, GetSlotParams, GetSlotRpcConfig,
    GetTokenAccountBalanceParams, GetTransactionParams, Lamport, PrioritizationFee, RpcConfig,
    RpcResult, RpcSources, SendTransactionParams, Signature, TokenAmount, TransactionInfo,
};
use solana_account_decoder_client_types::token::UiTokenAmount;
use solana_clock::Slot;
use solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta;
use std::fmt::{Debug, Formatter};
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
    fn params(self, default_commitment_level: Option<CommitmentLevel>) -> Self::Params;
}

/// Endpoint on the SOL RPC canister triggering a call to Solana providers.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, EnumIter)]
pub enum SolRpcEndpoint {
    /// `getAccountInfo` endpoint.
    GetAccountInfo,
    /// `getBalance` endpoint.
    GetBalance,
    /// `getBlock` endpoint.
    GetBlock,
    /// `getRecentPrioritizationFees` endpoint.
    GetRecentPrioritizationFees,
    /// `getSlot` endpoint.
    GetSlot,
    /// `getTokenAccountBalance` endpoint.
    GetTokenAccountBalance,
    /// `getTransaction` endpoint.
    GetTransaction,
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
            SolRpcEndpoint::GetBalance => "getBalance",
            SolRpcEndpoint::GetBlock => "getBlock",
            SolRpcEndpoint::GetRecentPrioritizationFees => "getRecentPrioritizationFees",
            SolRpcEndpoint::GetSlot => "getSlot",
            SolRpcEndpoint::GetTokenAccountBalance => "getTokenAccountBalance",
            SolRpcEndpoint::GetTransaction => "getTransaction",
            SolRpcEndpoint::JsonRequest => "jsonRequest",
            SolRpcEndpoint::SendTransaction => "sendTransaction",
        }
    }

    /// Method name on the SOL RPC canister to estimate the amount of cycles for that request.
    pub fn cycles_cost_method(&self) -> &'static str {
        match &self {
            SolRpcEndpoint::GetAccountInfo => "getAccountInfoCyclesCost",
            SolRpcEndpoint::GetBalance => "getBalanceCyclesCost",
            SolRpcEndpoint::GetBlock => "getBlockCyclesCost",
            SolRpcEndpoint::GetRecentPrioritizationFees => "getRecentPrioritizationFeesCyclesCost",
            SolRpcEndpoint::GetSlot => "getSlotCyclesCost",
            SolRpcEndpoint::GetTransaction => "getTransactionCyclesCost",
            SolRpcEndpoint::GetTokenAccountBalance => "getTokenAccountBalanceCyclesCost",
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

    fn params(self, default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        let mut params = self.0;
        set_default(default_commitment_level, &mut params.commitment);
        params
    }
}

#[derive(Debug, Clone)]
pub struct GetBalanceRequest(GetBalanceParams);

impl GetBalanceRequest {
    pub fn new(params: GetBalanceParams) -> Self {
        Self(params)
    }
}

impl SolRpcRequest for GetBalanceRequest {
    type Config = RpcConfig;
    type Params = GetBalanceParams;
    type CandidOutput = sol_rpc_types::MultiRpcResult<Lamport>;
    type Output = sol_rpc_types::MultiRpcResult<Lamport>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::GetBalance
    }

    fn params(self, default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        let mut params = self.0;
        set_default(default_commitment_level, &mut params.commitment);
        params
    }
}

#[derive(Debug, Clone)]
pub struct GetBlockRequest(GetBlockParams);

impl GetBlockRequest {
    pub fn new(params: GetBlockParams) -> Self {
        Self(params)
    }
}

impl SolRpcRequest for GetBlockRequest {
    type Config = RpcConfig;
    type Params = GetBlockParams;
    type CandidOutput = sol_rpc_types::MultiRpcResult<Option<ConfirmedBlock>>;
    type Output = sol_rpc_types::MultiRpcResult<
        Option<solana_transaction_status_client_types::UiConfirmedBlock>,
    >;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::GetBlock
    }

    fn params(self, default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        let mut params = self.0;
        let default_block_commitment_level =
            default_commitment_level.map(|commitment| match commitment {
                CommitmentLevel::Processed => {
                    // The minimum commitment level for `getBlock` is `confirmed,
                    // `processed` is not supported.
                    // Not setting a value here would be equivalent to requiring the block to be `finalized`,
                    // which seems to go against the chosen `default_commitment_level` of `processed` and so `confirmed`
                    // is the best we can do here.
                    GetBlockCommitmentLevel::Confirmed
                }
                CommitmentLevel::Confirmed => GetBlockCommitmentLevel::Confirmed,
                CommitmentLevel::Finalized => GetBlockCommitmentLevel::Finalized,
            });
        set_default(default_block_commitment_level, &mut params.commitment);
        params
    }
}

#[derive(Debug, Clone, Default)]
pub struct GetRecentPrioritizationFeesRequest(GetRecentPrioritizationFeesParams);

impl SolRpcRequest for GetRecentPrioritizationFeesRequest {
    type Config = GetRecentPrioritizationFeesRpcConfig;
    type Params = GetRecentPrioritizationFeesParams;
    type CandidOutput = sol_rpc_types::MultiRpcResult<Vec<PrioritizationFee>>;
    type Output = Self::CandidOutput;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::GetRecentPrioritizationFees
    }

    fn params(self, _default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        // [getRecentPrioritizationFees](https://solana.com/de/docs/rpc/http/getrecentprioritizationfees)
        // does not use commitment levels
        self.0
    }
}

impl From<GetRecentPrioritizationFeesParams> for GetRecentPrioritizationFeesRequest {
    fn from(value: GetRecentPrioritizationFeesParams) -> Self {
        Self(value)
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

    fn params(self, default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        let mut params = self.0;
        if let Some(slot_params) = params.as_mut() {
            set_default(default_commitment_level, &mut slot_params.commitment);
            return params;
        }
        if let Some(commitment) = default_commitment_level {
            return Some(GetSlotParams {
                commitment: Some(commitment),
                ..Default::default()
            });
        }
        params
    }
}

#[derive(Debug, Clone)]
pub struct GetTokenAccountBalanceRequest(GetTokenAccountBalanceParams);

impl GetTokenAccountBalanceRequest {
    pub fn new(params: GetTokenAccountBalanceParams) -> Self {
        Self(params)
    }
}

impl SolRpcRequest for GetTokenAccountBalanceRequest {
    type Config = RpcConfig;
    type Params = GetTokenAccountBalanceParams;
    type CandidOutput = sol_rpc_types::MultiRpcResult<TokenAmount>;
    type Output = sol_rpc_types::MultiRpcResult<UiTokenAmount>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::GetTokenAccountBalance
    }

    fn params(self, default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        let mut params = self.0;
        set_default(default_commitment_level, &mut params.commitment);
        params
    }
}

#[derive(Debug, Clone)]
pub struct GetTransactionRequest(GetTransactionParams);

impl GetTransactionRequest {
    pub fn new(params: GetTransactionParams) -> Self {
        Self(params)
    }
}

impl SolRpcRequest for GetTransactionRequest {
    type Config = RpcConfig;
    type Params = GetTransactionParams;
    type CandidOutput = sol_rpc_types::MultiRpcResult<Option<TransactionInfo>>;
    type Output = sol_rpc_types::MultiRpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::GetTransaction
    }

    fn params(self, default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        let mut params = self.0;
        set_default(default_commitment_level, &mut params.commitment);
        params
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
    type CandidOutput = sol_rpc_types::MultiRpcResult<Signature>;
    type Output = sol_rpc_types::MultiRpcResult<solana_signature::Signature>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::SendTransaction
    }

    fn params(self, default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        let mut params = self.0;
        set_default(default_commitment_level, &mut params.preflight_commitment);
        params
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

    fn params(self, _default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
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

pub type GetRecentPrioritizationFeesRequestBuilder<R> = RequestBuilder<
    R,
    GetRecentPrioritizationFeesRpcConfig,
    GetRecentPrioritizationFeesParams,
    sol_rpc_types::MultiRpcResult<Vec<PrioritizationFee>>,
    sol_rpc_types::MultiRpcResult<Vec<PrioritizationFee>>,
>;

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

impl<Runtime: Debug, Config: Debug, Params: Debug, CandidOutput, Output> Debug
    for RequestBuilder<Runtime, Config, Params, CandidOutput, Output>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let RequestBuilder { client, request } = &self;
        f.debug_struct("RequestBuilder")
            .field("client", client)
            .field("request", request)
            .finish()
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
        let endpoint = rpc_request.endpoint();
        let params = rpc_request.params(client.config.default_commitment_level.clone());
        let request = Request {
            endpoint,
            rpc_sources: client.config.rpc_sources.clone(),
            rpc_config: client.config.rpc_config.clone().map(Config::from),
            params,
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

    /// Modify current parameters to send for that request.
    pub fn modify_params<F>(mut self, mutator: F) -> Self
    where
        F: FnOnce(&mut Params),
    {
        mutator(self.request.params_mut());
        self
    }

    /// Change the RPC configuration to use for that request.
    pub fn with_rpc_config(mut self, rpc_config: impl Into<Option<Config>>) -> Self {
        *self.request.rpc_config_mut() = rpc_config.into();
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
    RequestBuilder<Runtime, GetRecentPrioritizationFeesRpcConfig, Params, CandidOutput, Output>
{
    /// Change the rounding error for the maximum slot value for a `getRecentPrioritizationFees` request.
    pub fn with_max_slot_rounding_error(mut self, rounding_error: u64) -> Self {
        let config = self.request.rpc_config_mut().get_or_insert_default();
        config.max_slot_rounding_error = Some(rounding_error);
        self
    }

    /// Change the maximum number of entries for a `getRecentPrioritizationFees` response.
    pub fn with_max_length(mut self, len: u8) -> Self {
        let config = self.request.rpc_config_mut().get_or_insert_default();
        config.max_length = Some(len);
        self
    }
}

impl<Runtime, Params, CandidOutput, Output>
    RequestBuilder<Runtime, GetSlotRpcConfig, Params, CandidOutput, Output>
{
    /// Change the rounding error for `getSlot` request.
    pub fn with_rounding_error(mut self, rounding_error: u64) -> Self {
        let config = self.request.rpc_config_mut().get_or_insert_default();
        config.rounding_error = Some(rounding_error);
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

impl<Config: Debug, Params: Debug, CandidOutput, Output> Debug
    for Request<Config, Params, CandidOutput, Output>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Request {
            endpoint,
            rpc_sources,
            rpc_config,
            params,
            cycles,
            _candid_marker,
            _output_marker,
        } = &self;
        f.debug_struct("Request")
            .field("endpoint", endpoint)
            .field("rpc_sources", rpc_sources)
            .field("rpc_config", rpc_config)
            .field("params", params)
            .field("cycles", cycles)
            .field("_candid_marker", _candid_marker)
            .field("_output_marker", _output_marker)
            .finish()
    }
}

impl<Config: PartialEq, Params: PartialEq, CandidOutput, Output> PartialEq
    for Request<Config, Params, CandidOutput, Output>
{
    fn eq(
        &self,
        Request {
            endpoint,
            rpc_sources,
            rpc_config,
            params,
            cycles,
            _candid_marker,
            _output_marker,
        }: &Self,
    ) -> bool {
        &self.endpoint == endpoint
            && &self.rpc_sources == rpc_sources
            && &self.rpc_config == rpc_config
            && &self.params == params
            && &self.cycles == cycles
            && &self._candid_marker == _candid_marker
            && &self._output_marker == _output_marker
    }
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

fn set_default<T>(default_value: Option<T>, value: &mut Option<T>) {
    if default_value.is_some() && value.is_none() {
        *value = Some(default_value.unwrap())
    }
}
