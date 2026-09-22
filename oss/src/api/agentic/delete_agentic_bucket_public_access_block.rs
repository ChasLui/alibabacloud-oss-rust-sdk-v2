use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteAgenticBucketPublicAccessBlockRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteAgenticBucketPublicAccessBlockResult {
    pub common: ResultCommon,
}

impl Client {
    /// Deletes the Block Public Access configuration of an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteAgenticBucketPublicAccessBlockRequest`
    ///   containing the bucket prefix.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::DeleteAgenticBucketPublicAccessBlockRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = DeleteAgenticBucketPublicAccessBlockRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_agentic_bucket_public_access_block(&request).await {
    ///     Ok(result) => println!("deleted: {}", result.common.status),
    ///     Err(error) => eprintln!("failed to delete agentic bucket public access block: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn delete_agentic_bucket_public_access_block(
        &self,
        request: &DeleteAgenticBucketPublicAccessBlockRequest,
    ) -> Result<DeleteAgenticBucketPublicAccessBlockResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "DeleteAgenticBucketPublicAccessBlock".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            parameters: [("agenticBucket", ""), ("publicAccessBlock", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteAgenticBucketPublicAccessBlockResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, ensure_agentic_bucket};

    #[test]
    fn test_delete_agentic_bucket_public_access_block_request_has_no_payload_fields() {
        let request = DeleteAgenticBucketPublicAccessBlockRequest {
            bucket: "my-agentic".to_string(),
            ..Default::default()
        };

        assert!(request.header_map().is_empty());
        assert!(request.query_map().is_empty());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_agentic_bucket_public_access_block() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .delete_agentic_bucket_public_access_block(
                &DeleteAgenticBucketPublicAccessBlockRequest {
                    bucket: prefix.clone(),
                    ..Default::default()
                },
            )
            .await;
        if let Err(error) = &result {
            eprintln!(
                "delete_agentic_bucket_public_access_block rejected: {}",
                error
            );
        }
    }
}
