use sol_rpc_int_tests::Setup;
use sol_rpc_types::Provider;

#[tokio::test]
async fn should_get_providers() {
    let setup = Setup::new().await;
    let client = setup.client();

    let response = client.get_providers().await;

    assert_eq!(response, vec![]);

    setup.drop().await;
}

#[tokio::test]
async fn should_get_service_provider_map() {
    let setup = Setup::new().await;
    let client = setup.client();
    let providers = client.get_providers().await;
    let service_provider_map = client.get_service_provider_map().await;
    assert_eq!(providers.len(), service_provider_map.len());

    for (service, provider_id) in service_provider_map {
        let found_provider = providers
            .iter()
            .find(|p| p.provider_id == provider_id)
            .unwrap();
        assert!(matches!(
            found_provider,
            Some(Provider { alias: service, .. })
        ));
    }

    setup.drop().await;
}
