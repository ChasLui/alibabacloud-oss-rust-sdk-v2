use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketPolicyRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The bucket policy in JSON format.
    pub policy: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketPolicyResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Configures a policy for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketPolicyRequest` containing the bucket name
    ///   and the policy text in JSON format.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::PutBucketPolicyRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketPolicyRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     policy: "{\"Version\":\"1\",\"Statement\":[]}".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_policy(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket policy updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket policy: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_policy(
        &self,
        request: &PutBucketPolicyRequest,
    ) -> Result<PutBucketPolicyResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketPolicy".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("policy", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            // The request body is a plain JSON policy text, not XML.
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/json")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["policy".to_string()]),
        );

        input.body = Some(BodyContent::from_text(request.policy.clone(), None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketPolicyResult::default();
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
    async fn test_put_bucket_policy() {
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

        let policy = format!(
            "{{\"Version\":\"1\",\"Statement\":[{{\"Action\":[\"oss:GetObject\"],\"Effect\":\"Deny\",\"Principal\":[\"1234567890\"],\"Resource\":[\"acs:oss:*:*:{}/*\"]}}]}}",
            config.bucket
        );

        let result = client
            .put_bucket_policy(&PutBucketPolicyRequest {
                bucket: config.bucket.clone(),
                policy,
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "put_bucket_policy failed: {:?}",
            result.err()
        );

        // Clean up
        let _ = client
            .delete_bucket_policy(&crate::api::bucket::DeleteBucketPolicyRequest {
                bucket: config.bucket.clone(),
                ..Default::default()
            })
            .await;
    }
}
