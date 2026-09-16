use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput};
use crate::client::BodyDataReader;


#[derive(Debug, Default, Serialize, OssRequestModel)]
pub struct InitiateMultipartUploadRequest {
    /// The name of the bucket.
    #[serde(skip)]
    pub bucket: String,

    /// The name of the object.
    #[serde(skip)]
    pub key: String,

    /// Specify standard behaviors to download data by range
    /// If the value is "standard", the download behavior is modified when the
    /// specified range is not within the valid range. For an object whose
    /// size is 1,000 bytes:
    /// 1) If you set Range: bytes to 500-2000, the value at the end of the
    ///    range is invalid.
    /// In this case, OSS returns HTTP status code 206 and the data that is
    /// within the range of byte 500 to byte 999.
    /// 2) If you set Range: bytes to 1000-2000, the value at the start of the
    ///    range is invalid.
    /// In this case, OSS returns HTTP status code 416 and the InvalidRange
    /// error code.
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-range-behavior")]
    pub range_behavior: Option<String>,

    /// The caching behavior of the web page when the object is downloaded.
    #[serde(skip)]
    #[field(type = "header", rename = "Cache-Control")]
    pub cache_control: Option<String>,

    /// The method that is used to access the object.
    #[serde(skip)]
    #[field(type = "header", rename = "Content-Disposition")]
    pub content_disposition: Option<String>,

    /// The method that is used to encode the object.
    #[serde(skip)]
    #[field(type = "header", rename = "Content-Encoding")]
    pub content_encoding: Option<String>,

    /// The expiration time of the cache in UTC.
    #[serde(skip)]
    #[field(type = "header", rename = "Expires")]
    pub expires: Option<String>,

    /// Specifies whether the object that is uploaded by calling the InitiateMultipartUpload
    /// operation overwrites an existing object that has the same name.
    /// Valid values: true and false
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-forbid-overwrite")]
    pub forbid_overwrite: Option<String>,

    /// The encryption method on the server side when an object is created.
    /// Valid values: AES256 and KMS
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-server-side-encryption")]
    pub server_side_encryption: Option<String>,

    /// The ID of the customer master key (CMK) that is managed by Key
    /// Management Service (KMS). This header is valid only when the
    /// x-oss-server-side-encryption header is set to KMS.
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-server-side-data-encryption")]
    pub server_side_data_encryption: Option<String>,

    /// The ID of the customer master key (CMK) that is managed by Key
    /// Management Service (KMS).
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-server-side-encryption-key-id")]
    pub sse_kms_key_id: Option<String>,

    /// The access control list (ACL) of the object.
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-object-acl")]
    pub object_acl: String,

    /// The storage class of the object.
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-storage-class")]
    pub storage_class: String,

    /// The metadata of the object that you want to upload.
    // #[field(type = "header", rename = "x-oss-object-meta-")]
    // pub metadata: HashMap<String, String>, // TODO `input:"header,x-oss-meta-,usermeta"`

    /// The tags that are specified for the object using a key-value pair.
    /// You can specify multiple tags for an object. Example: TagA=A&TagB=B.
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-tagging")]
    pub tagging: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(default)]
pub struct InitiateMultipartUploadResult {
    /// The container that stores the result of the Initiate Multipart Upload request.
    #[serde(rename = "InitiateMultipartUploadResult", skip_serializing_if = "Option::is_none")]
    pub initiate_multipart_upload_result: Option<InitiateMultipartUploadResultInner>,

    /// The bucket name of the multipart upload.
    #[serde(rename = "Bucket", skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// The key of the object for multipart upload.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The unique ID of the multipart upload event.
    #[serde(rename = "UploadId", skip_serializing_if = "Option::is_none")]
    pub upload_id: Option<String>,

    /// The encoding type of the returned result.
    #[serde(rename = "EncodingType", skip_serializing_if = "Option::is_none")]
    pub encoding_type: Option<String>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct InitiateMultipartUploadResultInner {
    /// The bucket name of the multipart upload.
    #[serde(rename = "Bucket", skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// The key of the object for multipart upload.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The unique ID of the multipart upload event.
    #[serde(rename = "UploadId", skip_serializing_if = "Option::is_none")]
    pub upload_id: Option<String>,

    /// The encoding type of the returned result.
    #[serde(rename = "EncodingType", skip_serializing_if = "Option::is_none")]
    pub encoding_type: Option<String>,
}

impl Client {
    /// Initiates a multipart upload to the OSS bucket.
    ///
    /// This method sends a POST request with the `?uploads` parameter to notify OSS
    /// to initialize a multipart upload event. It returns an UploadId that uniquely
    /// identifies this multipart upload event, which is used in subsequent operations
    /// such as uploading parts and completing the multipart upload.
    ///
    /// # Arguments
    ///
    /// * `request` - The `InitiateMultipartUploadRequest` containing the necessary
    ///   information for the multipart upload initialization.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `InitiateMultipartUploadResult` if the
    /// operation is successful, or an error if it fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::InitiateMultipartUploadRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = InitiateMultipartUploadRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.initiate_multipart_upload(&request).await {
    ///     Ok(initiate_result) => {
    ///         println!("Multipart upload initiated successfully: {:?}", initiate_result.upload_id);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to initiate multipart upload: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn initiate_multipart_upload(
        &self,
        request: &InitiateMultipartUploadRequest,
    ) -> Result<InitiateMultipartUploadResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "InitiateMultipartUpload".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("uploads", "")]  // This is required to indicate multipart upload initiation
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
        let result: InitiateMultipartUploadResult = quick_xml::de::from_str(&data_str)?;

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
    use crate::api::object::{AbortMultipartUploadRequest, ListMultipartUploadsRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::SignatureVersionType;
    use crate::test_utils::{load_test_config, TestConfig, generate_unique_object_name};

    #[tokio::test]
    #[serial_test::serial]
    async fn test_initiate_multipart_upload_basic() {
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
        let object_name = generate_unique_object_name("multipart-init");

        // Initiate multipart upload
        let request = InitiateMultipartUploadRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            object_acl: "default".to_string(),
            ..Default::default()
        };

        match client.initiate_multipart_upload(&request).await {
            Ok(result) => {
                println!("Multipart upload initiated successfully: {:?}", result);
                assert!(result.upload_id.is_some(), "UploadId should be present in the response");
                assert!(result.bucket.is_some(), "Bucket should be present in the response");
                assert!(result.key.is_some(), "Key should be present in the response");

                let upload_id = result.upload_id.unwrap();
                println!("Got UploadId: {}", upload_id);
                
                // Verify the multipart upload was created by listing multipart uploads
                match client.list_multipart_uploads(&ListMultipartUploadsRequest {
                    bucket: config.bucket.to_string(),
                    ..Default::default()
                }).await {
                    Ok(list_result) => {
                        // Check if our upload_id is in the list of uploads
                        let found = list_result.uploads.iter().any(|upload| {
                            upload.upload_id == upload_id
                        });
                        assert!(found, "The initiated multipart upload should be in the list");
                        println!("Verified that upload_id {} exists in multipart uploads", upload_id);
                    },
                    Err(err) => {
                        eprintln!("Failed to list multipart uploads for verification: {:?}", err);
                        // Don't fail the test if listing uploads fails, since it's just for verification
                    }
                }
                
                // Clean up: abort the multipart upload to release resources
                match client.abort_multipart_upload(&AbortMultipartUploadRequest {
                    bucket: config.bucket.to_string(),
                    key: object_name,
                    upload_id: upload_id.clone(),
                    ..Default::default()  // 使用默认值填充其他字段
                }).await {
                    Ok(_) => println!("Successfully aborted multipart upload for cleanup"),
                    Err(err) => eprintln!("Failed to abort multipart upload during cleanup: {:?}", err),
                }
            }
            Err(err) => panic!("Initiate multipart upload failed: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_initiate_multipart_upload_with_storage_class() {
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
        let object_name = generate_unique_object_name("multipart-init-storage");

        // Initiate multipart upload with storage class
        let request = InitiateMultipartUploadRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            object_acl: "default".to_string(),
            storage_class: "Standard".to_string(),
            ..Default::default()
        };

        match client.initiate_multipart_upload(&request).await {
            Ok(result) => {
                println!("Multipart upload initiated with storage class: {:?}", result);
                assert!(result.upload_id.is_some(), "UploadId should be present in the response");

                let upload_id = result.upload_id.unwrap();
                println!("Got UploadId with storage class: {}", upload_id);
                
                // Verify the multipart upload was created by listing multipart uploads
                match client.list_multipart_uploads(&ListMultipartUploadsRequest {
                    bucket: config.bucket.to_string(),
                    ..Default::default()
                }).await {
                    Ok(list_result) => {
                        // Check if our upload_id is in the list of uploads
                        let found = list_result.uploads.iter().any(|upload| {
                            upload.upload_id == upload_id
                        });
                        assert!(found, "The initiated multipart upload should be in the list");
                        println!("Verified that upload_id {} exists in multipart uploads", upload_id);
                    },
                    Err(err) => {
                        eprintln!("Failed to list multipart uploads for verification: {:?}", err);
                        // Don't fail the test if listing uploads fails, since it's just for verification
                    }
                }
                
                // Clean up: abort the multipart upload to release resources
                match client.abort_multipart_upload(&AbortMultipartUploadRequest {
                    bucket: config.bucket.to_string(),
                    key: object_name,
                    upload_id: upload_id.clone(),
                    ..Default::default()  // 使用默认值填充其他字段
                }).await {
                    Ok(_) => println!("Successfully aborted multipart upload for cleanup"),
                    Err(err) => eprintln!("Failed to abort multipart upload during cleanup: {:?}", err),
                }
            }
            Err(err) => panic!("Initiate multipart upload with storage class failed: {:?}", err),
        }
    }
}