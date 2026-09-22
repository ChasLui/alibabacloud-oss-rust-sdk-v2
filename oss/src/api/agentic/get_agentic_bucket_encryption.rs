use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::bucket::ServerSideEncryptionRule;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetAgenticBucketEncryptionRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetAgenticBucketEncryptionResult {
    /// The server-side encryption rule of the agentic bucket.
    #[serde(skip)]
    pub server_side_encryption_rule: Option<ServerSideEncryptionRule>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the server-side encryption rule of an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetAgenticBucketEncryptionRequest` containing the
    ///   bucket prefix.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::GetAgenticBucketEncryptionRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = GetAgenticBucketEncryptionRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_agentic_bucket_encryption(&request).await {
    ///     Ok(result) => println!("{:?}", result.server_side_encryption_rule),
    ///     Err(error) => eprintln!("failed to get agentic bucket encryption: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn get_agentic_bucket_encryption(
        &self,
        request: &GetAgenticBucketEncryptionRequest,
    ) -> Result<GetAgenticBucketEncryptionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetAgenticBucketEncryption".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("agenticBucket", ""), ("encryption", "")]
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

        let body_data = output.get_all_data().await?;
        let mut result = GetAgenticBucketEncryptionResult::default();
        if !body_data.is_empty() {
            let rule: ServerSideEncryptionRule =
                quick_xml::de::from_str(&String::from_utf8_lossy(&body_data))?;
            result.server_side_encryption_rule = Some(rule);
        }
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, ensure_agentic_bucket};

    #[test]
    fn test_get_agentic_bucket_encryption_result_deserialize() {
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<ServerSideEncryptionRule>
  <ApplyServerSideEncryptionByDefault>
    <SSEAlgorithm>KMS</SSEAlgorithm>
    <KMSMasterKeyID>9468da86-3509-4f8d-a61e-6eab****</KMSMasterKeyID>
    <KMSDataEncryption>SM4</KMSDataEncryption>
  </ApplyServerSideEncryptionByDefault>
</ServerSideEncryptionRule>"#;

        let rule: ServerSideEncryptionRule = quick_xml::de::from_str(body).unwrap();
        let default = rule.apply_server_side_encryption_by_default.unwrap();

        assert_eq!(default.sse_algorithm.as_deref(), Some("KMS"));
        assert_eq!(
            default.kms_master_key_id.as_deref(),
            Some("9468da86-3509-4f8d-a61e-6eab****")
        );
        assert_eq!(default.kms_data_encryption.as_deref(), Some("SM4"));
    }

    #[test]
    fn test_get_agentic_bucket_encryption_result_deserialize_empty_rule() {
        let rule: ServerSideEncryptionRule =
            quick_xml::de::from_str("<ServerSideEncryptionRule/>").unwrap();

        assert!(rule.apply_server_side_encryption_by_default.is_none());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_agentic_bucket_encryption() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .get_agentic_bucket_encryption(&GetAgenticBucketEncryptionRequest {
                bucket: prefix.clone(),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("get_agentic_bucket_encryption rejected: {}", error);
        }
    }
}
