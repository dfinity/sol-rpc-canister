use crate::{Runtime, SolRpcClient};
use candid::CandidType;
use serde::de::DeserializeOwned;
use sol_rpc_types::{GetSlotParams, RpcConfig, RpcSources};
use solana_clock::Slot;

pub trait SolRpcEndpoint {
    type Params;
    type Output;

    fn rpc_method(&self) -> &str;
    fn params(self) -> Self::Params;
}

pub struct GetSlotRequest(Option<GetSlotParams>);

impl From<Option<GetSlotParams>> for GetSlotRequest {
    fn from(value: Option<GetSlotParams>) -> Self {
        GetSlotRequest(value)
    }
}

impl SolRpcEndpoint for GetSlotRequest {
    type Params = Option<GetSlotParams>;
    type Output = sol_rpc_types::MultiRpcResult<Slot>;

    fn params(self) -> Self::Params {
        self.0
    }

    fn rpc_method(&self) -> &str {
        "getSlot"
    }
}

#[must_use = "RequestBuilder does nothing until you 'send' it"]
pub struct RequestBuilder<R, E> {
    client: SolRpcClient<R>,
    request: Request<E>,
}

impl<R, E> RequestBuilder<R, E> {
    pub fn new(client: SolRpcClient<R>, request: Request<E>) -> Self {
        RequestBuilder { client, request }
    }
}

impl<R: Runtime, E: SolRpcEndpoint> RequestBuilder<R, E> {
    pub async fn send(self) -> E::Output
    where
        E::Params: CandidType + Send,
        E::Output: CandidType + DeserializeOwned,
    {
        self.client.execute_request(self.request).await
    }
}

pub struct Request<E> {
    pub(super) endpoint: E,
    pub(super) rpc_sources: RpcSources,
    pub(super) rpc_config: Option<RpcConfig>,
    pub(super) cycles: u128,
}
