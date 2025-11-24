pub mod errors;

use crate::{
    add_latency_metric, add_metric_entry,
    constants::{COLLATERAL_CYCLES_PER_NODE, CONTENT_TYPE_VALUE},
    http::errors::HttpClientError,
    logs::Priority,
    memory::{next_request_id, read_state},
    metrics::{MetricRpcCallResponse, MetricRpcHost, MetricRpcMethod},
};
use canhttp::cycles::CyclesAccounting;
use canhttp::{
    convert::ConvertRequestLayer,
    cycles::ChargeCaller,
    http::{
        json::{
            ConsistentResponseIdFilterError, CreateJsonRpcIdFilter, HttpJsonRpcRequest,
            HttpJsonRpcResponse, Id, JsonRequestConverter, JsonResponseConversionError,
            JsonResponseConverter,
        },
        FilterNonSuccessfulHttpResponse, FilterNonSuccessfulHttpResponseError,
        HttpRequestConverter, HttpResponseConverter,
    },
    observability::ObservabilityLayer,
    retry::DoubleMaxResponseBytes,
    ConvertServiceBuilder, HttpsOutcallError, IcError,
};
use canlog::log;
use http::{header::CONTENT_TYPE, HeaderValue};
use ic_management_canister_types::HttpRequestArgs as IcHttpRequest;
use serde::{de::DeserializeOwned, Serialize};
use sol_rpc_types::{JsonRpcError, RpcError};
use std::fmt::Debug;
use tower::{
    layer::util::{Identity, Stack},
    retry::RetryLayer,
    util::MapRequestLayer,
    Service, ServiceBuilder,
};
use tower_http::{set_header::SetRequestHeaderLayer, ServiceBuilderExt};

pub fn http_client<I, O>(
    rpc_method: MetricRpcMethod,
    retry: bool,
) -> impl Service<HttpJsonRpcRequest<I>, Response = O, Error = RpcError>
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
        .map_result(extract_json_rpc_response)
        .map_err(|e: HttpClientError| RpcError::from(e))
        .option_layer(maybe_retry)
        .option_layer(maybe_unique_id)
        .layer(
            ObservabilityLayer::new()
                .on_request(move |req: &HttpJsonRpcRequest<I>| {
                    let req_data = MetricData {
                        method: rpc_method.clone(),
                        host: MetricRpcHost(req.uri().host().unwrap().to_string()),
                        request_id: req.body().id().clone(),
                        start_ns: ic_cdk::api::time(),
                    };
                    log!(Priority::TraceHttp, "JSON-RPC request with id `{}` to {}: {:?}",
                        req_data.request_id,
                        req_data.host.0,
                        req.body()
                    );
                    req_data
                })
                .on_response(|req_data: MetricData, response: &HttpJsonRpcResponse<O>| {
                    match response.body().as_result() {
                        Ok(_) => {
                            observe_response(MetricRpcCallResponse::Success, &req_data);
                        }
                        Err(_) => observe_error_with_status(response.status(), &req_data),
                    }
                    log!(
                        Priority::TraceHttp,
                        "Got response for request with id `{}`. Response with status {}: {:?}",
                        req_data.request_id,
                        response.status(),
                        response.body()
                    );
                })
                .on_error(
                    |req_data: MetricData, error: &HttpClientError| match error {
                        HttpClientError::IcError(error) => {
                            if error.is_response_too_large() {
                                observe_response(MetricRpcCallResponse::MaxResponseSizeExceeded, &req_data);
                            } else {
                                log!(
                                    Priority::TraceHttp,
                                    "IC error for request with id `{}`: {}",
                                    req_data.request_id,
                                    error
                                );

                                match error {
                                    IcError::CallRejected { code, .. } => {
                                        observe_response(MetricRpcCallResponse::IcError(code.to_string()), &req_data);
                                    }
                                    IcError::InsufficientLiquidCycleBalance { .. } => {}
                                }
                            }
                        }
                        HttpClientError::UnsuccessfulHttpResponse(
                            FilterNonSuccessfulHttpResponseError::UnsuccessfulResponse(response),
                        ) => {
                            observe_error_with_status(response.status().as_u16(), &req_data);
                            log!(
                                Priority::TraceHttp,
                                "Unsuccessful HTTP response for request with id `{}`. Response with status {}: {}",
                                req_data.request_id,
                                response.status(),
                                String::from_utf8_lossy(response.body())
                            );
                        }
                        HttpClientError::InvalidJsonResponse(
                            JsonResponseConversionError::InvalidJsonResponse {
                                status,
                                body: _,
                                parsing_error: _,
                            },
                        ) => {
                            observe_error_with_status(*status, &req_data);
                            log!(
                                Priority::TraceHttp,
                                "Invalid JSON RPC response for request with id `{}`: {}",
                                req_data.request_id,
                                error
                            );
                        }
                        HttpClientError::InvalidJsonResponseId(ConsistentResponseIdFilterError::InconsistentId { status, request_id: _, response_id: _ }) => {
                            observe_error_with_status(*status, &req_data);
                            log!(
                                Priority::TraceHttp,
                                "Invalid JSON RPC response for request with id `{}`: {}",
                                req_data.request_id,
                                error
                            );
                        }
                        HttpClientError::NotHandledError(e) => {
                            log!(Priority::Info, "BUG: Unexpected error: {}", e);
                        }
                        HttpClientError::CyclesAccountingError(_) => {}
                    },
                ),
        )
        .filter_response(CreateJsonRpcIdFilter::new())
        .layer(service_request_builder())
        .convert_response(JsonResponseConverter::new())
        .convert_response(FilterNonSuccessfulHttpResponse)
        .convert_response(HttpResponseConverter)
        .convert_request(CyclesAccounting::new(charging_policy_with_collateral()))
        .service(canhttp::Client::new_with_error::<HttpClientError>())
}

fn extract_json_rpc_response<O>(
    result: Result<HttpJsonRpcResponse<O>, RpcError>,
) -> Result<O, RpcError> {
    match result?.into_body().into_result() {
        Ok(value) => Ok(value),
        Err(json_rpc_error) => Err(RpcError::JsonRpcError(JsonRpcError {
            code: json_rpc_error.code,
            message: json_rpc_error.message,
        })),
    }
}

fn generate_request_id<I>(request: HttpJsonRpcRequest<I>) -> HttpJsonRpcRequest<I> {
    let (parts, mut body) = request.into_parts();
    body.set_id(next_request_id());
    http::Request::from_parts(parts, body)
}

fn observe_error_with_status(status: impl Into<u16>, req_data: &MetricData) {
    match status.into() {
        200 => observe_response(MetricRpcCallResponse::JsonRpcError, req_data),
        status => observe_response(MetricRpcCallResponse::HttpError(status.into()), req_data),
    }
}

fn observe_response(response: MetricRpcCallResponse, req_data: &MetricData) {
    match response {
        MetricRpcCallResponse::HttpError(_)
        | MetricRpcCallResponse::JsonRpcError
        | MetricRpcCallResponse::Success => add_latency_metric!(
            latencies,
            (req_data.method.clone(), req_data.host.clone()),
            req_data.start_ns
        ),
        MetricRpcCallResponse::IcError(_) | MetricRpcCallResponse::MaxResponseSizeExceeded => {
            // Don't record latency for IC errors
        }
    }
    add_metric_entry!(
        requests,
        (req_data.method.clone(), req_data.host.clone()),
        1
    );
    add_metric_entry!(
        responses,
        (req_data.method.clone(), req_data.host.clone(), response),
        1
    );
}

struct MetricData {
    method: MetricRpcMethod,
    host: MetricRpcHost,
    request_id: Id,
    start_ns: u64,
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

pub fn charging_policy_with_collateral(
) -> ChargeCaller<impl Fn(&IcHttpRequest, u128) -> u128 + Clone> {
    let charge_caller = if read_state(|s| s.is_demo_mode_active()) {
        |_request: &IcHttpRequest, _request_cost| 0
    } else {
        |_request: &IcHttpRequest, request_cost| {
            let collateral_cycles = COLLATERAL_CYCLES_PER_NODE
                .saturating_mul(read_state(|s| s.get_num_subnet_nodes()) as u128);
            request_cost + collateral_cycles
        }
    };
    ChargeCaller::new(charge_caller)
}
