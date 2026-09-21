use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput};
use std::collections::HashMap;

#[derive(Debug, Default, OssRequestModel)]
pub struct HeadObjectRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// If the ETag specified in the request matches the ETag value of the
    /// object, the object and 200 OK are returned. Otherwise, 412
    /// Precondition Failed is returned.
    #[field(type = "header", rename = "If-Match")]
    pub if_match: Option<String>,

    /// If the ETag specified in the request does not match the ETag value of
    /// the object, the object and 200 OK are returned. Otherwise, 304 Not
    /// Modified is returned.
    #[field(type = "header", rename = "If-None-Match")]
    pub if_none_match: Option<String>,

    /// If the time specified in this header is earlier than the object modified
    /// time or is invalid, the object and 200 OK are returned. Otherwise,
    /// 304 Not Modified is returned. The time must be in GMT. Example: Fri,
    /// 13 Nov 2015 14:47:53 GMT.
    #[field(type = "header", rename = "If-Modified-Since")]
    pub if_modified_since: Option<String>,

    /// If the time specified in this header is the same as or later than the
    /// object modified time, the object and 200 OK are returned. Otherwise,
    /// 412 Precondition Failed is returned. The time must be in GMT.
    /// Example: Fri, 13 Nov 2015 14:47:53 GMT.
    #[field(type = "header", rename = "If-Unmodified-Since")]
    pub if_unmodified_since: Option<String>,

    /// VersionId used to reference a specific version of the object.
    #[field(type = "query", rename = "versionId")]
    pub version_id: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct HeadObjectResult {
    /// Size of the body in bytes. -1 indicates that the Content-Length does not
    /// exist.
    #[field(type = "header", rename = "Content-Length")]
    pub content_length: Option<u64>,

    /// A standard MIME type describing the format of the object data.
    #[field(type = "header", rename = "Content-Type")]
    pub content_type: Option<String>,

    /// The entity tag (ETag). An ETag is created when an object is created to
    /// identify the content of the object.
    #[field(type = "header", rename = "ETag")]
    pub etag: Option<String>,

    /// The time when the returned objects were last modified.
    #[field(type = "header", rename = "Last-Modified")]
    pub last_modified: Option<String>,

    /// The storage class of the object.
    #[field(type = "header", rename = "x-oss-storage-class")]
    pub storage_class: Option<String>,

    /// Content-Md5 for the uploaded object.
    #[field(type = "header", rename = "Content-MD5")]
    pub content_md5: Option<String>,

    /// If the requested object is encrypted by using a server-side encryption
    /// algorithm based on entropy encoding, OSS automatically decrypts the
    /// object and returns the decrypted object after OSS receives the GetObject
    /// request. The x-oss-server-side-encryption header is included in the
    /// response to indicate the encryption algorithm used to encrypt the
    /// object on the server.
    #[field(type = "header", rename = "x-oss-server-side-encryption")]
    pub server_side_encryption: Option<String>,

    /// The ID of the customer master key (CMK) that is managed by Key
    /// Management Service (KMS).
    #[field(type = "header", rename = "x-oss-server-side-data-encryption")]
    pub server_side_data_encryption: Option<String>,

    /// The ID of the customer master key (CMK) that is managed by Key
    /// Management Service (KMS).
    #[field(type = "header", rename = "x-oss-server-side-encryption-key-id")]
    pub sse_kms_key_id: Option<String>,

    /// The type of the object.
    #[field(type = "header", rename = "x-oss-object-type")]
    pub object_type: Option<String>,

    /// The position for the next append operation.
    /// If the type of the object is Appendable, this header is included in the
    /// response.
    #[field(type = "header", rename = "x-oss-next-append-position")]
    pub next_append_position: Option<String>,

    /// The 64-bit CRC value of the object.
    /// This value is calculated based on the ECMA-182 standard.
    #[field(type = "header", rename = "x-oss-hash-crc64ecma")]
    pub hash_crc64: Option<String>,

    /// The lifecycle information about the object.
    /// If lifecycle rules are configured for the object, this header is
    /// included in the response. This header contains the following
    /// parameters: expiry-date that indicates the expiration time of the
    /// object, and rule-id that indicates the ID of the matched lifecycle
    /// rule.
    #[field(type = "header", rename = "x-oss-expiration")]
    pub expiration: Option<String>,

    /// The status of the object when you restore an object.
    /// If the storage class of the bucket is Archive and a RestoreObject
    /// request is submitted,
    #[field(type = "header", rename = "x-oss-restore")]
    pub restore: Option<String>,

    /// The result of an event notification that is triggered for the object.
    #[field(type = "header", rename = "x-oss-process-status")]
    pub process_status: Option<String>,

    /// The number of tags added to the object.
    /// This header is included in the response only when you have read
    /// permissions on tags.
    #[field(type = "header", rename = "x-oss-tagging-count")]
    pub tagging_count: Option<u32>,

    /// Specifies whether the object retrieved was (true) or was not (false) a
    /// Delete Marker.
    #[field(type = "header", rename = "x-oss-delete-marker")]
    pub delete_marker: Option<bool>,

    /// Version of the object.
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    /// The time when the object was sealed.
    #[field(type = "header", rename = "x-oss-sealed-time")]
    pub sealed_time: Option<String>,

    /// The time when the object was transitioned.
    #[field(type = "header", rename = "x-oss-transition-time")]
    pub transition_time: Option<String>,

    /// The last access time of the object.
    #[field(type = "header", rename = "x-oss-last-access-time")]
    pub last_access_time: Option<String>,

    /// The requester. This header is included in the response if the
    /// pay-by-requester mode is enabled for the bucket and the requester is not
    /// the bucket owner.
    #[field(type = "header", rename = "x-oss-request-charged")]
    pub request_charged: Option<String>,

    /// The origins allowed for cross-origin resource sharing (CORS).
    #[field(type = "header", rename = "Access-Control-Allow-Origin")]
    pub allow_origin: Option<String>,

    /// The methods allowed for CORS.
    #[field(type = "header", rename = "Access-Control-Allow-Methods")]
    pub allow_methods: Option<String>,

    /// The maximum caching period for CORS.
    #[field(type = "header", rename = "Access-Control-Allow-Age")]
    pub allow_age: Option<String>,

    /// The headers allowed for CORS.
    #[field(type = "header", rename = "Access-Control-Allow-Headers")]
    pub allow_headers: Option<String>,

    /// The headers that can be accessed by JavaScript applications on the
    /// client.
    #[field(type = "header", rename = "Access-Control-Expose-Headers")]
    pub expose_headers: Option<String>,

    /// The caching behavior of the web page when the object is downloaded.
    #[field(type = "header", rename = "Cache-Control")]
    pub cache_control: Option<String>,

    /// The method that is used to access the object.
    #[field(type = "header", rename = "Content-Disposition")]
    pub content_disposition: Option<String>,

    /// The method that is used to encode the object.
    #[field(type = "header", rename = "Content-Encoding")]
    pub content_encoding: Option<String>,

    /// The expiration time of the cache in UTC.
    #[field(type = "header", rename = "Expires")]
    pub expires: Option<String>,

    /// A map of metadata to store with the object.
    #[field(type = "header", rename = "x-oss-meta-", usermeta)]
    pub metadata: HashMap<String, String>,

    pub common: ResultCommon,
}

impl Client {
    /// Retrieves the metadata of an object in the OSS bucket without returning the content.
    ///
    /// This method sends a HEAD request to the OSS API to retrieve the metadata
    /// of the specified object. It takes a `HeadObjectRequest` as input and returns a
    /// `Result` containing the `HeadObjectResult` or an error implementing
    /// the `std::error::Error` trait.
    ///
    /// # Arguments
    ///
    /// * `request` - The `HeadObjectRequest` specifying the bucket and key of
    ///   the object to retrieve metadata for.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `HeadObjectResult` if the metadata was retrieved
    /// successfully, or an error implementing the `std::error::Error` trait
    /// if an error occurred.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::{HeadObjectRequest, HeadObjectResult};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = HeadObjectRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.head_object(request).await {
    ///     Ok(result) => {
    ///         // Metadata retrieved successfully
    ///         println!("Object content length: {:?}", result.content_length);
    ///         println!("Object ETag: {:?}", result.etag);
    ///     }
    ///     Err(err) => {
    ///         // Error occurred while retrieving the metadata
    ///         eprintln!("Error: {}", err);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn head_object(
        &self,
        request: HeadObjectRequest,
    ) -> Result<HeadObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "HeadObject".to_string(),
            method: http::Method::HEAD,
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

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = HeadObjectResult::default();

        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::object::tests::{delete_multiple, put, TEST_OBJECT_CONTENT, TEST_OBJECT_NAME};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::{SignatureVersionType, HTTP_HEADER_CONTENT_RANGE};
    use crate::test_utils::{load_test_config, TestConfig, generate_unique_object_name};

    // Configuration structure to hold test credentials
    // Using shared TestConfig from test_utils

    // Function to load test configuration from file
    // Using shared load_test_config from test_utils

    // Helper function to generate unique test object names
    // Using shared generate_unique_object_name from test_utils

    #[tokio::test]
    #[serial_test::serial]
    async fn test_head_object() {
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
        let test_object_name = generate_unique_object_name("basic");
        
        // Create a PutObjectRequest with the unique test object name
        use std::sync::{Arc, Mutex};
        use crate::api::object::PutObjectRequest;
        use crate::api::object::tests::TEST_OBJECT_CONTENT;

        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(), // Use unique test object name
            body: Some(crate::BodyContent::from_text(TEST_OBJECT_CONTENT.to_string(), None)),
            ..Default::default()
        };

        // put object first
        match client.put_object(put_request).await {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

        // test head object to retrieve metadata
        match client
            .head_object(HeadObjectRequest {
                bucket: config.bucket.to_string(),
                key: test_object_name.clone(), // Use unique test object name
                ..Default::default()
            })
            .await
        {
            Ok(result) => {
                println!("{:?}", result);

                // check status
                assert_eq!(result.common.status, http::StatusCode::OK);
                
                // check that content length matches expected content
                assert_eq!(result.content_length, Some(TEST_OBJECT_CONTENT.len() as u64));
            }
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

        // delete object using delete_multiple_objects with single object
        use crate::api::object::DeleteMultipleObjectsRequest;
        use crate::api::object::DeleteObject;
        match client.delete_multiple_objects(DeleteMultipleObjectsRequest {
            bucket: config.bucket.to_string(),
            objects: vec![DeleteObject {
                key: test_object_name, // Use unique test object name
                ..Default::default()
            }],
            ..Default::default()
        }).await {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_head_object_with_conditions() {
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
        let test_object_name = generate_unique_object_name("conditions");
        
        // Create a PutObjectRequest with the unique test object name
        use std::sync::{Arc, Mutex};
        use crate::api::object::PutObjectRequest;
        use crate::api::object::tests::TEST_OBJECT_CONTENT;

        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(), // Use unique test object name
            body: Some(crate::BodyContent::from_text(TEST_OBJECT_CONTENT.to_string(), None)),
            ..Default::default()
        };

        // put object first
        match client.put_object(put_request).await {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

        // test head object with if-none-match condition
        match client
            .head_object(HeadObjectRequest {
                bucket: config.bucket.to_string(),
                key: test_object_name.clone(), // Use unique test object name
                if_none_match: Some("\"non-matching-etag\"".to_string()), // Should return 200 since ETag doesn't match
                ..Default::default()
            })
            .await
        {
            Ok(result) => {
                println!("{:?}", result);

                // check status - should be 200 since ETag doesn't match
                assert_eq!(result.common.status, http::StatusCode::OK);
            }
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

        // delete object using delete_multiple_objects with single object
        use crate::api::object::DeleteMultipleObjectsRequest;
        use crate::api::object::DeleteObject;
        match client.delete_multiple_objects(DeleteMultipleObjectsRequest {
            bucket: config.bucket.to_string(),
            objects: vec![DeleteObject {
                key: test_object_name, // Use unique test object name
                ..Default::default()
            }],
            ..Default::default()
        }).await {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_head_object_with_user_defined_meta() {
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
        let test_object_name = generate_unique_object_name("meta");
        
        // Create a PutObjectRequest with custom metadata
        use std::sync::{Arc, Mutex};
        use crate::api::object::PutObjectRequest;
        use crate::api::object::tests::TEST_OBJECT_CONTENT;

        let mut put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(), // Use unique test object name
            body: Some(crate::BodyContent::from_text(TEST_OBJECT_CONTENT.to_string(), None)),
            ..Default::default()
        };

        // Add custom metadata using the common headers
        put_request.add_header("x-oss-meta-author", "rust-sdk-test");
        put_request.add_header("x-oss-meta-version", "1.0");
        put_request.add_header("x-oss-meta-description", "Test object with custom metadata");

        // put object with user defined meta
        match client.put_object(put_request).await {
            Ok(output) => println!("PutObject result: {:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

        // test head object to retrieve metadata including user-defined metadata
        match client
            .head_object(HeadObjectRequest {
                bucket: config.bucket.to_string(),
                key: test_object_name.clone(), // Use unique test object name
                ..Default::default()
            })
            .await
        {
            Ok(result) => {
                println!("{:?}", result);

                // check status
                assert_eq!(result.common.status, http::StatusCode::OK);
                
                // check that user-defined metadata is returned in the response headers
                assert_eq!(result.common.headers.get("x-oss-meta-author").unwrap(), "rust-sdk-test");
                assert_eq!(result.common.headers.get("x-oss-meta-version").unwrap(), "1.0");
                assert_eq!(result.common.headers.get("x-oss-meta-description").unwrap(), "Test object with custom metadata");
            }
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

        // delete object using delete_multiple_objects with single object
        use crate::api::object::DeleteMultipleObjectsRequest;
        use crate::api::object::DeleteObject;
        match client.delete_multiple_objects(DeleteMultipleObjectsRequest {
            bucket: config.bucket.to_string(),
            objects: vec![DeleteObject {
                key: test_object_name, // Use unique test object name
                ..Default::default()
            }],
            ..Default::default()
        }).await {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }
    }

    // A simple unit test that doesn't require network access
    #[test]
    fn test_head_object_request_creation() {
        let request = HeadObjectRequest {
            bucket: "test-bucket".to_string(),
            key: "test-key".to_string(),
            if_match: Some("test-etag".to_string()),
            ..Default::default()
        };

        assert_eq!(request.bucket, "test-bucket");
        assert_eq!(request.key, "test-key");
        assert_eq!(request.if_match, Some("test-etag".to_string()));
    }

    // A simple unit test that doesn't require network access
    #[test]
    fn test_head_object_result_creation() {
        let result = HeadObjectResult {
            content_length: Some(1024),
            content_type: Some("text/plain".to_string()),
            etag: Some("\"test-etag\"".to_string()),
            ..Default::default()
        };

        assert_eq!(result.content_length, Some(1024));
        assert_eq!(result.content_type, Some("text/plain".to_string()));
        assert_eq!(result.etag, Some("\"test-etag\"".to_string()));
    }
}