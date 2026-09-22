use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use super::SSERule;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores server-side encryption rules.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ServerSideEncryptionRule {
    /// The container that stores the default server-side encryption method.
    #[serde(
        rename = "ApplyServerSideEncryptionByDefault",
        skip_serializing_if = "Option::is_none"
    )]
    pub apply_server_side_encryption_by_default: Option<SSERule>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteBucketEncryptionRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteBucketEncryptionResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes encryption rules for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteBucketEncryptionRequest` containing the bucket
    ///   name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::DeleteBucketEncryptionRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteBucketEncryptionRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_bucket_encryption(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket encryption deleted: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete bucket encryption: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_bucket_encryption(
        &self,
        request: &DeleteBucketEncryptionRequest,
    ) -> Result<DeleteBucketEncryptionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteBucketEncryption".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            parameters: [("encryption", "")]
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
            std::rc::Rc::new(vec!["encryption".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteBucketEncryptionResult::default();
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
    async fn test_delete_bucket_encryption() {
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

        // Prepare an encryption rule to delete
        let _ = client
            .put_bucket_encryption(&crate::api::bucket::PutBucketEncryptionRequest {
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

        let result = client
            .delete_bucket_encryption(&DeleteBucketEncryptionRequest {
                bucket: config.bucket.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "delete_bucket_encryption failed: {:?}",
            result.err()
        );
    }
}
