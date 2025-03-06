#[test]
fn should_get_slot() {
    let solana_rpc_client = solana_client::rpc_client::RpcClient::new("http://localhost:8899");
    let slot = solana_rpc_client.get_slot().expect("Failed to get_slot");
    assert!(slot > 0);
}
