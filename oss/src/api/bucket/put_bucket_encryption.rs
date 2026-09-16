use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::delete_bucket_encryption::ServerSideEncryptionRule;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{
    BodyContent, OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE,
};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketEncryptionRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The container that stores server-side encryption rules.
    pub server_side_encryption_rule: ServerSideEncryptionRule,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketEncryptionResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Configures encryption rules for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketEncryptionRequest` containing the bucket
    ///   name and the encryption rules to set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{PutBucketEncryptionRequest, ServerSideEncryptionRule, SSERule};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketEncryptionRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     server_side_encryption_rule: ServerSideEncryptionRule {
    ///         apply_server_side_encryption_by_default: Some(SSERule {
    ///             sse_algorithm: Some("AES256".to_string()),
    ///             ..Default::default()
    ///         }),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_encryption(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket encryption updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket encryption: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_encryption(
        &self,
        request: &PutBucketEncryptionRequest,
    ) -> Result<PutBucketEncryptionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketEncryption".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("encryption", "")]
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
            std::rc::Rc::new(vec!["encryption".to_string()]),
        );

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

        let mut result = PutBucketEncryptionResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::SSERule;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_server_side_encryption_rule_serde_round_trip() {
        let rule = ServerSideEncryptionRule {
            apply_server_side_encryption_by_default: Some(SSERule {
                sse_algorithm: Some("KMS".to_string()),
                kms_master_key_id: Some("key-id-123".to_string()),
                kms_data_encryption: Some("SM4".to_string()),
            }),
        };

        let xml = quick_xml::se::to_string_with_root("ServerSideEncryptionRule", &rule).unwrap();
        assert!(xml.contains("<ServerSideEncryptionRule>"));
        assert!(xml.contains("<ApplyServerSideEncryptionByDefault>"));
        assert!(xml.contains("<SSEAlgorithm>KMS</SSEAlgorithm>"));
        assert!(xml.contains("<KMSMasterKeyID>key-id-123</KMSMasterKeyID>"));
        assert!(xml.contains("<KMSDataEncryption>SM4</KMSDataEncryption>"));

        let parsed: ServerSideEncryptionRule = quick_xml::de::from_str(&xml).unwrap();
        let default_rule = parsed.apply_server_side_encryption_by_default.unwrap();
        assert_eq!(default_rule.sse_algorithm.as_deref(), Some("KMS"));
        assert_eq!(default_rule.kms_master_key_id.as_deref(), Some("key-id-123"));
        assert_eq!(default_rule.kms_data_encryption.as_deref(), Some("SM4"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_encryption() {
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

        let result = client
            .put_bucket_encryption(&PutBucketEncryptionRequest {
                bucket: config.bucket.clone(),
                server_side_encryption_rule: ServerSideEncryptionRule {
                    apply_server_side_encryption_by_default: Some(SSERule {
                        sse_algorithm: Some("AES256".to_string()),
                        ..Default::default()
                    }),
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "put_bucket_encryption failed: {:?}",
            result.err()
        );

        // Clean up
        let _ = client
            .delete_bucket_encryption(&crate::api::bucket::DeleteBucketEncryptionRequest {
                bucket: config.bucket.clone(),
                ..Default::default()
            })
            .await;
    }
}
