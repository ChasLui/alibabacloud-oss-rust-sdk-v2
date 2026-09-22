use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};
use urlencoding;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5, xml_escape_str_ser};
use crate::{BodyContent, OperationInput, OperationOutput};

#[derive(Debug, Default, Serialize, Clone)]
pub struct CompleteMultipartUploadPart {
    /// The part number of the uploaded part.
    #[serde(rename = "PartNumber")]
    pub part_number: i32,

    /// The ETag returned when the part was uploaded.
    #[serde(rename = "ETag", with = "xml_escape_str_ser")]
    pub etag: String,
}

/// The `CompleteMultipartUpload` request body: a `Part` list in ascending
/// order.
#[derive(Serialize)]
struct CompleteMultipartUploadBody<'a> {
    #[serde(rename = "Part")]
    parts: &'a [CompleteMultipartUploadPart],
}

/// Serializes the `CompleteMultipartUpload` body with parts sorted by number.
/// OSS requires ascending part numbers; the Go SDK sorts before sending.
fn serialize_complete_body(
    parts: &[CompleteMultipartUploadPart],
) -> Result<String, quick_xml::DeError> {
    let mut sorted = parts.to_vec();
    sorted.sort_by_key(|part| part.part_number);
    let body = CompleteMultipartUploadBody { parts: &sorted };
    quick_xml::se::to_string_with_root("CompleteMultipartUpload", &body)
}

#[derive(Debug, Default, Serialize, OssRequestModel)]
pub struct CompleteMultipartUploadRequest {
    /// The name of the bucket.
    #[serde(skip)]
    pub bucket: String,

    /// The name of the object.
    #[serde(skip)]
    pub key: String,

    /// The upload ID of the multipart upload.
    #[serde(skip)]
    #[field(type = "query", rename = "uploadId")]
    pub upload_id: String,

    /// The encoding type of the object names in the response.
    #[serde(skip)]
    #[field(type = "query", rename = "encoding-type")]
    pub encoding_type: Option<String>,

    /// The size of the data in the HTTP message body. Unit: bytes.
    #[serde(skip)]
    #[field(type = "header", rename = "Content-Length")]
    pub content_length: Option<u64>,

    /// Specify whether to overwrite the target object if it already exists.
    /// Valid values: true and false
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-forbid-overwrite")]
    pub forbid_overwrite: Option<String>,

    /// Specify whether to list all uploaded parts for the current UploadId.
    /// Valid value: yes
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-complete-all")]
    pub complete_all: Option<String>,

    /// The access control list (ACL) of the object.
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-object-acl")]
    pub object_acl: Option<String>,

    /// The container that stores information about uploaded parts.
    #[serde(rename = "Part", default)]
    pub parts: Vec<CompleteMultipartUploadPart>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    /// A callback parameter is a Base64-encoded string that contains multiple
    /// fields in the JSON format.
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-callback")]
    pub callback: Option<String>,

    /// Configure custom parameters by using the callback-var parameter.
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-callback-var")]
    pub callback_var: Option<String>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "CompleteMultipartUploadResult")]
pub struct CompleteMultipartUploadResult {
    /// The encoding type of the returned result.
    #[serde(rename = "EncodingType", skip_serializing_if = "Option::is_none")]
    pub encoding_type: Option<String>,

    /// The URL of the created object.
    #[serde(rename = "Location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// The name of the bucket.
    #[serde(rename = "Bucket", skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// The name of the created object.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The ETag of the created object.
    #[serde(rename = "ETag", skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,

    /// Version of the object.
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    /// The 64-bit CRC value of the object.
    /// This value is calculated based on the ECMA-182 standard.
    #[field(type = "header", rename = "x-oss-hash-crc64ecma")]
    pub hash_crc64: Option<String>,

    /// The callback result of the upload. This field is populated with the
    /// JSON body returned by OSS when the request carries an
    /// `x-oss-callback` header and the callback is executed.
    #[serde(skip)]
    pub callback_result: std::collections::HashMap<String, serde_json::Value>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

/// Decodes URL-encoded fields in the result when the response reports
/// `EncodingType=url`. Mirrors Go `unmarshalEncodeType` for
/// `CompleteMultipartUploadResult`.
fn decode_result(result: &mut CompleteMultipartUploadResult) {
    let is_url_encoding = result
        .encoding_type
        .as_deref()
        .map(|v| v.eq_ignore_ascii_case("url"))
        .unwrap_or(false);
    if !is_url_encoding {
        return;
    }
    if let Some(key) = &mut result.key {
        *key = urlencoding::decode(key)
            .unwrap_or(std::borrow::Cow::Borrowed(key.as_str()))
            .into_owned();
    }
}

impl Client {
    /// Completes a multipart upload to the OSS bucket.
    ///
    /// This method sends a POST request with the upload ID parameter to
    /// complete a multipart upload. It takes a list of parts (with their
    /// part numbers and ETags) and combines them into a single object.
    ///
    /// # Arguments
    ///
    /// * `request` - The `CompleteMultipartUploadRequest` containing the
    ///   necessary information for the multipart upload completion, including
    ///   bucket, key, upload ID, and the list of parts to combine.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `CompleteMultipartUploadResult` if the
    /// operation is successful, or an error if it fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::{CompleteMultipartUploadRequest, CompleteMultipartUploadPart};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = CompleteMultipartUploadRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     upload_id: "upload-id-from-initiate-multipart-upload".to_string(),
    ///     parts: vec![
    ///         CompleteMultipartUploadPart {
    ///             part_number: 1,
    ///             etag: "\"3349DC700140D7F86A0784842780****\"".to_string(),
    ///         },
    ///         CompleteMultipartUploadPart {
    ///             part_number: 2,
    ///             etag: "\"8EFDA8BE206636A695359836FE0A****\"".to_string(),
    ///         },
    ///     ],
    ///     ..Default::default()
    /// };
    ///
    /// match client.complete_multipart_upload(&request).await {
    ///     Ok(result) => {
    ///         println!("Multipart upload completed: {:?}", result.etag);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to complete multipart upload: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn complete_multipart_upload(
        &self,
        request: &CompleteMultipartUploadRequest,
    ) -> Result<CompleteMultipartUploadResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "CompleteMultipartUpload".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [
                ("uploadId", request.upload_id.clone()),
                ("encoding-type", "url".to_string()),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
            ..Default::default()
        };

        // A caller-supplied encoding type overrides the hardcoded default.
        if let Some(encoding_type) = &request.encoding_type {
            input
                .parameters
                .insert("encoding-type".to_string(), encoding_type.clone());
        }

        // Serialize the parts as XML body (sorted ascending, like the Go SDK).
        let xml_body = serialize_complete_body(&request.parts)?;

        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        // When a callback is configured, OSS returns the callback result as a
        // JSON body instead of the usual XML result. Mirrors Go
        // `unmarshalCallbackBody`, which is only wired up in the callback
        // branch.
        if request.callback.is_some() {
            let body_data = output.get_all_data().await?;
            let mut result = CompleteMultipartUploadResult {
                callback_result: serde_json::from_slice(&body_data).unwrap_or_default(),
                ..Default::default()
            };
            result.update_result(&output);
            return Ok(result);
        }

        // Parse the XML response
        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let result: CompleteMultipartUploadResult = quick_xml::de::from_str(&data_str)?;

        // Update the common fields from the response
        let mut mutable_result = result;
        mutable_result.update_result(&output);

        // Decode the object key when the response is URL-encoded. Mirrors Go
        // `unmarshalEncodeType` for CompleteMultipartUploadResult.
        decode_result(&mut mutable_result);

        Ok(mutable_result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::object::{GetObjectRequest, InitiateMultipartUploadRequest, UploadPartRequest};
    use crate::client::BodyDataReader;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::test_utils::{generate_unique_object_name, load_test_config};
    use crate::SignatureVersionType;

    /// Out-of-order parts must be serialized in ascending part-number order;
    /// OSS rejects a body where `PartNumber` does not increase.
    #[test]
    fn test_complete_body_sorts_parts_ascending() {
        let parts = vec![
            CompleteMultipartUploadPart {
                part_number: 3,
                etag: "etag-3".to_string(),
            },
            CompleteMultipartUploadPart {
                part_number: 1,
                etag: "etag-1".to_string(),
            },
            CompleteMultipartUploadPart {
                part_number: 2,
                etag: "etag-2".to_string(),
            },
        ];
        let xml = serialize_complete_body(&parts).unwrap();
        let first = xml
            .find("<PartNumber>1</PartNumber>")
            .expect("part 1 present");
        let second = xml
            .find("<PartNumber>2</PartNumber>")
            .expect("part 2 present");
        let third = xml
            .find("<PartNumber>3</PartNumber>")
            .expect("part 3 present");
        assert!(
            first < second && second < third,
            "parts must ascend, got: {}",
            xml
        );
        // The caller's slice must not be reordered.
        assert_eq!(parts[0].part_number, 3);
    }

    /// `Key` must be decoded only when the response reports `EncodingType=url`;
    /// the request hardcodes `encoding-type=url`, so a server that ignores it
    /// (no `EncodingType` in the body) must leave the key untouched.
    #[test]
    fn test_decode_result_requires_url_encoding() {
        let mut result = CompleteMultipartUploadResult {
            encoding_type: Some("url".to_string()),
            key: Some("a%20b/c%2Bd".to_string()),
            ..Default::default()
        };
        decode_result(&mut result);
        assert_eq!(result.key.as_deref(), Some("a b/c+d"));

        // No EncodingType marker: leave the key as returned.
        let mut plain = CompleteMultipartUploadResult {
            encoding_type: None,
            key: Some("a%20b".to_string()),
            ..Default::default()
        };
        decode_result(&mut plain);
        assert_eq!(plain.key.as_deref(), Some("a%20b"));

        // Explicit non-url encoding type: also leave it alone.
        let mut other = CompleteMultipartUploadResult {
            encoding_type: Some("base64".to_string()),
            key: Some("a%20b".to_string()),
            ..Default::default()
        };
        decode_result(&mut other);
        assert_eq!(other.key.as_deref(), Some("a%20b"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_complete_multipart_upload_basic() {
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
                .with_signature_version(SignatureVersionType::V4)
                .with_log_level(LogLevel::Debug),
        );

        // Generate a unique object name for this test
        let object_name = generate_unique_object_name("complete-multipart");

        // First, initiate a multipart upload to get an upload ID
        let initiate_request = InitiateMultipartUploadRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            ..Default::default()
        };

        let initiate_result = match client.initiate_multipart_upload(&initiate_request).await {
            Ok(result) => {
                println!("Multipart upload initiated: {:?}", result.upload_id);
                result
            }
            Err(err) => panic!("Initiate multipart upload failed: {:?}", err),
        };

        let upload_id = initiate_result.upload_id.unwrap();

        // Prepare parts content for later verification
        // OSS requires all parts except the last one to be >= 100KB
        let mut expected_content = Vec::new();
        let mut parts = Vec::new();

        // Create 3 parts: first two parts >= 100KB, last part can be smaller
        let part_sizes = [100 * 1024, 150 * 1024, 50 * 1024]; // 100KB, 150KB,
                                                              // 50KB

        for (index, &size) in part_sizes.iter().enumerate() {
            let part_num = (index + 1) as i32;

            // Create part data with specified size
            let mut part_data = Vec::with_capacity(size);
            for i in 0..size {
                // Fill with predictable pattern for verification
                part_data.push(b'A' + (i % 26) as u8);
            }

            expected_content.extend_from_slice(&part_data);

            println!(
                "Uploading part {} with size: {} bytes",
                part_num,
                part_data.len()
            );

            let upload_part_request = UploadPartRequest {
                bucket: config.bucket.to_string(),
                key: object_name.clone(),
                part_number: part_num,
                upload_id: upload_id.clone(),
                body: Some(BodyContent::from_bytes(part_data, None)),
                ..Default::default()
            };

            match client.upload_part(upload_part_request).await {
                Ok(upload_result) => {
                    println!(
                        "Part {} uploaded successfully: {:?}",
                        part_num, upload_result
                    );
                    assert!(
                        upload_result.etag.is_some(),
                        "ETag should be present in the response"
                    );
                    parts.push(CompleteMultipartUploadPart {
                        part_number: part_num,
                        etag: upload_result.etag.unwrap().trim_matches('"').to_string(),
                    });
                }
                Err(err) => panic!("Upload part {} failed: {:?}", part_num, err),
            }
        }

        // Now complete the multipart upload
        let complete_request = CompleteMultipartUploadRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            upload_id: upload_id.clone(),
            parts,
            ..Default::default()
        };

        match client.complete_multipart_upload(&complete_request).await {
            Ok(result) => {
                println!("Multipart upload completed successfully: {:?}", result);
                assert!(
                    result.etag.is_some(),
                    "ETag should be present in the response"
                );
                println!("Object created with ETag: {:?}", result.etag);
                println!("Object location: {:?}", result.location);

                // Verify the content of the completed object
                match client
                    .get_object(GetObjectRequest {
                        bucket: config.bucket.to_string(),
                        key: object_name.clone(),
                        ..Default::default()
                    })
                    .await
                {
                    Ok(mut get_result) => {
                        let actual_content_bytes =
                            get_result.get_all_data().await.unwrap_or_default();
                        let expected_content_string =
                            String::from_utf8(expected_content).unwrap_or_default();
                        let actual_content =
                            String::from_utf8_lossy(&actual_content_bytes).into_owned();
                        println!(
                            "Retrieved object size: {} bytes",
                            actual_content_bytes.len()
                        );
                        println!(
                            "Expected object size: {} bytes",
                            expected_content_string.len()
                        );

                        // Compare the content
                        assert_eq!(
                            actual_content_bytes.len(),
                            expected_content_string.len(),
                            "Retrieved content length should match expected content length"
                        );
                        assert_eq!(
                            actual_content, expected_content_string,
                            "Retrieved content should match expected content"
                        );

                        println!(
                            "Content verification successful! Retrieved {} bytes",
                            actual_content_bytes.len()
                        );

                        println!("Expected content: {}", expected_content_string);
                        println!("Actual content: {}", actual_content);
                    }
                    Err(err) => panic!(
                        "Failed to get object after multipart upload completion: {:?}",
                        err
                    ),
                }
            }
            Err(err) => panic!("Complete multipart upload failed: {:?}", err),
        }
    }
}
