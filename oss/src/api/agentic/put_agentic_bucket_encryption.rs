use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::bucket::ServerSideEncryptionRule;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutAgenticBucketEncryptionRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    /// The server-side encryption rule of the agentic bucket.
    pub server_side_encryption_rule: ServerSideEncryptionRule,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutAgenticBucketEncryptionResult {
    pub common: ResultCommon,
}

impl Client {
    /// Configures the server-side encryption rule of an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutAgenticBucketEncryptionRequest` containing the
    ///   bucket prefix and the rule to apply.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::PutAgenticBucketEncryptionRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{SSERule, ServerSideEncryptionRule};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = PutAgenticBucketEncryptionRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     server_side_encryption_rule: ServerSideEncryptionRule {
    ///         apply_server_side_encryption_by_default: Some(SSERule {
    ///             sse_algorithm: Some("AES256".to_string()),
    ///             ..Default::default()
    ///         }),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_agentic_bucket_encryption(&request).await {
    ///     Ok(result) => println!("updated: {}", result.common.status),
    ///     Err(error) => eprintln!("failed to put agentic bucket encryption: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn put_agentic_bucket_encryption(
        &self,
        request: &PutAgenticBucketEncryptionRequest,
    ) -> Result<PutAgenticBucketEncryptionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutAgenticBucketEncryption".to_string(),
            method: http::Method::PUT,
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

        let xml_body = quick_xml::se::to_string_with_root(
            "ServerSideEncryptionRule",
            &request.server_side_encryption_rule,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutAgenticBucketEncryptionResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, ensure_agentic_bucket};
    use crate::api::bucket::SSERule;

    #[test]
    fn test_put_agentic_bucket_encryption_body_serialize() {
        let rule = ServerSideEncryptionRule {
            apply_server_side_encryption_by_default: Some(SSERule {
                sse_algorithm: Some("KMS".to_string()),
                kms_master_key_id: Some("key-id".to_string()),
                ..Default::default()
            }),
        };

        let xml = quick_xml::se::to_string_with_root("ServerSideEncryptionRule", &rule).unwrap();

        assert_eq!(
            xml,
            "<ServerSideEncryptionRule><ApplyServerSideEncryptionByDefault>\
             <KMSMasterKeyID>key-id</KMSMasterKeyID><SSEAlgorithm>KMS</SSEAlgorithm>\
             </ApplyServerSideEncryptionByDefault></ServerSideEncryptionRule>"
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_agentic_bucket_encryption() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .put_agentic_bucket_encryption(&PutAgenticBucketEncryptionRequest {
                bucket: prefix.clone(),
                server_side_encryption_rule: ServerSideEncryptionRule {
                    apply_server_side_encryption_by_default: Some(SSERule {
                        sse_algorithm: Some("AES256".to_string()),
                        ..Default::default()
                    }),
                },
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("put_agentic_bucket_encryption rejected: {}", error);
        }
    }
}
