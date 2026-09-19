use crate::HTTP_HEADER_CONTENT_TYPE;
use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::create_bucket_data_redundancy_transition::BucketDataRedundancyTransition;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::OperationInput;
use crate::OperationOutput;

#[derive(Debug, Default, OssRequestModel)]
pub struct ListBucketDataRedundancyTransitionRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl ListBucketDataRedundancyTransitionRequest {
    pub fn new(bucket: &str) -> Self {
        Self {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct ListBucketDataRedundancyTransitionResult {
    /// Indicates that this ListBucketDataRedundancyTransition request contains
    /// subsequent results. You must set continuation-token to
    /// NextContinuationToken to continue obtaining the results.
    #[serde(rename = "NextContinuationToken", skip_serializing_if = "Option::is_none")]
    pub next_continuation_token: Option<String>,

    /// The container in which the redundancy type conversion task is stored.
    #[serde(rename = "BucketDataRedundancyTransition", default)]
    pub bucket_data_redundancy_transitions: Vec<BucketDataRedundancyTransition>,

    /// Indicates whether the returned results are truncated.
    /// Valid values:
    /// true: indicates that not all results are returned for the request.
    /// false: indicates that all results are returned for the request.
    #[serde(rename = "IsTruncated", skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists all redundancy type conversion tasks of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListBucketDataRedundancyTransitionRequest` containing
    ///   the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::ListBucketDataRedundancyTransitionRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListBucketDataRedundancyTransitionRequest::new("my-bucket");
    ///
    /// match client.list_bucket_data_redundancy_transition(&request).await {
    ///     Ok(result) => {
    ///         println!("Transitions: {:?}", result.bucket_data_redundancy_transitions.len());
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to list redundancy transitions: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn list_bucket_data_redundancy_transition(
        &self,
        request: &ListBucketDataRedundancyTransitionRequest,
    ) -> Result<ListBucketDataRedundancyTransitionResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "ListBucketDataRedundancyTransition".to_string(),
            method: http::Method::GET,
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
        let mut result: ListBucketDataRedundancyTransitionResult =
            quick_xml::de::from_str(&data_str)?;

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
    fn test_list_bucket_data_redundancy_transition_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketDataRedundancyTransition>
  <NextContinuationToken>token-1</NextContinuationToken>
  <IsTruncated>true</IsTruncated>
  <BucketDataRedundancyTransition>
    <Bucket>my-bucket</Bucket>
    <TaskId>task-1</TaskId>
    <Status>Processing</Status>
    <ProcessPercentage>40</ProcessPercentage>
    <EstimatedRemainingTime>2</EstimatedRemainingTime>
    <CreateTime>2024-01-01T00:00:00.000Z</CreateTime>
    <StartTime>2024-01-01T01:00:00.000Z</StartTime>
  </BucketDataRedundancyTransition>
  <BucketDataRedundancyTransition>
    <Bucket>my-bucket</Bucket>
    <TaskId>task-2</TaskId>
    <Status>Finished</Status>
    <CreateTime>2024-01-02T00:00:00.000Z</CreateTime>
    <StartTime>2024-01-02T01:00:00.000Z</StartTime>
    <EndTime>2024-01-02T02:00:00.000Z</EndTime>
  </BucketDataRedundancyTransition>
</ListBucketDataRedundancyTransition>"#;
        let result: ListBucketDataRedundancyTransitionResult =
            quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.next_continuation_token.as_deref(), Some("token-1"));
        assert_eq!(result.is_truncated, Some(true));
        assert_eq!(result.bucket_data_redundancy_transitions.len(), 2);
        assert_eq!(
            result.bucket_data_redundancy_transitions[0].task_id.as_deref(),
            Some("task-1")
        );
        assert_eq!(
            result.bucket_data_redundancy_transitions[0].process_percentage,
            Some(40)
        );
        assert_eq!(
            result.bucket_data_redundancy_transitions[1].end_time.as_deref(),
            Some("2024-01-02T02:00:00.000Z")
        );
        assert_eq!(
            result.bucket_data_redundancy_transitions[1]
                .process_percentage,
            None
        );

        // Empty list
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketDataRedundancyTransition>
  <IsTruncated>false</IsTruncated>
</ListBucketDataRedundancyTransition>"#;
        let result: ListBucketDataRedundancyTransitionResult =
            quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.is_truncated, Some(false));
        assert!(result.bucket_data_redundancy_transitions.is_empty());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_bucket_data_redundancy_transition() {
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
            .list_bucket_data_redundancy_transition(
                &ListBucketDataRedundancyTransitionRequest::new(&config.bucket),
            )
            .await;
        assert!(
            result.is_ok(),
            "list_bucket_data_redundancy_transition failed: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().common.status, http::StatusCode::OK);
    }
}
