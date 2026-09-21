use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{
    BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE,
};

/// The container of the bucket configuration sent as the request body.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CreateBucketConfiguration {
    /// The storage class of the bucket.
    #[serde(rename = "StorageClass", skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,

    /// The redundancy type of the bucket.
    #[serde(rename = "DataRedundancyType", skip_serializing_if = "Option::is_none")]
    pub data_redundancy_type: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct CreateBucketRequest {
    /// The name of the bucket to create.
    pub bucket: String,

    /// The ACL for the bucket.
    #[field(type = "header", rename = "x-oss-acl")]
    pub acl: Option<String>,

    /// The container of the bucket configuration sent as the request body.
    /// `None` sends no body, matching the Go SDK's nil `CreateBucketConfiguration`.
    pub create_bucket_configuration: Option<CreateBucketConfiguration>,

    /// The ID of the resource group.
    #[field(type = "header", rename = "x-oss-resource-group-id")]
    pub resource_group_id: Option<String>,

    /// The agentic bucket name.
    #[field(type = "header", rename = "x-oss-agentic-bucket")]
    pub agentic_bucket: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct CreateBucketResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Creates a new bucket.
    ///
    /// This method sends a PUT request to create a new bucket with the specified name and properties.
    ///
    /// # Arguments
    ///
    /// * `request` - The `CreateBucketRequest` containing the bucket name and properties to create.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `CreateBucketResult` if the creation is
    /// successful, or an error if it fails. Note that:
    /// - Bucket names must be globally unique across all users of OSS
    /// - Bucket names must conform to specific naming rules
    /// - Creating a bucket that already exists will return an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::CreateBucketRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = CreateBucketRequest {
    ///     bucket: "my-new-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.create_bucket(&request).await {
    ///     Ok(create_bucket_result) => {
    ///         println!("Bucket created successfully");
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to create bucket: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn create_bucket(
        &self,
        request: &CreateBucketRequest,
    ) -> Result<CreateBucketResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            // Go names this operation "PutBucket" (api_op_bucket.go); OSS serves
            // bucket creation under PUT /?bucket, and upstream labels it
            // PutBucket. op_name feeds logging and operation-error wrapping; it
            // must match upstream so logs stay comparable across SDKs.
            op_name: "PutBucket".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        if let Some(config) = &request.create_bucket_configuration {
            let xml_body =
                quick_xml::se::to_string_with_root("CreateBucketConfiguration", config)?;
            input.body = Some(BodyContent::from_text(xml_body, None));
        }

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = CreateBucketResult::default();

        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::DeleteBucketRequest;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::SignatureVersionType;
    use crate::test_utils::{load_test_config, TestConfig, generate_unique_bucket_name};

    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_bucket_basic() {
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
        let bucket_name = generate_unique_bucket_name("create-bucket-test");

        let create_request = CreateBucketRequest {
            bucket: bucket_name.clone(),
            ..Default::default()
        };

        match client.create_bucket(&create_request).await {
            Ok(result) => {
                println!("Bucket created successfully: {:?}", result);
                assert_eq!(result.common.status, http::StatusCode::OK);
                
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
                    Err(err) => eprintln!("Failed to delete bucket when creat err: {:?}", err),
                }
                panic!("Create bucket failed: {:?}", err);
            }
        }
    }
}