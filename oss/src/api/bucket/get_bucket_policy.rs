use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketPolicyRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetBucketPolicyResult {
    /// The configurations of the bucket policy in JSON format.
    pub policy: Option<String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Queries the policies configured for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketPolicyRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketPolicyRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketPolicyRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_policy(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket policy: {:?}", result.policy);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket policy: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_policy(
        &self,
        request: &GetBucketPolicyRequest,
    ) -> Result<GetBucketPolicyResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketPolicy".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("policy", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
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

        let mut output = self.invoke_operation(input, vec![]).await?;

        // The response body is a plain JSON policy text, not XML.
        let body_data = output.get_all_data().await?;
        let mut result = GetBucketPolicyResult::default();
        result.policy = Some(String::from_utf8_lossy(&body_data).into_owned());
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
    async fn test_get_bucket_policy() {
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

        // Prepare a bucket policy
        let policy = format!(
            "{{\"Version\":\"1\",\"Statement\":[{{\"Action\":[\"oss:GetObject\"],\"Effect\":\"Deny\",\"Principal\":[\"1234567890\"],\"Resource\":[\"acs:oss:*:*:{}/*\"]}}]}}",
            config.bucket
        );
        client
            .put_bucket_policy(&crate::api::bucket::PutBucketPolicyRequest {
                bucket: config.bucket.clone(),
                policy,
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .get_bucket_policy(&GetBucketPolicyRequest {
                bucket: config.bucket.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_policy failed: {:?}",
            result.err()
        );
        let body = result.unwrap().policy.unwrap_or_default();
        assert!(body.contains("\"Version\""));

        // Clean up
        let _ = client
            .delete_bucket_policy(&crate::api::bucket::DeleteBucketPolicyRequest {
                bucket: config.bucket.clone(),
                ..Default::default()
            })
            .await;
    }
}
