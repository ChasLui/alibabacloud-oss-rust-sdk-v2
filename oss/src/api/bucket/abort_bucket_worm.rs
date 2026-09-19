use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct AbortBucketWormRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl AbortBucketWormRequest {
    pub fn new(bucket: &str) -> Self {
        AbortBucketWormRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct AbortBucketWormResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes an unlocked retention policy for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `AbortBucketWormRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::AbortBucketWormRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = AbortBucketWormRequest::new("my-bucket");
    ///
    /// match client.abort_bucket_worm(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket worm aborted: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to abort bucket worm: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn abort_bucket_worm(
        &self,
        request: &AbortBucketWormRequest,
    ) -> Result<AbortBucketWormResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "AbortBucketWorm".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            parameters: [("worm", "")]
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
            .set(SUB_RESOURCE, Rc::new(vec!["worm".to_string()]));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = AbortBucketWormResult::default();
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
    async fn test_abort_bucket_worm() {
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

        let bucket_name = crate::test_utils::generate_unique_bucket_name("worm-abort");

        client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        client
            .initiate_bucket_worm(&crate::api::bucket::InitiateBucketWormRequest {
                bucket: bucket_name.clone(),
                initiate_worm_configuration:
                    crate::api::bucket::InitiateWormConfiguration {
                        retention_period_in_days: Some(1),
                    },
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .abort_bucket_worm(&AbortBucketWormRequest::new(&bucket_name))
            .await;
        assert!(
            result.is_ok(),
            "abort_bucket_worm failed: {:?}",
            result.err()
        );

        // Clean up
        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
