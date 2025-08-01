use canhttp::http::json::JsonRpcRequest;
use ic_cdk::api::call::RejectionCode;
use pocket_ic::common::rest::{
    CanisterHttpHeader, CanisterHttpMethod, CanisterHttpReject, CanisterHttpReply,
    CanisterHttpRequest, CanisterHttpResponse,
};
use serde_json::Value;
use std::{collections::BTreeSet, str::FromStr};
use url::{Host, Url};

pub struct MockOutcallBody(pub Vec<u8>);

impl From<&serde_json::Value> for MockOutcallBody {
    fn from(value: &serde_json::Value) -> Self {
        value.to_string().into()
    }
}
impl From<serde_json::Value> for MockOutcallBody {
    fn from(value: serde_json::Value) -> Self {
        Self::from(serde_json::to_vec(&value).unwrap())
    }
}
impl From<String> for MockOutcallBody {
    fn from(string: String) -> Self {
        string.as_bytes().to_vec().into()
    }
}
impl<'a> From<&'a str> for MockOutcallBody {
    fn from(string: &'a str) -> Self {
        string.to_string().into()
    }
}
impl From<Vec<u8>> for MockOutcallBody {
    fn from(bytes: Vec<u8>) -> Self {
        MockOutcallBody(bytes)
    }
}

#[derive(Clone, Debug)]
pub struct MockOutcallBuilder(MockOutcall);

impl MockOutcallBuilder {
    pub fn new(status: u16, body: impl Into<MockOutcallBody>) -> Self {
        Self(MockOutcall {
            method: None,
            url: None,
            host: None,
            request_headers: None,
            request_body: None,
            max_response_bytes: None,
            response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
                status,
                headers: vec![],
                body: body.into().0,
            }),
        })
    }

    pub fn new_error(code: RejectionCode, message: impl ToString) -> Self {
        Self(MockOutcall {
            method: None,
            url: None,
            host: None,
            request_headers: None,
            request_body: None,
            max_response_bytes: None,
            response: CanisterHttpResponse::CanisterHttpReject(CanisterHttpReject {
                reject_code: code as u64,
                message: message.to_string(),
            }),
        })
    }

    pub fn with_method(mut self, method: CanisterHttpMethod) -> Self {
        self.0.method = Some(method);
        self
    }

    pub fn with_url(mut self, url: impl ToString) -> Self {
        self.0.url = Some(url.to_string());
        self
    }

    pub fn with_host(mut self, host: &str) -> Self {
        self.0.host = Some(Host::parse(host).expect("BUG: invalid host for a URL"));
        self
    }

    pub fn with_request_headers(mut self, headers: Vec<(impl ToString, impl ToString)>) -> Self {
        self.0.request_headers = Some(
            headers
                .into_iter()
                .map(|(name, value)| CanisterHttpHeader {
                    name: name.to_string(),
                    value: value.to_string(),
                })
                .collect(),
        );
        self
    }

    pub fn with_raw_request_body(self, body: &str) -> Self {
        self.with_request_body(serde_json::from_str(body).unwrap())
    }

    pub fn with_request_body(mut self, body: serde_json::Value) -> Self {
        self.0.request_body = Some(serde_json::from_value(body).unwrap());
        self
    }

    pub fn with_max_response_bytes(mut self, max_response_bytes: u64) -> Self {
        self.0.max_response_bytes = Some(max_response_bytes);
        self
    }

    pub fn build(self) -> MockOutcall {
        self.0
    }
}

impl From<MockOutcallBuilder> for MockOutcall {
    fn from(builder: MockOutcallBuilder) -> Self {
        builder.build()
    }
}

#[derive(Clone, Debug)]
pub struct MockOutcall {
    pub method: Option<CanisterHttpMethod>,
    pub url: Option<String>,
    pub host: Option<Host>,
    pub request_headers: Option<Vec<CanisterHttpHeader>>,
    pub request_body: Option<JsonRpcRequest<Value>>,
    pub max_response_bytes: Option<u64>,
    pub response: CanisterHttpResponse,
}

impl MockOutcall {
    pub fn assert_matches(&self, request: &CanisterHttpRequest) {
        let req_url = Url::from_str(&request.url).expect("BUG: invalid URL");
        if let Some(ref url) = self.url {
            let mock_url = Url::from_str(url).unwrap();
            assert_eq!(mock_url, req_url);
        }
        if let Some(ref host) = self.host {
            assert_eq!(
                host,
                &req_url.host().expect("BUG: missing host in URL").to_owned()
            );
        }
        if let Some(ref method) = self.method {
            assert_eq!(method, &request.http_method);
        }
        if let Some(ref headers) = self.request_headers {
            assert_eq!(
                headers.iter().collect::<BTreeSet<_>>(),
                request.headers.iter().collect::<BTreeSet<_>>()
            );
        }
        if let Some(ref expected_body) = self.request_body {
            let actual_body: JsonRpcRequest<Value> = serde_json::from_slice(&request.body)
                .expect("BUG: failed to parse JSON request body");
            assert_eq!(expected_body, &actual_body);
        }
        if let Some(max_response_bytes) = self.max_response_bytes {
            assert_eq!(Some(max_response_bytes), request.max_response_bytes);
        }
    }
}
