use canhttp::{
    cycles::ChargeCallerError,
    http::{
        json::{
            ConsistentResponseIdFilterError, JsonRequestConversionError,
            JsonResponseConversionError,
        },
        FilterNonSuccessfulHttpResponseError, HttpRequestConversionError,
        HttpResponseConversionError,
    },
    HttpsOutcallError, IcError,
};
use derive_more::From;
use sol_rpc_types::{HttpOutcallError, LegacyRejectionCode, ProviderError, RpcError};
use thiserror::Error;

#[derive(Clone, Debug, Error, From)]
pub enum HttpClientError {
    #[error("IC error: {0}")]
    IcError(IcError),
    #[error("unknown error (most likely sign of a bug): {0}")]
    NotHandledError(String),
    #[error("cycles accounting error: {0}")]
    CyclesAccountingError(ChargeCallerError),
    #[error("HTTP response was not successful: {0}")]
    UnsuccessfulHttpResponse(FilterNonSuccessfulHttpResponseError<Vec<u8>>),
    #[error("Error converting response to JSON: {0}")]
    InvalidJsonResponse(JsonResponseConversionError),
    #[error("Invalid JSON-RPC response ID: {0}")]
    InvalidJsonResponseId(ConsistentResponseIdFilterError),
}

impl From<HttpRequestConversionError> for HttpClientError {
    fn from(value: HttpRequestConversionError) -> Self {
        HttpClientError::NotHandledError(value.to_string())
    }
}

impl From<HttpResponseConversionError> for HttpClientError {
    fn from(value: HttpResponseConversionError) -> Self {
        // Replica should return valid http::Response
        HttpClientError::NotHandledError(value.to_string())
    }
}

impl From<JsonRequestConversionError> for HttpClientError {
    fn from(value: JsonRequestConversionError) -> Self {
        HttpClientError::NotHandledError(value.to_string())
    }
}

impl TryFrom<HttpClientError> for RpcError {
    type Error = HttpClientError;

    fn try_from(error: HttpClientError) -> Result<Self, Self::Error> {
        match error {
            HttpClientError::IcError(IcError::CallRejected { code, message }) => {
                Ok(RpcError::HttpOutcallError(HttpOutcallError::IcError {
                    code: LegacyRejectionCode::from(code),
                    message,
                }))
            }
            e @ HttpClientError::IcError(IcError::InsufficientLiquidCycleBalance { .. }) => Err(e),
            HttpClientError::NotHandledError(e) => Ok(RpcError::ValidationError(e)),
            HttpClientError::CyclesAccountingError(
                ChargeCallerError::InsufficientCyclesError { expected, received },
            ) => Ok(RpcError::ProviderError(ProviderError::TooFewCycles {
                expected,
                received,
            })),
            HttpClientError::InvalidJsonResponse(
                JsonResponseConversionError::InvalidJsonResponse {
                    status,
                    body,
                    parsing_error,
                },
            ) => Ok(RpcError::HttpOutcallError(
                HttpOutcallError::InvalidHttpJsonRpcResponse {
                    status,
                    body,
                    parsing_error: Some(parsing_error),
                },
            )),
            HttpClientError::UnsuccessfulHttpResponse(
                FilterNonSuccessfulHttpResponseError::UnsuccessfulResponse(response),
            ) => Ok(RpcError::HttpOutcallError(
                HttpOutcallError::InvalidHttpJsonRpcResponse {
                    status: response.status().as_u16(),
                    body: String::from_utf8_lossy(response.body()).to_string(),
                    parsing_error: None,
                },
            )),
            HttpClientError::InvalidJsonResponseId(e) => {
                Ok(RpcError::ValidationError(e.to_string()))
            }
        }
    }
}

impl HttpsOutcallError for HttpClientError {
    fn is_response_too_large(&self) -> bool {
        match self {
            HttpClientError::IcError(e) => e.is_response_too_large(),
            HttpClientError::NotHandledError(_)
            | HttpClientError::CyclesAccountingError(_)
            | HttpClientError::UnsuccessfulHttpResponse(_)
            | HttpClientError::InvalidJsonResponseId(_)
            | HttpClientError::InvalidJsonResponse(_) => false,
        }
    }
}
