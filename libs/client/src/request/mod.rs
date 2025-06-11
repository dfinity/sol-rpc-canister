#[cfg(test)]
mod tests;

use crate::{Runtime, SolRpcClient};
use candid::CandidType;
use derive_more::From;
use serde::de::DeserializeOwned;
use sol_rpc_types::{
    AccountInfo, CommitmentLevel, ConfirmedBlock, ConfirmedTransactionStatusWithSignature,
    DataSlice, EncodedConfirmedTransactionWithStatusMeta, GetAccountInfoEncoding,
    GetAccountInfoParams, GetBalanceParams, GetBlockCommitmentLevel, GetBlockParams,
    GetRecentPrioritizationFeesParams, GetRecentPrioritizationFeesRpcConfig,
    GetSignatureStatusesParams, GetSignaturesForAddressLimit, GetSignaturesForAddressParams,
    GetSlotParams, GetSlotRpcConfig, GetTokenAccountBalanceParams, GetTransactionEncoding,
    GetTransactionParams, Lamport, MultiRpcResult, NonZeroU8, PrioritizationFee, RoundingError,
    RpcConfig, RpcError, RpcResult, RpcSource, RpcSources, SendTransactionParams, Signature, Slot,
    TokenAmount, TransactionDetails, TransactionStatus,
};
use solana_account_decoder_client_types::token::UiTokenAmount;
use solana_hash::Hash;
use solana_transaction_status_client_types::UiConfirmedBlock;
use std::{
    fmt::{Debug, Formatter},
    num::NonZeroUsize,
    str::FromStr,
};
use strum::EnumIter;
use thiserror::Error;

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
    /// `getSignaturesForAddress` endpoint.
    GetSignaturesForAddress,
    /// `getSignatureStatuses` endpoint.
    GetSignatureStatuses,
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
            SolRpcEndpoint::GetSignatureStatuses => "getSignatureStatuses",
            SolRpcEndpoint::GetSignaturesForAddress => "getSignaturesForAddress",
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
            SolRpcEndpoint::GetSignaturesForAddress => "getSignaturesForAddressCyclesCost",
            SolRpcEndpoint::GetSignatureStatuses => "getSignatureStatusesCyclesCost",
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
    type CandidOutput = MultiRpcResult<Option<AccountInfo>>;
    type Output = MultiRpcResult<Option<solana_account_decoder_client_types::UiAccount>>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::GetAccountInfo
    }

    fn params(self, default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        let mut params = self.0;
        set_default(default_commitment_level, &mut params.commitment);
        params
    }
}

pub type GetAccountInfoRequestBuilder<R> = RequestBuilder<
    R,
    RpcConfig,
    GetAccountInfoParams,
    MultiRpcResult<Option<AccountInfo>>,
    MultiRpcResult<Option<solana_account_decoder_client_types::UiAccount>>,
>;

impl<R> GetAccountInfoRequestBuilder<R> {
    /// Change the `commitment` parameter for a `getAccountInfo` request.
    pub fn with_commitment(mut self, commitment: impl Into<CommitmentLevel>) -> Self {
        self.request.params.commitment = Some(commitment.into());
        self
    }

    /// Change the `encoding` parameter for a `getAccountInfo` request.
    pub fn with_encoding(mut self, encoding: impl Into<GetAccountInfoEncoding>) -> Self {
        self.request.params.encoding = Some(encoding.into());
        self
    }

    /// Change the `dataSlice` parameter for a `getAccountInfo` request.
    pub fn with_data_slice(mut self, data_slice: impl Into<DataSlice>) -> Self {
        self.request.params.data_slice = Some(data_slice.into());
        self
    }

    /// Change the `minContextSlot` parameter for a `getAccountInfo` request.
    pub fn with_min_context_slot(mut self, slot: Slot) -> Self {
        self.request.params.min_context_slot = Some(slot);
        self
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
    type CandidOutput = MultiRpcResult<Lamport>;
    type Output = MultiRpcResult<Lamport>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::GetBalance
    }

    fn params(self, default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        let mut params = self.0;
        set_default(default_commitment_level, &mut params.commitment);
        params
    }
}

pub type GetBalanceRequestBuilder<R> = RequestBuilder<
    R,
    RpcConfig,
    GetBalanceParams,
    MultiRpcResult<Lamport>,
    MultiRpcResult<Lamport>,
>;

impl<R> GetBalanceRequestBuilder<R> {
    /// Change the `commitment` parameter for a `getBalance` request.
    pub fn with_commitment(mut self, commitment_level: impl Into<CommitmentLevel>) -> Self {
        self.request.params.commitment = Some(commitment_level.into());
        self
    }

    /// Change the `minContextSlot` parameter for a `getBalance` request.
    pub fn with_min_context_slot(mut self, slot: Slot) -> Self {
        self.request.params.min_context_slot = Some(slot);
        self
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
    type CandidOutput = MultiRpcResult<Option<ConfirmedBlock>>;
    type Output = MultiRpcResult<Option<UiConfirmedBlock>>;

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

pub type GetBlockRequestBuilder<R> = RequestBuilder<
    R,
    RpcConfig,
    GetBlockParams,
    MultiRpcResult<Option<ConfirmedBlock>>,
    MultiRpcResult<Option<UiConfirmedBlock>>,
>;

impl<R> GetBlockRequestBuilder<R> {
    /// Change the `commitment` parameter for a `getBlock` request.
    pub fn with_commitment(mut self, commitment_level: impl Into<GetBlockCommitmentLevel>) -> Self {
        self.request.params.commitment = Some(commitment_level.into());
        self
    }

    /// Change the `maxSupportedTransactionVersion` parameter for a `getBlock` request.
    pub fn with_max_supported_transaction_version(mut self, version: u8) -> Self {
        self.request.params.max_supported_transaction_version = Some(version);
        self
    }

    /// Change the `transactionDetails` parameter for a `getBlock` request.
    pub fn with_transaction_details(
        mut self,
        transaction_details: impl Into<TransactionDetails>,
    ) -> Self {
        self.request.params.transaction_details = Some(transaction_details.into());
        self.update_cycles()
    }

    /// Change the `rewards` parameter for a `getBlock` request to `false`.
    pub fn without_rewards(mut self) -> Self {
        self.request.params.rewards = Some(false);
        self.update_cycles()
    }

    /// Update the cycles estimate for this request
    pub fn update_cycles(self) -> Self {
        let cycles = match self.request.params.transaction_details.unwrap_or_default() {
            TransactionDetails::Accounts => 1_000_000_000_000,
            TransactionDetails::Signatures => 100_000_000_000,
            TransactionDetails::None => match self.request.params.rewards {
                Some(true) | None => 20_000_000_000,
                Some(false) => 10_000_000_000,
            },
        };
        self.with_cycles(cycles)
    }
}

#[derive(Debug, Clone, Default)]
pub struct GetRecentPrioritizationFeesRequest(GetRecentPrioritizationFeesParams);

impl SolRpcRequest for GetRecentPrioritizationFeesRequest {
    type Config = GetRecentPrioritizationFeesRpcConfig;
    type Params = GetRecentPrioritizationFeesParams;
    type CandidOutput = MultiRpcResult<Vec<PrioritizationFee>>;
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

#[derive(Debug, Clone, From)]
pub struct GetSignaturesForAddressRequest(GetSignaturesForAddressParams);

impl SolRpcRequest for GetSignaturesForAddressRequest {
    type Config = RpcConfig;
    type Params = GetSignaturesForAddressParams;
    type CandidOutput = Self::Output;
    type Output = MultiRpcResult<Vec<ConfirmedTransactionStatusWithSignature>>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::GetSignaturesForAddress
    }

    fn params(self, default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        let mut params = self.0;
        set_default(default_commitment_level, &mut params.commitment);
        params
    }
}

pub type GetSignaturesForAddressRequestBuilder<R> = RequestBuilder<
    R,
    RpcConfig,
    GetSignaturesForAddressParams,
    MultiRpcResult<Vec<ConfirmedTransactionStatusWithSignature>>,
    MultiRpcResult<Vec<ConfirmedTransactionStatusWithSignature>>,
>;

impl<R> GetSignaturesForAddressRequestBuilder<R> {
    /// Change the `commitment` parameter for a `getSignaturesForAddress` request.
    pub fn with_commitment(mut self, commitment_level: CommitmentLevel) -> Self {
        self.request.params.commitment = Some(commitment_level);
        self
    }

    /// Change the `minContextSlot` parameter for a `getSignaturesForAddress` request.
    pub fn with_min_context_slot(mut self, slot: Slot) -> Self {
        self.request.params.min_context_slot = Some(slot);
        self
    }

    /// Change the `limit` parameter for a `getSignaturesForAddress` request.
    pub fn with_limit(mut self, limit: GetSignaturesForAddressLimit) -> Self {
        self.request.params.limit = Some(limit);
        self
    }

    /// Change the `until` parameter for a `getSignaturesForAddress` request.
    pub fn with_until(mut self, until: impl Into<Signature>) -> Self {
        self.request.params.until = Some(until.into());
        self
    }

    /// Change the `before` parameter for a `getSignaturesForAddress` request.
    pub fn with_before(mut self, before: impl Into<Signature>) -> Self {
        self.request.params.before = Some(before.into());
        self
    }
}

#[derive(Debug, Clone, Default, From)]
pub struct GetSignatureStatusesRequest(GetSignatureStatusesParams);

impl SolRpcRequest for GetSignatureStatusesRequest {
    type Config = RpcConfig;
    type Params = GetSignatureStatusesParams;
    type CandidOutput = MultiRpcResult<Vec<Option<TransactionStatus>>>;
    type Output =
        MultiRpcResult<Vec<Option<solana_transaction_status_client_types::TransactionStatus>>>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::GetSignatureStatuses
    }

    fn params(self, _default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        self.0
    }
}

pub type GetSignatureStatusesRequestBuilder<R> = RequestBuilder<
    R,
    RpcConfig,
    GetSignatureStatusesParams,
    MultiRpcResult<Vec<Option<TransactionStatus>>>,
    MultiRpcResult<Vec<Option<solana_transaction_status_client_types::TransactionStatus>>>,
>;

impl<R> GetSignatureStatusesRequestBuilder<R> {
    /// Change the `searchTransactionHistory` parameter for a `getSignatureStatuses` request.
    pub fn with_search_transaction_history(mut self, search_transaction_history: bool) -> Self {
        self.request.params.search_transaction_history = Some(search_transaction_history);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct GetSlotRequest(Option<GetSlotParams>);

impl SolRpcRequest for GetSlotRequest {
    type Config = GetSlotRpcConfig;
    type Params = Option<GetSlotParams>;
    type CandidOutput = Self::Output;
    type Output = MultiRpcResult<Slot>;

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

pub type GetSlotRequestBuilder<R> = RequestBuilder<
    R,
    GetSlotRpcConfig,
    Option<GetSlotParams>,
    MultiRpcResult<Slot>,
    MultiRpcResult<Slot>,
>;

impl<R> GetSlotRequestBuilder<R> {
    /// Change the `commitment` parameter for a `getSlot` request.
    pub fn with_commitment(mut self, commitment_level: CommitmentLevel) -> Self {
        self.request.params.get_or_insert_default().commitment = Some(commitment_level);
        self
    }

    /// Change the `minContextSlot` parameter for a `getSlot` request.
    pub fn with_min_context_slot(mut self, slot: Slot) -> Self {
        self.request.params.get_or_insert_default().min_context_slot = Some(slot);
        self
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
    type CandidOutput = MultiRpcResult<TokenAmount>;
    type Output = MultiRpcResult<UiTokenAmount>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::GetTokenAccountBalance
    }

    fn params(self, default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        let mut params = self.0;
        set_default(default_commitment_level, &mut params.commitment);
        params
    }
}

pub type GetTokenAccountBalanceRequestBuilder<R> = RequestBuilder<
    R,
    RpcConfig,
    GetTokenAccountBalanceParams,
    MultiRpcResult<TokenAmount>,
    MultiRpcResult<UiTokenAmount>,
>;

impl<R> GetTokenAccountBalanceRequestBuilder<R> {
    /// Change the `commitment` parameter for a `getTokenAccountBalance` request.
    pub fn with_commitment(mut self, commitment_level: CommitmentLevel) -> Self {
        self.request.params.commitment = Some(commitment_level);
        self
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
    type CandidOutput = MultiRpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>;
    type Output = MultiRpcResult<
        Option<solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta>,
    >;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::GetTransaction
    }

    fn params(self, default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        let mut params = self.0;
        set_default(default_commitment_level, &mut params.commitment);
        params
    }
}

pub type GetTransactionRequestBuilder<R> = RequestBuilder<
    R,
    RpcConfig,
    GetTransactionParams,
    MultiRpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>,
    MultiRpcResult<
        Option<solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta>,
    >,
>;

impl<R> GetTransactionRequestBuilder<R> {
    /// Change the `commitment` parameter for a `getTransaction` request.
    pub fn with_commitment(mut self, commitment_level: CommitmentLevel) -> Self {
        self.request.params.commitment = Some(commitment_level);
        self
    }

    /// Change the `maxSupportedTransaction_version` parameter for a `getTransaction` request.
    pub fn with_max_supported_transaction_version(mut self, version: u8) -> Self {
        self.request.params.max_supported_transaction_version = Some(version);
        self
    }

    /// Change the `encoding` parameter for a `getTransaction` request.
    pub fn with_encoding(mut self, encoding: GetTransactionEncoding) -> Self {
        self.request.params.encoding = Some(encoding);
        self
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
    type CandidOutput = MultiRpcResult<Signature>;
    type Output = MultiRpcResult<solana_signature::Signature>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::SendTransaction
    }

    fn params(self, default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        let mut params = self.0;
        set_default(default_commitment_level, &mut params.preflight_commitment);
        params
    }
}

pub type SendTransactionRequestBuilder<R> = RequestBuilder<
    R,
    RpcConfig,
    SendTransactionParams,
    MultiRpcResult<Signature>,
    MultiRpcResult<solana_signature::Signature>,
>;

impl<R> SendTransactionRequestBuilder<R> {
    /// Change the `skipPreflight` parameter for a `sendTransaction` request.
    pub fn with_skip_preflight(mut self, skip_preflight: bool) -> Self {
        self.request.params.skip_preflight = Some(skip_preflight);
        self
    }

    /// Change the `preflightCommitment` parameter for a `sendTransaction` request.
    pub fn with_preflight_commitment(mut self, preflight_commitment: CommitmentLevel) -> Self {
        self.request.params.preflight_commitment = Some(preflight_commitment);
        self
    }

    /// Change the `maxRetries` parameter for a `sendTransaction` request.
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.request.params.max_retries = Some(max_retries);
        self
    }

    /// Change the `minContextSlot` parameter for a `sendTransaction` request.
    pub fn with_min_context_slot(mut self, slot: Slot) -> Self {
        self.request.params.min_context_slot = Some(slot);
        self
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
    type CandidOutput = MultiRpcResult<String>;
    type Output = MultiRpcResult<String>;

    fn endpoint(&self) -> SolRpcEndpoint {
        SolRpcEndpoint::JsonRequest
    }

    fn params(self, _default_commitment_level: Option<CommitmentLevel>) -> Self::Params {
        self.0
    }
}

pub type JsonRequestBuilder<R> =
    RequestBuilder<R, RpcConfig, String, MultiRpcResult<String>, MultiRpcResult<String>>;

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
    MultiRpcResult<Vec<PrioritizationFee>>,
    MultiRpcResult<Vec<PrioritizationFee>>,
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
    pub fn with_rpc_config(mut self, rpc_config: impl Into<Config>) -> Self {
        *self.request.rpc_config_mut() = Some(rpc_config.into());
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
    pub fn with_max_slot_rounding_error<T: Into<RoundingError>>(
        mut self,
        rounding_error: T,
    ) -> Self {
        let config = self.request.rpc_config_mut().get_or_insert_default();
        config.max_slot_rounding_error = Some(rounding_error.into());
        self
    }

    /// Change the maximum number of entries for a `getRecentPrioritizationFees` response.
    pub fn with_max_length<T: Into<NonZeroU8>>(mut self, len: T) -> Self {
        let config = self.request.rpc_config_mut().get_or_insert_default();
        config.set_max_length(len.into());
        self
    }
}

impl<Runtime, Params, CandidOutput, Output>
    RequestBuilder<Runtime, GetSlotRpcConfig, Params, CandidOutput, Output>
{
    /// Change the rounding error for `getSlot` request.
    pub fn with_rounding_error<T: Into<RoundingError>>(mut self, rounding_error: T) -> Self {
        let config = self.request.rpc_config_mut().get_or_insert_default();
        config.rounding_error = Some(rounding_error.into());
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

/// An error that occurred while trying to fetch a recent blockhash.
/// See [`SolRpcClient::estimate_recent_blockhash`]
#[derive(Debug, Clone, PartialEq, Error)]
pub enum EstimateRecentBlockhashError {
    /// The results from the different providers were not consistent for a `getSlot` call.
    #[error("Inconsistent result while fetching slot: {0:?}")]
    GetSlotConsensusError(Vec<(RpcSource, RpcResult<Slot>)>),
    /// The results from the different providers were not consistent for a `getBlock` call.
    #[error("Inconsistent result while fetching block: {0:?}")]
    GetBlockConsensusError(Vec<(RpcSource, RpcResult<Option<UiConfirmedBlock>>)>),
    /// An error occurred during a `getSlot` call.
    #[error("Error while fetching slot: {0}")]
    #[from(ignore)]
    GetSlotRpcError(RpcError),
    /// An error occurred during a `getBlock` call.
    #[error("Error while fetching block: {0}")]
    #[from(ignore)]
    GetBlockRpcError(RpcError),
    /// There was no matching block for the fetched slot.
    #[error("No block for slot: {0}")]
    MissingBlock(Slot),
}

type EstimateRecentBlockhashResult<T> = Result<T, EstimateRecentBlockhashError>;

/// A builder to build a request to fetch a recent blockhash.
/// See [`SolRpcClient::estimate_recent_blockhash`].
#[must_use = "EstimateBlockhashRequestBuilder does nothing until you 'send' it"]
pub struct EstimateBlockhashRequestBuilder<R> {
    client: SolRpcClient<R>,
    num_tries: NonZeroUsize,
    rounding_error: Option<RoundingError>,
    rpc_config: Option<RpcConfig>,
}

impl<R> EstimateBlockhashRequestBuilder<R> {
    /// Create a new [`EstimateBlockhashRequestBuilder`] request with the given [`SolRpcClient`]
    /// and default parameters.
    ///
    /// The maximum number of attempts that will be performed to retrieve a blockhash is set to 3.
    pub fn new(client: SolRpcClient<R>) -> Self {
        Self {
            client,
            num_tries: NonZeroUsize::new(3).unwrap(),
            rounding_error: None,
            rpc_config: None,
        }
    }

    /// Sets the maximum number of attempts that will be performed to retrieve a blockhash.
    ///
    /// Each attempt consists of at most one `getSlot` and one `getBlock` call, such that the
    /// maximum number of RPC calls performed is `2 * num_tries`.
    pub fn with_num_tries(mut self, num_tries: NonZeroUsize) -> Self {
        self.num_tries = num_tries;
        self
    }

    /// Sets an [`RpcConfig`] for the `getSlot` and `getBlock` calls. If not set, the default
    /// client [`RpcConfig`] is used.
    pub fn with_rpc_config(mut self, rpc_config: RpcConfig) -> Self {
        self.rpc_config = Some(rpc_config);
        self
    }

    /// Sets a [`RoundingError`] for the `getSlot` calls. If not set, the default value for the
    /// rounding error is used.
    pub fn with_get_slot_rounding_error(mut self, rounding_error: RoundingError) -> Self {
        self.rounding_error = Some(rounding_error);
        self
    }
}

impl<R: Runtime> EstimateBlockhashRequestBuilder<R> {
    /// Constructs the required `getSlot` and `getBlock` requests and try to estimate a recent
    /// blockhash using the [`SolRpcClient`], possibly with re-tries (see
    /// [`SolRpcClient::estimate_recent_blockhash`]).
    pub async fn send(self) -> Result<Hash, Vec<EstimateRecentBlockhashError>> {
        let mut errors = Vec::with_capacity(self.num_tries.into());
        while errors.len() < usize::from(self.num_tries) {
            match self.get_slot_then_get_blockhash().await {
                Ok(hash) => return Ok(hash),
                Err(error) => errors.push(error),
            }
        }
        Err(errors)
    }

    async fn get_slot_then_get_blockhash(&self) -> EstimateRecentBlockhashResult<Hash> {
        let slot = self.get_slot().await?;
        let block = self.get_block(slot).await?;
        match Hash::from_str(&block.blockhash) {
            Ok(blockhash) => Ok(blockhash),
            Err(e) => Err(EstimateRecentBlockhashError::GetBlockRpcError(e.into())),
        }
    }

    async fn get_slot(&self) -> EstimateRecentBlockhashResult<Slot> {
        let mut request = self.client.get_slot();
        if let Some(rpc_config) = self.rpc_config.as_ref() {
            request = request.with_rpc_config(rpc_config.clone());
        }
        if let Some(rounding_error) = self.rounding_error {
            request = request.with_rounding_error(rounding_error);
        }
        match request.send().await {
            MultiRpcResult::Consistent(Ok(slot)) => Ok(slot),
            MultiRpcResult::Consistent(Err(e)) => {
                Err(EstimateRecentBlockhashError::GetSlotRpcError(e))
            }
            MultiRpcResult::Inconsistent(results) => {
                Err(EstimateRecentBlockhashError::GetSlotConsensusError(results))
            }
        }
    }

    async fn get_block(&self, slot: Slot) -> EstimateRecentBlockhashResult<UiConfirmedBlock> {
        let mut request = self
            .client
            .get_block(slot)
            .with_transaction_details(TransactionDetails::None)
            .with_max_supported_transaction_version(0)
            .without_rewards();
        if let Some(rpc_config) = self.rpc_config.as_ref() {
            request = request.with_rpc_config(rpc_config.clone());
        }
        match request.send().await {
            MultiRpcResult::Consistent(Ok(Some(block))) => Ok(block),
            MultiRpcResult::Consistent(Ok(None)) => {
                Err(EstimateRecentBlockhashError::MissingBlock(slot))
            }
            MultiRpcResult::Consistent(Err(e)) => {
                Err(EstimateRecentBlockhashError::GetBlockRpcError(e))
            }
            MultiRpcResult::Inconsistent(results) => Err(
                EstimateRecentBlockhashError::GetBlockConsensusError(results),
            ),
        }
    }
}
