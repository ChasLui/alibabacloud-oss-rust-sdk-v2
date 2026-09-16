use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::put_bucket_replication::{ReplicationDestination, ReplicationPrefixSet};
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::OperationInput;
use crate::OperationOutput;

/// The container that stores the progress of the data replication task.
#[derive(Debug, Default, Deserialize)]
pub struct ReplicationProgressInformation {
    /// The percentage of the replicated historical data. This parameter is
    /// valid only when HistoricalObjectReplication is set to enabled.
    #[serde(rename = "HistoricalObject", skip_serializing_if = "Option::is_none")]
    pub historical_object: Option<String>,

    /// The time used to determine whether data is replicated to the
    /// destination bucket, in the GMT format.
    #[serde(rename = "NewObject", skip_serializing_if = "Option::is_none")]
    pub new_object: Option<String>,
}

/// The progress of the data replication task corresponding to a data
/// replication rule.
#[derive(Debug, Default, Deserialize)]
pub struct ReplicationProgressRule {
    /// The container that stores the information about the destination bucket.
    #[serde(rename = "Destination", skip_serializing_if = "Option::is_none")]
    pub destination: Option<ReplicationDestination>,

    /// The status of the data replication task. Valid values: starting, doing,
    /// closing.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Specifies whether to replicate historical data that exists before data
    /// replication is enabled.
    #[serde(rename = "HistoricalObjectReplication", skip_serializing_if = "Option::is_none")]
    pub historical_object_replication: Option<String>,

    /// The container that stores the progress of the data replication task.
    /// This parameter is returned only when the data replication task is in
    /// the doing state.
    #[serde(rename = "Progress", skip_serializing_if = "Option::is_none")]
    pub progress: Option<ReplicationProgressInformation>,

    /// The ID of the data replication rule.
    #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The container that stores prefixes.
    #[serde(rename = "PrefixSet", skip_serializing_if = "Option::is_none")]
    pub prefix_set: Option<ReplicationPrefixSet>,

    /// The operations that are synchronized to the destination bucket.
    #[serde(rename = "Action", skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketReplicationProgressRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The ID of the data replication rule. You can call the
    /// GetBucketReplication operation to query the ID.
    #[field(type = "query", rename = "rule-id")]
    pub rule_id: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetBucketReplicationProgressResult {
    /// The container that stores the progress of the data replication task
    /// corresponding to each data replication rule.
    #[serde(rename = "Rule", default)]
    pub rules: Vec<ReplicationProgressRule>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the information about the data replication process of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketReplicationProgressRequest` containing the
    ///   bucket name and the ID of the data replication rule.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketReplicationProgressRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketReplicationProgressRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     rule_id: Some("rule-id".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_replication_progress(&request).await {
    ///     Ok(result) => {
    ///         println!("Replication progress rules: {:?}", result.rules.len());
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket replication progress: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_replication_progress(
        &self,
        request: &GetBucketReplicationProgressRequest,
    ) -> Result<GetBucketReplicationProgressResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketReplicationProgress".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("replicationProgress", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["replicationProgress".to_string()]),
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
        let mut result: GetBucketReplicationProgressResult = quick_xml::de::from_str(&data_str)?;
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
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_get_bucket_replication_progress_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ReplicationProgress>
  <Rule>
    <ID>rule-id-1</ID>
    <Status>doing</Status>
    <PrefixSet>
      <Prefix>abc</Prefix>
    </PrefixSet>
    <Action>ALL</Action>
    <Destination>
      <Bucket>dest-bucket</Bucket>
      <Location>oss-cn-beijing</Location>
      <TransferType>internal</TransferType>
    </Destination>
    <HistoricalObjectReplication>enabled</HistoricalObjectReplication>
    <Progress>
      <HistoricalObject>0.85</HistoricalObject>
      <NewObject>Thu, 24 Sep 2015 15:39:18 GMT</NewObject>
    </Progress>
  </Rule>
</ReplicationProgress>"#;

        let result: GetBucketReplicationProgressResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.rules.len(), 1);
        let rule = &result.rules[0];
        assert_eq!(rule.id.as_deref(), Some("rule-id-1"));
        assert_eq!(rule.status.as_deref(), Some("doing"));
        assert_eq!(rule.action.as_deref(), Some("ALL"));
        assert_eq!(
            rule.destination.as_ref().unwrap().bucket.as_deref(),
            Some("dest-bucket")
        );
        assert_eq!(
            rule.prefix_set.as_ref().unwrap().prefixs,
            vec!["abc".to_string()]
        );
        assert_eq!(
            rule.historical_object_replication.as_deref(),
            Some("enabled")
        );
        let progress = rule.progress.as_ref().unwrap();
        assert_eq!(progress.historical_object.as_deref(), Some("0.85"));
        assert_eq!(
            progress.new_object.as_deref(),
            Some("Thu, 24 Sep 2015 15:39:18 GMT")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_replication_progress() {
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

        let bucket_name = generate_unique_bucket_name("replication-progress-test");

        let created = client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(created.is_ok(), "create_bucket failed: {:?}", created.err());

        // A fresh bucket has no replication rule; the server is expected to
        // reject the query.
        let result = client
            .get_bucket_replication_progress(&GetBucketReplicationProgressRequest {
                bucket: bucket_name.clone(),
                rule_id: Some("non-existent-rule".to_string()),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_err(),
            "get_bucket_replication_progress without a rule should fail"
        );

        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
    }
}
