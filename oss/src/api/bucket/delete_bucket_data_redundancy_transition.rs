use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::OperationInput;
use crate::OperationOutput;

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteBucketDataRedundancyTransitionRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The ID of the redundancy type change task.
    #[field(type = "query", rename = "x-oss-redundancy-transition-taskid")]
    pub redundancy_transition_taskid: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteBucketDataRedundancyTransitionResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes a redundancy type conversion task of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteBucketDataRedundancyTransitionRequest`
    ///   containing the bucket name and the ID of the redundancy type change
    ///   task.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::DeleteBucketDataRedundancyTransitionRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteBucketDataRedundancyTransitionRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     redundancy_transition_taskid: Some("task-id".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_bucket_data_redundancy_transition(&request).await {
    ///     Ok(result) => {
    ///         println!("Transition deleted: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete redundancy transition: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_bucket_data_redundancy_transition(
        &self,
        request: &DeleteBucketDataRedundancyTransitionRequest,
    ) -> Result<DeleteBucketDataRedundancyTransitionResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "DeleteBucketDataRedundancyTransition".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            parameters: [("redundancyTransition", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["redundancyTransition".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteBucketDataRedundancyTransitionResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::CreateBucketDataRedundancyTransitionRequest;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_bucket_data_redundancy_transition() {
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

        // Create a task to delete; creating a real LRS->ZRS conversion may be
        // rejected depending on the bucket state, in which case skip.
        let created = client
            .create_bucket_data_redundancy_transition(
                &CreateBucketDataRedundancyTransitionRequest {
                    bucket: config.bucket.clone(),
                    target_redundancy_type: Some("ZRS".to_string()),
                    ..Default::default()
                },
            )
            .await;

        let task_id = match created {
            Ok(result) => result
                .bucket_data_redundancy_transition
                .and_then(|t| t.task_id),
            Err(error) => {
                eprintln!(
                    "create_bucket_data_redundancy_transition rejected, skipping delete: {}",
                    error
                );
                return;
            }
        };

        if let Some(task_id) = task_id {
            let result = client
                .delete_bucket_data_redundancy_transition(
                    &DeleteBucketDataRedundancyTransitionRequest {
                        bucket: config.bucket.clone(),
                        redundancy_transition_taskid: Some(task_id),
                        ..Default::default()
                    },
                )
                .await;
            assert!(
                result.is_ok(),
                "delete_bucket_data_redundancy_transition failed: {:?}",
                result.err()
            );
        }
    }
}
