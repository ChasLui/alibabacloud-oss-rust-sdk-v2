use std::collections::HashMap;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput};
use crate::client::BodyDataReader;


#[derive(Debug, Default, Serialize, OssRequestModel)]
pub struct UploadPartCopyRequest {
    /// The name of the destination bucket.
    #[serde(skip)]
    pub bucket: String,

    /// The name of the destination object.
    #[serde(skip)]
    pub key: String,

    /// The part number of the part to upload.
    /// Each part has an identifier number (partNumber).
    /// Range: 1~10000
    #[serde(skip)]
    #[field(type = "query", rename = "partNumber")]
    pub part_number: i32,

    /// The upload ID of the multipart upload.
    #[serde(skip)]
    #[field(type = "query", rename = "uploadId")]
    pub upload_id: String,

    /// The source bucket and object to copy from.
    /// Format: /SourceBucketName/SourceObjectName
    #[field(type = "header", rename = "x-oss-copy-source")]
    pub copy_source: String,

    /// The range of bytes to copy from the source object.
    /// Format: bytes=first-last
    #[field(type = "header", rename = "x-oss-copy-source-range")]
    pub copy_source_range: Option<String>,

    /// Copy condition: only copy if the source object's ETag matches this value.
    #[field(type = "header", rename = "x-oss-copy-source-if-match")]
    pub copy_source_if_match: Option<String>,

    /// Copy condition: only copy if the source object's ETag does not match this value.
    #[field(type = "header", rename = "x-oss-copy-source-if-none-match")]
    pub copy_source_if_none_match: Option<String>,

    /// Copy condition: only copy if the source object has not been modified since this time.
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-copy-source-if-unmodified-since")]
    pub copy_source_if_unmodified_since: Option<DateTime<Utc>>,

    /// Copy condition: only copy if the source object has been modified since this time.
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-copy-source-if-modified-since")]
    pub copy_source_if_modified_since: Option<DateTime<Utc>>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    /// The speed limit value. The speed limit value ranges from 245760 to
    /// 838860800, in bit/s.
    #[field(type = "header", rename = "x-oss-traffic-limit")]
    pub traffic_limit: Option<u64>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "CopyPartResult")]
pub struct UploadPartCopyResult {
    /// The time when the part was last modified.
    #[serde(rename = "LastModified", skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,  // Using String to avoid deserialization issues

    /// The ETag of the uploaded part.
    #[serde(rename = "ETag", skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,

    /// The version ID of the source object that was copied.
    #[field(type = "header", rename = "x-oss-copy-source-version-id")]
    pub copy_source_version_id: Option<String>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Uploads a part by copying data from an existing object.
    ///
    /// This method copies data from a source object to upload a part of a multipart upload.
    /// It sends a PUT request with the part number and upload ID parameters, along with
    /// the x-oss-copy-source header specifying the source object to copy from.
    ///
    /// # Arguments
    ///
    /// * `request` - The `UploadPartCopyRequest` containing the necessary information
    ///   for the part copy operation, including destination bucket/key, part number,
    ///   upload ID, and source object specification.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `UploadPartCopyResult` if the copy is
    /// successful, or an error if it fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::UploadPartCopyRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = UploadPartCopyRequest {
    ///     bucket: "destination-bucket".to_string(),
    ///     key: "destination-object".to_string(),
    ///     part_number: 1,  // part number must be between 1 and 10000
    ///     upload_id: "upload-id-from-initiate-multipart-upload".to_string(),
    ///     copy_source: "/source-bucket/source-object".to_string(),
    ///     copy_source_range: Some("bytes=0-1048575".to_string()), // Copy first 1MB
    ///     ..Default::default()
    /// };
    ///
    /// match client.upload_part_copy(&request).await {
    ///     Ok(upload_part_copy_result) => {
    ///         println!("Part copied successfully: {:?}", upload_part_copy_result.etag);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to copy part: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn upload_part_copy(
        &self,
        request: &UploadPartCopyRequest,
    ) -> Result<UploadPartCopyResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "UploadPartCopy".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [
                ("partNumber", request.part_number.to_string()),
                ("uploadId", request.upload_id.clone()),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_bytes = output.get_all_data().await?;
        let body_data = String::from_utf8_lossy(&body_bytes).into_owned();
        let result: UploadPartCopyResult = quick_xml::de::from_str(&body_data)?;

        // Update the common fields from the response
        let mut mutable_result = result;
        mutable_result.update_result(&output);

        Ok(mutable_result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::object::{InitiateMultipartUploadRequest, PutObjectRequest, AbortMultipartUploadRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::SignatureVersionType;
    use crate::test_utils::{load_test_config, TestConfig, generate_unique_object_name};

    #[tokio::test]
    #[serial_test::serial]
    async fn test_upload_part_copy_basic() {
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

        // Generate unique object names for this test
        let source_object_name = generate_unique_object_name("source-upload-part-copy");
        let dest_object_name = generate_unique_object_name("dest-upload-part-copy");

        // First, upload a source object to copy from
        let source_content = b"This is the content of the source object for upload part copy test.";
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: source_object_name.clone(),
            body: Some(crate::BodyContent::from_bytes(source_content.to_vec(), None)),
            ..Default::default()
        };

        match client.put_object(put_request).await {
            Ok(put_result) => {
                println!("Source object uploaded: {:?}", put_result);
            }
            Err(err) => panic!("Failed to upload source object: {:?}", err),
        }

        // Then, initiate a multipart upload to get an upload ID
        let initiate_request = InitiateMultipartUploadRequest {
            bucket: config.bucket.to_string(),
            key: dest_object_name.clone(),
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

        // Now copy a part from the source object
        let copy_request = UploadPartCopyRequest {
            bucket: config.bucket.to_string(),
            key: dest_object_name.clone(),
            part_number: 1,
            upload_id: upload_id.clone(),
            copy_source: format!("/{}/{}", config.bucket, source_object_name),
            copy_source_range: Some("bytes=0-29".to_string()), // Copy first 30 bytes
            ..Default::default()
        };

        match client.upload_part_copy(&copy_request).await {
            Ok(result) => {
                println!("Part copied successfully: {:?}", result);
                assert!(result.etag.is_some(), "ETag should be present in the response");
                println!("Got ETag: {:?}", result.etag);
                println!("Last modified: {:?}", result.last_modified);

                // Clean up: abort the multipart upload to release resources
                match client.abort_multipart_upload(&AbortMultipartUploadRequest {
                    bucket: config.bucket.to_string(),
                    key: dest_object_name,
                    upload_id: upload_id.clone(),
                    ..Default::default()
                }).await {
                    Ok(_) => println!("Successfully aborted multipart upload for cleanup"),
                    Err(err) => eprintln!("Failed to abort multipart upload during cleanup: {:?}", err),
                }
            }
            Err(err) => {
                eprintln!("Upload part copy failed: {:?}", err);
                
                // Even if upload part copy fails, we should still clean up
                match client.abort_multipart_upload(&AbortMultipartUploadRequest {
                    bucket: config.bucket.to_string(),
                    key: dest_object_name,
                    upload_id: upload_id.clone(),
                    ..Default::default()
                }).await {
                    Ok(_) => println!("Successfully aborted multipart upload for cleanup after failure"),
                    Err(err) => eprintln!("Failed to abort multipart upload during cleanup: {:?}", err),
                }
                
                panic!("Upload part copy failed: {:?}", err);
            }
        }
    }
}