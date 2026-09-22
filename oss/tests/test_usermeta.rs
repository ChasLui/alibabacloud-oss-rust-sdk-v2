//! Tests for `usermeta` header mapping (x-oss-meta-*), matching the Go SDK's
//! `input/output:"header,x-oss-meta-,usermeta"` behavior: requests expand a
//! map into `prefix+key` headers, responses capture every `prefix*` header into
//! the map keyed by the remainder.

use std::collections::HashMap;

use alibabacloud_oss_sdk_rust_v2::api::object::{
    GetObjectResult, HeadObjectResult, PutObjectRequest, PutSymlinkRequest,
};

/// Request side: map entries become individual `x-oss-meta-<key>` headers.
#[test]
fn put_object_metadata_expands_to_meta_headers() {
    let mut request = PutObjectRequest {
        bucket: "b".to_string(),
        key: "k".to_string(),
        ..Default::default()
    };
    request.metadata = HashMap::from([
        ("author".to_string(), "alice".to_string()),
        ("version".to_string(), "1.0".to_string()),
    ]);

    let headers = request.header_map();
    assert_eq!(
        headers.get("x-oss-meta-author").map(String::as_str),
        Some("alice")
    );
    assert_eq!(
        headers.get("x-oss-meta-version").map(String::as_str),
        Some("1.0")
    );
    // the bare map name must not leak into the headers
    assert!(!headers.contains_key("metadata"));
}

/// Symlink requests use the same usermeta expansion (no `usermeta` keyword
/// needed on the map; the prefix comes from `rename`).
#[test]
fn put_symlink_metadata_expands_to_meta_headers() {
    let mut request = PutSymlinkRequest {
        bucket: "b".to_string(),
        key: "link".to_string(),
        target: Some("target".to_string()),
        ..Default::default()
    };
    request.metadata = HashMap::from([("tag".to_string(), "blue".to_string())]);

    let headers = request.header_map();
    assert_eq!(
        headers.get("x-oss-meta-tag").map(String::as_str),
        Some("blue")
    );
}

/// Result side: every `x-oss-meta-*` response header lands in the map under the
/// suffix, case-insensitively.
#[test]
fn head_object_result_captures_meta_headers() {
    let mut result = HeadObjectResult::default();
    result.update_result(&mock_output(HashMap::from([
        ("x-oss-meta-Author".to_string(), "alice".to_string()),
        ("X-Oss-Meta-Version".to_string(), "1.0".to_string()),
        ("ETag".to_string(), "\"abc\"".to_string()),
    ])));

    assert_eq!(
        result.metadata.get("author").map(String::as_str),
        Some("alice")
    );
    assert_eq!(
        result.metadata.get("version").map(String::as_str),
        Some("1.0")
    );
    // non-meta headers are not captured
    assert_eq!(result.metadata.len(), 2);
}

#[test]
fn get_object_result_captures_meta_headers() {
    let mut result = GetObjectResult::default();
    result.update_result(&mock_output(HashMap::from([(
        "x-oss-meta-owner".to_string(),
        "bob".to_string(),
    )])));

    assert_eq!(
        result.metadata.get("owner").map(String::as_str),
        Some("bob")
    );
}

fn mock_output(headers: HashMap<String, String>) -> alibabacloud_oss_sdk_rust_v2::OperationOutput {
    alibabacloud_oss_sdk_rust_v2::OperationOutput {
        headers,
        ..Default::default()
    }
}
