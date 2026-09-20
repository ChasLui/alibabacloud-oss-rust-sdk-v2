use std::error::Error;
use std::fmt;
use std::time::SystemTime;

use serde::Deserialize;

/// Error returned by the OSS service.
///
/// Every field is optional on the wire: real OSS error bodies omit fields
/// (e.g. `RequestId` on malformed responses), and Go's `xml.Unmarshal`
/// tolerates that by zero-filling. Requiring every element here would make
/// the whole body fail to parse, lose `Code`, and break error-code-based
/// retry decisions, so each field defaults.
#[derive(Debug, Deserialize)]
pub struct ServiceError {
    #[serde(rename = "Code", default)]
    pub code: String,

    #[serde(rename = "Message", default)]
    pub message: String,

    #[serde(rename = "RequestId", default)]
    pub request_id: String,

    #[serde(rename = "EC", default)]
    pub ec: String,

    #[serde(skip)]
    pub status_code: http::StatusCode,

    #[serde(skip)]
    pub snapshot: Vec<u8>,

    #[serde(skip)]
    pub timestamp: Option<SystemTime>,

    #[serde(skip)]
    pub request_target: String,

    #[serde(skip)]
    pub headers: http::HeaderMap,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error returned by Service. Http Status Code: {}. Error Code: {}. Request Id: {}. \
             Message: {}. EC: {}. Timestamp: {:?}. Request Endpoint: {}.",
            self.status_code.as_u16(),
            self.code,
            self.request_id,
            self.message,
            self.ec,
            self.timestamp,
            self.request_target
        )
    }
}

impl Error for ServiceError {}

impl ServiceError {
    pub fn http_status_code(&self) -> http::StatusCode {
        self.status_code
    }

    pub fn error_code(&self) -> &str {
        &self.code
    }
}

#[derive(Debug)]
pub struct ClientError {
    pub code: String,
    pub message: String,
    pub err: Box<dyn std::error::Error + Send + Sync>,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "client error: {}, {}", self.message, self.err)
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.err.as_ref())
    }
}

#[derive(Debug)]
pub struct OperationError {
    pub name: String,
    pub err: Box<dyn std::error::Error>,
}

impl OperationError {
    pub fn operation(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "operation error {}: {}", self.name, self.err)
    }
}

impl Error for OperationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.err.as_ref())
    }
}

#[derive(Debug)]
pub struct DeserializationError {
    pub err: Box<dyn std::error::Error>,
    pub snapshot: Vec<u8>,
}

impl fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "deserialization failed: {}", self.err)
    }
}

impl Error for DeserializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.err.as_ref())
    }
}

#[derive(Debug)]
pub struct SerializationError {
    pub err: Box<dyn std::error::Error>,
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "serialization failed: {}", self.err)
    }
}

impl Error for SerializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.err.as_ref())
    }
}

#[derive(Debug)]
pub struct CanceledError {
    pub err: Box<dyn std::error::Error>,
}

impl CanceledError {
    pub fn canceled_error(&self) -> bool {
        true
    }
}

impl fmt::Display for CanceledError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "canceled: {}", self.err)
    }
}

impl Error for CanceledError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.err.as_ref())
    }
}

pub trait InvalidParamErrorTrait: Error {
    fn field(&self) -> String;
    fn set_context(&mut self, context: String);
}

#[derive(Debug)]
struct InvalidParamErrorImpl {
    pub context: String,
    pub field: String,
    pub reason: String,
}

impl fmt::Display for InvalidParamErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}.", self.reason, self.field())
    }
}

impl Error for InvalidParamErrorImpl {}

impl InvalidParamErrorTrait for InvalidParamErrorImpl {
    fn field(&self) -> String {
        if self.context.is_empty() {
            self.field.clone()
        } else {
            format!("{}.{}", self.context, self.field)
        }
    }

    fn set_context(&mut self, context: String) {
        self.context = context;
    }
}

pub fn new_err_param_required(field: &str) -> Box<dyn InvalidParamErrorTrait> {
    Box::new(InvalidParamErrorImpl {
        context: String::new(),
        field: field.to_string(),
        reason: "missing required field".to_string(),
    })
}

pub fn new_err_param_invalid(field: &str) -> Box<dyn InvalidParamErrorTrait> {
    Box::new(InvalidParamErrorImpl {
        context: String::new(),
        field: field.to_string(),
        reason: "invalid field".to_string(),
    })
}

pub fn new_err_param_null(field: &str) -> Box<dyn InvalidParamErrorTrait> {
    Box::new(InvalidParamErrorImpl {
        context: String::new(),
        field: field.to_string(),
        reason: "null field".to_string(),
    })
}

pub fn new_err_param_type_not_support(field: &str) -> Box<dyn InvalidParamErrorTrait> {
    Box::new(InvalidParamErrorImpl {
        context: String::new(),
        field: field.to_string(),
        reason: "type not support".to_string(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_error_display() {
        let error = ServiceError {
            code: "500".to_string(),
            message: "Internal Server Error".to_string(),
            request_id: "1234567890".to_string(),
            ec: "EC123".to_string(),
            status_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            snapshot: vec![],
            timestamp: None,
            request_target: "/api/service".to_string(),
            headers: http::HeaderMap::new(),
        };

        assert_eq!(
            error.to_string(),
            "Error returned by Service. Http Status Code: 500. Error Code: 500. Request Id: \
             1234567890. Message: Internal Server Error. EC: EC123. Timestamp: None. Request \
             Endpoint: /api/service."
        );
    }

    #[test]
    fn test_client_error_display() {
        let error = ClientError {
            code: "400".to_string(),
            message: "Bad Request".to_string(),
            err: Box::new(ServiceError {
                code: "400".to_string(),
                message: "Bad Request".to_string(),
                request_id: "0987654321".to_string(),
                ec: "EC456".to_string(),
                status_code: http::StatusCode::BAD_REQUEST,
                snapshot: vec![],
                timestamp: None,
                request_target: "/api/client".to_string(),
                headers: http::HeaderMap::new(),
            }),
        };

        assert_eq!(
            error.to_string(),
            "client error: Bad Request, Error returned by Service. Http Status Code: 400. Error \
             Code: 400. Request Id: 0987654321. Message: Bad Request. EC: EC456. Timestamp: None. \
             Request Endpoint: /api/client."
        );
    }

    #[test]
    fn test_operation_error_display() {
        let error = OperationError {
            name: "Operation".to_string(),
            err: Box::new(ClientError {
                code: "500".to_string(),
                message: "Internal Server Error".to_string(),
                err: Box::new(ServiceError {
                    code: "500".to_string(),
                    message: "Internal Server Error".to_string(),
                    request_id: "1234567890".to_string(),
                    ec: "EC123".to_string(),
                    status_code: http::StatusCode::INTERNAL_SERVER_ERROR,
                    snapshot: vec![],
                    timestamp: None,
                    request_target: "/api/operation".to_string(),
                    headers: http::HeaderMap::new(),
                }),
            }),
        };

        assert_eq!(
            error.to_string(),
            "operation error Operation: client error: Internal Server Error, Error returned by \
             Service. Http Status Code: 500. Error Code: 500. Request Id: 1234567890. Message: \
             Internal Server Error. EC: EC123. Timestamp: None. Request Endpoint: /api/operation."
        );
    }

    #[test]
    fn test_deserialization_error_display() {
        let error = DeserializationError {
            err: Box::new(ServiceError {
                code: "400".to_string(),
                message: "Bad Request".to_string(),
                request_id: "0987654321".to_string(),
                ec: "EC456".to_string(),
                status_code: http::StatusCode::BAD_REQUEST,
                snapshot: vec![],
                timestamp: None,
                request_target: "/api/deserialization".to_string(),
                headers: http::HeaderMap::new(),
            }),
            snapshot: vec![1, 2, 3],
        };

        assert_eq!(
            error.to_string(),
            "deserialization failed: Error returned by Service. Http Status Code: 400. Error \
             Code: 400. Request Id: 0987654321. Message: Bad Request. EC: EC456. Timestamp: None. \
             Request Endpoint: /api/deserialization."
        );
    }

    #[test]
    fn test_serialization_error_display() {
        let error = SerializationError {
            err: Box::new(ServiceError {
                code: "500".to_string(),
                message: "Internal Server Error".to_string(),
                request_id: "1234567890".to_string(),
                ec: "EC123".to_string(),
                status_code: http::StatusCode::INTERNAL_SERVER_ERROR,
                snapshot: vec![],
                timestamp: None,
                request_target: "/api/serialization".to_string(),
                headers: http::HeaderMap::new(),
            }),
        };

        assert_eq!(
            error.to_string(),
            "serialization failed: Error returned by Service. Http Status Code: 500. Error Code: \
             500. Request Id: 1234567890. Message: Internal Server Error. EC: EC123. Timestamp: \
             None. Request Endpoint: /api/serialization."
        );
    }

    #[test]
    fn test_canceled_error_display() {
        let error = CanceledError {
            err: Box::new(ServiceError {
                code: "500".to_string(),
                message: "Internal Server Error".to_string(),
                request_id: "1234567890".to_string(),
                ec: "EC123".to_string(),
                status_code: http::StatusCode::INTERNAL_SERVER_ERROR,
                snapshot: vec![],
                timestamp: None,
                request_target: "/api/canceled".to_string(),
                headers: http::HeaderMap::new(),
            }),
        };

        assert_eq!(
            error.to_string(),
            "canceled: Error returned by Service. Http Status Code: 500. Error Code: 500. Request \
             Id: 1234567890. Message: Internal Server Error. EC: EC123. Timestamp: None. Request \
             Endpoint: /api/canceled."
        );
    }

    #[test]
    fn test_invalid_param_error_display() {
        let error = InvalidParamErrorImpl {
            context: "Context".to_string(),
            field: "Field".to_string(),
            reason: "invalid value".to_string(),
        };

        assert_eq!(error.to_string(), "invalid value, Context.Field.");
    }

    #[test]
    fn test_new_err_param_required() {
        let error = new_err_param_required("Field");

        assert_eq!(error.field(), "Field");

        assert_eq!(error.to_string(), "missing required field, Field.");
    }

    #[test]
    fn test_new_err_param_invalid() {
        let error = new_err_param_invalid("Field");

        assert_eq!(error.field(), "Field");

        assert_eq!(error.to_string(), "invalid field, Field.");
    }

    #[test]
    fn test_new_err_param_null() {
        let error = new_err_param_null("Field");

        assert_eq!(error.field(), "Field");

        assert_eq!(error.to_string(), "null field, Field.");
    }

    #[test]
    fn test_new_err_param_type_not_support() {
        let error = new_err_param_type_not_support("Field");

        assert_eq!(error.field(), "Field");

        assert_eq!(error.to_string(), "type not support, Field.");
    }

    #[test]
    fn test_service_error_http_status_code() {
        let error = ServiceError {
            code: "400".to_string(),
            message: "Bad Request".to_string(),
            request_id: "0987654321".to_string(),
            ec: "EC456".to_string(),
            status_code: http::StatusCode::BAD_REQUEST,
            snapshot: Vec::new(),
            timestamp: None,
            request_target: "/api/service".to_string(),
            headers: http::HeaderMap::new(),
        };
        assert_eq!(error.http_status_code(), http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_service_error_error_code() {
        let error = ServiceError {
            code: "400".to_string(),
            message: "Bad Request".to_string(),
            request_id: "0987654321".to_string(),
            ec: "EC456".to_string(),
            status_code: http::StatusCode::BAD_REQUEST,
            snapshot: Vec::new(),
            timestamp: None,
            request_target: "/api/service".to_string(),
            headers: http::HeaderMap::new(),
        };
        assert_eq!(error.error_code(), "400");
    }

    #[test]
    fn test_client_error_source() {
        let inner_error = ServiceError {
            code: "400".to_string(),
            message: "Bad Request".to_string(),
            request_id: "0987654321".to_string(),
            ec: "EC456".to_string(),
            status_code: http::StatusCode::BAD_REQUEST,
            snapshot: Vec::new(),
            timestamp: None,
            request_target: "/api/client".to_string(),
            headers: http::HeaderMap::new(),
        };
        let error = ClientError {
            code: "500".to_string(),
            message: "Internal Server Error".to_string(),
            err: Box::new(inner_error),
        };
        assert_eq!(
            error.source().unwrap().to_string(),
            "Error returned by Service. Http Status Code: 400. Error Code: 400. Request Id: \
             0987654321. Message: Bad Request. EC: EC456. Timestamp: None. Request Endpoint: \
             /api/client."
        );
    }

    #[test]
    fn test_operation_error_operation() {
        let inner_error = ServiceError {
            code: "400".to_string(),
            message: "Bad Request".to_string(),
            request_id: "0987654321".to_string(),
            ec: "EC456".to_string(),
            status_code: http::StatusCode::BAD_REQUEST,
            snapshot: Vec::new(),
            timestamp: None,
            request_target: "/api/operation".to_string(),
            headers: http::HeaderMap::new(),
        };
        let error = OperationError {
            name: "Operation".to_string(),
            err: Box::new(inner_error),
        };
        assert_eq!(error.operation(), "Operation");
    }

    #[test]
    fn test_deserialization_error_source() {
        let inner_error = ServiceError {
            code: "400".to_string(),
            message: "Bad Request".to_string(),
            request_id: "0987654321".to_string(),
            ec: "EC456".to_string(),
            status_code: http::StatusCode::BAD_REQUEST,
            snapshot: Vec::new(),
            timestamp: None,
            request_target: "/api/deserialization".to_string(),
            headers: http::HeaderMap::new(),
        };
        let error = DeserializationError {
            err: Box::new(inner_error),
            snapshot: Vec::new(),
        };
        assert_eq!(
            error.source().unwrap().to_string(),
            "Error returned by Service. Http Status Code: 400. Error Code: 400. Request Id: \
             0987654321. Message: Bad Request. EC: EC456. Timestamp: None. Request Endpoint: \
             /api/deserialization."
        );
    }

    #[test]
    fn test_serialization_error_source() {
        let inner_error = ServiceError {
            code: "500".to_string(),
            message: "Internal Server Error".to_string(),
            request_id: "1234567890".to_string(),
            ec: "EC123".to_string(),
            status_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            snapshot: Vec::new(),
            timestamp: None,
            request_target: "/api/serialization".to_string(),
            headers: http::HeaderMap::new(),
        };
        let error = SerializationError {
            err: Box::new(inner_error),
        };
        assert_eq!(
            error.source().unwrap().to_string(),
            "Error returned by Service. Http Status Code: 500. Error Code: 500. Request Id: \
             1234567890. Message: Internal Server Error. EC: EC123. Timestamp: None. Request \
             Endpoint: /api/serialization."
        );
    }

    #[test]
    fn test_canceled_error_canceled_error() {
        let inner_error = ServiceError {
            code: "500".to_string(),
            message: "Internal Server Error".to_string(),
            request_id: "1234567890".to_string(),
            ec: "EC123".to_string(),
            status_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            snapshot: Vec::new(),
            timestamp: None,
            request_target: "/api/canceled".to_string(),
            headers: http::HeaderMap::new(),
        };
        let error = CanceledError {
            err: Box::new(inner_error),
        };
        assert!(error.canceled_error());
    }

    #[test]
    fn test_invalid_param_error_field() {
        let error = InvalidParamErrorImpl {
            context: "Context".to_string(),
            field: "Field".to_string(),
            reason: "invalid value".to_string(),
        };
        assert_eq!(error.field(), "Context.Field");
    }

    #[test]
    fn test_invalid_param_error_set_context() {
        let mut error = InvalidParamErrorImpl {
            context: "Context".to_string(),
            field: "Field".to_string(),
            reason: "invalid value".to_string(),
        };
        error.set_context("New Context".to_string());
        assert_eq!(error.context, "New Context");
    }

    #[test]
    fn test_new_err_param_required_field() {
        let error = new_err_param_required("Field");
        assert_eq!(error.field(), "Field");
    }

    #[test]
    fn test_new_err_param_invalid_field() {
        let error = new_err_param_invalid("Field");
        assert_eq!(error.field(), "Field");
    }

    #[test]
    fn test_new_err_param_null_field() {
        let error = new_err_param_null("Field");
        assert_eq!(error.field(), "Field");
    }

    #[test]
    fn test_new_err_param_type_not_support_field() {
        let error = new_err_param_type_not_support("Field");
        assert_eq!(error.field(), "Field");
    }
}
