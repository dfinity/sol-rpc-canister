use sol_rpc_int_tests::Setup;

#[tokio::test]
async fn should_get_providers() {
    let setup = Setup::new().await;
    let client = setup.client();

    let response = client.get_providers().await;

    assert_eq!(response, vec![]);

    setup.drop().await;
}
