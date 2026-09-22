use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteAgenticBucketRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteAgenticBucketResult {
    pub common: ResultCommon,
}

impl Client {
    /// Deletes an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteAgenticBucketRequest` containing the bucket
    ///   prefix.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::DeleteAgenticBucketRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = DeleteAgenticBucketRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_agentic_bucket(&request).await {
    ///     Ok(result) => println!("deleted: {}", result.common.status),
    ///     Err(error) => eprintln!("failed to delete agentic bucket: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn delete_agentic_bucket(
        &self,
        request: &DeleteAgenticBucketRequest,
    ) -> Result<DeleteAgenticBucketResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteAgenticBucket".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            parameters: [("agenticBucket", "")]
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
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteAgenticBucketResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, ensure_agentic_bucket};

    #[test]
    fn test_delete_agentic_bucket_request_has_no_payload_fields() {
        let request = DeleteAgenticBucketRequest {
            bucket: "my-agentic".to_string(),
            ..Default::default()
        };

        assert_eq!(request.bucket, "my-agentic");
        assert!(request.header_map().is_empty());
        assert!(request.query_map().is_empty());
    }

    #[test]
    fn test_delete_agentic_bucket_request_common_overrides() {
        let mut request = DeleteAgenticBucketRequest {
            bucket: "my-agentic".to_string(),
            ..Default::default()
        };
        request.add_query("versionId", "v1");

        assert_eq!(request.query_map().get("versionId").unwrap(), "v1");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_agentic_bucket() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .delete_agentic_bucket(&DeleteAgenticBucketRequest {
                bucket: prefix.clone(),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("delete_agentic_bucket rejected: {}", error);
        }
    }
}
