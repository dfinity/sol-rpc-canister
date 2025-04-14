use crate::rpc_client::GetSlotRequest;
use sol_rpc_types::GetSlotParams;

mod request_serialization_tests {
    use super::*;
    use sol_rpc_types::{GetSlotRpcConfig, RpcSources, SolanaCluster};

    #[test]
    fn should_serialize_get_slot_request() {
        let request = GetSlotRequest::get_slot(
            RpcSources::Default(SolanaCluster::Mainnet),
            GetSlotRpcConfig::default(),
            GetSlotParams::default(),
        )
        .unwrap();

        let serialized = serde_json::to_vec(&request.request);
    }
}
