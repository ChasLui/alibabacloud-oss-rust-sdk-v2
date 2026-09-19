use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteBucketPolicyRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteBucketPolicyResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes a policy for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteBucketPolicyRequest` containing the bucket
    ///   name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::DeleteBucketPolicyRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteBucketPolicyRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_bucket_policy(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket policy deleted: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete bucket policy: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_bucket_policy(
        &self,
        request: &DeleteBucketPolicyRequest,
    ) -> Result<DeleteBucketPolicyResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteBucketPolicy".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            parameters: [("policy", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["policy".to_string()]),
        );

        modify_request(&mut input, request.header_map(), request.query_map(), vec![])?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteBucketPolicyResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_bucket_policy() {
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
                .with_signature_version(SignatureVersionType::V4),
        );

        // Prepare a bucket policy to delete
        let policy = format!(
            "{{\"Version\":\"1\",\"Statement\":[{{\"Action\":[\"oss:GetObject\"],\"Effect\":\"Deny\",\"Principal\":[\"1234567890\"],\"Resource\":[\"acs:oss:*:*:{}/*\"]}}]}}",
            config.bucket
        );
        let _ = client
            .put_bucket_policy(&crate::api::bucket::PutBucketPolicyRequest {
                bucket: config.bucket.clone(),
                policy,
                ..Default::default()
            })
            .await;

        let result = client
            .delete_bucket_policy(&DeleteBucketPolicyRequest {
                bucket: config.bucket.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "delete_bucket_policy failed: {:?}",
            result.err()
        );
    }
}
