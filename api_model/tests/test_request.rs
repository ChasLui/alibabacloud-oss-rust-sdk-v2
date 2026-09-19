use std::collections::HashMap;

use alibabacloud_oss_sdk_rust_v2_api_model::OssRequestModel;

// #[add_field]
#[derive(Default, OssRequestModel)]
struct Request {
    #[field(type = "header", rename = "x-header1")]
    header1: i32,

    #[field(type = "query")]
    param1: String,

    #[field(type = "query", rename = "param-2")]
    param2: Option<bool>,

    #[field(type = "header")]
    header2: Option<String>,

    #[field(type = "header")]
    none: Option<String>,

    _normal_field: i32,

    common: Common,
}

#[derive(Default)]
struct Common {
    headers: HashMap<String, String>,
    parameters: HashMap<String, String>,
}

#[test]
fn test_request_macro() {
    let mut request = Request {
        header1: 123,
        param1: "Hello".to_string(),
        param2: Some(false),
        header2: Some("World".to_string()),
        none: None,
        _normal_field: 6,
        ..Default::default()
    };

    // assert init status
    assert_eq!(
        [("x-header1", "123"), ("header2", "World")]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<std::collections::HashMap<String, String>>(),
        request.header_map()
    );
    assert_eq!(
        [("param1", "Hello"), ("param-2", "false")]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<std::collections::HashMap<String, String>>(),
        request.query_map()
    );

    // add header and map
    request.add_header("new-header", "new-value");
    request.add_query("new-param", "new-value");
    assert_eq!(
        [
            ("x-header1", "123"),
            ("header2", "World"),
            ("new-header", "new-value")
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<std::collections::HashMap<String, String>>(),
        request.header_map()
    );
    assert_eq!(
        [
            ("param1", "Hello"),
            ("param-2", "false"),
            ("new-param", "new-value")
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<std::collections::HashMap<String, String>>(),
        request.query_map()
    );
}

/// Non-`Option` fields are skipped when empty/zero, mirroring Go's
/// `isEmptyValue` check in `marshalInput`. Prior behavior inserted `""`/`0`.
#[test]
fn test_request_macro_skips_empty_non_option_values() {
    let request = Request::default();
    let headers = request.header_map();
    let queries = request.query_map();

    assert_eq!(
        headers.get("x-header1").map(String::as_str),
        None,
        "zero i32 must not be sent"
    );
    assert_eq!(
        queries.get("param1").map(String::as_str),
        None,
        "empty String must not be sent"
    );

    // ...but a real value still goes through.
    let request = Request {
        header1: 7,
        param1: "x".to_string(),
        ..Default::default()
    };
    assert_eq!(
        request.header_map().get("x-header1").map(String::as_str),
        Some("7")
    );
    assert_eq!(
        request.query_map().get("param1").map(String::as_str),
        Some("x")
    );
}
