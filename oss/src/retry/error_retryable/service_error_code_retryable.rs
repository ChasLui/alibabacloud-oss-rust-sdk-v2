use crate::retry::traits::ErrorRetryable;
use crate::ServiceError;

pub struct ServiceErrorCodeRetryable;

static RETRY_SERVICE_ERROR_CODES: &[&str] = &[
    "RequestTimeTooSkewed",
    "BadRequest",
];

impl ErrorRetryable for ServiceErrorCodeRetryable {
    fn is_error_retryable(&self, err: &(dyn std::error::Error + 'static)) -> bool {
        // Match the service error code exactly. A substring scan over the
        // formatted error would also match unrelated messages that happen to
        // mention the code. Mirrors Go `ServiceErrorCodeRetryable`, which
        // reads the parsed code via an `ErrorCode()` interface.
        if let Some(service_error) = err.downcast_ref::<ServiceError>() {
            return RETRY_SERVICE_ERROR_CODES.contains(&service_error.code.as_str());
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service_error_with_code(code: &str) -> ServiceError {
        ServiceError {
            code: code.to_string(),
            message: String::new(),
            request_id: String::new(),
            ec: String::new(),
            status_code: http::StatusCode::BAD_REQUEST,
            snapshot: Vec::new(),
            timestamp: None,
            request_target: String::new(),
            headers: Default::default(),
        }
    }

    #[test]
    fn test_is_error_retryable_with_retryable_error_code() {
        let retryable = ServiceErrorCodeRetryable;
        assert!(retryable.is_error_retryable(&service_error_with_code("RequestTimeTooSkewed")));
        assert!(retryable.is_error_retryable(&service_error_with_code("BadRequest")));
    }

    #[test]
    fn test_is_error_retryable_with_non_retryable_error_code() {
        let retryable = ServiceErrorCodeRetryable;
        assert!(!retryable.is_error_retryable(&service_error_with_code("NoSuchKey")));
    }

    /// The code must match exactly: a message that merely mentions a
    /// retryable code must not make the error retryable.
    #[test]
    fn test_code_match_is_not_substring_based() {
        let retryable = ServiceErrorCodeRetryable;
        let mut err = service_error_with_code("NoSuchKey");
        err.message = "the request was a BadRequest in the past".to_string();
        assert!(!retryable.is_error_retryable(&err));

        let non_service_error =
            std::io::Error::new(std::io::ErrorKind::Other, "BadRequest");
        assert!(!retryable.is_error_retryable(&non_service_error));
    }
}