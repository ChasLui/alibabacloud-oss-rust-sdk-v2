use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::Owner;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{acl_grant_de, modify_request};
use crate::{
    OperationInput, OperationOutput, ServiceError, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE,
};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketAclRequest {
    /// The name of the bucket containing the objects
    pub bucket: String,

    pub common: RequestCommon,
}

impl GetBucketAclRequest {
    pub fn new(bucket: &str) -> Self {
        Self {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, OssResultModel)]
pub struct GetBucketAclResult {
    /// The container that stores the access control list (ACL) information
    /// about the bucket.
    #[serde(rename = "AccessControlList", with = "acl_grant_de")]
    pub acl: Option<String>,

    /// The container that stores information about the bucket owner.
    #[serde(rename = "Owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<Owner>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Retrieves the access control list (ACL) for a bucket.
    ///
    /// This method sends a GET request to the OSS server to retrieve the ACL
    /// for the specified bucket. It returns a `Result` containing the
    /// `GetBucketAclResult` if the request is successful, or an error if it
    /// fails.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketAclRequest` object containing the bucket
    ///   name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketAclRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketAclRequest::new("my-bucket");
    ///
    /// match client.get_bucket_acl(&request).await {
    ///     Ok(acl) => {
    ///         // Handle the ACL response
    ///     }
    ///     Err(err) => {
    ///         // Handle the error
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_acl(
        &self,
        request: &GetBucketAclRequest,
    ) -> Result<GetBucketAclResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketAcl".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("acl", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
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

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetBucketAclResult =
            quick_xml::de::from_str(&data_str)?;

        result.update_result(&output);

        Ok(result)
    }

    /// Checks whether a bucket exists.
    ///
    /// The probe is `GetBucketAcl`. Only `NoSuchBucket` means the bucket is
    /// absent (`Ok(false)`); any other *service* error still proves the bucket
    /// was reached, so it reports `Ok(true)` — a permission or region error is
    /// not evidence that the bucket is missing. A failure that never reached
    /// the service (transport error) is propagated. Mirrors Go
    /// `Client.IsBucketExist`.
    pub async fn is_bucket_exist(
        &self,
        bucket: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let request = GetBucketAclRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        };

        match self.get_bucket_acl(&request).await {
            Ok(_) => Ok(true),
            Err(err) => match err.downcast_ref::<ServiceError>() {
                Some(service_error) if service_error.code == "NoSuchBucket" => Ok(false),
                Some(_) => Ok(true),
                None => Err(err),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config, TestConfig};
    use crate::SignatureVersionType;

    /// Builds a client pointed at a mock server.
    fn mock_client(server: &mockito::ServerGuard) -> Client {
        Client::new(
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
        )
    }

    /// `NoSuchBucket` is the one answer that means the bucket is absent.
    #[tokio::test]
    async fn test_is_bucket_exist_no_such_bucket_is_false() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(404)
            .with_header("content-type", "application/xml")
            .with_body("<Error><Code>NoSuchBucket</Code><Message>no</Message></Error>")
            .create_async()
            .await;

        let client = mock_client(&server);

        assert!(!client
            .is_bucket_exist("missing-bucket")
            .await
            .expect("a missing bucket must not be an error"));
        mock.assert_async().await;
    }

    /// A reachable bucket answers `true`.
    #[tokio::test]
    async fn test_is_bucket_exist_present_bucket_is_true() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/xml")
            .with_body("<AccessControlPolicy><AccessControlList><Grant>private</Grant></AccessControlList><Owner><ID>o</ID></Owner></AccessControlPolicy>")
            .create_async()
            .await;

        let client = mock_client(&server);

        assert!(client
            .is_bucket_exist("bucket")
            .await
            .expect("a present bucket must not be an error"));
        mock.assert_async().await;
    }

    /// A service error other than `NoSuchBucket` still proves the bucket was
    /// reached, so it reports `true` rather than failing — an access-denied
    /// answer is not evidence that the bucket is gone. Mirrors Go
    /// `Client.IsBucketExist`, which returns `true` for any other service
    /// error and only propagates failures that never reached the service.
    #[tokio::test]
    async fn test_is_bucket_exist_other_service_error_is_true() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(403)
            .with_header("content-type", "application/xml")
            .with_body("<Error><Code>AccessDenied</Code><Message>denied</Message></Error>")
            .create_async()
            .await;

        let client = mock_client(&server);

        assert!(client
            .is_bucket_exist("private-bucket")
            .await
            .expect("an access-denied answer must not be an error"));
        mock.assert_async().await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_acl() {
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

        // Generate a unique bucket name for this test
        let bucket_name = generate_unique_bucket_name("get-bucket-acl");

        // Create the bucket first
        let create_request = CreateBucketRequest {
            bucket: bucket_name.clone(),
            ..Default::default()
        };

        match client.create_bucket(&create_request).await {
            Ok(_) => println!("Bucket created: {}", bucket_name),
            Err(err) => panic!("Failed to create bucket: {:?}", err),
        };

        // Perform the test
        match client.get_bucket_acl(&GetBucketAclRequest::new(&bucket_name)).await {
            Ok(output) => {
                println!("{:?}", output);
                // Clean up: delete the bucket
                let delete_request = DeleteBucketRequest {
                    bucket: bucket_name.clone(),
                    ..Default::default()
                };
                match client.delete_bucket(&delete_request).await {
                    Ok(_) => println!("Bucket deleted: {}", bucket_name),
                    Err(err) => eprintln!("Failed to delete bucket: {:?}", err),
                }
            }
            Err(err) => {
                // Even if the test fails, try to clean up
                let delete_request = DeleteBucketRequest {
                    bucket: bucket_name.clone(),
                    ..Default::default()
                };
                match client.delete_bucket(&delete_request).await {
                    Ok(_) => println!("Bucket deleted: {}", bucket_name),
                    Err(err) => eprintln!("Failed to delete bucket: {:?}", err),
                }
                panic!("Invoke operation failed: {:?}", err);
            }
        }
    }
}
