use std::collections::HashMap;

use alibabacloud_oss_sdk_rust_v2_api_model::OssResultModel;

/// The shape `update_result` reads from.
///
/// The macro names `OperationOutput` from the caller's scope rather than from a
/// crate of its own, so the test only needs a type with the two fields the
/// generated code touches. Depending on the SDK crate here would make this
/// crate's tests depend on its own consumer.
#[derive(Debug, Default)]
pub struct OperationOutput {
    pub status: http::StatusCode,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Default)]
pub struct ResultCommon {
    pub status: http::StatusCode,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Default, OssResultModel)]
pub struct Result {
    #[field(type = "header", rename = "x-version-id")]
    pub version_id: Option<String>,

    #[field(type = "header")]
    pub another_version_id: Option<String>,

    pub common: ResultCommon,
}

#[test]
fn test_result_macro() {
    let mut result = Result::default();

    let headers = [
        ("x-version-id", "123"),
        ("x-another-version-id", "456"),
        ("another_version_id", "789"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<std::collections::HashMap<String, String>>();

    let output = OperationOutput {
        status: http::StatusCode::OK,
        headers,
    };

    result.update_result(&output);

    assert_eq!(result.version_id, Some("123".to_string()));
    assert_ne!(result.another_version_id, Some("456".to_string()));
    assert_eq!(result.another_version_id, Some("789".to_string()));

    assert_eq!(result.common.headers.get("x-version-id").unwrap(), "123");
    assert_eq!(
        result.common.headers.get("x-another-version-id").unwrap(),
        "456"
    );
    assert_eq!(
        result.common.headers.get("another_version_id").unwrap(),
        "789"
    );
}
