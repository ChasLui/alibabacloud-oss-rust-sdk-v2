use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{acl_grant_de, modify_request};
use crate::{OperationInput, OperationOutput};

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
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::object::tests::{delete_multiple, put, put_with_meta, TEST_OBJECT_CONTENT, TEST_OBJECT_NAME, generate_unique_object_name};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::{SignatureVersionType, HTTP_HEADER_CONTENT_RANGE};
    use crate::test_utils::{load_test_config, TestConfig};

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


