use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::create_bucket_data_redundancy_transition::BucketDataRedundancyTransition;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::OperationInput;
use crate::OperationOutput;

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketDataRedundancyTransitionRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The ID of the redundancy change task.
    #[field(type = "query", rename = "x-oss-redundancy-transition-taskid")]
    pub redundancy_transition_taskid: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetBucketDataRedundancyTransitionResult {
    /// The container for a specific redundancy type change task.
    pub bucket_data_redundancy_transition: Option<BucketDataRedundancyTransition>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Queries the redundancy type conversion tasks of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketDataRedundancyTransitionRequest` containing
    ///   the bucket name and the ID of the redundancy change task.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketDataRedundancyTransitionRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketDataRedundancyTransitionRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     redundancy_transition_taskid: Some("task-id".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_data_redundancy_transition(&request).await {
    ///     Ok(result) => {
    ///         println!("Transition: {:?}", result.bucket_data_redundancy_transition);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get redundancy transition: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_data_redundancy_transition(
        &self,
        request: &GetBucketDataRedundancyTransitionRequest,
    ) -> Result<GetBucketDataRedundancyTransitionResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "GetBucketDataRedundancyTransition".to_string(),
            method: http::Method::GET,
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

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let transition: BucketDataRedundancyTransition = quick_xml::de::from_str(&data_str)?;

        let mut result = GetBucketDataRedundancyTransitionResult {
            bucket_data_redundancy_transition: Some(transition),
            ..Default::default()
        };
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

    #[test]
    fn test_get_bucket_data_redundancy_transition_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<BucketDataRedundancyTransition>
  <Bucket>my-bucket</Bucket>
  <TaskId>task-1</TaskId>
  <Status>Finished</Status>
  <ProcessPercentage>100</ProcessPercentage>
  <CreateTime>2024-01-01T00:00:00.000Z</CreateTime>
  <StartTime>2024-01-01T01:00:00.000Z</StartTime>
  <EndTime>2024-01-01T02:00:00.000Z</EndTime>
</BucketDataRedundancyTransition>"#;
        let transition: BucketDataRedundancyTransition = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(transition.task_id.as_deref(), Some("task-1"));
        assert_eq!(transition.status.as_deref(), Some("Finished"));
        assert_eq!(transition.process_percentage, Some(100));
        assert_eq!(
            transition.end_time.as_deref(),
            Some("2024-01-01T02:00:00.000Z")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_data_redundancy_transition() {
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

        // A nonexistent task ID must be rejected by the service.
        let result = client
            .get_bucket_data_redundancy_transition(&GetBucketDataRedundancyTransitionRequest {
                bucket: config.bucket.clone(),
                redundancy_transition_taskid: Some("nonexistent-task-id".to_string()),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_err(),
            "get with a nonexistent task id should fail, got: {:?}",
            result.ok()
        );
    }
}
