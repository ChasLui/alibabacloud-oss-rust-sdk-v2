// Integration tests for retry functionality

use alibabacloud_oss_sdk_rust_v2::retry::{Retryer, NopRetryer, Standard};
use std::io::{Error as IoError, ErrorKind};
use std::time::Duration;

#[test]
fn test_nop_retryer_max_attempts() {
    let retryer = NopRetryer::new();
    assert_eq!(retryer.max_attempts(), 1);
}

#[test]
fn test_nop_retryer_not_retryable() {
    let retryer = NopRetryer::new();
    let error = IoError::new(ErrorKind::Other, "test error");
    
    assert!(!retryer.is_error_retryable(&error));
}

#[test]
fn test_nop_retryer_retry_delay_error() {
    let retryer = NopRetryer::new();
    let error = IoError::new(ErrorKind::Other, "test error");
    
    let result = retryer.retry_delay(1, &error);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Not retrying"));
}

#[test]
fn test_standard_retryer_default() {
    let retryer = Standard::default();
    
    // Default max attempts should be 3
    assert_eq!(retryer.max_attempts(), 3);
}

#[test]
fn test_standard_retryer_custom_max_attempts() {
    let retryer = Standard::default().with_max_attempts(5);
    
    assert_eq!(retryer.max_attempts(), 5);
}

#[test]
fn test_standard_retryer_connection_error_retryable() {
    let retryer = Standard::default();
    
    // Connection errors should be retryable
    let connection_error = IoError::new(ErrorKind::ConnectionRefused, "connection refused");
    assert!(retryer.is_error_retryable(&connection_error));
    
    let connection_reset = IoError::new(ErrorKind::ConnectionReset, "connection reset");
    assert!(retryer.is_error_retryable(&connection_reset));
}

#[test]
fn test_standard_retryer_non_retryable_error() {
    let retryer = Standard::default();
    
    // Some errors should not be retryable
    let invalid_input = IoError::new(ErrorKind::InvalidInput, "invalid input");
    assert!(!retryer.is_error_retryable(&invalid_input));
    
    let not_found = IoError::new(ErrorKind::NotFound, "not found");
    assert!(!retryer.is_error_retryable(&not_found));
}

#[test]
fn test_standard_retryer_retry_delay_increases() {
    let retryer = Standard::default();
    let error = IoError::new(ErrorKind::ConnectionRefused, "connection refused");
    
    let delay1 = retryer.retry_delay(1, &error).unwrap();
    let delay2 = retryer.retry_delay(2, &error).unwrap();
    let delay3 = retryer.retry_delay(3, &error).unwrap();
    
    // Delays should generally increase with each attempt (with jitter)
    assert!(delay1 <= delay2 || delay2 <= delay3);
    
    // All delays should be reasonable (less than 30 seconds)
    assert!(delay1 < Duration::from_secs(30));
    assert!(delay2 < Duration::from_secs(30));
    assert!(delay3 < Duration::from_secs(30));
}

#[test]
fn test_standard_retryer_max_attempts_behavior() {
    let retryer = Standard::default().with_max_attempts(3);
    let error = IoError::new(ErrorKind::ConnectionRefused, "connection refused");
    
    // Should allow retries up to max attempts
    for attempt in 1..=3 {
        let result = retryer.retry_delay(attempt, &error);
        assert!(result.is_ok(), "Attempt {} should succeed", attempt);
    }
}

#[test]
fn test_standard_retryer_builder_pattern() {
    let retryer = Standard::default().with_max_attempts(10);
    
    assert_eq!(retryer.max_attempts(), 10);
    
    let error = IoError::new(ErrorKind::ConnectionRefused, "connection refused");
    assert!(retryer.is_error_retryable(&error));
}
