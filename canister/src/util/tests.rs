use crate::util::hostname_from_url;

#[test]
fn test_hostname_from_url() {
    assert_eq!(
        hostname_from_url("https://example.com"),
        Some("example.com".to_string())
    );
    assert_eq!(
        hostname_from_url("https://example.com?k=v"),
        Some("example.com".to_string())
    );
    assert_eq!(
        hostname_from_url("https://example.com/{API_KEY}"),
        Some("example.com".to_string())
    );
    assert_eq!(
        hostname_from_url("https://example.com/path/{API_KEY}"),
        Some("example.com".to_string())
    );
    assert_eq!(
        hostname_from_url("https://example.com/path/{API_KEY}?k=v"),
        Some("example.com".to_string())
    );
    assert_eq!(hostname_from_url("https://{API_KEY}"), None);
    assert_eq!(hostname_from_url("https://{API_KEY}/path/"), None);
    assert_eq!(hostname_from_url("https://{API_KEY}.com"), None);
    assert_eq!(hostname_from_url("https://{API_KEY}.com/path/"), None);
    assert_eq!(hostname_from_url("https://example.{API_KEY}"), None);
    assert_eq!(hostname_from_url("https://example.{API_KEY}/path/"), None);
}
