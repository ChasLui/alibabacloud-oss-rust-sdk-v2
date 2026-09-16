use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::put_bucket_replication::ReplicationRule;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::OperationInput;
use crate::OperationOutput;

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketReplicationRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetBucketReplicationResult {
    /// The container that stores the data replication rules.
    #[serde(rename = "Rule", default)]
    pub rules: Vec<ReplicationRule>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the data replication rules configured for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketReplicationRequest` containing the bucket
    ///   name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketReplicationRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketReplicationRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_replication(&request).await {
    ///     Ok(result) => {
    ///         println!("Replication rules: {:?}", result.rules.len());
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket replication: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_replication(
        &self,
        request: &GetBucketReplicationRequest,
    ) -> Result<GetBucketReplicationResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketReplication".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("replication", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["replication".to_string()]),
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
        let mut result: GetBucketReplicationResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_get_bucket_replication_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ReplicationConfiguration>
  <Rule>
    <ID>rule-id-1</ID>
    <Status>doing</Status>
    <PrefixSet>
      <Prefix>abc</Prefix>
    </PrefixSet>
    <Destination>
      <Bucket>dest-bucket</Bucket>
      <Location>oss-cn-beijing</Location>
      <TransferType>internal</TransferType>
    </Destination>
    <HistoricalObjectReplication>enabled</HistoricalObjectReplication>
    <Action>ALL</Action>
  </Rule>
</ReplicationConfiguration>"#;

        let result: GetBucketReplicationResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.rules.len(), 1);
        let rule = &result.rules[0];
        assert_eq!(rule.id.as_deref(), Some("rule-id-1"));
        assert_eq!(rule.status.as_deref(), Some("doing"));
        assert_eq!(
            rule.prefix_set.as_ref().unwrap().prefixs,
            vec!["abc".to_string()]
        );
        let dest = rule.destination.as_ref().unwrap();
        assert_eq!(dest.bucket.as_deref(), Some("dest-bucket"));
        assert_eq!(dest.location.as_deref(), Some("oss-cn-beijing"));
        assert_eq!(dest.transfer_type.as_deref(), Some("internal"));
        assert_eq!(
            rule.historical_object_replication.as_deref(),
            Some("enabled")
        );
        assert_eq!(rule.action.as_deref(), Some("ALL"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_replication() {
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

        let bucket_name = generate_unique_bucket_name("get-replication-test");

        let created = client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(created.is_ok(), "create_bucket failed: {:?}", created.err());

        // A fresh bucket has no replication rule; the server is expected to
        // return a NoSuchReplication error.
        let result = client
            .get_bucket_replication(&GetBucketReplicationRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_err(),
            "get_bucket_replication on a fresh bucket should fail"
        );

        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
    }
}
