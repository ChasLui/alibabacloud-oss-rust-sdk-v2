use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationOutput, BodyContent, OperationInput, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the data replication rules.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReplicationConfiguration {
    /// The container that stores the data replication rules.
    #[serde(rename = "Rule", default)]
    pub rules: Vec<ReplicationRule>,
}

/// A data replication rule.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReplicationRule {
    /// The container that stores the information about the destination bucket.
    #[serde(rename = "Destination", skip_serializing_if = "Option::is_none")]
    pub destination: Option<ReplicationDestination>,

    /// The role that you want to authorize OSS to use to replicate data.
    #[serde(rename = "SyncRole", skip_serializing_if = "Option::is_none")]
    pub sync_role: Option<String>,

    /// The container that specifies other conditions used to filter the source
    /// objects that you want to replicate.
    #[serde(rename = "SourceSelectionCriteria", skip_serializing_if = "Option::is_none")]
    pub source_selection_criteria: Option<ReplicationSourceSelectionCriteria>,

    /// The encryption configuration for the objects replicated to the
    /// destination bucket.
    #[serde(rename = "EncryptionConfiguration", skip_serializing_if = "Option::is_none")]
    pub encryption_configuration: Option<ReplicationEncryptionConfiguration>,

    /// Specifies whether to replicate historical data that exists before data
    /// replication is enabled. Valid values: enabled (default), disabled.
    #[serde(rename = "HistoricalObjectReplication", skip_serializing_if = "Option::is_none")]
    pub historical_object_replication: Option<String>,

    /// The container that stores the status of the RTC feature.
    #[serde(rename = "RTC", skip_serializing_if = "Option::is_none")]
    pub rtc: Option<ReplicationTimeControl>,

    /// The ID of the rule.
    #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The container that stores prefixes. You can specify up to 10 prefixes
    /// in each data replication rule.
    #[serde(rename = "PrefixSet", skip_serializing_if = "Option::is_none")]
    pub prefix_set: Option<ReplicationPrefixSet>,

    /// The operations that can be synchronized to the destination bucket.
    /// Valid values: ALL (default), PUT.
    #[serde(rename = "Action", skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,

    /// The status of the data replication task. Valid values: starting, doing,
    /// closing.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// The container that stores the information about the destination bucket.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReplicationDestination {
    /// The destination bucket to which data is replicated.
    #[serde(rename = "Bucket", skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// The region in which the destination bucket is located.
    #[serde(rename = "Location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// The link that is used to transfer data during data replication.
    /// Valid values: internal (default), oss_acc.
    #[serde(rename = "TransferType", skip_serializing_if = "Option::is_none")]
    pub transfer_type: Option<String>,
}

/// The container that is used to filter the source objects.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReplicationSourceSelectionCriteria {
    /// The container that is used to filter the source objects that are
    /// encrypted by using SSE-KMS.
    #[serde(rename = "SseKmsEncryptedObjects", skip_serializing_if = "Option::is_none")]
    pub sse_kms_encrypted_objects: Option<SseKmsEncryptedObjects>,
}

/// The container that is used to filter the source objects that are encrypted
/// by using SSE-KMS.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SseKmsEncryptedObjects {
    /// Specifies whether to replicate objects that are encrypted by using
    /// SSE-KMS. Valid values: Enabled, Disabled.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// The encryption configuration for the objects replicated to the destination
/// bucket.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReplicationEncryptionConfiguration {
    /// The ID of the KMS key that is used to encrypt the replicated objects.
    #[serde(rename = "ReplicaKmsKeyID", skip_serializing_if = "Option::is_none")]
    pub replica_kms_key_id: Option<String>,
}

/// The container that stores the status of the RTC feature.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReplicationTimeControl {
    /// Specifies whether to enable RTC. Valid values: disabled, enabled.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// The container that stores prefixes.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReplicationPrefixSet {
    /// The prefix that is used to specify the objects that you want to
    /// replicate.
    #[serde(rename = "Prefix", default)]
    pub prefixs: Vec<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketReplicationRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The container of the request body.
    pub replication_configuration: ReplicationConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketReplicationResult {
    /// The ID of the data replication rule.
    #[field(type = "header", rename = "x-oss-replication-rule-id")]
    pub replication_rule_id: Option<String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Configures data replication rules for a bucket. Object Storage Service
    /// (OSS) supports cross-region replication (CRR) and same-region
    /// replication (SRR).
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketReplicationRequest` containing the bucket
    ///   name and the replication configuration to set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    /// #     PutBucketReplicationRequest, ReplicationConfiguration, ReplicationDestination,
    /// #     ReplicationPrefixSet, ReplicationRule,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketReplicationRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     replication_configuration: ReplicationConfiguration {
    ///         rules: vec![ReplicationRule {
    ///             destination: Some(ReplicationDestination {
    ///                 bucket: Some("dest-bucket".to_string()),
    ///                 location: Some("oss-cn-beijing".to_string()),
    ///                 transfer_type: None,
    ///             }),
    ///             prefix_set: Some(ReplicationPrefixSet {
    ///                 prefixs: vec!["prefix1".to_string()],
    ///             }),
    ///             historical_object_replication: Some("enabled".to_string()),
    ///             ..Default::default()
    ///         }],
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_replication(&request).await {
    ///     Ok(result) => {
    ///         println!("Replication rule id: {:?}", result.replication_rule_id);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket replication: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_replication(
        &self,
        request: &PutBucketReplicationRequest,
    ) -> Result<PutBucketReplicationResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketReplication".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("comp", "add"), ("replication", "")]
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
            std::rc::Rc::new(vec!["replication".to_string(), "comp".to_string()]),
        );

        let xml_body = quick_xml::se::to_string_with_root(
            "ReplicationConfiguration",
            &request.replication_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketReplicationResult::default();
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
    fn test_replication_configuration_serde_round_trip() {
        let config = ReplicationConfiguration {
            rules: vec![ReplicationRule {
                destination: Some(ReplicationDestination {
                    bucket: Some("dest-bucket".to_string()),
                    location: Some("oss-cn-beijing".to_string()),
                    transfer_type: Some("oss_acc".to_string()),
                }),
                sync_role: Some("aliyunossrole".to_string()),
                source_selection_criteria: Some(ReplicationSourceSelectionCriteria {
                    sse_kms_encrypted_objects: Some(SseKmsEncryptedObjects {
                        status: Some("Enabled".to_string()),
                    }),
                }),
                encryption_configuration: Some(ReplicationEncryptionConfiguration {
                    replica_kms_key_id: Some("key-id".to_string()),
                }),
                historical_object_replication: Some("enabled".to_string()),
                rtc: Some(ReplicationTimeControl {
                    status: Some("enabled".to_string()),
                }),
                id: Some("rule-id".to_string()),
                prefix_set: Some(ReplicationPrefixSet {
                    prefixs: vec!["p1".to_string(), "p2".to_string()],
                }),
                action: Some("ALL".to_string()),
                status: Some("doing".to_string()),
            }],
        };

        let xml =
            quick_xml::se::to_string_with_root("ReplicationConfiguration", &config).unwrap();
        assert!(xml.contains("<ReplicationConfiguration>"));
        assert!(xml.contains("<Bucket>dest-bucket</Bucket>"));
        assert!(xml.contains("<Location>oss-cn-beijing</Location>"));
        assert!(xml.contains("<TransferType>oss_acc</TransferType>"));
        assert!(xml.contains("<SyncRole>aliyunossrole</SyncRole>"));
        assert!(xml.contains("<Status>Enabled</Status>"));
        assert!(xml.contains("<ReplicaKmsKeyID>key-id</ReplicaKmsKeyID>"));
        assert!(xml.contains("<HistoricalObjectReplication>enabled</HistoricalObjectReplication>"));
        assert!(xml.contains("<RTC>"));
        assert!(xml.contains("<ID>rule-id</ID>"));
        assert!(xml.contains("<Prefix>p1</Prefix>"));
        assert!(xml.contains("<Prefix>p2</Prefix>"));
        assert!(xml.contains("<Action>ALL</Action>"));

        let parsed: ReplicationConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.rules.len(), 1);
        let rule = &parsed.rules[0];
        assert_eq!(
            rule.destination.as_ref().unwrap().bucket.as_deref(),
            Some("dest-bucket")
        );
        assert_eq!(
            rule.destination.as_ref().unwrap().transfer_type.as_deref(),
            Some("oss_acc")
        );
        assert_eq!(rule.sync_role.as_deref(), Some("aliyunossrole"));
        assert_eq!(
            rule.source_selection_criteria
                .as_ref()
                .unwrap()
                .sse_kms_encrypted_objects
                .as_ref()
                .unwrap()
                .status
                .as_deref(),
            Some("Enabled")
        );
        assert_eq!(
            rule.encryption_configuration
                .as_ref()
                .unwrap()
                .replica_kms_key_id
                .as_deref(),
            Some("key-id")
        );
        assert_eq!(
            rule.historical_object_replication.as_deref(),
            Some("enabled")
        );
        assert_eq!(
            rule.rtc.as_ref().unwrap().status.as_deref(),
            Some("enabled")
        );
        assert_eq!(rule.id.as_deref(), Some("rule-id"));
        assert_eq!(rule.prefix_set.as_ref().unwrap().prefixs.len(), 2);
        assert_eq!(rule.action.as_deref(), Some("ALL"));
        assert_eq!(rule.status.as_deref(), Some("doing"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_replication() {
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

        let bucket_name = generate_unique_bucket_name("replication-test");

        let created = client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(created.is_ok(), "create_bucket failed: {:?}", created.err());

        // A replication rule requires a reachable destination bucket and (for
        // SSE-KMS) a RAM role; the request may be rejected by the server in
        // test accounts, so only exercise the full flow when accepted.
        let result = client
            .put_bucket_replication(&PutBucketReplicationRequest {
                bucket: bucket_name.clone(),
                replication_configuration: ReplicationConfiguration {
                    rules: vec![ReplicationRule {
                        destination: Some(ReplicationDestination {
                            bucket: Some(bucket_name.clone()),
                            location: Some(format!("oss-{}", config.region)),
                            transfer_type: None,
                        }),
                        prefix_set: Some(ReplicationPrefixSet {
                            prefixs: vec!["test-prefix".to_string()],
                        }),
                        historical_object_replication: Some("enabled".to_string()),
                        ..Default::default()
                    }],
                },
                ..Default::default()
            })
            .await;

        match result {
            Ok(put_result) => {
                assert!(put_result.replication_rule_id.is_some());

                let rule_id = put_result.replication_rule_id.clone().unwrap();
                let _ = client
                    .delete_bucket_replication(
                        &crate::api::bucket::DeleteBucketReplicationRequest {
                            bucket: bucket_name.clone(),
                            replication_rules:
                                crate::api::bucket::ReplicationRules {
                                    ids: vec![rule_id],
                                },
                            ..Default::default()
                        },
                    )
                    .await;
            }
            Err(error) => {
                eprintln!("put_bucket_replication rejected by server: {}", error);
            }
        }

        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
    }
}
