use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{acl_grant_de, modify_request};
use crate::{OperationInput, OperationOutput, ServiceError};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetObjectMetaRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// The version ID of the source object.
    #[field(type = "query", rename = "versionId")]
    pub version_id: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}


#[derive(Debug, Default,Deserialize, OssResultModel)]
pub struct GetObjectMetaResult {
    #[field(type = "header", rename = "Content-Length")]
    pub content_length: Option<u64>,

    #[field(type = "header", rename = "ETag")]
    pub etag: Option<String>,

    #[field(type = "header", rename = "x-oss-transition-time")]
    pub x_oss_transition_time: Option<String>,

    #[field(type = "header", rename = "x-oss-last-access-time")]
    pub x_oss_last_access_time: Option<String>,

    #[field(type = "header", rename = "Last-Modified")]
    pub last_modified: Option<String>,

    #[field(type = "header", rename = "x-oss-sealed-time")]
    pub x_oss_sealed_time: Option<String>,

    /// Version of the object.
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    /// The 64-bit CRC value of the object.
    /// This value is calculated based on the ECMA-182 standard.
    #[field(type = "header", rename = "x-oss-hash-crc64ecma")]
    pub hash_crc64: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Retrieves the metadata for an object in the OSS bucket.
    ///
    /// This method sends a HEAD request to the OSS server to retrieve the metadata
    /// for the specified object. It returns a `GetObjectMetaResult` struct
    /// containing the metadata information.
    ///
    /// # Arguments
    ///
    /// * `request` - A reference to a `GetObjectMetaRequest` struct that
    ///   specifies the bucket and key of the object.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `GetObjectMetaResult` on success, or a boxed
    /// `dyn std::error::Error` on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::GetObjectMetaRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetObjectMetaRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_object_meta(&request).await {
    ///     Ok(result) => {
    ///         println!("Content Length: {:?}", result.content_length);
    ///         println!("ETag: {:?}", result.etag);
    ///     }
    ///     Err(err) => {
    ///         eprintln!("Error: {}", err);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_object_meta(
        &self,
        request: &GetObjectMetaRequest,
    ) -> Result<GetObjectMetaResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetObjectMeta".to_string(),
            method: http::Method::HEAD,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("objectMeta", "")]
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

        let output = self.invoke_operation(input, vec![]).await?;

        // let mut result: GetObjectMetaResult =
        //     quick_xml::de::from_str(output.body.clone().unwrap_or_default().as_str())?;

        let mut result: GetObjectMetaResult = GetObjectMetaResult::default();

        result.update_result(&output);

        Ok(result)
    }

    /// Checks whether an object exists.
    ///
    /// A `NoSuchKey` response means the object is absent, which is reported as
    /// `Ok(false)`; every other failure — including permission and network
    /// errors — is propagated, so a caller never mistakes "could not check"
    /// for "does not exist". The probe is `GetObjectMeta`, whose HEAD request
    /// returns the same object-not-found code without transferring the body.
    /// Mirrors Go `Client.IsObjectExist`.
    pub async fn is_object_exist(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        self.is_object_exist_with_options(bucket, key, IsObjectExistOptions::default())
            .await
    }

    /// [`Client::is_object_exist`] with the version ID and payer options.
    pub async fn is_object_exist_with_options(
        &self,
        bucket: &str,
        key: &str,
        options: IsObjectExistOptions,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let err = match self
            .get_object_meta(&GetObjectMetaRequest {
                bucket: bucket.to_string(),
                key: key.to_string(),
                version_id: options.version_id,
                request_payer: options.request_payer,
                ..Default::default()
            })
            .await
        {
            Ok(_) => return Ok(true),
            Err(err) => err,
        };

        // A missing object may arrive as `NoSuchKey`, or as a 404 whose body
        // OSS replaced with a plain error (`BadErrorResponse`), in which case
        // the real code never makes it into the parsed error. Mirrors the two
        // conditions in Go `Client.IsObjectExist`.
        if let Some(service_error) = err.downcast_ref::<ServiceError>() {
            if service_error.code == "NoSuchKey"
                || (service_error.status_code == http::StatusCode::NOT_FOUND
                    && service_error.code == "BadErrorResponse")
            {
                return Ok(false);
            }
        }

        Err(err)
    }
}

/// Optional parameters for [`Client::is_object_exist_with_options`].
#[derive(Debug, Default)]
pub struct IsObjectExistOptions {
    /// The version ID of the object.
    pub version_id: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs.
    pub request_payer: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;

    /// Builds a client pointed at a mock server, so the existence probes can
    /// be driven without real credentials.
    pub(crate) async fn mock_client(server: &mockito::ServerGuard) -> Client {
        Client::new(
            &Config::default()
                .with_endpoint(server.url().as_str())
                .with_region("cn-hangzhou")
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    "test-ak", "test-sk", &[],
                )))
                .with_signature_version(SignatureVersionType::V1)
                .with_log_level(LogLevel::Off),
        )
    }
    use crate::api::object::tests::{delete_multiple, put, put_with_meta, TEST_OBJECT_CONTENT, TEST_OBJECT_NAME, generate_unique_object_name};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::{SignatureVersionType, HTTP_HEADER_CONTENT_RANGE};
    use crate::test_utils::{load_test_config, TestConfig};

    /// A missing object is reported as `false`, not as an error.
    #[tokio::test]
    async fn test_is_object_exist_missing_object_is_false() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(404)
            .with_header("content-type", "application/xml")
            .with_body("<Error><Code>NoSuchKey</Code><Message>no</Message></Error>")
            .create_async()
            .await;

        let client = super::tests::mock_client(&server).await;

        assert!(!client
            .is_object_exist("test-bucket", "missing")
            .await
            .expect("a missing object must not be an error"));
        mock.assert_async().await;
    }

    /// A present object is reported as `true`.
    #[tokio::test]
    async fn test_is_object_exist_present_object_is_true() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", "3")
            .create_async()
            .await;

        let client = super::tests::mock_client(&server).await;

        assert!(client
            .is_object_exist("test-bucket", "present")
            .await
            .expect("a present object must not be an error"));
        mock.assert_async().await;
    }

    /// A non-404 service failure must surface as an error rather than being
    /// silently turned into `false`.
    ///
    /// A HEAD response carries no body, so the real error code never reaches
    /// the client and every service failure is reported as `BadErrorResponse`.
    /// That makes the status code the only discriminator — exactly why Go's
    /// `IsObjectExist` pairs `BadErrorResponse` with a 404 check, and why a 403
    /// has to fall through to the error branch.
    #[tokio::test]
    async fn test_is_object_exist_non_404_failure_is_an_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(403)
            .create_async()
            .await;

        let client = super::tests::mock_client(&server).await;

        let err = client
            .is_object_exist("test-bucket", "secret")
            .await
            .expect_err("a permission failure must not be reported as a missing object");
        assert!(
            err.to_string().contains("403"),
            "the underlying failure must survive: {}",
            err
        );
        mock.assert_async().await;
    }

    /// A 404 without a parseable error body is how OSS reports a missing
    /// object to a HEAD probe, and must be reported as `false`.
    #[tokio::test]
    async fn test_is_object_exist_404_bad_error_response_is_false() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(404)
            .create_async()
            .await;

        let client = super::tests::mock_client(&server).await;

        assert!(!client
            .is_object_exist("test-bucket", "missing")
            .await
            .expect("a bodiless 404 must count as a missing object"));
        mock.assert_async().await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_object_meta() {
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

        // put object
        match client.put_object(put_request).await {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

       match client.get_object_meta(&GetObjectMetaRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.to_string(), // Use unique test object name
            ..Default::default()
        })
        .await {
            Ok(result) => {
                println!("getObjectMeta Result {:?}", result);
                // check status
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
    async fn test_get_object_meta_with_user_defined_meta() {
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
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

        match client.get_object_meta(&GetObjectMetaRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.to_string(), // Use unique test object name
            ..Default::default()
        })
            .await {
            Ok(result) => {
                println!("getObjectMeta Result {:?}", result);
                // check status
                assert_eq!(result.common.status, http::StatusCode::OK);

                // Note: User-defined metadata headers (x-oss-meta-*) do not return in the response
                assert_eq!(result.common.headers.get("x-oss-meta-author"), None);
                assert_eq!(result.common.headers.get("x-oss-meta-version"), None);
                assert_eq!(result.common.headers.get("x-oss-meta-description"), None);
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
}


