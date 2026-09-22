use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;
use urlencoding;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, Clone, OssRequestModel)]
pub struct ListPartsRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// The upload ID of the multipart upload.
    #[field(type = "query", rename = "uploadId")]
    pub upload_id: String,

    /// The maximum number of parts to return.
    /// Maximum value is 1000, default is 1000.
    #[field(type = "query", rename = "max-parts")]
    pub max_parts: Option<i32>,

    /// The part number marker for specifying the starting position of the list.
    /// Only parts with part numbers greater than this value will be listed.
    #[field(type = "query", rename = "part-number-marker")]
    pub part_number_marker: Option<i32>,

    /// The encoding type for the returned content.
    #[field(type = "query", rename = "encoding-type")]
    pub encoding_type: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "ListPartsResult")]
pub struct ListPartsResult {
    /// The bucket name.
    #[serde(rename = "Bucket", skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// The encoding type of the returned content.
    #[serde(rename = "EncodingType", skip_serializing_if = "Option::is_none")]
    pub encoding_type: Option<String>,

    /// The object key.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The upload ID.
    #[serde(rename = "UploadId", skip_serializing_if = "Option::is_none")]
    pub upload_id: Option<String>,

    /// The part number marker for the list.
    #[serde(rename = "PartNumberMarker", skip_serializing_if = "Option::is_none")]
    pub part_number_marker: Option<i32>,

    /// The next part number marker if the results are truncated.
    #[serde(
        rename = "NextPartNumberMarker",
        skip_serializing_if = "Option::is_none"
    )]
    pub next_part_number_marker: Option<i32>,

    /// The maximum number of parts returned.
    #[serde(rename = "MaxParts", skip_serializing_if = "Option::is_none")]
    pub max_parts: Option<i32>,

    /// Whether the results are truncated.
    #[serde(rename = "IsTruncated", skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,

    /// The list of parts.
    #[serde(rename = "Part", default)]
    pub parts: Vec<Part>,

    /// The wrapped content key, base64, for a client-side-encrypted object.
    /// Mirrors Go's `ClientEncryptionKey`.
    #[serde(
        rename = "ClientEncryptionKey",
        skip_serializing_if = "Option::is_none"
    )]
    pub client_encryption_key: Option<String>,

    /// The wrapped IV, base64, for a client-side-encrypted object.
    /// Mirrors Go's `ClientEncryptionStart`.
    #[serde(
        rename = "ClientEncryptionStart",
        skip_serializing_if = "Option::is_none"
    )]
    pub client_encryption_start: Option<String>,

    /// The content algorithm of a client-side-encrypted object.
    #[serde(
        rename = "ClientEncryptionCekAlg",
        skip_serializing_if = "Option::is_none"
    )]
    pub client_encryption_cek_alg: Option<String>,

    /// The wrapping algorithm of a client-side-encrypted object.
    #[serde(
        rename = "ClientEncryptionWrapAlg",
        skip_serializing_if = "Option::is_none"
    )]
    pub client_encryption_wrap_alg: Option<String>,

    /// The total plaintext size recorded when the upload was initiated.
    #[serde(
        rename = "ClientEncryptionDataSize",
        skip_serializing_if = "Option::is_none"
    )]
    pub client_encryption_data_size: Option<i64>,

    /// The plaintext part size recorded when the upload was initiated.
    #[serde(
        rename = "ClientEncryptionPartSize",
        skip_serializing_if = "Option::is_none"
    )]
    pub client_encryption_part_size: Option<i64>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

#[derive(Debug, Default, Deserialize)]
pub struct Part {
    /// The part number.
    #[serde(rename = "PartNumber")]
    pub part_number: i32,

    /// The last modified time of the part.
    #[serde(rename = "LastModified", skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>, /* Using String instead of DateTime to avoid
                                        * deserialization issues */

    /// The ETag of the part.
    #[serde(rename = "ETag")]
    pub etag: String,

    /// The size of the part.
    #[serde(rename = "Size")]
    pub size: i64,

    /// The 64-bit CRC value of the part, per the ECMA-182 standard.
    #[serde(rename = "HashCrc64ecma", skip_serializing_if = "Option::is_none")]
    pub hash_crc64: Option<String>,
}

impl Client {
    /// Lists all parts of a multipart upload.
    ///
    /// This method sends a GET request with the upload ID parameter to list
    /// all parts that have been successfully uploaded for a specific multipart
    /// upload. The results are sorted in ascending order by part number.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListPartsRequest` containing the necessary
    ///   information for listing parts, including bucket, key, and upload ID.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `ListPartsResult` if the operation
    /// is successful, or an error if it fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::ListPartsRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListPartsRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     upload_id: "upload-id-from-initiate-multipart-upload".to_string(),
    ///     max_parts: Some(100),
    ///     ..Default::default()
    /// };
    ///
    /// match client.list_parts(&request).await {
    ///     Ok(result) => {
    ///         println!("Found {} parts", result.parts.len());
    ///         for part in result.parts {
    ///             println!(
    ///                 "Part {}: ETag={}, Size={} bytes",
    ///                 part.part_number, part.etag, part.size
    ///             );
    ///         }
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to list parts: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn list_parts(
        &self,
        request: &ListPartsRequest,
    ) -> Result<ListPartsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListParts".to_string(),
            method: http::Method::GET,
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

        // Add optional parameters
        if let Some(max_parts) = request.max_parts {
            input
                .parameters
                .insert("max-parts".to_string(), max_parts.to_string());
        }
        if let Some(part_number_marker) = request.part_number_marker {
            input.parameters.insert(
                "part-number-marker".to_string(),
                part_number_marker.to_string(),
            );
        }
        if let Some(encoding_type) = &request.encoding_type {
            input
                .parameters
                .insert("encoding-type".to_string(), encoding_type.clone());
        }

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        // Parse the XML response body
        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: ListPartsResult = quick_xml::de::from_str(&data_str)?;

        // Update the common fields from the response
        result.update_result(&output);

        // Decode the object key when the response is URL-encoded. Mirrors Go
        // `unmarshalEncodeType` for ListPartsResult.
        let is_url_encoding = result
            .encoding_type
            .as_deref()
            .map(|v| v.eq_ignore_ascii_case("url"))
            .unwrap_or(false);
        if is_url_encoding {
            if let Some(key) = &mut result.key {
                *key = urlencoding::decode(key)
                    .unwrap_or(std::borrow::Cow::Borrowed(key.as_str()))
                    .into_owned();
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::object::{
        AbortMultipartUploadRequest, InitiateMultipartUploadRequest, UploadPartRequest,
    };
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::test_utils::{generate_unique_object_name, load_test_config};
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_parts_basic() {
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
        let object_name = generate_unique_object_name("list-parts");

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

        // Upload a few parts
        for part_num in 1..=3 {
            let part_data = format!("Content of part {}", part_num).into_bytes();
            let upload_part_request = UploadPartRequest {
                bucket: config.bucket.to_string(),
                key: object_name.clone(),
                part_number: part_num,
                upload_id: upload_id.clone(),
                body: Some(crate::BodyContent::from_bytes(part_data, None)),
                ..Default::default()
            };

            match client.upload_part(upload_part_request).await {
                Ok(result) => {
                    println!("Part {} uploaded successfully: {:?}", part_num, result.etag);
                }
                Err(err) => panic!("Upload part {} failed: {:?}", part_num, err),
            }
        }

        // Now list the parts
        let list_request = ListPartsRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            upload_id: upload_id.clone(),
            max_parts: Some(1000),
            ..Default::default()
        };

        match client.list_parts(&list_request).await {
            Ok(result) => {
                println!("Found {} parts", result.parts.len());
                for part in result.parts {
                    println!(
                        "Part {}: ETag={}, Size={} bytes",
                        part.part_number, part.etag, part.size
                    );
                }

                // Clean up: abort the multipart upload to release resources
                match client
                    .abort_multipart_upload(&AbortMultipartUploadRequest {
                        bucket: config.bucket.to_string(),
                        key: object_name,
                        upload_id: upload_id.clone(),
                        ..Default::default()
                    })
                    .await
                {
                    Ok(_) => println!("Successfully aborted multipart upload for cleanup"),
                    Err(err) => {
                        eprintln!("Failed to abort multipart upload during cleanup: {:?}", err)
                    }
                }
            }
            Err(err) => {
                // Even if list parts fails, we should still clean up
                eprintln!("List parts failed: {:?}", err);

                // Clean up: abort the multipart upload to release resources
                match client
                    .abort_multipart_upload(&AbortMultipartUploadRequest {
                        bucket: config.bucket.to_string(),
                        key: object_name,
                        upload_id: upload_id.clone(),
                        ..Default::default()
                    })
                    .await
                {
                    Ok(_) => {
                        println!("Successfully aborted multipart upload for cleanup after failure")
                    }
                    Err(err) => {
                        eprintln!("Failed to abort multipart upload during cleanup: {:?}", err)
                    }
                }

                panic!("List parts failed: {:?}", err);
            }
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_parts_with_marker() {
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
        let object_name = generate_unique_object_name("list-parts-marker");

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

        // Upload a few parts
        for part_num in 1..=5 {
            let part_data = format!("Content of part {}", part_num).into_bytes();
            let upload_part_request = UploadPartRequest {
                bucket: config.bucket.to_string(),
                key: object_name.clone(),
                part_number: part_num,
                upload_id: upload_id.clone(),
                body: Some(crate::BodyContent::from_bytes(part_data, None)),
                ..Default::default()
            };

            match client.upload_part(upload_part_request).await {
                Ok(result) => {
                    println!("Part {} uploaded successfully: {:?}", part_num, result.etag);
                }
                Err(err) => panic!("Upload part {} failed: {:?}", part_num, err),
            }
        }

        // List parts with a marker to get only parts with numbers greater than
        // 2
        let list_request = ListPartsRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            upload_id: upload_id.clone(),
            part_number_marker: Some(2),
            max_parts: Some(1000),
            ..Default::default()
        };

        match client.list_parts(&list_request).await {
            Ok(result) => {
                println!("Found {} parts with number > 2", result.parts.len());
                for part in result.parts {
                    println!(
                        "Part {}: ETag={}, Size={} bytes",
                        part.part_number, part.etag, part.size
                    );
                }

                // Clean up: abort the multipart upload to release resources
                match client
                    .abort_multipart_upload(&AbortMultipartUploadRequest {
                        bucket: config.bucket.to_string(),
                        key: object_name,
                        upload_id: upload_id.clone(),
                        ..Default::default()
                    })
                    .await
                {
                    Ok(_) => println!("Successfully aborted multipart upload for cleanup"),
                    Err(err) => {
                        eprintln!("Failed to abort multipart upload during cleanup: {:?}", err)
                    }
                }
            }
            Err(err) => {
                // Even if list parts fails, we should still clean up
                eprintln!("List parts with marker failed: {:?}", err);

                // Clean up: abort the multipart upload to release resources
                match client
                    .abort_multipart_upload(&AbortMultipartUploadRequest {
                        bucket: config.bucket.to_string(),
                        key: object_name,
                        upload_id: upload_id.clone(),
                        ..Default::default()
                    })
                    .await
                {
                    Ok(_) => {
                        println!("Successfully aborted multipart upload for cleanup after failure")
                    }
                    Err(err) => {
                        eprintln!("Failed to abort multipart upload during cleanup: {:?}", err)
                    }
                }

                panic!("List parts with marker failed: {:?}", err);
            }
        }
    }
}
