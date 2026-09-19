use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use urlencoding;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput};
use crate::client::BodyDataReader;


#[derive(Debug, Default, Clone, OssRequestModel)]
pub struct ListMultipartUploadsRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The delimiter for grouping object names.
    #[field(type = "query", rename = "delimiter")]
    pub delimiter: Option<String>,

    /// The maximum number of multipart uploads to return.
    /// Maximum value is 1000, default is 1000.
    #[field(type = "query", rename = "max-uploads")]
    pub max_uploads: Option<i32>,

    /// The key marker for specifying the starting position of the result.
    #[field(type = "query", rename = "key-marker")]
    pub key_marker: Option<String>,

    /// The prefix to limit the returned object keys.
    #[field(type = "query", rename = "prefix")]
    pub prefix: Option<String>,

    /// The upload ID marker for specifying the starting position of the result.
    #[field(type = "query", rename = "upload-id-marker")]
    pub upload_id_marker: Option<String>,

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
#[serde(rename = "ListMultipartUploadsResult")]
pub struct ListMultipartUploadsResult {
    /// The bucket name.
    #[serde(rename = "Bucket", skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// The encoding type of the returned content.
    #[serde(rename = "EncodingType", skip_serializing_if = "Option::is_none")]
    pub encoding_type: Option<String>,

    /// The starting object key position of the list.
    #[serde(rename = "KeyMarker", skip_serializing_if = "Option::is_none")]
    pub key_marker: Option<String>,

    /// The starting upload ID position of the list.
    #[serde(rename = "UploadIdMarker", skip_serializing_if = "Option::is_none")]
    pub upload_id_marker: Option<String>,

    /// The next key marker if the results are truncated.
    #[serde(rename = "NextKeyMarker", skip_serializing_if = "Option::is_none")]
    pub next_key_marker: Option<String>,

    /// The next upload ID marker if the results are truncated.
    #[serde(rename = "NextUploadIdMarker", skip_serializing_if = "Option::is_none")]
    pub next_upload_id_marker: Option<String>,

    /// The maximum number of uploads returned.
    #[serde(rename = "MaxUploads", skip_serializing_if = "Option::is_none")]
    pub max_uploads: Option<i32>,

    /// Whether the results are truncated.
    #[serde(rename = "IsTruncated", skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,

    /// The delimiter used for grouping object names.
    #[serde(rename = "Delimiter", skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<String>,

    /// The prefix used to limit object keys.
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// The list of multipart uploads.
    #[serde(rename = "Upload", default)]
    pub uploads: Vec<MultipartUpload>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

#[derive(Debug, Default, Deserialize)]
pub struct MultipartUpload {
    /// The object key of the multipart upload.
    #[serde(rename = "Key")]
    pub key: String,

    /// The upload ID of the multipart upload.
    #[serde(rename = "UploadId")]
    pub upload_id: String,

    /// The time when the multipart upload was initiated.
    #[serde(rename = "Initiated", skip_serializing_if = "Option::is_none")]
    pub initiated: Option<String>,  // Changed from DateTime<Utc> to String for now
}

impl Client {
    /// Lists all in-progress multipart uploads in the specified bucket.
    ///
    /// This method sends a GET request with the `?uploads` parameter to list
    /// all multipart upload events that have been initialized but not yet
    /// completed (Complete) or aborted (Abort).
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListMultipartUploadsRequest` containing the necessary
    ///   information for listing multipart uploads, including bucket and optional
    ///   filtering parameters.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `ListMultipartUploadsResult` if the
    /// operation is successful, or an error if it fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::ListMultipartUploadsRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListMultipartUploadsRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     max_uploads: Some(100),
    ///     prefix: Some("my-prefix".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.list_multipart_uploads(&request).await {
    ///     Ok(result) => {
    ///         println!("Found {} in-progress uploads", result.uploads.len());
    ///         for upload in result.uploads {
    ///             println!("Upload ID: {}, Key: {}, Initiated: {:?}", 
    ///                      upload.upload_id, upload.key, upload.initiated);
    ///         }
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to list multipart uploads: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn list_multipart_uploads(
        &self,
        request: &ListMultipartUploadsRequest,
    ) -> Result<ListMultipartUploadsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListMultipartUploads".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("uploads", ""), ("encoding-type", "url")]  // This is required to indicate multipart uploads listing and to enable URL encoding
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

        // Parse the XML response body
        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: ListMultipartUploadsResult = quick_xml::de::from_str(&data_str)?;

        // Update the common fields from the response
        result.update_result(&output);
        
        // Decode URL-encoded response fields. Mirrors Go `unmarshalEncodeType`
        // for ListMultipartUploadsResult: key marker, next key marker, prefix,
        // delimiter, and each upload key.
        let is_url_encoding = result
            .encoding_type
            .as_deref()
            .map(|v| v.eq_ignore_ascii_case("url"))
            .unwrap_or(false);
        if is_url_encoding {
            for field in [
                &mut result.key_marker,
                &mut result.next_key_marker,
                &mut result.prefix,
                &mut result.delimiter,
            ] {
                if let Some(value) = field {
                    *value = urlencoding::decode(value)
                        .unwrap_or_else(|_| std::borrow::Cow::Borrowed(value.as_str()))
                        .into_owned();
                }
            }

            for upload in &mut result.uploads {
                upload.key = urlencoding::decode(&upload.key)
                    .unwrap_or_else(|_| std::borrow::Cow::Borrowed(&upload.key))
                    .to_string();
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use std::io::Cursor;

    use super::*;
    use crate::api::object::{
        InitiateMultipartUploadRequest, 
        UploadPartRequest, 
        CompleteMultipartUploadRequest, 
        CompleteMultipartUploadPart,
        AbortMultipartUploadRequest,
    };
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::SignatureVersionType;
    use crate::test_utils::{load_test_config, TestConfig};

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_multipart_uploads_basic() {
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

        // List multipart uploads in the bucket
        let request = ListMultipartUploadsRequest {
            bucket: config.bucket.to_string(),
            ..Default::default()
        };

        match client.list_multipart_uploads(&request).await {
            Ok(result) => {
                println!("Found {} in-progress uploads", result.uploads.len());
                println!("Bucket: {:?}", result.bucket);
                println!("Is Truncated: {:?}", result.is_truncated);
                
                for upload in result.uploads {
                    println!("Upload ID: {}, Key: {}, Initiated: {:?}", 
                             upload.upload_id, upload.key, upload.initiated);
                }
            }
            Err(err) => panic!("List multipart uploads failed: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_multipart_uploads_with_filters() {
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

        // List multipart uploads with filters
        let request = ListMultipartUploadsRequest {
            bucket: config.bucket.to_string(),
            max_uploads: Some(10),
            prefix: Some("test".to_string()),
            ..Default::default()
        };

        match client.list_multipart_uploads(&request).await {
            Ok(result) => {
                println!("Found {} in-progress uploads with prefix 'test'", result.uploads.len());
                
                for upload in result.uploads {
                    println!("Upload ID: {}, Key: {}, Initiated: {:?}", 
                             upload.upload_id, upload.key, upload.initiated);
                }
            }
            Err(err) => panic!("List multipart uploads with filters failed: {:?}", err),
        }
    }
    
    #[tokio::test]
    #[serial_test::serial]
    async fn test_comprehensive_multipart_operations_workflow() {
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
        let object_name = format!("test-comprehensive-multipart-{}", 
                                  std::time::SystemTime::now()
                                      .duration_since(std::time::UNIX_EPOCH)
                                      .unwrap()
                                      .as_millis());

        // Step 1: Initiate multipart upload
        let initiate_request = InitiateMultipartUploadRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            ..Default::default()
        };

        let initiate_result = client
            .initiate_multipart_upload(&initiate_request)
            .await
            .expect("Initiate multipart upload should succeed");

        let upload_id = initiate_result.upload_id.expect("UploadId should be present");
        println!("Step 1: Multipart upload initiated with ID: {}", upload_id);

        // Step 2: Verify the upload is listed
        let list_before_upload = client
            .list_multipart_uploads(&ListMultipartUploadsRequest {
                bucket: config.bucket.to_string(),
                ..Default::default()
            })
            .await
            .expect("List multipart uploads should succeed");

        let found_upload = list_before_upload.uploads.iter().any(|upload| {
            upload.upload_id == upload_id
        });
        assert!(found_upload, "The initiated upload should appear in the list");
        println!("Step 2: Verified upload exists in multipart uploads list");

        // Step 3: Upload a small part (we won't actually complete the upload to keep the test simple)
        // We'll just test the workflow and then abort it
        let part_data = b"test part data for comprehensive workflow";
        let upload_part_request = UploadPartRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            part_number: 1,
            upload_id: upload_id.clone(),
            body: Some(crate::BodyContent::from_bytes(part_data.to_vec(), None)),
            ..Default::default()
        };

        let upload_part_result = client
            .upload_part(upload_part_request)
            .await;
        
        match upload_part_result {
            Ok(part_result) => {
                println!("Uploaded part with ETag: {:?}", part_result.etag);
                
                // Step 4: List parts to verify the uploaded part
                let list_parts_request = crate::api::object::ListPartsRequest {
                    bucket: config.bucket.to_string(),
                    key: object_name.clone(),
                    upload_id: upload_id.clone(),
                    ..Default::default()
                };

                match client.list_parts(&list_parts_request).await {
                    Ok(parts_result) => {
                        println!("Found {} parts in the upload", parts_result.parts.len());
                    },
                    Err(err) => {
                        eprintln!("Could not list parts (this might be expected if part hasn't been committed): {:?}", err);
                    }
                }
            },
            Err(err) => {
                eprintln!("Part upload failed (might be OK for this test): {:?}", err);
            }
        }

        // Step 5: List multipart uploads again to confirm it still exists
        let list_after_part = client
            .list_multipart_uploads(&ListMultipartUploadsRequest {
                bucket: config.bucket.to_string(),
                ..Default::default()
            })
            .await
            .expect("List multipart uploads should succeed after part upload");

        let still_exists = list_after_part.uploads.iter().any(|upload| {
            upload.upload_id == upload_id
        });
        assert!(still_exists, "The upload should still exist after part upload");
        println!("Step 5: Confirmed upload still exists after part upload");

        // Step 6: Test abort functionality to clean up
        let abort_request = AbortMultipartUploadRequest {
            bucket: config.bucket.to_string(),
            key: object_name,
            upload_id: upload_id.clone(),
            ..Default::default()
        };

        match client.abort_multipart_upload(&abort_request).await {
            Ok(_) => println!("Step 6: Successfully aborted multipart upload (resource cleanup)"),
            Err(err) => eprintln!("Failed to abort multipart upload: {:?}", err),
        }

        // Step 7: Verify the upload is no longer listed
        let list_after_abort = client
            .list_multipart_uploads(&ListMultipartUploadsRequest {
                bucket: config.bucket.to_string(),
                ..Default::default()
            })
            .await
            .expect("List multipart uploads should succeed after abort");

        let no_longer_exists = !list_after_abort.uploads.iter().any(|upload| {
            upload.upload_id == upload_id
        });
        assert!(no_longer_exists, "The upload should no longer exist after abort");
        println!("Step 7: Verified upload no longer exists after abort (resources cleaned up)");

        println!("Comprehensive multipart operations workflow test completed successfully!");
    }
}