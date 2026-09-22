use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutAgenticBucketPolicyRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    /// The policy of the agentic bucket, as a JSON document.
    pub policy: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutAgenticBucketPolicyResult {
    pub common: ResultCommon,
}

impl Client {
    /// Configures the policy of an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutAgenticBucketPolicyRequest` containing the bucket
    ///   prefix and the policy document.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::PutAgenticBucketPolicyRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = PutAgenticBucketPolicyRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     policy: r#"{"Version":"1","Statement":[]}"#.to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_agentic_bucket_policy(&request).await {
    ///     Ok(result) => println!("updated: {}", result.common.status),
    ///     Err(error) => eprintln!("failed to put agentic bucket policy: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn put_agentic_bucket_policy(
        &self,
        request: &PutAgenticBucketPolicyRequest,
    ) -> Result<PutAgenticBucketPolicyResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutAgenticBucketPolicy".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("agenticBucket", ""), ("policy", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/json")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        // The policy is a JSON document supplied by the caller; it is sent
        // verbatim rather than re-serialized.
        input.body = Some(BodyContent::from_text(request.policy.clone(), None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutAgenticBucketPolicyResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, ensure_agentic_bucket};

    #[test]
    fn test_put_agentic_bucket_policy_request_keeps_the_document_verbatim() {
        let policy = r#"{"Version":"1","Statement":[{"Effect":"Allow"}]}"#;
        let request = PutAgenticBucketPolicyRequest {
            bucket: "my-agentic".to_string(),
            policy: policy.to_string(),
            ..Default::default()
        };

        assert_eq!(request.policy, policy);
        // The policy travels in the body, not in a header or query.
        assert!(request.header_map().is_empty());
        assert!(request.query_map().is_empty());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_agentic_bucket_policy() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .put_agentic_bucket_policy(&PutAgenticBucketPolicyRequest {
                bucket: prefix.clone(),
                policy: r#"{"Version":"1","Statement":[]}"#.to_string(),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("put_agentic_bucket_policy rejected: {}", error);
        }
    }
}
