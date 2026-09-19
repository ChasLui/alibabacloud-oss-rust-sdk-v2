use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::SSERule;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketEncryptionRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetBucketEncryptionResult {
    /// The container that stores the default server-side encryption method.
    #[serde(rename = "ApplyServerSideEncryptionByDefault", skip_serializing_if = "Option::is_none")]
    pub apply_server_side_encryption_by_default: Option<SSERule>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the encryption rules configured for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketEncryptionRequest` containing the bucket
    ///   name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketEncryptionRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketEncryptionRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_encryption(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket encryption: {:?}", result.apply_server_side_encryption_by_default);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket encryption: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_encryption(
        &self,
        request: &GetBucketEncryptionRequest,
    ) -> Result<GetBucketEncryptionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketEncryption".to_string(),
            method: http::Method::GET,
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

        modify_request(&mut input, request.header_map(), request.query_map(), vec![])?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetBucketEncryptionResult = quick_xml::de::from_str(&data_str)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{PutBucketEncryptionRequest, ServerSideEncryptionRule};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_encryption() {
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

        // Prepare an encryption rule
        client
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
            .await
            .unwrap();

        let result = client
            .get_bucket_encryption(&GetBucketEncryptionRequest {
                bucket: config.bucket.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_encryption failed: {:?}",
            result.err()
        );
        let rule = result.unwrap().apply_server_side_encryption_by_default.unwrap();
        assert_eq!(rule.sse_algorithm.as_deref(), Some("AES256"));

        // Clean up
        let _ = client
            .delete_bucket_encryption(&crate::api::bucket::DeleteBucketEncryptionRequest {
                bucket: config.bucket.clone(),
                ..Default::default()
            })
            .await;
    }
}
