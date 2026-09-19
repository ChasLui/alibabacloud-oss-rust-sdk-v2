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
pub struct ListUserDataRedundancyTransitionRequest {
    /// The token from which the list operation must start.
    #[field(type = "query", rename = "continuation-token")]
    pub continuation_token: Option<String>,

    /// The maximum number of redundancy type conversion tasks that can be
    /// returned. Valid values: 1 to 100.
    #[field(type = "query", rename = "max-keys")]
    pub max_keys: Option<i32>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct ListUserDataRedundancyTransitionResult {
    /// Indicates that this ListUserDataRedundancyTransition request contains
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
    /// Lists all redundancy type conversion tasks of the requester.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListUserDataRedundancyTransitionRequest` containing
    ///   the optional continuation token and the maximum number of tasks to
    ///   return.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::ListUserDataRedundancyTransitionRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListUserDataRedundancyTransitionRequest {
    ///     max_keys: Some(10),
    ///     ..Default::default()
    /// };
    ///
    /// match client.list_user_data_redundancy_transition(&request).await {
    ///     Ok(result) => {
    ///         println!("Transitions: {:?}", result.bucket_data_redundancy_transitions.len());
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to list user redundancy transitions: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn list_user_data_redundancy_transition(
        &self,
        request: &ListUserDataRedundancyTransitionRequest,
    ) -> Result<ListUserDataRedundancyTransitionResult, Box<dyn std::error::Error + Send + Sync>>
    {
        // Account-level operation: no bucket is set on the OperationInput.
        let mut input = OperationInput {
            op_name: "ListUserDataRedundancyTransition".to_string(),
            method: http::Method::GET,
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
        let mut result: ListUserDataRedundancyTransitionResult =
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
    fn test_list_user_data_redundancy_transition_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketDataRedundancyTransition>
  <IsTruncated>false</IsTruncated>
  <BucketDataRedundancyTransition>
    <Bucket>my-bucket</Bucket>
    <TaskId>task-1</TaskId>
    <Status>Queueing</Status>
    <CreateTime>2024-01-01T00:00:00.000Z</CreateTime>
  </BucketDataRedundancyTransition>
</ListBucketDataRedundancyTransition>"#;
        let result: ListUserDataRedundancyTransitionResult =
            quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.is_truncated, Some(false));
        assert_eq!(result.next_continuation_token, None);
        assert_eq!(result.bucket_data_redundancy_transitions.len(), 1);
        assert_eq!(
            result.bucket_data_redundancy_transitions[0].status.as_deref(),
            Some("Queueing")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_user_data_redundancy_transition() {
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
            .list_user_data_redundancy_transition(&ListUserDataRedundancyTransitionRequest {
                max_keys: Some(10),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "list_user_data_redundancy_transition failed: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().common.status, http::StatusCode::OK);
    }
}
