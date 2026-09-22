use std::collections::HashMap;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{add_crc64_check, modify_request, update_content_length};
use crate::{BodyContent, FeatureFlagsType, OperationInput, OperationOutput};

#[derive(Default, OssRequestModel)]
pub struct PutObjectRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// The caching behavior of the web page when the object is downloaded.
    #[field(type = "header", rename = "Cache-Control")]
    pub cache_control: Option<String>,

    /// The method that is used to access the object.
    #[field(type = "header", rename = "Content-Disposition")]
    pub content_disposition: Option<String>,

    /// The method that is used to encode the object.
    #[field(type = "header", rename = "Content-Encoding")]
    pub content_encoding: Option<String>,

    /// The size of the data in the HTTP message body. Unit: bytes.
    #[field(type = "header", rename = "Content-Length")]
    pub content_length: Option<u64>,

    /// The MD5 hash of the object that you want to upload.
    #[field(type = "header", rename = "Content-MD5")]
    pub content_md5: Option<String>,

    /// A standard MIME type describing the format of the contents.
    #[field(type = "header", rename = "Content-Type")]
    pub content_type: Option<String>,

    /// The expiration time of the cache in UTC.
    #[field(type = "header", rename = "Expires")]
    pub expires: Option<String>,

    /// Specifies whether the object that is uploaded by calling the PutObject
    /// operation overwrites an existing object that has the same name.
    /// Valid values: true and false
    #[field(type = "header", rename = "x-oss-forbid-overwrite")]
    pub forbid_overwrite: Option<String>,

    /// The encryption method on the server side when an object is created.
    /// Valid values: AES256 and KMS
    #[field(type = "header", rename = "x-oss-server-side-encryption")]
    pub server_side_encryption: Option<String>,

    /// The ID of the customer master key (CMK) that is managed by Key
    /// Management Service (KMS). This header is valid only when the
    /// x-oss-server-side-encryption header is set to KMS.
    #[field(type = "header", rename = "x-oss-server-side-data-encryption")]
    pub server_side_data_encryption: Option<String>,

    /// The ID of the customer master key (CMK) that is managed by Key
    /// Management Service (KMS).
    #[field(type = "header", rename = "x-oss-server-side-encryption-key-id")]
    pub sse_kms_key_id: Option<String>,

    /// The access control list (ACL) of the object.
    #[field(type = "header", rename = "x-oss-object-acl")]
    pub object_acl: String,

    /// The storage class of the object.
    #[field(type = "header", rename = "x-oss-storage-class")]
    pub storage_class: String,

    /// The metadata of the object that you want to upload.
    #[field(type = "header", rename = "x-oss-meta-", usermeta)]
    pub metadata: HashMap<String, String>,

    /// The tags that are specified for the object using a key-value pair.
    /// You can specify multiple tags for an object. Example: TagA=A&TagB=B.
    #[field(type = "header", rename = "x-oss-tagging")]
    pub tagging: Option<String>,

    /// A callback parameter is a Base64-encoded string that contains multiple
    /// fields in the JSON format.
    #[field(type = "header", rename = "x-oss-callback")]
    pub callback: Option<String>,

    /// Configure custom parameters by using the callback-var parameter.
    #[field(type = "header", rename = "x-oss-callback-var")]
    pub callback_var: Option<String>,

    /// Specify the speed limit value. The speed limit value ranges from 245760
    /// to 838860800, with a unit of bit/s.
    #[field(type = "header", rename = "x-oss-traffic-limit")]
    pub traffic_limit: Option<u64>,

    /// Object data.
    pub body: Option<BodyContent>, /* TODO find if rust supports Seek at
                                    * runtime */

    /// Progress callback function
    pub progress_fn: Option<Box<dyn Fn(i64, i64) + Send + Sync>>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutObjectResult {
    /// Content-Md5 for the uploaded object.
    #[field(type = "header", rename = "Content-Md5")]
    pub content_md5: Option<String>,

    /// Entity tag for the uploaded object.
    #[field(type = "header", rename = "ETag")]
    pub etag: Option<String>,

    /// The 64-bit CRC value of the object.
    /// This value is calculated based on the ECMA-182 standard.
    #[field(type = "header", rename = "x-oss-hash-crc64ecma")]
    pub hash_crc64: Option<String>,

    /// Version of the object.
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    /// Callback result. Populated with the JSON body returned by OSS when the
    /// request carries an `x-oss-callback` header and the callback is executed.
    pub callback_result: HashMap<String, serde_json::Value>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Uploads an object to the OSS bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutObjectRequest` containing the necessary
    ///   information for the upload.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `PutObjectResult` if the upload is
    /// successful, or an error if it fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::{
    /// #     GetObjectRequest, GetObjectResult, PutObjectAclRequest, PutObjectRequest,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// # use alibabacloud_oss_sdk_rust_v2::BodyContent;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutObjectRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     body: Some(BodyContent::from_text(
    ///         "Content of the object".to_string(),
    ///         None,
    ///     )),
    ///     object_acl: "default".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_object(request).await {
    ///     Ok(put_object_result) => {
    ///         println!("Object uploaded successfully: {:?}", put_object_result);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to upload object: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_object(
        &self,
        mut request: PutObjectRequest,
    ) -> Result<PutObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let headers = request.header_map();
        let queries = request.query_map();
        let bucket = request.bucket.clone();
        let key = request.key.clone();

        let mut input = OperationInput {
            op_name: "PutObject".to_string(),
            method: http::Method::PUT,
            bucket: Some(bucket),
            key: Some(key),
            body: request.body, // Move body from request
            ..Default::default()
        };

        modify_request(&mut input, headers, queries, vec![update_content_length])?;

        // Client-side CRC64 check over the bytes actually sent. Mirrors Go
        // `Client.PutObject`'s `c.addCrcCheck`.
        add_crc64_check(
            &mut input,
            0,
            self.options
                .feature_flags
                .contains(FeatureFlagsType::ENABLE_CRC64_CHECK_UPLOAD),
        );

        // The callback was declared on the request but never called; wiring it
        // here is what makes it report anything at all.
        if let Some(progress_fn) = request.progress_fn.take() {
            let total = input
                .body
                .as_ref()
                .and_then(|body| body.content_length())
                .map(|len| len as i64)
                .unwrap_or(0);
            crate::utils::add_progress_tracker(&mut input, progress_fn, total);
        }

        let mut output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutObjectResult::default();

        // When a callback is configured, OSS returns the callback result as a
        // JSON body. Mirrors Go `unmarshalCallbackBody`, which is only wired up
        // in the callback branch.
        if request.callback.is_some() {
            let body_data = output.get_all_data().await?;
            result.callback_result = serde_json::from_slice(&body_data).unwrap_or_default();
        }

        result.update_result(&output);

        Ok(result)
    }

    /// Uploads a local file as an object.
    ///
    /// The file is streamed rather than buffered, so a large upload never has
    /// to fit in memory. The file's length is taken from its metadata and
    /// becomes the request's `Content-Length`. Mirrors Go
    /// `Client.PutObjectFromFile`.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutObjectRequest`; any `body` it carries is replaced
    ///   by the file.
    /// * `file_path` - The local file to upload. A missing or unreadable file
    ///   fails before any request is sent.
    pub async fn put_object_from_file(
        &self,
        mut request: PutObjectRequest,
        file_path: impl AsRef<std::path::Path>,
    ) -> Result<PutObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = file_path.as_ref();
        let len = tokio::fs::metadata(file_path).await?.len();
        request.body = Some(BodyContent::from_file_path(file_path, len, None).await?);

        self.put_object(request).await
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use bytes::Bytes;
    use futures_util::StreamExt;

    use super::*;
    use crate::api::object::{DeleteObjectRequest, GetObjectRequest};
    use crate::client::BodyDataReader;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::test_utils::{generate_unique_object_name, load_test_config};
    use crate::SignatureVersionType;

    /// The uploaded file's bytes must arrive intact, and the request must
    /// carry the file's real length — a stream without a known length would
    /// otherwise be sent chunked.
    #[tokio::test]
    async fn test_put_object_from_file_streams_file_contents() {
        let mut server = mockito::Server::new_async().await;

        let expected = {
            let mut crc = crate::utils::Crc64::new(0);
            crc.write(b"file contents").unwrap();
            crc.sum64().to_string()
        };

        // The CRC check only passes if the body really was the file's bytes.
        let mock = server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(200)
            .with_header("x-oss-hash-crc64ecma", expected.as_str())
            .with_body("")
            .create_async()
            .await;

        let client = Client::new(
            &Config::default()
                .with_endpoint(server.url().as_str())
                .with_region("cn-hangzhou")
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    "test-ak",
                    "test-sk",
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V1)
                .with_log_level(LogLevel::Off),
        );

        let dir = std::env::temp_dir().join("oss-put-from-file-test");
        std::fs::create_dir_all(&dir).expect("Failed to create temp dir");
        let path = dir.join("upload.bin");
        std::fs::write(&path, b"file contents").expect("Failed to write temp file");

        client
            .put_object_from_file(
                PutObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "from-file".to_string(),
                    ..Default::default()
                },
                &path,
            )
            .await
            .expect("uploading a readable file must succeed");
        mock.assert_async().await;

        let _ = std::fs::remove_file(&path);
    }

    /// A missing file must fail before any request is sent.
    #[tokio::test]
    async fn test_put_object_from_file_missing_file_fails() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(200)
            .expect(0) // no request may be sent
            .create_async()
            .await;

        let client = Client::new(
            &Config::default()
                .with_endpoint(server.url().as_str())
                .with_region("cn-hangzhou")
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    "test-ak",
                    "test-sk",
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V1)
                .with_log_level(LogLevel::Off),
        );

        client
            .put_object_from_file(
                PutObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "missing".to_string(),
                    ..Default::default()
                },
                std::env::temp_dir().join("oss-put-from-file-test/does-not-exist.bin"),
            )
            .await
            .expect_err("a missing file must fail");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_put_object_crc_check_is_wired_into_the_operation() {
        let mut server = mockito::Server::new_async().await;

        // The CRC64 of the body the client will send.
        let expected = {
            let mut crc = crate::utils::Crc64::new(0);
            crc.write(b"payload").unwrap();
            crc.sum64().to_string()
        };

        // A wrong server CRC must fail the operation, proving `put_object`
        // actually attaches the check (an unattached check would succeed).
        let bad = server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(200)
            .with_header("x-oss-hash-crc64ecma", "1")
            .with_body("")
            .create_async()
            .await;

        let client = Client::new(
            &Config::default()
                .with_endpoint(server.url().as_str())
                .with_region("cn-hangzhou")
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    "test-ak",
                    "test-sk",
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V1)
                .with_log_level(LogLevel::Off)
                .with_retryer(Rc::new(crate::retry::NopRetryer::new())),
        );

        let request = PutObjectRequest {
            bucket: "test-bucket".to_string(),
            key: "crc-op-object".to_string(),
            body: Some(BodyContent::from_bytes(
                Bytes::from_static(b"payload"),
                None,
            )),
            ..Default::default()
        };

        let err = client
            .put_object(request)
            .await
            .expect_err("a CRC mismatch must fail put_object");
        assert!(
            err.to_string().contains("crc is inconsistent"),
            "unexpected error: {}",
            err
        );
        bad.assert_async().await;

        // The same operation with the correct CRC succeeds, and the expected
        // value matches what the SDK computes for this body.
        server.reset();
        let good = server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(200)
            .with_header("x-oss-hash-crc64ecma", expected.as_str())
            .with_body("")
            .create_async()
            .await;

        let request = PutObjectRequest {
            bucket: "test-bucket".to_string(),
            key: "crc-op-object".to_string(),
            body: Some(BodyContent::from_bytes(
                Bytes::from_static(b"payload"),
                None,
            )),
            ..Default::default()
        };

        client
            .put_object(request)
            .await
            .expect("a matching CRC must succeed");
        good.assert_async().await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_object_basic() {
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
        let test_object_name = generate_unique_object_name("put-object-basic");
        let test_content = "Test content for basic put object operation";

        // Create a PutObjectRequest
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            body: Some(crate::BodyContent::from_text(
                test_content.to_string(),
                None,
            )),
            object_acl: "default".to_string(),
            ..Default::default()
        };

        // Execute the put_object operation
        match client.put_object(put_request).await {
            Ok(result) => {
                println!("Object uploaded successfully: {:?}", result);
                assert!(result.etag.is_some()); // ETag should be returned upon
                                                // successful upload
            }
            Err(err) => panic!("Put object failed: {:?}", err),
        }

        // Verify the object was uploaded by getting it back
        let get_request = GetObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            ..Default::default()
        };

        match client.get_object(get_request).await {
            Ok(mut result) => {
                let content_bytes = result.get_all_data().await.unwrap_or_default();
                let content = String::from_utf8_lossy(&content_bytes).into_owned();
                assert_eq!(content, test_content);
                println!("Verified object content matches what was uploaded");
            }
            Err(err) => panic!("Failed to get object after upload: {:?}", err),
        }

        // Clean up: delete the test object
        match client
            .delete_object(DeleteObjectRequest {
                bucket: config.bucket.to_string(),
                key: test_object_name.clone(),
                ..Default::default()
            })
            .await
        {
            Ok(_) => println!("Test object cleaned up successfully"),
            Err(err) => eprintln!("Failed to clean up test object: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_object_with_custom_headers() {
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
        let test_object_name = generate_unique_object_name("put-object-headers");
        let test_content = "Test content with custom headers";

        // Create a PutObjectRequest with custom headers
        let mut put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            body: Some(crate::BodyContent::from_text(
                test_content.to_string(),
                None,
            )),
            object_acl: "private".to_string(),
            content_type: Some("text/plain".to_string()),
            cache_control: Some("max-age=3600".to_string()),
            ..Default::default()
        };

        // Add custom headers via common field
        put_request
            .common
            .headers
            .insert("x-oss-meta-author".to_string(), "test-user".to_string());

        // Execute the put_object operation
        match client.put_object(put_request).await {
            Ok(result) => {
                println!("Object uploaded with custom headers: {:?}", result);
                assert!(result.etag.is_some());
            }
            Err(err) => panic!("Put object with custom headers failed: {:?}", err),
        }

        // Clean up: delete the test object
        match client
            .delete_object(DeleteObjectRequest {
                bucket: config.bucket.to_string(),
                key: test_object_name.clone(),
                ..Default::default()
            })
            .await
        {
            Ok(_) => println!("Test object with custom headers cleaned up successfully"),
            Err(err) => eprintln!("Failed to clean up test object: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_object_empty_content() {
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
        let test_object_name = generate_unique_object_name("put-object-empty");
        let test_content = ""; // Empty content

        // Create a PutObjectRequest with empty content
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            body: Some(crate::BodyContent::from_text(
                test_content.to_string(),
                None,
            )),
            object_acl: "default".to_string(),
            ..Default::default()
        };

        // Execute the put_object operation
        match client.put_object(put_request).await {
            Ok(result) => {
                println!("Empty object uploaded successfully: {:?}", result);
                assert!(result.etag.is_some());
            }
            Err(err) => panic!("Put object with empty content failed: {:?}", err),
        }

        // Verify the object was uploaded with empty content
        let get_request = GetObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            ..Default::default()
        };

        match client.get_object(get_request).await {
            Ok(mut result) => {
                let content_bytes = result.get_all_data().await.unwrap_or_default();
                let content = String::from_utf8_lossy(&content_bytes).into_owned();
                assert_eq!(content, test_content);
                println!("Verified empty object content");
            }
            Err(err) => panic!("Failed to get empty object after upload: {:?}", err),
        }

        // Clean up: delete the test object
        match client
            .delete_object(DeleteObjectRequest {
                bucket: config.bucket.to_string(),
                key: test_object_name.clone(),
                ..Default::default()
            })
            .await
        {
            Ok(_) => println!("Empty test object cleaned up successfully"),
            Err(err) => eprintln!("Failed to clean up test object: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_object_with_bytes() {
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
        let test_object_name = generate_unique_object_name("put-object-bytes");
        let test_content = b"Test content using bytes";

        // Create a PutObjectRequest with bytes content
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            body: Some(crate::BodyContent::from_bytes(test_content.to_vec(), None)),
            object_acl: "default".to_string(),
            ..Default::default()
        };

        // Execute the put_object operation
        match client.put_object(put_request).await {
            Ok(result) => {
                println!("Object with bytes uploaded successfully: {:?}", result);
                assert!(result.etag.is_some());
            }
            Err(err) => panic!("Put object with bytes failed: {:?}", err),
        }

        // Verify the object was uploaded with bytes content
        let get_request = GetObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            ..Default::default()
        };

        match client.get_object(get_request).await {
            Ok(mut result) => {
                let content_bytes = result.get_all_data().await.unwrap_or_default();
                let content = String::from_utf8_lossy(&content_bytes).into_owned();
                assert_eq!(content, String::from_utf8_lossy(test_content).into_owned());
                println!("Verified object content matches bytes content");
            }
            Err(err) => panic!("Failed to get object after bytes upload: {:?}", err),
        }

        // Clean up: delete the test object
        match client
            .delete_object(DeleteObjectRequest {
                bucket: config.bucket.to_string(),
                key: test_object_name.clone(),
                ..Default::default()
            })
            .await
        {
            Ok(_) => println!("Test object with bytes cleaned up successfully"),
            Err(err) => eprintln!("Failed to clean up test object: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_object_with_stream() {
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
        let test_object_name = generate_unique_object_name("put-object-stream");

        // Create 10MB of test data
        const SIZE_10MB: usize = 10 * 1024 * 1024;
        let mut test_content = Vec::with_capacity(SIZE_10MB);
        for i in 0..SIZE_10MB {
            // Generate predictable pattern: A-Z repeating
            test_content.push(b'A' + (i % 26) as u8);
        }

        // Create a stream from the 10MB test content
        // Split the data into chunks for streaming
        const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks
        let stream =
            futures_util::stream::unfold((test_content.clone(), 0), |(data, pos)| async move {
                if pos >= data.len() {
                    None
                } else {
                    let end = std::cmp::min(pos + CHUNK_SIZE, data.len());
                    let chunk = Bytes::from(data[pos..end].to_vec());
                    Some((Ok(chunk), (data, end)))
                }
            });
        let byte_stream = crate::client::ByteStream::new(stream);

        // Create a PutObjectRequest with stream content
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            body: Some(crate::BodyContent::from_stream(
                byte_stream,
                SIZE_10MB as u64,
                None,
            )),
            object_acl: "default".to_string(),
            ..Default::default()
        };

        // Execute the put_object operation
        match client.put_object(put_request).await {
            Ok(result) => {
                println!("Object with stream uploaded successfully: {:?}", result);
                assert!(result.etag.is_some());
            }
            Err(err) => panic!("Put object with stream failed: {:?}", err),
        }

        // Verify the object was uploaded with stream content
        let get_request = GetObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            ..Default::default()
        };

        match client.get_object(get_request).await {
            Ok(mut result) => {
                let content_bytes = result.get_all_data().await.unwrap_or_default();
                assert_eq!(content_bytes.len(), SIZE_10MB);
                assert_eq!(content_bytes, test_content);
                println!(
                    "Verified object content matches stream content ({} bytes)",
                    content_bytes.len()
                );
            }
            Err(err) => panic!("Failed to get object after stream upload: {:?}", err),
        }

        // Clean up: delete the test object
        match client
            .delete_object(DeleteObjectRequest {
                bucket: config.bucket.to_string(),
                key: test_object_name.clone(),
                ..Default::default()
            })
            .await
        {
            Ok(_) => println!("Test object with stream cleaned up successfully"),
            Err(err) => eprintln!("Failed to clean up test object: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_object_with_large_file_stream() {
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
        let test_object_name = generate_unique_object_name("put-object-large-file-stream");

        // Create a temporary 4MB file for testing
        const SIZE_4MB: u64 = 4 * 1024 * 1024; // Changed to 1MB to avoid issues
        let temp_file_path = std::env::temp_dir().join("test_4mb_file_stream.txt");

        // Write 4MB of data to the temporary file
        let mut file_data = Vec::with_capacity(SIZE_4MB as usize);
        for i in 0..SIZE_4MB {
            file_data.push((i % 256) as u8); // Generate some pattern
        }

        std::fs::write(&temp_file_path, &file_data).expect("Failed to write temp file");

        // Create a file-based stream using tokio::fs::File and tokio-util -
        // this reads the file in chunks
        let file = tokio::fs::File::open(&temp_file_path)
            .await
            .expect("Failed to open temp file");
        let stream = tokio_util::codec::FramedRead::new(file, tokio_util::codec::BytesCodec::new());
        let byte_stream = crate::client::ByteStream::new(stream.map(|result| {
            result
                .map(|bytes_mut| bytes_mut.freeze()) // BytesMut → Bytes
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                    Box::new(e) // std::io::Error → Box<dyn Error + Send + Sync>
                })
        }));

        // Create a PutObjectRequest with file content as stream
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            body: Some(crate::BodyContent::from_stream(byte_stream, SIZE_4MB, None)),
            object_acl: "default".to_string(),
            ..Default::default()
        };

        // Execute the put_object operation
        match client.put_object(put_request).await {
            Ok(result) => {
                println!(
                    "Large object with file stream uploaded successfully {}: {:?}",
                    test_object_name, result
                );
                assert!(result.etag.is_some());
                println!("Uploaded 4MB file stream with ETag: {:?}", result.etag);
            }
            Err(err) => panic!("Put large object with file stream failed: {:?}", err),
        }

        // Verify the object was uploaded with correct size
        let get_request = GetObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            ..Default::default()
        };

        match client.get_object(get_request).await {
            Ok(mut result) => match result.get_all_data().await {
                Ok(content) => {
                    assert_eq!(content.len(), SIZE_4MB as usize);
                    println!(
                        "Verified large object content size: {} bytes",
                        content.len()
                    );
                }
                Err(err) => panic!(
                    "Failed to get large object after file stream upload: {:?}",
                    err
                ),
            },
            Err(err) => panic!(
                "Failed to get large object after file stream upload: {:?}",
                err
            ),
        }

        // Clean up: delete the test object
        match client
            .delete_object(DeleteObjectRequest {
                bucket: config.bucket.to_string(),
                key: test_object_name.clone(),
                ..Default::default()
            })
            .await
        {
            Ok(_) => println!("Large test object with file stream cleaned up successfully"),
            Err(err) => eprintln!("Failed to clean up large test object: {:?}", err),
        }

        // Remove the temporary file
        if temp_file_path.exists() {
            std::fs::remove_file(temp_file_path).expect("Failed to remove temp file");
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_object_with_file_stream_content() {
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
        let test_object_name = generate_unique_object_name("put-object-file-stream");

        // Create a temporary file for testing
        let temp_file_path = std::env::temp_dir().join("test_file_stream.txt");
        let file_content = "This is test content for file stream upload functionality";
        std::fs::write(&temp_file_path, file_content).expect("Failed to write temp file");

        // Create a file-based stream using tokio::fs::File and tokio-util -
        // this reads the file in chunks
        let file = tokio::fs::File::open(&temp_file_path)
            .await
            .expect("Failed to open temp file");
        let stream = tokio_util::codec::FramedRead::new(file, tokio_util::codec::BytesCodec::new());
        let byte_stream = crate::client::ByteStream::new(stream.map(|result| {
            result
                .map(|bytes_mut| bytes_mut.freeze()) // BytesMut → Bytes
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                    Box::new(e) // std::io::Error → Box<dyn Error + Send + Sync>
                })
        }));

        // Create a PutObjectRequest with file content as stream
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            body: Some(crate::BodyContent::from_stream(
                byte_stream,
                file_content.len() as u64,
                None,
            )),
            object_acl: "default".to_string(),
            ..Default::default()
        };

        // Execute the put_object operation
        match client.put_object(put_request).await {
            Ok(result) => {
                println!(
                    "Object with file stream content uploaded successfully: {:?}",
                    result
                );
                assert!(result.etag.is_some());
                println!("Uploaded file with ETag: {:?}", result.etag);
            }
            Err(err) => panic!("Put object with file stream content failed: {:?}", err),
        }

        // Verify the object was uploaded with correct content
        let get_request = GetObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            ..Default::default()
        };

        match client.get_object(get_request).await {
            Ok(mut result) => {
                let content_bytes = result.get_all_data().await.unwrap_or_default();
                let content = String::from_utf8_lossy(&content_bytes).into_owned();
                assert_eq!(content, file_content);
                println!("Verified file object content matches: {}", content);
            }
            Err(err) => panic!("Failed to get object after file stream upload: {:?}", err),
        }

        // Clean up: delete the test object
        match client
            .delete_object(DeleteObjectRequest {
                bucket: config.bucket.to_string(),
                key: test_object_name.clone(),
                ..Default::default()
            })
            .await
        {
            Ok(_) => println!("Test object with file stream content cleaned up successfully"),
            Err(err) => eprintln!("Failed to clean up test object: {:?}", err),
        }

        // Remove the temporary file
        if temp_file_path.exists() {
            std::fs::remove_file(temp_file_path).expect("Failed to remove temp file");
        }
    }
}
