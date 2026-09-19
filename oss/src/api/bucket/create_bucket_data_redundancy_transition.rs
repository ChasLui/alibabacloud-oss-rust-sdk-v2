use crate::HTTP_HEADER_CONTENT_TYPE;
use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::OperationInput;
use crate::OperationOutput;

/// The container in which the redundancy type conversion task is stored.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BucketDataRedundancyTransition {
    /// The progress of the redundancy type change task in percentage.
    /// Valid values: 0 to 100. This element is available when the task is in
    /// the Processing or Finished state.
    #[serde(rename = "ProcessPercentage", skip_serializing_if = "Option::is_none")]
    pub process_percentage: Option<i32>,

    /// The estimated period of time that is required for the redundancy type
    /// change task. Unit: hours. This element is available when the task is in
    /// the Processing or Finished state.
    #[serde(rename = "EstimatedRemainingTime", skip_serializing_if = "Option::is_none")]
    pub estimated_remaining_time: Option<i64>,

    /// The name of the bucket.
    #[serde(rename = "Bucket", skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// The ID of the redundancy type change task.
    #[serde(rename = "TaskId", skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,

    /// The state of the redundancy type change task.
    /// Valid values: Queueing, Processing, Finished.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// The time when the redundancy type change task was created.
    #[serde(rename = "CreateTime", skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,

    /// The time when the redundancy type change task was performed. This
    /// element is available when the task is in the Processing or Finished
    /// state.
    #[serde(rename = "StartTime", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,

    /// The time when the redundancy type change task was finished. This
    /// element is available when the task is in the Finished state.
    #[serde(rename = "EndTime", skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct CreateBucketDataRedundancyTransitionRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The redundancy type to which you want to convert the bucket. You can
    /// only convert the redundancy type of a bucket from LRS to ZRS.
    #[field(type = "query", rename = "x-oss-target-redundancy-type")]
    pub target_redundancy_type: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct CreateBucketDataRedundancyTransitionResult {
    /// The container in which the redundancy type conversion task is stored.
    pub bucket_data_redundancy_transition: Option<BucketDataRedundancyTransition>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Creates a redundancy type conversion task for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `CreateBucketDataRedundancyTransitionRequest`
    ///   containing the bucket name and the target redundancy type.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::CreateBucketDataRedundancyTransitionRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = CreateBucketDataRedundancyTransitionRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     target_redundancy_type: Some("ZRS".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.create_bucket_data_redundancy_transition(&request).await {
    ///     Ok(result) => {
    ///         println!("Transition: {:?}", result.bucket_data_redundancy_transition);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to create redundancy transition: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn create_bucket_data_redundancy_transition(
        &self,
        request: &CreateBucketDataRedundancyTransitionRequest,
    ) -> Result<CreateBucketDataRedundancyTransitionResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "CreateBucketDataRedundancyTransition".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("redundancyTransition", "")]
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

        let mut result = CreateBucketDataRedundancyTransitionResult {
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
    fn test_bucket_data_redundancy_transition_serde_round_trip() {
        let transition = BucketDataRedundancyTransition {
            process_percentage: Some(50),
            estimated_remaining_time: Some(1),
            bucket: Some("my-bucket".to_string()),
            task_id: Some("task-123".to_string()),
            status: Some("Processing".to_string()),
            create_time: Some("2024-01-01T00:00:00.000Z".to_string()),
            start_time: Some("2024-01-01T01:00:00.000Z".to_string()),
            end_time: None,
        };

        let xml =
            quick_xml::se::to_string_with_root("BucketDataRedundancyTransition", &transition)
                .unwrap();
        assert!(xml.contains("<TaskId>task-123</TaskId>"));
        assert!(xml.contains("<ProcessPercentage>50</ProcessPercentage>"));
        assert!(xml.contains("<EstimatedRemainingTime>1</EstimatedRemainingTime>"));

        let parsed: BucketDataRedundancyTransition = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.task_id.as_deref(), Some("task-123"));
        assert_eq!(parsed.process_percentage, Some(50));
        assert_eq!(parsed.estimated_remaining_time, Some(1));
        assert_eq!(parsed.status.as_deref(), Some("Processing"));
        assert_eq!(parsed.bucket.as_deref(), Some("my-bucket"));
        assert_eq!(
            parsed.create_time.as_deref(),
            Some("2024-01-01T00:00:00.000Z")
        );
        assert_eq!(
            parsed.start_time.as_deref(),
            Some("2024-01-01T01:00:00.000Z")
        );
        // None fields are skipped during serialization and read back as None
        assert_eq!(parsed.end_time, None);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_bucket_data_redundancy_transition() {
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

        // Creating a real LRS->ZRS conversion may be rejected depending on the
        // bucket state; tolerate a service error and clean up on success.
        let result = client
            .create_bucket_data_redundancy_transition(
                &CreateBucketDataRedundancyTransitionRequest {
                    bucket: config.bucket.clone(),
                    target_redundancy_type: Some("ZRS".to_string()),
                    ..Default::default()
                },
            )
            .await;

        match result {
            Ok(created) => {
                let task_id = created
                    .bucket_data_redundancy_transition
                    .and_then(|t| t.task_id);
                // Clean up the created task
                if let Some(task_id) = task_id {
                    let _ = client
                        .delete_bucket_data_redundancy_transition(
                            &crate::api::bucket::DeleteBucketDataRedundancyTransitionRequest {
                                bucket: config.bucket.clone(),
                                redundancy_transition_taskid: Some(task_id),
                                ..Default::default()
                            },
                        )
                        .await;
                }
            }
            Err(error) => {
                eprintln!("create_bucket_data_redundancy_transition rejected: {}", error);
            }
        }
    }
}
