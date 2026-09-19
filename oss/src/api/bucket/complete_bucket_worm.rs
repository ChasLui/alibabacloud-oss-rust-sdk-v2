use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct CompleteBucketWormRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The ID of the retention policy.
    #[field(type = "query", rename = "wormId")]
    pub worm_id: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct CompleteBucketWormResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Locks a retention policy.
    ///
    /// # Arguments
    ///
    /// * `request` - The `CompleteBucketWormRequest` containing the bucket
    ///   name and the ID of the retention policy.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::CompleteBucketWormRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = CompleteBucketWormRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     worm_id: Some("worm-id".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.complete_bucket_worm(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket worm locked: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to complete bucket worm: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn complete_bucket_worm(
        &self,
        request: &CompleteBucketWormRequest,
    ) -> Result<CompleteBucketWormResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "CompleteBucketWorm".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };
        input
            .op_metadata
            .set(SUB_RESOURCE, Rc::new(vec!["wormId".to_string()]));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = CompleteBucketWormResult::default();
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
    use crate::SignatureVersionType;
    use crate::test_utils::load_test_config;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_complete_bucket_worm() {
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

        let bucket_name = crate::test_utils::generate_unique_bucket_name("worm-complete");

        client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let worm_id = client
            .initiate_bucket_worm(&crate::api::bucket::InitiateBucketWormRequest {
                bucket: bucket_name.clone(),
                initiate_worm_configuration:
                    crate::api::bucket::InitiateWormConfiguration {
                        retention_period_in_days: Some(1),
                    },
                ..Default::default()
            })
            .await
            .unwrap()
            .worm_id
            .unwrap();

        let result = client
            .complete_bucket_worm(&CompleteBucketWormRequest {
                bucket: bucket_name.clone(),
                worm_id: Some(worm_id),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "complete_bucket_worm failed: {:?}",
            result.err()
        );

        // Clean up: the empty bucket can be deleted even with a locked policy
        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
