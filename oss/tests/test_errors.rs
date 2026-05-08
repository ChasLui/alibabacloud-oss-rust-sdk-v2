// Integration tests for error types

use std::error::Error;
use alibabacloud_oss_sdk_rust_v2::{
    ServiceError, ClientError, OperationError, DeserializationError, 
    SerializationError, CanceledError,
    new_err_param_required, new_err_param_invalid, new_err_param_null,
    new_err_param_type_not_support,
};
use http::StatusCode;

#[test]
fn test_service_error_creation() {
    let error = ServiceError {
        code: "NoSuchBucket".to_string(),
        message: "The specified bucket does not exist".to_string(),
        request_id: "request-123".to_string(),
        ec: "0003-00000001".to_string(),
        status_code: StatusCode::NOT_FOUND,
        snapshot: vec![],
        timestamp: None,
        request_target: "/bucket".to_string(),
        headers: http::HeaderMap::new(),
    };
    
    assert_eq!(error.code, "NoSuchBucket");
    assert_eq!(error.message, "The specified bucket does not exist");
    assert_eq!(error.request_id, "request-123");
    assert_eq!(error.status_code, StatusCode::NOT_FOUND);
}

#[test]
fn test_service_error_display_format() {
    let error = ServiceError {
        code: "AccessDenied".to_string(),
        message: "Access Denied".to_string(),
        request_id: "req-456".to_string(),
        ec: "0002-00000001".to_string(),
        status_code: StatusCode::FORBIDDEN,
        snapshot: vec![],
        timestamp: None,
        request_target: "/bucket/object".to_string(),
        headers: http::HeaderMap::new(),
    };
    
    let display = format!("{}", error);
    assert!(display.contains("403"));
    assert!(display.contains("AccessDenied"));
    assert!(display.contains("Access Denied"));
    assert!(display.contains("req-456"));
}

#[test]
fn test_service_error_http_status_code() {
    let error = ServiceError {
        code: "BadRequest".to_string(),
        message: "Bad Request".to_string(),
        request_id: "req-789".to_string(),
        ec: "0001-00000001".to_string(),
        status_code: StatusCode::BAD_REQUEST,
        snapshot: vec![],
        timestamp: None,
        request_target: "/".to_string(),
        headers: http::HeaderMap::new(),
    };
    
    assert_eq!(error.http_status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_service_error_error_code() {
    let error = ServiceError {
        code: "InvalidArgument".to_string(),
        message: "Invalid argument".to_string(),
        request_id: "req-abc".to_string(),
        ec: "0001-00000002".to_string(),
        status_code: StatusCode::BAD_REQUEST,
        snapshot: vec![],
        timestamp: None,
        request_target: "/".to_string(),
        headers: http::HeaderMap::new(),
    };
    
    assert_eq!(error.error_code(), "InvalidArgument");
}

#[test]
fn test_client_error_with_source() {
    let service_error = ServiceError {
        code: "ServiceError".to_string(),
        message: "Service failed".to_string(),
        request_id: "req-xyz".to_string(),
        ec: "0000-00000001".to_string(),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        snapshot: vec![],
        timestamp: None,
        request_target: "/".to_string(),
        headers: http::HeaderMap::new(),
    };
    
    let client_error = ClientError {
        code: "ClientError".to_string(),
        message: "Client operation failed".to_string(),
        err: Box::new(service_error),
    };
    
    assert!(client_error.source().is_some());
    let source_msg = format!("{}", client_error.source().unwrap());
    assert!(source_msg.contains("ServiceError"));
}

#[test]
fn test_operation_error_chain() {
    let service_error = ServiceError {
        code: "ServiceError".to_string(),
        message: "Service failed".to_string(),
        request_id: "req-op".to_string(),
        ec: "0000-00000001".to_string(),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        snapshot: vec![],
        timestamp: None,
        request_target: "/bucket".to_string(),
        headers: http::HeaderMap::new(),
    };
    
    let client_error = ClientError {
        code: "ClientError".to_string(),
        message: "Client failed".to_string(),
        err: Box::new(service_error),
    };
    
    let operation_error = OperationError {
        name: "PutObject".to_string(),
        err: Box::new(client_error),
    };
    
    assert_eq!(operation_error.operation(), "PutObject");
    assert!(format!("{}", operation_error).contains("PutObject"));
}

#[test]
fn test_deserialization_error_with_snapshot() {
    let inner_error = std::io::Error::new(std::io::ErrorKind::Other, "parse error");
    let snapshot = vec![1, 2, 3, 4, 5];
    
    let error = DeserializationError {
        err: Box::new(inner_error),
        snapshot: snapshot.clone(),
    };
    
    assert_eq!(error.snapshot, snapshot);
    assert!(format!("{}", error).contains("deserialization failed"));
}

#[test]
fn test_serialization_error() {
    let inner_error = std::io::Error::new(std::io::ErrorKind::Other, "serialize error");
    
    let error = SerializationError {
        err: Box::new(inner_error),
    };
    
    assert!(format!("{}", error).contains("serialization failed"));
}

#[test]
fn test_canceled_error() {
    let inner_error = std::io::Error::new(std::io::ErrorKind::Other, "canceled");
    
    let error = CanceledError {
        err: Box::new(inner_error),
    };
    
    assert!(error.canceled_error());
    assert!(format!("{}", error).contains("canceled"));
}

#[test]
fn test_invalid_param_required() {
    let error = new_err_param_required("BucketName");
    
    assert_eq!(error.field(), "BucketName");
    assert!(format!("{}", error).contains("missing required field"));
    assert!(format!("{}", error).contains("BucketName"));
}

#[test]
fn test_invalid_param_invalid() {
    let error = new_err_param_invalid("ObjectKey");
    
    assert_eq!(error.field(), "ObjectKey");
    assert!(format!("{}", error).contains("invalid field"));
}

#[test]
fn test_invalid_param_null() {
    let error = new_err_param_null("Region");
    
    assert_eq!(error.field(), "Region");
    assert!(format!("{}", error).contains("null field"));
}

#[test]
fn test_invalid_param_type_not_support() {
    let error = new_err_param_type_not_support("UploadType");
    
    assert_eq!(error.field(), "UploadType");
    assert!(format!("{}", error).contains("type not support"));
}

#[test]
fn test_invalid_param_with_context() {
    let mut error = new_err_param_required("FieldName");
    error.set_context("ParentField".to_string());
    
    assert_eq!(error.field(), "ParentField.FieldName");
}

#[test]
fn test_service_error_different_status_codes() {
    let status_codes = vec![
        StatusCode::BAD_REQUEST,
        StatusCode::UNAUTHORIZED,
        StatusCode::FORBIDDEN,
        StatusCode::NOT_FOUND,
        StatusCode::INTERNAL_SERVER_ERROR,
        StatusCode::SERVICE_UNAVAILABLE,
    ];
    
    for status in status_codes {
        let error = ServiceError {
            code: format!("Error{}", status.as_u16()),
            message: format!("Error with status {}", status),
            request_id: "req-test".to_string(),
            ec: "0000-00000000".to_string(),
            status_code: status,
            snapshot: vec![],
            timestamp: None,
            request_target: "/".to_string(),
            headers: http::HeaderMap::new(),
        };
        
        assert_eq!(error.http_status_code(), status);
        assert!(format!("{}", error).contains(&status.as_u16().to_string()));
    }
}

#[test]
fn test_error_trait_implementations() {
    // Test that all error types implement Error trait
    let service_error = ServiceError {
        code: "TestError".to_string(),
        message: "Test".to_string(),
        request_id: "req".to_string(),
        ec: "0000".to_string(),
        status_code: StatusCode::BAD_REQUEST,
        snapshot: vec![],
        timestamp: None,
        request_target: "/".to_string(),
        headers: http::HeaderMap::new(),
    };
    
    let _: &dyn std::error::Error = &service_error;
    
    let client_error = ClientError {
        code: "Test".to_string(),
        message: "Test".to_string(),
        err: Box::new(service_error),
    };
    
    let _: &dyn std::error::Error = &client_error;
}
