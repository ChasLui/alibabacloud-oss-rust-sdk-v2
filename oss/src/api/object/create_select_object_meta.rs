use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use base64::Engine;
use serde::Serialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};

use crate::{BodyContent, OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};
const DATA_FRAME_TYPE: i32 = 8388609;
const CONTINUOUS_FRAME_TYPE: i32 = 8388612;
const END_FRAME_TYPE: i32 = 8388613;
const META_END_FRAME_CSV_TYPE: i32 = 8388614;
const META_END_FRAME_JSON_TYPE: i32 = 8388615;

/// The format of the input data for a CSV meta request.
#[derive(Debug, Default, Clone, Serialize)]
pub struct InputSerializationCsv {
    /// The value used to separate individual records in the input data. It is
    /// base64-encoded before the request is sent.
    #[serde(rename = "RecordDelimiter", skip_serializing_if = "Option::is_none")]
    pub record_delimiter: Option<String>,

    /// The value used to separate individual fields in a record. It is
    /// base64-encoded before the request is sent.
    #[serde(rename = "FieldDelimiter", skip_serializing_if = "Option::is_none")]
    pub field_delimiter: Option<String>,

    /// The value used as the escape character. It is base64-encoded before
    /// the request is sent.
    #[serde(rename = "QuoteCharacter", skip_serializing_if = "Option::is_none")]
    pub quote_character: Option<String>,
}

/// The format of the input data for a JSON meta request.
#[derive(Debug, Default, Clone, Serialize)]
pub struct InputSerializationJson {
    /// The type of the JSON content. Valid values: DOCUMENT and LINES.
    #[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
    pub json_type: Option<String>,
}

/// The format of the input data.
#[derive(Debug, Default, Clone, Serialize)]
pub struct InputSerialization {
    /// The CSV input format.
    #[serde(rename = "CSV", skip_serializing_if = "Option::is_none")]
    pub csv: Option<InputSerializationCsv>,

    /// The JSON input format.
    #[serde(rename = "JSON", skip_serializing_if = "Option::is_none")]
    pub json: Option<InputSerializationJson>,

    /// The compression type of the object. Valid values: None and GZIP.
    #[serde(rename = "CompressionType", skip_serializing_if = "Option::is_none")]
    pub compression_type: Option<String>,
}

/// The CSV meta request body.
#[derive(Debug, Default, Clone, Serialize)]
pub struct CsvMetaRequest {
    /// The container that stores the input serialization settings.
    #[serde(rename = "InputSerialization", skip_serializing_if = "Option::is_none")]
    pub input_serialization: Option<InputSerialization>,

    /// Specifies whether to overwrite the existing select meta.
    #[serde(rename = "OverwriteIfExists", skip_serializing_if = "Option::is_none")]
    pub overwrite_if_exists: Option<bool>,
}

/// The JSON meta request body.
#[derive(Debug, Default, Clone, Serialize)]
pub struct JsonMetaRequest {
    /// The container that stores the input serialization settings.
    #[serde(rename = "InputSerialization", skip_serializing_if = "Option::is_none")]
    pub input_serialization: Option<InputSerialization>,

    /// Specifies whether to overwrite the existing select meta.
    #[serde(rename = "OverwriteIfExists", skip_serializing_if = "Option::is_none")]
    pub overwrite_if_exists: Option<bool>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct CreateSelectObjectMetaRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// The CSV meta request. Exactly one of `csv_meta_request` and
    /// `json_meta_request` must be set; setting both or neither returns an
    /// error (mirrors Go's `MetaRequest` union type).
    pub csv_meta_request: Option<CsvMetaRequest>,

    /// The JSON meta request. Mutually exclusive with `csv_meta_request`.
    pub json_meta_request: Option<JsonMetaRequest>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct CreateSelectObjectMetaResult {
    /// The total number of bytes scanned.
    pub total_scanned: i64,

    /// The status of the meta operation.
    pub meta_status: i32,

    /// The number of splits.
    pub splits_count: i32,

    /// The total number of rows.
    pub rows_count: i64,

    /// The total number of columns (CSV only, 0 for JSON).
    pub columns_count: i32,

    /// The error message returned in the end frame, empty on success.
    pub error_msg: String,

    pub common: ResultCommon,
}

/// Builds the XML body and the `x-oss-process` value from the request.
/// Base64-encodes the CSV delimiters as the Go SDK does.
fn build_meta_body(
    request: &CreateSelectObjectMetaRequest,
) -> Result<(String, &'static str), Box<dyn std::error::Error + Send + Sync>> {
    match (&request.csv_meta_request, &request.json_meta_request) {
        (Some(csv), None) => {
            let mut csv = csv.clone();
            if let Some(input_serialization) = csv.input_serialization.as_mut() {
                if let Some(csv_input) = input_serialization.csv.as_mut() {
                    for field in [
                        &mut csv_input.record_delimiter,
                        &mut csv_input.field_delimiter,
                        &mut csv_input.quote_character,
                    ] {
                        if let Some(value) = field.as_mut() {
                            *value = base64::engine::general_purpose::STANDARD
                                .encode(value.as_bytes());
                        }
                    }
                }
            }
            Ok((
                quick_xml::se::to_string_with_root("CsvMetaRequest", &csv)?,
                "csv/meta",
            ))
        }
        (None, Some(json)) => Ok((
            quick_xml::se::to_string_with_root("JsonMetaRequest", json)?,
            "json/meta",
        )),
        _ => Err(
            "invalid MetaRequest: exactly one of csv_meta_request or json_meta_request must be set"
                .into(),
        ),
    }
}

/// Parses Select response frames from the full response body and fills the
/// result from the trailing MetaEnd frame (CSV or JSON). Payload CRC is not
/// validated, matching the Go SDK default (EnablePayloadCrc == false).
fn parse_meta_frames(
    body: &[u8],
    result: &mut CreateSelectObjectMetaResult,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut pos = 0usize;
    while pos + 20 <= body.len() {
        // 20-byte header: version(1) + frame_type(3) + payload_len(4) +
        // header_crc(4) + offset(8). All multi-byte fields are big-endian.
        let frame_type = i32::from_be_bytes([0, body[pos + 1], body[pos + 2], body[pos + 3]]);
        let payload_length = i32::from_be_bytes(body[pos + 4..pos + 8].try_into().unwrap());
        let data_len = match payload_length.checked_sub(8) {
            Some(n) if n >= 0 => n as usize,
            _ => return Err(format!("invalid frame payload length: {}", payload_length).into()),
        };
        let payload_start = pos + 20;
        let payload_end = payload_start + data_len;
        // 4-byte payload CRC32 trailer follows the payload data.
        if payload_end + 4 > body.len() {
            return Err("truncated select frame".into());
        }
        let payload = &body[payload_start..payload_end];

        match frame_type {
            META_END_FRAME_CSV_TYPE => {
                if payload.len() < 28 {
                    return Err("invalid meta end csv frame payload".into());
                }
                result.total_scanned = i64::from_be_bytes(payload[0..8].try_into().unwrap());
                result.meta_status = i32::from_be_bytes(payload[8..12].try_into().unwrap());
                result.splits_count = i32::from_be_bytes(payload[12..16].try_into().unwrap());
                result.rows_count = i64::from_be_bytes(payload[16..24].try_into().unwrap());
                result.columns_count = i32::from_be_bytes(payload[24..28].try_into().unwrap());
                result.error_msg = String::from_utf8_lossy(&payload[28..]).into_owned();
                return Ok(());
            }
            META_END_FRAME_JSON_TYPE => {
                if payload.len() < 24 {
                    return Err("invalid meta end json frame payload".into());
                }
                result.total_scanned = i64::from_be_bytes(payload[0..8].try_into().unwrap());
                result.meta_status = i32::from_be_bytes(payload[8..12].try_into().unwrap());
                result.splits_count = i32::from_be_bytes(payload[12..16].try_into().unwrap());
                result.rows_count = i64::from_be_bytes(payload[16..24].try_into().unwrap());
                result.error_msg = String::from_utf8_lossy(&payload[24..]).into_owned();
                return Ok(());
            }
            END_FRAME_TYPE => {
                // End frame: total_scanned(8) + http_status(4) + error_msg.
                if payload.len() >= 12 {
                    let http_status = i32::from_be_bytes(payload[8..12].try_into().unwrap());
                    if http_status >= 400 {
                        let error_msg = String::from_utf8_lossy(&payload[12..]).into_owned();
                        return Err(format!(
                            "select operation failed, status: {}, message: {}",
                            http_status, error_msg
                        )
                        .into());
                    }
                }
                return Ok(());
            }
            DATA_FRAME_TYPE | CONTINUOUS_FRAME_TYPE => {}
            _ => return Err(format!("unexpected frame type: {}", frame_type).into()),
        }
        pos = payload_end + 4;
    }
    Ok(())
}

impl Client {
    /// Obtains the meta information of a CSV or JSON object, such as the
    /// total number of rows and the number of splits.
    ///
    /// # Arguments
    ///
    /// * `request` - The `CreateSelectObjectMetaRequest` containing the bucket
    ///   name, object key and exactly one of `csv_meta_request` /
    ///   `json_meta_request`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::{
    /// #     CreateSelectObjectMetaRequest, CsvMetaRequest, InputSerialization, InputSerializationCsv,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = CreateSelectObjectMetaRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object.csv".to_string(),
    ///     csv_meta_request: Some(CsvMetaRequest {
    ///         input_serialization: Some(InputSerialization {
    ///             csv: Some(InputSerializationCsv {
    ///                 record_delimiter: Some("\n".to_string()),
    ///                 field_delimiter: Some(",".to_string()),
    ///                 quote_character: Some("\"".to_string()),
    ///             }),
    ///             ..Default::default()
    ///         }),
    ///         overwrite_if_exists: Some(false),
    ///     }),
    ///     ..Default::default()
    /// };
    ///
    /// match client.create_select_object_meta(&request).await {
    ///     Ok(result) => {
    ///         println!("rows: {}, splits: {}", result.rows_count, result.splits_count);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to create select object meta: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn create_select_object_meta(
        &self,
        request: &CreateSelectObjectMetaRequest,
    ) -> Result<CreateSelectObjectMetaResult, Box<dyn std::error::Error + Send + Sync>> {
        let (xml_body, process) = build_meta_body(request)?;

        let mut input = OperationInput {
            op_name: "CreateSelectObjectMeta".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("x-oss-process", process)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        // The response is not XML: it is a sequence of Select frames, the
        // trailing MetaEnd frame carries the meta information.
        let body_data = output.get_all_data().await?;
        let mut result = CreateSelectObjectMetaResult::default();
        parse_meta_frames(&body_data, &mut result)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    /// Builds a Select frame: version(1) + frame_type(3) + payload_len(4) +
    /// header_crc(4) + offset(8) + payload + payload_crc(4). CRC fields are
    /// zeroed because parsing does not validate them.
    fn build_frame(frame_type: i32, offset: u64, payload: &[u8]) -> Vec<u8> {
        let mut frame = Vec::new();
        frame.push(0u8);
        frame.extend_from_slice(&frame_type.to_be_bytes()[1..4]);
        frame.extend_from_slice(&((payload.len() + 8) as u32).to_be_bytes());
        frame.extend_from_slice(&0u32.to_be_bytes());
        frame.extend_from_slice(&offset.to_be_bytes());
        frame.extend_from_slice(payload);
        frame.extend_from_slice(&0u32.to_be_bytes());
        frame
    }

    fn csv_meta_payload(error_msg: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&12345i64.to_be_bytes()); // total_scanned
        payload.extend_from_slice(&200i32.to_be_bytes()); // status
        payload.extend_from_slice(&4i32.to_be_bytes()); // splits_count
        payload.extend_from_slice(&1000i64.to_be_bytes()); // rows_count
        payload.extend_from_slice(&6i32.to_be_bytes()); // columns_count
        payload.extend_from_slice(error_msg);
        payload
    }

    #[test]
    fn test_parse_meta_end_csv_frame() {
        let frame = build_frame(META_END_FRAME_CSV_TYPE, 0, &csv_meta_payload(b""));
        let mut result = CreateSelectObjectMetaResult::default();
        parse_meta_frames(&frame, &mut result).unwrap();

        assert_eq!(result.total_scanned, 12345);
        assert_eq!(result.meta_status, 200);
        assert_eq!(result.splits_count, 4);
        assert_eq!(result.rows_count, 1000);
        assert_eq!(result.columns_count, 6);
        assert_eq!(result.error_msg, "");
    }

    #[test]
    fn test_parse_meta_end_csv_frame_with_error_msg() {
        let frame = build_frame(META_END_FRAME_CSV_TYPE, 0, &csv_meta_payload(b"bad csv"));
        let mut result = CreateSelectObjectMetaResult::default();
        parse_meta_frames(&frame, &mut result).unwrap();
        assert_eq!(result.error_msg, "bad csv");
    }

    #[test]
    fn test_parse_meta_end_json_frame() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&999i64.to_be_bytes()); // total_scanned
        payload.extend_from_slice(&200i32.to_be_bytes()); // status
        payload.extend_from_slice(&2i32.to_be_bytes()); // splits_count
        payload.extend_from_slice(&50i64.to_be_bytes()); // rows_count
        payload.extend_from_slice(b""); // error_msg
        let frame = build_frame(META_END_FRAME_JSON_TYPE, 0, &payload);

        let mut result = CreateSelectObjectMetaResult::default();
        parse_meta_frames(&frame, &mut result).unwrap();

        assert_eq!(result.total_scanned, 999);
        assert_eq!(result.meta_status, 200);
        assert_eq!(result.splits_count, 2);
        assert_eq!(result.rows_count, 50);
        assert_eq!(result.columns_count, 0);
    }

    #[test]
    fn test_parse_skips_data_frames() {
        let mut body = build_frame(DATA_FRAME_TYPE, 0, b"col1,col2\n1,2\n");
        body.extend(build_frame(CONTINUOUS_FRAME_TYPE, 15, b""));
        body.extend(build_frame(META_END_FRAME_CSV_TYPE, 15, &csv_meta_payload(b"")));

        let mut result = CreateSelectObjectMetaResult::default();
        parse_meta_frames(&body, &mut result).unwrap();
        assert_eq!(result.rows_count, 1000);
    }

    #[test]
    fn test_parse_end_frame_error() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&0i64.to_be_bytes()); // total_scanned
        payload.extend_from_slice(&500i32.to_be_bytes()); // http status
        payload.extend_from_slice(b"internal error");
        let frame = build_frame(END_FRAME_TYPE, 0, &payload);

        let mut result = CreateSelectObjectMetaResult::default();
        let err = parse_meta_frames(&frame, &mut result).unwrap_err();
        assert!(err.to_string().contains("500"));
        assert!(err.to_string().contains("internal error"));
    }

    #[test]
    fn test_build_meta_body_csv() {
        let request = CreateSelectObjectMetaRequest {
            csv_meta_request: Some(CsvMetaRequest {
                input_serialization: Some(InputSerialization {
                    csv: Some(InputSerializationCsv {
                        record_delimiter: Some("\n".to_string()),
                        field_delimiter: Some(",".to_string()),
                        quote_character: Some("\"".to_string()),
                    }),
                    ..Default::default()
                }),
                overwrite_if_exists: Some(false),
            }),
            ..Default::default()
        };
        let (xml, process) = build_meta_body(&request).unwrap();
        assert_eq!(process, "csv/meta");
        assert!(xml.starts_with("<CsvMetaRequest>"));
        assert!(xml.contains("<RecordDelimiter>Cg==</RecordDelimiter>"));
        assert!(xml.contains("<FieldDelimiter>LA==</FieldDelimiter>"));
        assert!(xml.contains("<QuoteCharacter>Ig==</QuoteCharacter>"));
        assert!(xml.contains("<OverwriteIfExists>false</OverwriteIfExists>"));
    }

    #[test]
    fn test_build_meta_body_json() {
        let request = CreateSelectObjectMetaRequest {
            json_meta_request: Some(JsonMetaRequest {
                input_serialization: Some(InputSerialization {
                    json: Some(InputSerializationJson {
                        json_type: Some("LINES".to_string()),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (xml, process) = build_meta_body(&request).unwrap();
        assert_eq!(process, "json/meta");
        assert!(xml.starts_with("<JsonMetaRequest>"));
        assert!(xml.contains("<Type>LINES</Type>"));
    }

    #[test]
    fn test_build_meta_body_invalid() {
        // Neither set.
        assert!(build_meta_body(&CreateSelectObjectMetaRequest::default()).is_err());
        // Both set.
        let both = CreateSelectObjectMetaRequest {
            csv_meta_request: Some(CsvMetaRequest::default()),
            json_meta_request: Some(JsonMetaRequest::default()),
            ..Default::default()
        };
        assert!(build_meta_body(&both).is_err());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_select_object_meta() {
        let config = match load_test_config() {
            Some(cfg) => cfg,
            None => {
                eprintln!("Test configuration not found. Skipping test.");
                return;
            }
        };

        let client = Client::new(
            &Config::default()
                .with_region(&config.region)
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    &config.access_key_id,
                    &config.access_key_secret,
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V4),
        );

        let object_name = crate::test_utils::generate_unique_object_name("select-meta");

        // Prepare a CSV object
        client
            .put_object(crate::api::object::PutObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                body: Some(BodyContent::from_text(
                    "a,b,c\n1,2,3\n4,5,6\n".to_string(),
                    None,
                )),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .create_select_object_meta(&CreateSelectObjectMetaRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                csv_meta_request: Some(CsvMetaRequest {
                    input_serialization: Some(InputSerialization {
                        csv: Some(InputSerializationCsv {
                            record_delimiter: Some("\n".to_string()),
                            field_delimiter: Some(",".to_string()),
                            quote_character: Some("\"".to_string()),
                        }),
                        ..Default::default()
                    }),
                    overwrite_if_exists: Some(true),
                }),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "create_select_object_meta failed: {:?}",
            result.err()
        );
        let result = result.unwrap();
        assert_eq!(result.rows_count, 3);
        assert!(result.splits_count >= 1);

        // Clean up
        let _ = client
            .delete_object(crate::api::object::DeleteObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name,
                ..Default::default()
            })
            .await;
    }
}
