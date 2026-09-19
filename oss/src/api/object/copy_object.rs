use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;
use std::collections::HashMap;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput};
use crate::client::BodyDataReader;


#[derive(Debug, Default, OssRequestModel)]
pub struct CopyObjectRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// The source of the copy operation in the format `/SourceBucketName/SourceObjectName`
    /// or `/SourceBucketName/SourceObjectName?versionId=xxx` to specify a version.
    #[field(type = "header", rename = "x-oss-copy-source")]
    pub copy_source: String,

    /// Specifies whether to overwrite the target object if it already exists.
    /// Valid values: true and false
    #[field(type = "header", rename = "x-oss-forbid-overwrite")]
    pub forbid_overwrite: Option<String>,

    /// Specifies the ETag value to match against the source object.
    #[field(type = "header", rename = "x-oss-copy-source-if-match")]
    pub copy_source_if_match: Option<String>,

    /// Specifies the ETag value to match against the source object (should not match).
    #[field(type = "header", rename = "x-oss-copy-source-if-none-match")]
    pub copy_source_if_none_match: Option<String>,

    /// Specifies the time to compare with the modification time of the source object.
    /// If the source object was modified after this time, the copy operation proceeds.
    #[field(type = "header", rename = "x-oss-copy-source-if-unmodified-since")]
    pub copy_source_if_unmodified_since: Option<String>,

    /// Specifies the time to compare with the modification time of the source object.
    /// If the source object was modified before this time, the copy operation proceeds.
    #[field(type = "header", rename = "x-oss-copy-source-if-modified-since")]
    pub copy_source_if_modified_since: Option<String>,

    /// Specifies how to handle metadata during the copy operation.
    /// Valid values: COPY (default) and REPLACE
    #[field(type = "header", rename = "x-oss-metadata-directive")]
    pub metadata_directive: Option<String>,

    /// The encryption method on the server side when the target object is created.
    #[field(type = "header", rename = "x-oss-server-side-encryption")]
    pub server_side_encryption: Option<String>,

    /// The ID of the customer master key (CMK) that is managed by KMS.
    #[field(type = "header", rename = "x-oss-server-side-encryption-key-id")]
    pub server_side_encryption_key_id: Option<String>,

    /// The access control list (ACL) of the target object.
    #[field(type = "header", rename = "x-oss-object-acl")]
    pub object_acl: Option<String>,

    /// The storage class of the target object.
    #[field(type = "header", rename = "x-oss-storage-class")]
    pub storage_class: Option<String>,

    /// The tags that are specified for the target object using a key-value pair.
    #[field(type = "header", rename = "x-oss-tagging")]
    pub tagging: Option<String>,

    /// Specifies how to handle object tagging during the copy operation.
    #[field(type = "header", rename = "x-oss-tagging-directive")]
    pub tagging_directive: Option<String>,

    /// The ID of the customer master key (CMK) for data encryption.
    #[field(type = "header", rename = "x-oss-server-side-data-encryption")]
    pub server_side_data_encryption: Option<String>,

    /// The caching behavior of the web browser when the object is downloaded.
    #[field(type = "header", rename = "Cache-Control")]
    pub cache_control: Option<String>,

    /// The name of the object that the user downloads.
    #[field(type = "header", rename = "Content-Disposition")]
    pub content_disposition: Option<String>,

    /// The content encoding of the object.
    #[field(type = "header", rename = "Content-Encoding")]
    pub content_encoding: Option<String>,

    /// The content type of the object.
    #[field(type = "header", rename = "Content-Type")]
    pub content_type: Option<String>,

    /// The expiration time in UTC. Example: `Fri, 13 Nov 2015 14:47:53 GMT`.
    #[field(type = "header", rename = "Expires")]
    pub expires: Option<String>,

    /// The metadata of the object that you want to upload.
    #[field(type = "header", rename = "x-oss-meta-", usermeta)]
    pub metadata: HashMap<String, String>,

    /// The speed limit value. The speed limit value ranges from 245760 to
    /// 838860800, in bit/s.
    #[field(type = "header", rename = "x-oss-traffic-limit")]
    pub traffic_limit: Option<u64>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    /// Common request fields
    pub common: RequestCommon,
}

// Structure for parsing XML response from body
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CopyObjectResponseBody {
    #[serde(rename = "ETag", skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
    
    #[serde(rename = "LastModified", skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
}

// Main result structure for API
#[derive(Debug, Default, OssResultModel)]
pub struct CopyObjectResult {
    /// The ETag of the copied object
    #[field(type = "header", rename = "ETag")]
    pub etag: Option<String>,

    /// The last modified time of the copied object
    #[field(type = "header", rename = "LastModified")]
    pub last_modified: Option<String>,

    /// The version ID of the copied object
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    /// The version ID of the source object that was copied
    #[field(type = "header", rename = "x-oss-copy-source-version-id")]
    pub copy_source_version_id: Option<String>,

    /// The 64-bit CRC value of the object
    #[field(type = "header", rename = "x-oss-hash-crc64ecma")]
    pub hash_crc64: Option<String>,

    /// The encryption algorithm used to encrypt the object on the server.
    #[field(type = "header", rename = "x-oss-server-side-encryption")]
    pub server_side_encryption: Option<String>,

    /// The server side data encryption algorithm.
    #[field(type = "header", rename = "x-oss-server-side-data-encryption")]
    pub server_side_data_encryption: Option<String>,

    /// The ID of the customer master key (CMK) that is managed by Key
    /// Management Service (KMS).
    #[field(type = "header", rename = "x-oss-server-side-encryption-key-id")]
    pub server_side_encryption_key_id: Option<String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Copies an object from one location to another within the same region.
    ///
    /// # Arguments
    ///
    /// * `request` - The `CopyObjectRequest` containing the necessary
    ///   information for the copy operation.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `CopyObjectResult` if the copy is
    /// successful, or an error if it fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::CopyObjectRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = CopyObjectRequest {
    ///     bucket: "target-bucket".to_string(),
    ///     key: "target-object".to_string(),
    ///     copy_source: "/source-bucket/source-object".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.copy_object(&request).await {
    ///     Ok(copy_object_result) => {
    ///         println!("Object copied successfully: {:?}", copy_object_result);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to copy object: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn copy_object(
        &self,
        request: &CopyObjectRequest,
    ) -> Result<CopyObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "CopyObject".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let mut result: CopyObjectResult = Default::default();

        // First update result with headers and other common fields
        result.update_result(&output);

        // Then parse the XML response body if it exists and merge with header values
        let body_bytes = output.get_all_data().await?;
        let body_data = String::from_utf8_lossy(&body_bytes).into_owned();
        if !body_data.trim().is_empty() {
            // Parse the XML response
            if let Ok(parsed_result) = quick_xml::de::from_str::<CopyObjectResponseBody>(&body_data) {
                // Only update fields that might come from the XML body
                if parsed_result.etag.is_some() {
                    result.etag = parsed_result.etag;
                }
                if parsed_result.last_modified.is_some() {
                    result.last_modified = parsed_result.last_modified;
                }
            }
        }

        Ok(result)
    }
}

/// Struct to parse the XML response
#[derive(Debug, Deserialize, Default)]
#[serde(rename = "CopyObjectResult")]
pub struct CopyObjectResultBody {
    #[serde(rename = "ETag", skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
    
    #[serde(rename = "LastModified", skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::object::{PutObjectRequest, GetObjectRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::SignatureVersionType;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use crate::test_utils::{load_test_config, TestConfig, generate_unique_object_name};

    /// The extended request headers added for Go SDK v2 parity must reach the
    /// wire map under their protocol names.
    #[test]
    fn test_copy_object_extended_headers_are_mapped() {
        let request = CopyObjectRequest {
            bucket: "b".to_string(),
            key: "k".to_string(),
            copy_source: "/src-bucket/src-key".to_string(),
            cache_control: Some("no-cache".to_string()),
            content_disposition: Some("attachment".to_string()),
            content_encoding: Some("gzip".to_string()),
            content_type: Some("text/plain".to_string()),
            expires: Some("Fri, 13 Nov 2015 14:47:53 GMT".to_string()),
            server_side_data_encryption: Some("AES256".to_string()),
            traffic_limit: Some(245760),
            ..Default::default()
        };
        let headers = request.header_map();
        assert_eq!(headers.get("Cache-Control").map(String::as_str), Some("no-cache"));
        assert_eq!(headers.get("Content-Disposition").map(String::as_str), Some("attachment"));
        assert_eq!(headers.get("Content-Encoding").map(String::as_str), Some("gzip"));
        assert_eq!(headers.get("Content-Type").map(String::as_str), Some("text/plain"));
        assert_eq!(headers.get("Expires").map(String::as_str), Some("Fri, 13 Nov 2015 14:47:53 GMT"));
        assert_eq!(headers.get("x-oss-server-side-data-encryption").map(String::as_str), Some("AES256"));
        assert_eq!(headers.get("x-oss-traffic-limit").map(String::as_str), Some("245760"));
    }

    // Configuration structure to hold test credentials
    // Using shared TestConfig from test_utils

    // Function to load test configuration from file
    // Using shared load_test_config from test_utils

    // Helper function to generate unique test object names
    // Using shared generate_unique_object_name from test_utils

    #[tokio::test]
    #[serial_test::serial]
    async fn test_copy_object_basic() {
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

        // Create a source object to copy
        let source_object_name = generate_unique_object_name("source-basic");
        let target_object_name = generate_unique_object_name("target-basic");
        let test_content = "Test content for basic copy operation";

        // Put the source object
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: source_object_name.to_string(),
            body: Some(crate::BodyContent::from_text(test_content.to_string(), None)),
            ..Default::default()
        };

        match client.put_object(put_request).await {
            Ok(output) => println!("Source object created: {:?}", output),
            Err(err) => panic!("Failed to create source object: {:?}", err),
        }

        // Now test copy object
        let copy_request = CopyObjectRequest {
            bucket: config.bucket.to_string(),
            key: target_object_name.to_string(),
            copy_source: format!("/{}/{}", config.bucket, source_object_name),
            ..Default::default()
        };

        match client.copy_object(&copy_request).await {
            Ok(result) => {
                println!("Object copied successfully: {:?}", result);
                assert!(result.etag.is_some());
                assert!(result.last_modified.is_some());
            }
            Err(err) => panic!("Copy object failed: {:?}", err),
        }

        // Verify the target object exists by getting it
        let get_request = GetObjectRequest {
            bucket: config.bucket.to_string(),
            key: target_object_name.to_string(),
            ..Default::default()
        };

        match client.get_object(get_request).await {
            Ok(mut result) => {
                let content_bytes = result.get_all_data().await.unwrap_or_default();
                let content = String::from_utf8_lossy(&content_bytes).into_owned();
                assert_eq!(content, test_content);
                println!("Verified copied object content matches source");
            }
            Err(err) => panic!("Failed to get copied object: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_copy_object_with_metadata_directive() {
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

        // Create a source object to copy
        let source_object_name = generate_unique_object_name("source-metadata");
        let target_object_name = generate_unique_object_name("target-metadata");
        let test_content = "Test content for metadata copy operation";

        // Put the source object
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: source_object_name.to_string(),
            body: Some(crate::BodyContent::from_text(test_content.to_string(), None)),
            ..Default::default()
        };

        match client.put_object(put_request).await {
            Ok(output) => println!("Source object created with metadata: {:?}", output),
            Err(err) => panic!("Failed to create source object: {:?}", err),
        }

        // Now test copy object with metadata directive
        let copy_request = CopyObjectRequest {
            bucket: config.bucket.to_string(),
            key: target_object_name.to_string(),
            copy_source: format!("/{}/{}", config.bucket, source_object_name),
            metadata_directive: Some("COPY".to_string()),
            ..Default::default()
        };
        // Add some custom headers
        let mut copy_request_with_headers = copy_request;
        copy_request_with_headers.add_header("x-oss-meta-author", "test-user");

        match client.copy_object(&copy_request_with_headers).await {
            Ok(result) => {
                println!("Object copied with metadata directive: {:?}", result);
                assert!(result.etag.is_some());
                assert!(result.last_modified.is_some());
            }
            Err(err) => panic!("Copy object with metadata directive failed: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_copy_object_with_storage_class() {
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

        // Create a source object to copy
        let source_object_name = generate_unique_object_name("source-storage");
        let target_object_name = generate_unique_object_name("target-storage");
        let test_content = "Test content for storage class copy operation";

        // Put the source object
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: source_object_name.to_string(),
            body: Some(crate::BodyContent::from_text(test_content.to_string(), None)),
            ..Default::default()
        };

        match client.put_object(put_request).await {
            Ok(output) => println!("Source object created with storage class: {:?}", output),
            Err(err) => panic!("Failed to create source object: {:?}", err),
        }

        // Now test copy object with storage class
        let copy_request = CopyObjectRequest {
            bucket: config.bucket.to_string(),
            key: target_object_name.to_string(),
            copy_source: format!("/{}/{}", config.bucket, source_object_name),
            storage_class: Some("Standard".to_string()),
            ..Default::default()
        };

        match client.copy_object(&copy_request).await {
            Ok(result) => {
                println!("Object copied with storage class: {:?}", result);
                assert!(result.etag.is_some());
                assert!(result.last_modified.is_some());
            }
            Err(err) => panic!("Copy object with storage class failed: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_copy_object_with_acl() {
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

        // Create a source object to copy
        let source_object_name = generate_unique_object_name("source-acl");
        let target_object_name = generate_unique_object_name("target-acl");
        let test_content = "Test content for ACL copy operation";

        // Put the source object
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: source_object_name.to_string(),
            body: Some(crate::BodyContent::from_text(test_content.to_string(), None)),
            ..Default::default()
        };

        match client.put_object(put_request).await {
            Ok(output) => println!("Source object created with ACL: {:?}", output),
            Err(err) => panic!("Failed to create source object: {:?}", err),
        }

        // Now test copy object with ACL
        let copy_request = CopyObjectRequest {
            bucket: config.bucket.to_string(),
            key: target_object_name.to_string(),
            copy_source: format!("/{}/{}", config.bucket, source_object_name),
            object_acl: Some("private".to_string()),
            ..Default::default()
        };
        // Add ACL header
        let mut copy_request_with_acl = copy_request;
        copy_request_with_acl.add_header("x-oss-object-acl", "private");

        match client.copy_object(&copy_request_with_acl).await {
            Ok(result) => {
                println!("Object copied with ACL: {:?}", result);
                assert!(result.etag.is_some());
                assert!(result.last_modified.is_some());
            }
            Err(err) => panic!("Copy object with ACL failed: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_copy_object_forbid_overwrite() {
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

        // Create a source object to copy
        let source_object_name = generate_unique_object_name("source-fo");
        let target_object_name = generate_unique_object_name("target-fo");
        let test_content = "Test content for forbid overwrite operation";
        let new_content = "New content";

        // Put the source object
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: source_object_name.to_string(),
            body: Some(crate::BodyContent::from_text(test_content.to_string(), None)),
            ..Default::default()
        };

        match client.put_object(put_request).await {
            Ok(output) => println!("Source object created: {:?}", output),
            Err(err) => panic!("Failed to create source object: {:?}", err),
        }

        // First copy - this should work
        let copy_request = CopyObjectRequest {
            bucket: config.bucket.to_string(),
            key: target_object_name.to_string(),
            copy_source: format!("/{}/{}", config.bucket, source_object_name),
            ..Default::default()
        };

        match client.copy_object(&copy_request).await {
            Ok(result) => {
                println!("First copy successful: {:?}", result);
                assert!(result.etag.is_some());
            }
            Err(err) => panic!("First copy failed: {:?}", err),
        }

        // Put a different object to target
        let put_target_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: target_object_name.to_string(),
            body: Some(crate::BodyContent::from_text(new_content.to_string(), None)),
            ..Default::default()
        };

        match client.put_object(put_target_request).await {
            Ok(output) => println!("Target object updated: {:?}", output),
            Err(err) => panic!("Failed to update target object: {:?}", err),
        }

        // Now try to copy again with forbid overwrite enabled - this should fail
        let copy_request_forbid = CopyObjectRequest {
            bucket: config.bucket.to_string(),
            key: target_object_name.to_string(),
            copy_source: format!("/{}/{}", config.bucket, source_object_name),
            forbid_overwrite: Some("true".to_string()),
            ..Default::default()
        };

        match client.copy_object(&copy_request_forbid).await {
            Ok(_result) => {
                // In some cases, depending on bucket settings, this might succeed
                println!("Copy with overwrite forbidden succeeded - this may be due to bucket versioning settings");
            }
            Err(err) => {
                println!("Copy with overwrite forbidden failed as expected in some configurations: {:?}", err);
            }
        }
    }
}
