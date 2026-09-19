use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationOutput, BodyContent, OperationInput, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the IDs of the data replication rules to delete.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReplicationRules {
    /// The ID of data replication rules that you want to delete. You can call
    /// the GetBucketReplication operation to obtain the ID.
    #[serde(rename = "ID", default)]
    pub ids: Vec<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteBucketReplicationRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The container of the request body.
    pub replication_rules: ReplicationRules,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteBucketReplicationResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Disables data replication for a bucket and deletes the data replication
    /// rule configured for the bucket. After you call this operation, all
    /// operations performed on the source bucket are not synchronized to the
    /// destination bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteBucketReplicationRequest` containing the
    ///   bucket name and the IDs of the replication rules to delete.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    /// #     DeleteBucketReplicationRequest, ReplicationRules,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteBucketReplicationRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     replication_rules: ReplicationRules {
    ///         ids: vec!["rule-id".to_string()],
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_bucket_replication(&request).await {
    ///     Ok(result) => {
    ///         println!("Replication deleted: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete bucket replication: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_bucket_replication(
        &self,
        request: &DeleteBucketReplicationRequest,
    ) -> Result<DeleteBucketReplicationResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteBucketReplication".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("comp", "delete"), ("replication", "")]
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
            std::rc::Rc::new(vec!["comp".to_string(), "replication".to_string()]),
        );

        let xml_body =
            quick_xml::se::to_string_with_root("ReplicationRules", &request.replication_rules)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteBucketReplicationResult::default();
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
    fn test_replication_rules_serde_round_trip() {
        let rules = ReplicationRules {
            ids: vec!["rule-id-1".to_string(), "rule-id-2".to_string()],
        };

        let xml = quick_xml::se::to_string_with_root("ReplicationRules", &rules).unwrap();
        assert!(xml.contains("<ReplicationRules>"));
        assert!(xml.contains("<ID>rule-id-1</ID>"));
        assert!(xml.contains("<ID>rule-id-2</ID>"));

        let parsed: ReplicationRules = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.ids, rules.ids);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_bucket_replication() {
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

        let bucket_name = generate_unique_bucket_name("del-replication-test");

        let created = client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(created.is_ok(), "create_bucket failed: {:?}", created.err());

        // A fresh bucket has no replication rule; the server is expected to
        // reject the deletion.
        let result = client
            .delete_bucket_replication(&DeleteBucketReplicationRequest {
                bucket: bucket_name.clone(),
                replication_rules: ReplicationRules {
                    ids: vec!["non-existent-rule".to_string()],
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_err(),
            "delete_bucket_replication without a rule should fail"
        );

        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
    }
}
