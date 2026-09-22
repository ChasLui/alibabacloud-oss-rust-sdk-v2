use crate::retry::traits::ErrorRetryable;
use crate::ServiceError;

pub struct HTTPStatusCodeRetryable;

/// retryable HTTP status codes
static RETRY_ERROR_CODES: &[http::StatusCode] = &[
    http::StatusCode::UNAUTHORIZED,
    http::StatusCode::REQUEST_TIMEOUT,
    http::StatusCode::TOO_MANY_REQUESTS,
];

impl ErrorRetryable for HTTPStatusCodeRetryable {
    fn is_error_retryable(&self, err: &(dyn std::error::Error + 'static)) -> bool {
        // Any 5xx is retryable, plus the three specific 4xx codes. Mirrors Go
        // `HTTPStatusCodeRetryable`.
        if let Some(service_error) = err.downcast_ref::<ServiceError>() {
            let status = service_error.status_code;
            return status.is_server_error() || RETRY_ERROR_CODES.contains(&status);
        }
        if let Some(http_error) = err.downcast_ref::<reqwest::Error>() {
            if let Some(status) = http_error.status() {
                return status.is_server_error() || RETRY_ERROR_CODES.contains(&status);
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service_error_with_status(status: http::StatusCode) -> ServiceError {
        ServiceError {
            code: "SomeCode".to_string(),
            message: String::new(),
            request_id: String::new(),
            ec: String::new(),
            status_code: status,
            snapshot: Vec::new(),
            timestamp: None,
            request_target: String::new(),
            headers: Default::default(),
        }
    }

    #[test]
    fn test_server_errors_are_retryable() {
        let retryable = HTTPStatusCodeRetryable;
        for status in [500, 502, 503, 504] {
            let err = service_error_with_status(http::StatusCode::from_u16(status).unwrap());
            assert!(
                retryable.is_error_retryable(&err),
                "{} must be retryable",
                status
            );
        }
    }

    #[test]
    fn test_selected_4xx_are_retryable() {
        let retryable = HTTPStatusCodeRetryable;
        for status in [401, 408, 429] {
            let err = service_error_with_status(http::StatusCode::from_u16(status).unwrap());
            assert!(
                retryable.is_error_retryable(&err),
                "{} must be retryable",
                status
            );
        }
    }

    #[test]
    fn test_other_4xx_are_not_retryable() {
        let retryable = HTTPStatusCodeRetryable;
        for status in [400, 403, 404, 409] {
            let err = service_error_with_status(http::StatusCode::from_u16(status).unwrap());
            assert!(
                !retryable.is_error_retryable(&err),
                "{} must not be retryable",
                status
            );
        }
    }

    #[test]
    fn test_is_error_retryable_with_non_http_error() {
        let retryable = HTTPStatusCodeRetryable;
        let err = std::io::Error::new(std::io::ErrorKind::Other, "Some error");
        assert!(!retryable.is_error_retryable(&err));
    }
}
