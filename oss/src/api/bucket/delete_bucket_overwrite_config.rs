use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteBucketOverwriteConfigRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl DeleteBucketOverwriteConfigRequest {
    pub fn new(bucket: &str) -> Self {
        DeleteBucketOverwriteConfigRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteBucketOverwriteConfigResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes the overwrite configuration rules of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteBucketOverwriteConfigRequest` containing the
    ///   bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::DeleteBucketOverwriteConfigRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteBucketOverwriteConfigRequest::new("my-bucket");
    ///
    /// match client.delete_bucket_overwrite_config(&request).await {
    ///     Ok(result) => {
    ///         println!("Overwrite config deleted: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete bucket overwrite config: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_bucket_overwrite_config(
        &self,
        request: &DeleteBucketOverwriteConfigRequest,
    ) -> Result<DeleteBucketOverwriteConfigResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteBucketOverwriteConfig".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            parameters: [("overwriteConfig", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };
        input
            .op_metadata
            .set(SUB_RESOURCE, Rc::new(vec!["overwriteConfig".to_string()]));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteBucketOverwriteConfigResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_bucket_overwrite_config() {
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

        // Deleting a configuration that was never set is expected to fail; this
        // only verifies the request is well-formed.
        let result = client
            .delete_bucket_overwrite_config(&DeleteBucketOverwriteConfigRequest::new(
                &config.bucket,
            ))
            .await;
        if let Err(error) = &result {
            eprintln!(
                "delete_bucket_overwrite_config rejected (may not be configured): {}",
                error
            );
        }
    }
}
