use sol_rpc_int_tests::Setup;

#[tokio::test]
async fn should_get_providers_and_get_service_provider_map_be_consistent() {
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
        assert_eq!(found_provider.alias, Some(service));
    }

    setup.drop().await;
}
