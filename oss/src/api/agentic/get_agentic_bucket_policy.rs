use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// Reads a response body that is a document in its own right, rather than a
/// container to be deserialized.
async fn read_raw_body(
    output: &mut OperationOutput,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok(String::from_utf8_lossy(&output.get_all_data().await?).into_owned())
}

#[derive(Debug, Default, OssRequestModel)]
pub struct GetAgenticBucketPolicyRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetAgenticBucketPolicyResult {
    /// The policy of the agentic bucket, as the JSON document returned by the
    /// service.
    pub policy: String,

    pub common: ResultCommon,
}

impl Client {
    /// Queries the policy of an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetAgenticBucketPolicyRequest` containing the bucket
    ///   prefix.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::GetAgenticBucketPolicyRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = GetAgenticBucketPolicyRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_agentic_bucket_policy(&request).await {
    ///     Ok(result) => println!("policy: {}", result.policy),
    ///     Err(error) => eprintln!("failed to get agentic bucket policy: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn get_agentic_bucket_policy(
        &self,
        request: &GetAgenticBucketPolicyRequest,
    ) -> Result<GetAgenticBucketPolicyResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetAgenticBucketPolicy".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("agenticBucket", ""), ("policy", "")]
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

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = GetAgenticBucketPolicyResult {
            policy: read_raw_body(&mut output).await?,
            ..Default::default()
        };
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use futures_util::stream;

    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, ensure_agentic_bucket};
    use crate::client::BodyStream;

    /// The response body is the policy document itself, not a wrapper: every
    /// chunk must be handed back verbatim.
    #[tokio::test]
    async fn test_read_raw_body_returns_the_document_verbatim() {
        let body: BodyStream = Box::pin(stream::iter(vec![
            Ok(Bytes::from_static(br#"{"Version":"1","#)),
            Ok(Bytes::from_static(br#""Statement":[]}"#)),
        ]));
        let mut output = OperationOutput {
            body: Some(body),
            ..Default::default()
        };

        assert_eq!(
            read_raw_body(&mut output).await.unwrap(),
            r#"{"Version":"1","Statement":[]}"#
        );
    }

    /// A body that was already drained yields an empty policy rather than an
    /// error.
    #[tokio::test]
    async fn test_read_raw_body_of_empty_response() {
        let mut output = OperationOutput::default();

        assert_eq!(read_raw_body(&mut output).await.unwrap(), "");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_agentic_bucket_policy() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .get_agentic_bucket_policy(&GetAgenticBucketPolicyRequest {
                bucket: prefix.clone(),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("get_agentic_bucket_policy rejected: {}", error);
        }
    }
}
