use crate::{Runtime, SolRpcClient};
use candid::CandidType;
use serde::de::DeserializeOwned;
use sol_rpc_types::{GetSlotParams, RpcConfig, RpcSources};
use solana_clock::Slot;

/// Solana RPC endpoint supported by the SOL RPC canister.
pub trait SolRpcRequest {
    /// The type of parameters taken by this endpoint.
    type Params;
    /// The type returned by this endpoint.
    type Output;

    /// The name of the endpoint on the SOL RPC canister.
    fn rpc_method(&self) -> &str;

    /// Return the request parameters.
    fn params(self) -> Self::Params;
}

pub struct GetSlotRequest(Option<GetSlotParams>);

impl From<Option<GetSlotParams>> for GetSlotRequest {
    fn from(value: Option<GetSlotParams>) -> Self {
        GetSlotRequest(value)
    }
}

impl SolRpcRequest for GetSlotRequest {
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
pub struct RequestBuilder<R, E> {
    client: SolRpcClient<R>,
    request: Request<E>,
}

impl<R, E> RequestBuilder<R, E> {
    pub(super) fn new(client: SolRpcClient<R>, request: Request<E>) -> Self {
        RequestBuilder { client, request }
    }

    /// Change the amount of cycles to send for that request.
    pub fn with_cycles(mut self, cycles: u128) -> Self {
        *self.request.cycles_mut() = cycles;
        self
    }
}

impl<R: Runtime, E: SolRpcRequest> RequestBuilder<R, E> {
    /// Constructs the [`Request`] and send it using the [`SolRpcClient`].
    pub async fn send(self) -> E::Output
    where
        E::Params: CandidType + Send,
        E::Output: CandidType + DeserializeOwned,
    {
        self.client.execute_request(self.request).await
    }
}

/// A request which can be executed with `SolRpcClient::execute_request`.
pub struct Request<E> {
    pub(super) endpoint: E,
    pub(super) rpc_sources: RpcSources,
    pub(super) rpc_config: Option<RpcConfig>,
    pub(super) cycles: u128,
}

impl<E> Request<E> {
    /// Get a mutable reference to the cycles.
    #[inline]
    pub fn cycles_mut(&mut self) -> &mut u128 {
        &mut self.cycles
    }
}
