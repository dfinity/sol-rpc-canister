use canhttp::{
    http::{
        json::{JsonRequestConversionError, JsonResponseConversionError},
        FilterNonSuccessfulHttpResponseError, HttpRequestConversionError,
        HttpResponseConversionError,
    },
    CyclesAccountingError, HttpsOutcallError, IcError,
};
use derive_more::From;
use sol_rpc_types::{HttpOutcallError, ProviderError, RpcError};
use thiserror::Error;

#[derive(Clone, Debug, Error, From)]
pub enum HttpClientError {
    #[error("IC error: {0}")]
    IcError(IcError),
    #[error("unknown error (most likely sign of a bug): {0}")]
    NotHandledError(String),
    #[error("cycles accounting error: {0}")]
    CyclesAccountingError(CyclesAccountingError),
    #[error("HTTP response was not successful: {0}")]
    UnsuccessfulHttpResponse(FilterNonSuccessfulHttpResponseError<Vec<u8>>),
    #[error("Error converting response to JSON: {0}")]
    InvalidJsonResponse(JsonResponseConversionError),
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

impl From<HttpClientError> for RpcError {
    fn from(error: HttpClientError) -> Self {
        match error {
            HttpClientError::IcError(IcError { code, message }) => {
                RpcError::HttpOutcallError(HttpOutcallError::IcError { code, message })
            }
            HttpClientError::NotHandledError(e) => RpcError::ValidationError(e),
            HttpClientError::CyclesAccountingError(
                CyclesAccountingError::InsufficientCyclesError { expected, received },
            ) => RpcError::ProviderError(ProviderError::TooFewCycles { expected, received }),
            HttpClientError::InvalidJsonResponse(
                JsonResponseConversionError::InvalidJsonResponse {
                    status,
                    body,
                    parsing_error,
                },
            ) => RpcError::HttpOutcallError(HttpOutcallError::InvalidHttpJsonRpcResponse {
                status,
                body,
                parsing_error: Some(parsing_error),
            }),
            HttpClientError::UnsuccessfulHttpResponse(
                FilterNonSuccessfulHttpResponseError::UnsuccessfulResponse(response),
            ) => RpcError::HttpOutcallError(HttpOutcallError::InvalidHttpJsonRpcResponse {
                status: response.status().as_u16(),
                body: String::from_utf8_lossy(response.body()).to_string(),
                parsing_error: None,
            }),
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
            | HttpClientError::InvalidJsonResponse(_) => false,
        }
    }
}
