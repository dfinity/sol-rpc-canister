use super::*;

mod normalization_tests {
    use super::*;

    #[test]
    fn should_normalize_raw_response() {
        assert_normalized_equal(
            &ResponseTransform::Raw,
            r#"{"k1":"v1","k2":"v2"}"#,
            r#"{"k1":"v1","k2":"v2"}"#,
        );
        assert_normalized_equal(
            &ResponseTransform::Raw,
            r#"{"k1":"v1","k2":"v2"}"#,
            r#"{"k2":"v2","k1":"v1"}"#,
        );
        assert_normalized_not_equal(
            &ResponseTransform::Raw,
            r#"{"k1":"v1","k2":"v2"}"#,
            r#"{"k1":"v1","k3":"v3"}"#,
        );
    }

    #[test]
    fn should_normalize_get_slot_response() {
        assert_normalized_equal(&ResponseTransform::GetSlot, "329535108", "329535108");
        assert_normalized_equal(&ResponseTransform::GetSlot, "329535108", "329535116");
        assert_normalized_not_equal(&ResponseTransform::GetSlot, "329535108", "329535128");
    }

    fn normalize_response(transform: &ResponseTransform, response: &str) -> String {
        fn add_envelope(reply: &str) -> Vec<u8> {
            format!("{{\"jsonrpc\": \"2.0\", \"id\": 1, \"result\": {}}}", reply).into_bytes()
        }
        let mut response = add_envelope(response);
        transform.apply(&mut response);
        String::from_utf8(response).unwrap()
    }

    fn assert_normalized_equal(transform: &ResponseTransform, left: &str, right: &str) {
        assert_eq!(
            normalize_response(transform, left),
            normalize_response(transform, right)
        );
    }

    fn assert_normalized_not_equal(transform: &ResponseTransform, left: &str, right: &str) {
        assert_ne!(
            normalize_response(transform, left),
            normalize_response(transform, right)
        );
    }
}
