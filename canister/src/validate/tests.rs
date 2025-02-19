use super::*;

#[test]
pub fn test_validate_api_key() {
    assert_eq!(validate_api_key("abc"), Ok(()));
    assert_eq!(
        validate_api_key("?a=b"),
        Err("Invalid character in API key")
    );
    assert_eq!(validate_api_key("/"), Err("Invalid character in API key"));
    assert_eq!(
        validate_api_key("abc/def"),
        Err("Invalid character in API key")
    );
    assert_eq!(
        validate_api_key("../def"),
        Err("Invalid character in API key")
    );
    assert_eq!(
        validate_api_key("abc/:key"),
        Err("Invalid character in API key")
    );
    assert_eq!(
        validate_api_key(
            std::iter::repeat("a")
                .take(513)
                .collect::<String>()
                .as_str()
        ),
        Err("API key must be <= 512 bytes")
    );
}
