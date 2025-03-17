use crate::rpc_client::sol_rpc::{HttpResponsePayload, ResponseTransform};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Slot(pub u64);

impl HttpResponsePayload for Slot {
    fn response_transform() -> Option<ResponseTransform> {
        Some(ResponseTransform::Slot)
    }
}
