use std::io::Read;
use std::sync::{Arc, Mutex};

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, BodyContent};

#[derive(Default, OssRequestModel)]
pub struct UploadPartRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// The part number of the part to upload.
    /// Each part has an identifier number (partNumber).
    /// Range: 1~10000
    #[field(type = "query", rename = "partNumber")]
    pub part_number: i32,

    /// The upload ID of the multipart upload.
    /// uploadId is used to uniquely identify which Object the uploaded Part belongs to.
    #[field(type = "query", rename = "uploadId")]
    pub upload_id: String,

    /// The data to upload as a part.
    pub body: Option<BodyContent>,

    /// The MD5 hash of the part content.
    #[field(type = "header", rename = "Content-MD5")]
    pub content_md5: Option<String>,

    /// Progress callback function
    pub progress_fn: Option<Box<dyn Fn(i64, i64)>>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    /// The speed limit value. The speed limit value ranges from 245760 to
    /// 838860800, in bit/s.
    #[field(type = "header", rename = "x-oss-traffic-limit")]
    pub traffic_limit: Option<u64>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct UploadPartResult {
    /// The MD5 hash of the part that was uploaded.
    #[field(type = "header", rename = "ETag")]
    pub etag: Option<String>,

    /// The MD5 hash of the part content.
    #[field(type = "header", rename = "Content-MD5")]
    pub content_md5: Option<String>,

    /// The 64-bit CRC value of the part.
    /// This value is calculated based on the ECMA-182 standard.
    #[field(type = "header", rename = "x-oss-hash-crc64ecma")]
    pub hash_crc64: Option<String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Uploads a part to the OSS bucket as part of a multipart upload.
    ///
    /// This method sends a PUT request with the part number and upload ID parameters
    /// to upload a part of the object as part of a multipart upload. The part number
    /// must be between 1 and 10000. Each upload of the same part number will overwrite
    /// the previous data.
    ///
    /// # Arguments
    ///
    /// * `request` - The `UploadPartRequest` containing the necessary information
    ///   for the part upload, including bucket, key, part number, upload ID, and the part data.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `UploadPartResult` if the upload is
    /// successful, or an error if it fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::UploadPartRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// # use alibabacloud_oss_sdk_rust_v2::BodyContent;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let part_data = b"Content of the part to upload";
    /// let request = UploadPartRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     part_number: 1,  // part number must be between 1 and 10000
    ///     upload_id: "upload-id-from-initiate-multipart-upload".to_string(),
    ///     body: Some(BodyContent::from_bytes(part_data.to_vec(), None)),
    ///     ..Default::default()
    /// };
    ///
    /// match client.upload_part(request).await {
    ///     Ok(upload_part_result) => {
    ///         println!("Part uploaded successfully: {:?}", upload_part_result.etag);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to upload part: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn upload_part(
        &self,
        request: UploadPartRequest,
    ) -> Result<UploadPartResult, Box<dyn std::error::Error + Send + Sync>> {
        let headers = request.header_map();
        let queries = request.query_map();
        let bucket = request.bucket.clone();
        let key = request.key.clone();

        let mut input = OperationInput {
            op_name: "UploadPart".to_string(),
            method: http::Method::PUT,
            bucket: Some(bucket),
            key: Some(key),
            body: request.body,
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
            headers,
            queries,
            vec![update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = UploadPartResult::default();

        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::api::object::{InitiateMultipartUploadRequest, AbortMultipartUploadRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::SignatureVersionType;
    use crate::test_utils::{load_test_config, TestConfig, generate_unique_object_name};

    #[tokio::test]
    #[serial_test::serial]
    async fn test_upload_part_basic() {
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
        let object_name = generate_unique_object_name("upload-part");

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

        // Now upload a part
        let part_data = b"This is the content of the first part";
        let upload_part_request = UploadPartRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            part_number: 1,  // part number must be between 1 and 10000
            upload_id: upload_id.clone(),
            body: Some(crate::BodyContent::from_bytes(part_data.to_vec(), None)),
            ..Default::default()
        };

        match client.upload_part(upload_part_request).await {
            Ok(result) => {
                println!("Part uploaded successfully: {:?}", result);
                assert!(result.etag.is_some(), "ETag should be present in the response");
                println!("Got ETag: {:?}", result.etag);

                // Clean up: abort the multipart upload to release resources
                match client.abort_multipart_upload(&AbortMultipartUploadRequest {
                    bucket: config.bucket.to_string(),
                    key: object_name,
                    upload_id: upload_id.clone(),
                    ..Default::default()
                }).await {
                    Ok(_) => println!("Successfully aborted multipart upload for cleanup"),
                    Err(err) => eprintln!("Failed to abort multipart upload during cleanup: {:?}", err),
                }
            }
            Err(err) => {
                eprintln!("Upload part failed: {:?}", err);
                
                // Even if upload part fails, we should still clean up
                match client.abort_multipart_upload(&AbortMultipartUploadRequest {
                    bucket: config.bucket.to_string(),
                    key: object_name,
                    upload_id: upload_id.clone(),
                    ..Default::default()
                }).await {
                    Ok(_) => println!("Successfully aborted multipart upload for cleanup after failure"),
                    Err(err) => eprintln!("Failed to abort multipart upload during cleanup: {:?}", err),
                }
                
                panic!("Upload part failed: {:?}", err);
            }
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_upload_part_multiple_parts() {
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
        let object_name = generate_unique_object_name("upload-part-multi");

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

        // Upload multiple parts with different part numbers
        let mut part_etags = Vec::new();

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
                    println!("Part {} uploaded successfully: {:?}", part_num, result);
                    assert!(result.etag.is_some(), "ETag should be present in the response");
                    part_etags.push(result.etag.unwrap());
                }
                Err(err) => panic!("Upload part {} failed: {:?}", part_num, err),
            }
        }

        println!("Uploaded {} parts with ETags: {:?}", part_etags.len(), part_etags);
        
        // Clean up: abort the multipart upload to release resources
        match client.abort_multipart_upload(&AbortMultipartUploadRequest {
            bucket: config.bucket.to_string(),
            key: object_name,
            upload_id: upload_id.clone(),
            ..Default::default()
        }).await {
            Ok(_) => println!("Successfully aborted multipart upload for cleanup"),
            Err(err) => eprintln!("Failed to abort multipart upload during cleanup: {:?}", err),
        }
    }
}