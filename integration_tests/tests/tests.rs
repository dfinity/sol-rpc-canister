use sol_rpc_int_tests::Setup;
use sol_rpc_types::{DummyRequest, DummyResponse};

#[tokio::test]
async fn should_greet() {
    let setup = Setup::new().await;
    let client = setup.client();

    let response = client
        .greet(DummyRequest {
            input: "world".to_string(),
        })
        .await;

    assert_eq!(
        response,
        DummyResponse {
            output: "Hello, world!".to_string()
        }
    );

    setup.drop().await;
}
