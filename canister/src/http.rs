use crate::{
    constants::{CONTENT_TYPE_HEADER_LOWERCASE, CONTENT_TYPE_VALUE},
    state::{read_state, State},
    types::ResolvedRpcService,
};
use canhttp::{CyclesAccounting, CyclesAccountingError, CyclesChargingPolicy};
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};
use num_traits::ToPrimitive;
use sol_rpc_types::{HttpOutcallError, Mode, ProviderError, RpcError, RpcResult, ValidationError};
use tower::{BoxError, Service, ServiceBuilder};

pub fn json_rpc_request_arg(
    service: ResolvedRpcService,
    json_rpc_payload: &str,
    max_response_bytes: u64,
) -> RpcResult<CanisterHttpRequestArgument> {
    let api = service.api(&read_state(|s| s.get_override_provider()))?;
    let mut request_headers = api.headers.unwrap_or_default();
    if !request_headers
        .iter()
        .any(|header| header.name.to_lowercase() == CONTENT_TYPE_HEADER_LOWERCASE)
    {
        request_headers.push(HttpHeader {
            name: CONTENT_TYPE_HEADER_LOWERCASE.to_string(),
            value: CONTENT_TYPE_VALUE.to_string(),
        });
    }
    Ok(CanisterHttpRequestArgument {
        url: api.url,
        max_response_bytes: Some(max_response_bytes),
        method: HttpMethod::POST,
        headers: request_headers,
        body: Some(json_rpc_payload.as_bytes().to_vec()),
        transform: Some(TransformContext::from_name(
            "__transform_json_rpc".to_string(),
            vec![],
        )),
    })
}

pub async fn json_rpc_request(
    service: ResolvedRpcService,
    rpc_method: &str,
    json_rpc_payload: &str,
    max_response_bytes: u64,
) -> RpcResult<HttpResponse> {
    let request = json_rpc_request_arg(service, json_rpc_payload, max_response_bytes)?;
    http_request(rpc_method, request).await
}

pub async fn http_request(
    rpc_method: &str,
    request: CanisterHttpRequestArgument,
) -> RpcResult<HttpResponse> {
    let url = request.url.clone();
    let parsed_url = match url::Url::parse(&url) {
        Ok(url) => url,
        Err(_) => {
            return Err(ValidationError::Custom(format!("Error parsing URL: {}", url)).into())
        }
    };
    let _host = match parsed_url.host_str() {
        Some(host) => host,
        None => {
            return Err(ValidationError::Custom(format!(
                "Error parsing hostname from URL: {}",
                url
            ))
            .into())
        }
    };
    http_client(rpc_method).call(request).await
}

pub fn http_client(
    _rpc_method: &str,
) -> impl Service<CanisterHttpRequestArgument, Response = HttpResponse, Error = RpcError> {
    let cycles_accounting = read_state(|s| {
        CyclesAccounting::new(
            s.get_num_subnet_nodes(),
            ChargingPolicyWithCollateral::new_from_state(s),
        )
    });
    ServiceBuilder::new()
        .map_err(map_error)
        .filter(cycles_accounting)
        .service(canhttp::Client)
}

fn map_error(e: BoxError) -> RpcError {
    if let Some(charging_error) = e.downcast_ref::<CyclesAccountingError>() {
        return match charging_error {
            CyclesAccountingError::InsufficientCyclesError { expected, received } => {
                ProviderError::TooFewCycles {
                    expected: *expected,
                    received: *received,
                }
                .into()
            }
        };
    }
    if let Some(canhttp::IcError { code, message }) = e.downcast_ref::<canhttp::IcError>() {
        return HttpOutcallError::IcError {
            code: *code,
            message: message.clone(),
        }
        .into();
    }
    RpcError::ProviderError(ProviderError::InvalidRpcConfig(format!(
        "Unknown error: {}",
        e
    )))
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
            0,
        )
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

pub fn canonicalize_json_rpc_response(args: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: args.response.status,
        body: canonicalize_json(&args.response.body).unwrap_or(args.response.body),
        // Remove headers (which may contain a timestamp) for consensus
        headers: vec![],
    }
}

pub fn get_http_response_status(status: candid::Nat) -> u16 {
    status.0.to_u16().unwrap_or(u16::MAX)
}

pub fn get_http_response_body(response: HttpResponse) -> Result<String, RpcError> {
    String::from_utf8(response.body).map_err(|e| {
        HttpOutcallError::InvalidHttpJsonRpcResponse {
            status: get_http_response_status(response.status),
            body: "".to_string(),
            parsing_error: Some(format!("{e}")),
        }
        .into()
    })
}

fn canonicalize_json(json_rpc_response: &[u8]) -> Option<Vec<u8>> {
    let json = serde_json::from_slice::<serde_json::Value>(json_rpc_response).ok()?;
    serde_json::to_vec(&json).ok()
}
