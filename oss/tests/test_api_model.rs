// Integration tests for API model types

use alibabacloud_oss_sdk_rust_v2_api_model::{OperationInput, OperationOutput, BodyContent};

#[test]
fn test_operation_input_default() {
    let input = OperationInput::default();
    assert!(input.bucket.is_none());
    assert!(input.key.is_none());
}

#[test]
fn test_operation_input_with_bucket() {
    let input = OperationInput {
        bucket: Some("test-bucket".to_string()),
        ..Default::default()
    };
    assert_eq!(input.bucket, Some("test-bucket".to_string()));
}

#[test]
fn test_operation_input_with_key() {
    let input = OperationInput {
        key: Some("test-key.txt".to_string()),
        ..Default::default()
    };
    assert_eq!(input.key, Some("test-key.txt".to_string()));
}

#[test]
fn test_operation_input_complete() {
    let input = OperationInput {
        bucket: Some("my-bucket".to_string()),
        key: Some("path/to/file.txt".to_string()),
        ..Default::default()
    };
    assert!(input.bucket.is_some());
    assert!(input.key.is_some());
}

#[test]
fn test_operation_output_default() {
    let output = OperationOutput::default();
    assert_eq!(output.status_code, 0);
}

#[test]
fn test_operation_output_with_status() {
    let output = OperationOutput {
        status_code: 200,
        ..Default::default()
    };
    assert_eq!(output.status_code, 200);
}

#[test]
fn test_operation_output_with_headers() {
    let mut headers = http::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    
    let output = OperationOutput {
        headers,
        ..Default::default()
    };
    assert!(!output.headers.is_empty());
}

#[test]
fn test_body_content_text_variant() {
    let content = BodyContent::Text {
        data: "Hello".to_string(),
        md5: None,
    };
    
    match content {
        BodyContent::Text { data, .. } => {
            assert_eq!(data, "Hello");
        }
        _ => panic!("Expected Text variant"),
    }
}

#[test]
fn test_body_content_bytes_variant() {
    let data = vec![1, 2, 3, 4, 5];
    let content = BodyContent::Bytes {
        data: data.clone(),
        md5: None,
    };
    
    match content {
        BodyContent::Bytes { data: d, .. } => {
            assert_eq!(d, data);
        }
        _ => panic!("Expected Bytes variant"),
    }
}

#[test]
fn test_operation_input_clone() {
    let input1 = OperationInput {
        bucket: Some("bucket".to_string()),
        key: Some("key".to_string()),
        ..Default::default()
    };
    
    let input2 = input1.clone();
    assert_eq!(input1.bucket, input2.bucket);
    assert_eq!(input1.key, input2.key);
}
