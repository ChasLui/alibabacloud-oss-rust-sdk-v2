use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeletePublicAccessBlockRequest {
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeletePublicAccessBlockResult {
    pub common: ResultCommon,
}

impl Client {
    /// Deletes the Block Public Access configurations of the Object Storage
    /// Service (OSS) resources of the current account.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeletePublicAccessBlockRequest`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::service::DeletePublicAccessBlockRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeletePublicAccessBlockRequest::default();
    ///
    /// match client.delete_public_access_block(&request).await {
    ///     Ok(result) => {
    ///         println!("status: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete public access block: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_public_access_block(
        &self,
        request: &DeletePublicAccessBlockRequest,
    ) -> Result<DeletePublicAccessBlockResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeletePublicAccessBlock".to_string(),
            method: http::Method::DELETE,
            parameters: [("publicAccessBlock", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        // The SubResource metadata is required by the V1 signer.
        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["publicAccessBlock".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = DeletePublicAccessBlockResult::default();
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
    async fn test_delete_public_access_block() {
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

        // Deleting a non-existent account-level configuration is idempotent
        // on the service side.
        let result = client
            .delete_public_access_block(&DeletePublicAccessBlockRequest::default())
            .await;
        assert!(
            result.is_ok(),
            "delete_public_access_block failed: {:?}",
            result.err()
        );
    }
}
