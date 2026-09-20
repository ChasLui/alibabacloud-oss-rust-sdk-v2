use std::time::SystemTime;

use base64::{self, Engine};
use chrono::DateTime;

use crate::{ServiceError, HEADER_OSS_ERR, HEADER_OSS_REQUEST_ID};
use crate::client::OssResponse;

/// Convert a response to a service error by parsing the response body.
/// 
/// **Note**: This function consumes the response, so it should be used carefully
/// to avoid conflicts with other response consumption.
pub fn try_convert_service_error(
    oss_response: &OssResponse ,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match oss_response {
        OssResponse::ErrResponse { status, body, headers, url } => {
            let timestamp: SystemTime = headers
                .get("Date")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| DateTime::parse_from_rfc2822(v).ok())
                .map(|datetime| datetime.into())
                .unwrap_or_else(SystemTime::now);

// An empty body with an `x-oss-err` header means the error payload
            // is base64-encoded in that header. Mirrors Go
            // `tryConvertServiceError`.
            let decoded_body;
            let body = if body.is_empty() {
                decoded_body = headers
                    .get(HEADER_OSS_ERR)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| {
                        base64::engine::general_purpose::STANDARD
                            .decode(v.as_bytes())
                            .ok()
                    })
                    .and_then(|bytes| String::from_utf8(bytes).ok());
                decoded_body.as_deref().unwrap_or(body.as_str())
            } else {
                body.as_str()
            };

            let mut service_error = ServiceError {
                status_code: status.clone(),
                code: "BadErrorResponse".to_string(),
                request_id: headers
                    .get(HEADER_OSS_REQUEST_ID)
                    .map(|v| v.to_str().unwrap_or("").to_string())
                    .unwrap_or_default(),
                timestamp: Some(timestamp),
                request_target: format!("{}", url), // TODO request method
                snapshot: body.as_bytes().clone().into(),
                headers: headers.clone(),
                message: String::new(),
                ec: String::new(),
            };

            // A callback failure is reported as a JSON body (`{"Error": {...}}`),
            // not XML. Mirrors the JSON branch of Go `tryConvertServiceError`.
            let is_json = headers
                .get("Content-Type")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.eq_ignore_ascii_case("application/json"))
                .unwrap_or(false);
            if is_json {
                if let Ok(raw) = serde_json::from_str::<serde_json::Value>(body) {
                    let payload = raw.get("Error").unwrap_or(&raw);
                    service_error.code = payload
                        .get("Code")
                        .and_then(|v| v.as_str())
                        .unwrap_or("BadErrorResponse")
                        .to_string();
                    service_error.message = payload
                        .get("Message")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    if let Some(rid) = payload.get("RequestId").and_then(|v| v.as_str()) {
                        service_error.request_id = rid.to_string();
                    }
                    if let Some(ec) = payload.get("EC").and_then(|v| v.as_str()) {
                        service_error.ec = ec.to_string();
                    }
                    return Err(service_error.into());
                }
            }

            match quick_xml::de::from_str::<ServiceError>(body) {
                Ok(parsed) => {
                    service_error.code = parsed.code;
                    service_error.message = parsed.message;
                    service_error.request_id = parsed.request_id;
                    service_error.ec = parsed.ec;
                }
                Err(err) => {
                    let len = body.as_bytes().len().min(256);
                    service_error.message = format!(
                        "Failed to parse xml from response body due to: {}. With part response body {}.",
                        err,
                        String::from_utf8_lossy(&body.as_bytes()[..len])
                    );
                }
            }
            return Err(service_error.into())
        }
        _ => {
            panic!("it will not go to this point")
        }
    }
}

// Helper function to convert HashMap to HeaderMap
fn convert_hashmap_to_headermap(
    hashmap: std::collections::HashMap<String, String>,
) -> Result<http::HeaderMap, Box<dyn std::error::Error + Send + Sync>> {
    use http::HeaderMap;
    let mut header_map = HeaderMap::new();
    
    for (key, value) in hashmap {
        if let (Ok(header_name), Ok(header_value)) = (
            http::HeaderName::from_bytes(key.as_bytes()),
            http::HeaderValue::from_str(&value)
        ) {
            header_map.insert(header_name, header_value);
        }
    }
    
    Ok(header_map)
}
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[tokio::test]
//     async fn test_try_convert_service_error_xml() {
//         let response = reqwest::ClientBuilder::new()
//             .build()
//             .unwrap()
//             .get("https://oss-cn-hangzhou.aliyuncs.com/")
//             .send()
//             .await
//             .unwrap();
//
//         let service_error = try_convert_service_error(OssResponse::from_response_and_context(response)).await.unwrap();
//
//         println!("{:?}", service_error);
//         assert!(!service_error.message.starts_with("Failed to parse xml"));
//     }
//
//     #[tokio::test]
//     async fn test_try_convert_service_error_parse_failed() {
//         let response = reqwest::ClientBuilder::new()
//             .build()
//             .unwrap()
//             .get("https://www.aliyun.com/")
//             .send()
//             .await
//             .unwrap();
//
//         let service_error = try_convert_service_error(response).await.unwrap();
//
//         println!("{:?}", service_error);
//         assert!(service_error.message.starts_with("Failed to parse xml"));
//     }
// }