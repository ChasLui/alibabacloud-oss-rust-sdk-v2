// Integration tests for constants and defaults

use alibabacloud_oss_sdk_rust_v2::{
    DEFAULT_USER_AGENT, HEADER_OSS_PREFIX, HTTP_HEADER_CONTENT_TYPE,
    URL_STYLE_VIRTUAL_HOSTED, URL_STYLE_PATH, URL_STYLE_CNAME,
};

#[test]
fn test_default_user_agent() {
    assert!(!DEFAULT_USER_AGENT.is_empty());
    assert!(DEFAULT_USER_AGENT.contains("aliyun-oss-sdk-rust"));
}

#[test]
fn test_header_oss_prefix() {
    assert_eq!(HEADER_OSS_PREFIX, "x-oss-");
}

#[test]
fn test_http_header_content_type() {
    assert_eq!(HTTP_HEADER_CONTENT_TYPE, "Content-Type");
}

#[test]
fn test_url_style_constants() {
    assert_eq!(URL_STYLE_VIRTUAL_HOSTED, "virtual-hosted");
    assert_eq!(URL_STYLE_PATH, "path");
    assert_eq!(URL_STYLE_CNAME, "cname");
}

#[test]
fn test_constants_are_static() {
    // Constants should be accessible without initialization
    let _ua = DEFAULT_USER_AGENT;
    let _prefix = HEADER_OSS_PREFIX;
    let _content_type = HTTP_HEADER_CONTENT_TYPE;
}

#[test]
fn test_url_style_values_different() {
    assert_ne!(URL_STYLE_VIRTUAL_HOSTED, URL_STYLE_PATH);
    assert_ne!(URL_STYLE_PATH, URL_STYLE_CNAME);
    assert_ne!(URL_STYLE_VIRTUAL_HOSTED, URL_STYLE_CNAME);
}

#[test]
fn test_header_prefix_format() {
    // OSS headers should start with the prefix
    assert!(HEADER_OSS_PREFIX.ends_with('-'));
}

#[test]
fn test_constants_immutability() {
    // Constants should be immutable
    let ua_copy = DEFAULT_USER_AGENT;
    let ua_copy2 = DEFAULT_USER_AGENT;
    assert_eq!(ua_copy, ua_copy2);
}

#[test]
fn test_content_type_header_case() {
    // Content-Type header should have proper casing
    assert_eq!(HTTP_HEADER_CONTENT_TYPE, "Content-Type");
}

#[test]
fn test_user_agent_format() {
    // User agent should follow standard format
    let ua = DEFAULT_USER_AGENT;
    assert!(ua.chars().all(|c| c.is_ascii()));
}
