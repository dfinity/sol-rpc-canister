use crate::DummyRequest;

#[test]
fn should_deser() {
    let request = DummyRequest { input: "Hello".to_string() };
    let encoded = candid::encode_one(&request).unwrap();
    let decoded: DummyRequest = candid::decode_one(&encoded).unwrap();
    assert_eq!(request, decoded);

    let response = DummyRequest { input: "Hello world!".to_string() };
    let encoded = candid::encode_one(&response).unwrap();
    let decoded: DummyRequest = candid::decode_one(&encoded).unwrap();
    assert_eq!(response, decoded);
}
