mod errors;

use crate::{
    constants::{COLLATERAL_CYCLES_PER_NODE, CONTENT_TYPE_VALUE},
    http::errors::HttpClientError,
    logs::Priority,
    state::{next_request_id, read_state, State},
};
use canhttp::{
    convert::ConvertRequestLayer,
    http::{
        json::{
            HttpJsonRpcRequest, HttpJsonRpcResponse, JsonRequestConverter, JsonResponseConverter,
        },
        FilterNonSuccessfulHttpResponse, HttpRequestConverter, HttpResponseConverter,
    },
    observability::ObservabilityLayer,
    retry::DoubleMaxResponseBytes,
    ConvertServiceBuilder, CyclesAccounting, CyclesChargingPolicy,
};
use canlog::log;
use http::{header::CONTENT_TYPE, HeaderValue};
use ic_cdk::api::management_canister::http_request::CanisterHttpRequestArgument;
use serde::{de::DeserializeOwned, Serialize};
use sol_rpc_types::{Mode, RpcError};
use std::fmt::Debug;
use tower::{
    layer::util::{Identity, Stack},
    retry::RetryLayer,
    util::MapRequestLayer,
    Service, ServiceBuilder,
};
use tower_http::{set_header::SetRequestHeaderLayer, ServiceBuilderExt};

pub fn http_client<I, O>(
    retry: bool,
) -> impl Service<HttpJsonRpcRequest<I>, Response = HttpJsonRpcResponse<O>, Error = RpcError>
where
    I: Serialize + Clone + Debug,
    O: DeserializeOwned + Debug,
{
    let maybe_retry = if retry {
        Some(RetryLayer::new(DoubleMaxResponseBytes))
    } else {
        None
    };
    let maybe_unique_id = if retry {
        Some(MapRequestLayer::new(generate_request_id))
    } else {
        None
    };
    ServiceBuilder::new()
        .map_err(|e: HttpClientError| RpcError::from(e))
        .option_layer(maybe_retry)
        .option_layer(maybe_unique_id)
        // TODO XC-292: Flesh out observability layer
        .layer(
            ObservabilityLayer::new().on_request(move |req: &HttpJsonRpcRequest<I>| {
                log!(
                    Priority::TraceHttp,
                    "JSON-RPC request with id `{}` to {}: {:?}",
                    req.body().id().clone(),
                    req.uri().host().unwrap().to_string(),
                    req.body()
                );
            }),
        )
        .layer(service_request_builder())
        .convert_response(JsonResponseConverter::new())
        .convert_response(FilterNonSuccessfulHttpResponse)
        .convert_response(HttpResponseConverter)
        .convert_request(CyclesAccounting::new(
            read_state(|s| s.get_num_subnet_nodes()),
            ChargingPolicyWithCollateral::default(),
        ))
        .service(canhttp::Client::new_with_error::<HttpClientError>())
}

fn generate_request_id<I>(request: HttpJsonRpcRequest<I>) -> HttpJsonRpcRequest<I> {
    let (parts, mut body) = request.into_parts();
    body.set_id(next_request_id());
    http::Request::from_parts(parts, body)
}

type JsonRpcServiceBuilder<I> = ServiceBuilder<
    Stack<
        ConvertRequestLayer<HttpRequestConverter>,
        Stack<
            ConvertRequestLayer<JsonRequestConverter<I>>,
            Stack<SetRequestHeaderLayer<HeaderValue>, Identity>,
        >,
    >,
>;

/// Middleware that takes care of transforming the request.
///
/// It's required to separate it from the other middlewares, to compute the exact request cost.
pub fn service_request_builder<I>() -> JsonRpcServiceBuilder<I> {
    ServiceBuilder::new()
        .insert_request_header_if_not_present(
            CONTENT_TYPE,
            HeaderValue::from_static(CONTENT_TYPE_VALUE),
        )
        .convert_request(JsonRequestConverter::<I>::new())
        .convert_request(HttpRequestConverter)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChargingPolicyWithCollateral {
    charge_user: bool,
    collateral_cycles: u128,
}

impl ChargingPolicyWithCollateral {
    pub fn new(
        num_nodes_in_subnet: u32,
        charge_user: bool,
        collateral_cycles_per_node: u128,
    ) -> Self {
        let collateral_cycles =
            collateral_cycles_per_node.saturating_mul(num_nodes_in_subnet as u128);
        Self {
            charge_user,
            collateral_cycles,
        }
    }

    fn new_from_state(s: &State) -> Self {
        Self::new(
            s.get_num_subnet_nodes(),
            !matches!(s.get_mode(), Mode::Demo),
            COLLATERAL_CYCLES_PER_NODE,
        )
    }
}

impl Default for ChargingPolicyWithCollateral {
    fn default() -> Self {
        read_state(|state| Self::new_from_state(state))
    }
}

impl CyclesChargingPolicy for ChargingPolicyWithCollateral {
    fn cycles_to_charge(
        &self,
        _request: &CanisterHttpRequestArgument,
        attached_cycles: u128,
    ) -> u128 {
        if self.charge_user {
            return attached_cycles.saturating_add(self.collateral_cycles);
        }
        0
    }
}
