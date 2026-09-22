use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutObjectAclRequest {
    /// The name of the bucket containing the objects
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// The access control list (ACL) of the object.
    /// Accepted values: public-read-write, public-read, private, default
    #[field(type = "header", rename = "x-oss-object-acl")]
    pub acl: String,

    /// The version ID of the source object.
    #[field(type = "query", rename = "versionId")]
    pub version_id: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutObjectAclResult {
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    pub common: ResultCommon,
}

impl Client {
    /// Sets the access control list (ACL) for an object in the OSS bucket.
    ///
    /// This method sends a PUT request to the OSS server to update the ACL of
    /// the specified object.
    ///
    /// # Arguments
    ///
    /// * `request` - A reference to a `PutObjectAclRequest` struct that
    ///   contains the necessary parameters for the request.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `PutObjectAclResult` struct if the
    /// operation is successful, or a boxed `dyn std::error::Error` trait object
    /// if an error occurs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::{
    /// #     GetObjectRequest, GetObjectResult, PutObjectAclRequest,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutObjectAclRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     acl: "public-read-write".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_object_acl(&request).await {
    ///     Ok(result) => {
    ///         println!("Object ACL updated successfully: {:?}", result);
    ///     }
    ///     Err(err) => {
    ///         eprintln!("Failed to update object ACL: {:?}", err);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_object_acl(
        &self,
        request: &PutObjectAclRequest,
    ) -> Result<PutObjectAclResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutObjectAcl".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("acl", "")]
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

        let mut result = PutObjectAclResult::default();

        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::object::{DeleteObjectRequest, GetObjectAclRequest, PutObjectRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::test_utils::{generate_unique_object_name, load_test_config};
    use crate::SignatureVersionType;

    // Configuration structure to hold test credentials
    // Using shared TestConfig from test_utils

    // Function to load test configuration from file
    // Using shared load_test_config from test_utils

    async fn put_acl(client: &Client, bucket: &str, key: &str, acl: &str) {
        match client
            .put_object_acl(&PutObjectAclRequest {
                bucket: bucket.to_string(),
                key: key.to_string(),
                acl: acl.to_string(),
                ..Default::default()
            })
            .await
        {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }
    }

    async fn assert_acl(client: &Client, bucket: &str, key: &str, acl: &str) {
        match client
            .get_object_acl(&GetObjectAclRequest {
                bucket: bucket.to_string(),
                key: key.to_string(),
                ..Default::default()
            })
            .await
        {
            Ok(output) => {
                println!("{:?}", output);
                assert_eq!(output.acl.as_ref().unwrap(), acl);
            }
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_object_acl() {
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
        let test_object_name = generate_unique_object_name("put-object-acl");

        // First create an object
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            body: Some(crate::BodyContent::from_text(
                "Test content for ACL test".to_string(),
                None,
            )),
            ..Default::default()
        };

        match client.put_object(put_request).await {
            Ok(output) => println!("Object created for ACL test: {:?}", output),
            Err(err) => panic!("Failed to create object for ACL test: {:?}", err),
        }

        // make modification
        put_acl(&client, &config.bucket, &test_object_name, "public-read").await;

        // check whether modification is made
        assert_acl(&client, &config.bucket, &test_object_name, "public-read").await;

        // revert to default
        put_acl(&client, &config.bucket, &test_object_name, "default").await;

        // check whether modification is reverted
        assert_acl(&client, &config.bucket, &test_object_name, "default").await;

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
}
