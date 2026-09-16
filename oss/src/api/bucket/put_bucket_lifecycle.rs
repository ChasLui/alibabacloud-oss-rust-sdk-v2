use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::object::Tag;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the lifecycle rules.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LifecycleConfiguration {
    /// The lifecycle rules.
    #[serde(rename = "Rule", default)]
    pub rules: Vec<LifecycleRule>,
}

/// A lifecycle rule of a bucket.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LifecycleRule {
    /// Specifies whether to enable the rule. Valid values: Enabled, Disabled.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// The delete operation that you want OSS to perform on the parts that are
    /// uploaded in incomplete multipart upload tasks when the parts expire.
    #[serde(rename = "AbortMultipartUpload", skip_serializing_if = "Option::is_none")]
    pub abort_multipart_upload: Option<LifecycleRuleAbortMultipartUpload>,

    /// Timestamp for when access tracking was enabled.
    #[serde(rename = "AtimeBase", skip_serializing_if = "Option::is_none")]
    pub atime_base: Option<i64>,

    /// The conversion of the storage class of previous versions of the objects
    /// that match the lifecycle rule when the previous versions expire.
    #[serde(rename = "NoncurrentVersionTransition", default)]
    pub noncurrent_version_transitions: Vec<NoncurrentVersionTransition>,

    /// The container that stores the Not parameter that is used to filter objects.
    #[serde(rename = "Filter", skip_serializing_if = "Option::is_none")]
    pub filter: Option<LifecycleRuleFilter>,

    /// The ID of the lifecycle rule.
    #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The prefix in the names of the objects to which the rule applies.
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// The delete operation to perform on objects based on the lifecycle rule.
    #[serde(rename = "Expiration", skip_serializing_if = "Option::is_none")]
    pub expiration: Option<LifecycleRuleExpiration>,

    /// The conversion of the storage class of objects that match the lifecycle
    /// rule when the objects expire.
    #[serde(rename = "Transition", default)]
    pub transitions: Vec<LifecycleRuleTransition>,

    /// The tags of the objects to which the lifecycle rule applies.
    #[serde(rename = "Tag", default)]
    pub tags: Vec<Tag>,

    /// The delete operation that you want OSS to perform on the previous versions
    /// of the objects that match the lifecycle rule when the previous versions expire.
    #[serde(rename = "NoncurrentVersionExpiration", skip_serializing_if = "Option::is_none")]
    pub noncurrent_version_expiration: Option<NoncurrentVersionExpiration>,
}

/// The delete operation performed on expired parts of incomplete multipart uploads.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LifecycleRuleAbortMultipartUpload {
    /// The number of days from when the objects were last modified to when the
    /// lifecycle rule takes effect.
    #[serde(rename = "Days", skip_serializing_if = "Option::is_none")]
    pub days: Option<i32>,

    /// The date based on which the lifecycle rule takes effect. Specify the time
    /// in the ISO 8601 standard. The time must be at 00:00:00 in UTC.
    #[serde(rename = "CreatedBeforeDate", skip_serializing_if = "Option::is_none")]
    pub created_before_date: Option<String>,

    /// Deprecated: please use days or created_before_date.
    #[serde(rename = "Date", skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

/// The Not condition used to filter objects in a lifecycle rule filter.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LifecycleRuleNot {
    /// The tag of the objects to which the lifecycle rule does not apply.
    #[serde(rename = "Tag", skip_serializing_if = "Option::is_none")]
    pub tag: Option<Tag>,

    /// The prefix in the names of the objects to which the lifecycle rule does not apply.
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
}

/// The container that stores the Not parameter that is used to filter objects.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LifecycleRuleFilter {
    /// The condition that is matched by objects to which the lifecycle rule does not apply.
    #[serde(rename = "Not", default)]
    pub nots: Vec<LifecycleRuleNot>,

    /// This lifecycle rule only applies to files larger than this size.
    #[serde(rename = "ObjectSizeGreaterThan", skip_serializing_if = "Option::is_none")]
    pub object_size_greater_than: Option<i64>,

    /// This lifecycle rule only applies to files smaller than this size.
    #[serde(rename = "ObjectSizeLessThan", skip_serializing_if = "Option::is_none")]
    pub object_size_less_than: Option<i64>,
}

/// The delete operation to perform on objects based on the lifecycle rule.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LifecycleRuleExpiration {
    /// The date based on which the lifecycle rule takes effect, in the
    /// yyyy-MM-ddT00:00:00.000Z format.
    #[serde(rename = "CreatedBeforeDate", skip_serializing_if = "Option::is_none")]
    pub created_before_date: Option<String>,

    /// The number of days from when the objects were last modified to when the
    /// lifecycle rule takes effect.
    #[serde(rename = "Days", skip_serializing_if = "Option::is_none")]
    pub days: Option<i32>,

    /// Specifies whether to automatically remove expired delete markers.
    #[serde(rename = "ExpiredObjectDeleteMarker", skip_serializing_if = "Option::is_none")]
    pub expired_object_delete_marker: Option<bool>,

    /// Deprecated: please use days or created_before_date.
    #[serde(rename = "Date", skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

/// The delete operation performed on expired previous versions of objects.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NoncurrentVersionExpiration {
    /// The number of days from when the objects became previous versions to when
    /// the lifecycle rule takes effect.
    #[serde(rename = "NoncurrentDays", skip_serializing_if = "Option::is_none")]
    pub noncurrent_days: Option<i32>,
}

/// The conversion of the storage class of previous versions of objects when the
/// previous versions expire.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NoncurrentVersionTransition {
    /// Specifies whether the lifecycle rule applies to objects based on their
    /// last access time.
    #[serde(rename = "IsAccessTime", skip_serializing_if = "Option::is_none")]
    pub is_access_time: Option<bool>,

    /// Specifies whether to convert the storage class of non-Standard objects
    /// back to Standard after the objects are accessed.
    #[serde(rename = "ReturnToStdWhenVisit", skip_serializing_if = "Option::is_none")]
    pub return_to_std_when_visit: Option<bool>,

    /// Specifies whether to convert the storage class of objects whose sizes are
    /// less than 64 KB to IA, Archive, or Cold Archive based on their last access time.
    #[serde(rename = "AllowSmallFile", skip_serializing_if = "Option::is_none")]
    pub allow_small_file: Option<bool>,

    /// The number of days from when the objects became previous versions to when
    /// the lifecycle rule takes effect.
    #[serde(rename = "NoncurrentDays", skip_serializing_if = "Option::is_none")]
    pub noncurrent_days: Option<i32>,

    /// The storage class to which objects are converted. Valid values: IA,
    /// Archive, ColdArchive.
    #[serde(rename = "StorageClass", skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,
}

/// The conversion of the storage class of objects when the objects expire.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LifecycleRuleTransition {
    /// The date based on which the lifecycle rule takes effect. Specify the time
    /// in the ISO 8601 standard. The time must be at 00:00:00 in UTC.
    #[serde(rename = "CreatedBeforeDate", skip_serializing_if = "Option::is_none")]
    pub created_before_date: Option<String>,

    /// The number of days from when the objects were last modified to when the
    /// lifecycle rule takes effect.
    #[serde(rename = "Days", skip_serializing_if = "Option::is_none")]
    pub days: Option<i32>,

    /// The storage class to which objects are converted. Valid values: IA,
    /// Archive, ColdArchive.
    #[serde(rename = "StorageClass", skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,

    /// Specifies whether the lifecycle rule applies to objects based on their
    /// last access time.
    #[serde(rename = "IsAccessTime", skip_serializing_if = "Option::is_none")]
    pub is_access_time: Option<bool>,

    /// Specifies whether to convert the storage class of non-Standard objects
    /// back to Standard after the objects are accessed.
    #[serde(rename = "ReturnToStdWhenVisit", skip_serializing_if = "Option::is_none")]
    pub return_to_std_when_visit: Option<bool>,

    /// Specifies whether to convert the storage class of objects whose sizes are
    /// less than 64 KB to IA, Archive, or Cold Archive based on their last access time.
    #[serde(rename = "AllowSmallFile", skip_serializing_if = "Option::is_none")]
    pub allow_small_file: Option<bool>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketLifecycleRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// Specifies whether to allow overlapped prefixes. Valid values: true, false.
    #[field(type = "header", rename = "x-oss-allow-same-action-overlap")]
    pub allow_same_action_overlap: Option<String>,

    /// The container of the request body.
    pub lifecycle_configuration: LifecycleConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketLifecycleResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Configures a lifecycle rule for a bucket. After you configure a lifecycle
    /// rule for a bucket, Object Storage Service (OSS) automatically deletes the
    /// objects that match the rule or converts the storage type of the objects
    /// based on the point in time that is specified in the lifecycle rule.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketLifecycleRequest` containing the bucket name
    ///   and the lifecycle configuration to set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    /// #     LifecycleConfiguration, LifecycleRule, LifecycleRuleExpiration, PutBucketLifecycleRequest,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketLifecycleRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     lifecycle_configuration: LifecycleConfiguration {
    ///         rules: vec![LifecycleRule {
    ///             id: Some("rule1".to_string()),
    ///             status: Some("Enabled".to_string()),
    ///             prefix: Some("logs/".to_string()),
    ///             expiration: Some(LifecycleRuleExpiration {
    ///                 days: Some(30),
    ///                 ..Default::default()
    ///             }),
    ///             ..Default::default()
    ///         }],
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_lifecycle(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket lifecycle updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket lifecycle: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_lifecycle(
        &self,
        request: &PutBucketLifecycleRequest,
    ) -> Result<PutBucketLifecycleResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketLifecycle".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("lifecycle", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["lifecycle".to_string()]),
        );

        let xml_body =
            quick_xml::se::to_string_with_root("LifecycleConfiguration", &request.lifecycle_configuration)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketLifecycleResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_lifecycle_configuration_serde_round_trip() {
        let config = LifecycleConfiguration {
            rules: vec![LifecycleRule {
                id: Some("rule1".to_string()),
                status: Some("Enabled".to_string()),
                prefix: Some("logs/".to_string()),
                expiration: Some(LifecycleRuleExpiration {
                    days: Some(30),
                    created_before_date: None,
                    expired_object_delete_marker: None,
                    date: None,
                }),
                abort_multipart_upload: Some(LifecycleRuleAbortMultipartUpload {
                    days: Some(7),
                    created_before_date: None,
                    date: None,
                }),
                transitions: vec![LifecycleRuleTransition {
                    days: Some(10),
                    storage_class: Some("IA".to_string()),
                    created_before_date: None,
                    is_access_time: None,
                    return_to_std_when_visit: None,
                    allow_small_file: None,
                }],
                noncurrent_version_expiration: Some(NoncurrentVersionExpiration {
                    noncurrent_days: Some(20),
                }),
                noncurrent_version_transitions: vec![NoncurrentVersionTransition {
                    noncurrent_days: Some(15),
                    storage_class: Some("Archive".to_string()),
                    is_access_time: Some(false),
                    return_to_std_when_visit: Some(false),
                    allow_small_file: Some(false),
                }],
                filter: Some(LifecycleRuleFilter {
                    nots: vec![LifecycleRuleNot {
                        prefix: Some("logs/tmp/".to_string()),
                        tag: Some(Tag {
                            key: Some("skip".to_string()),
                            value: Some("true".to_string()),
                        }),
                    }],
                    object_size_greater_than: Some(100),
                    object_size_less_than: Some(64000000),
                }),
                tags: vec![Tag {
                    key: Some("k1".to_string()),
                    value: Some("v1".to_string()),
                }],
                atime_base: None,
            }],
        };

        let xml = quick_xml::se::to_string_with_root("LifecycleConfiguration", &config).unwrap();
        assert!(xml.contains("<LifecycleConfiguration>"));
        assert!(xml.contains("<ID>rule1</ID>"));
        assert!(xml.contains("<Prefix>logs/</Prefix>"));
        assert!(xml.contains("<Days>30</Days>"));
        assert!(xml.contains("<StorageClass>IA</StorageClass>"));
        assert!(xml.contains("<NoncurrentDays>20</NoncurrentDays>"));
        assert!(xml.contains("<ObjectSizeGreaterThan>100</ObjectSizeGreaterThan>"));

        let parsed: LifecycleConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.rules.len(), 1);
        let rule = &parsed.rules[0];
        assert_eq!(rule.id.as_deref(), Some("rule1"));
        assert_eq!(rule.status.as_deref(), Some("Enabled"));
        assert_eq!(rule.prefix.as_deref(), Some("logs/"));
        assert_eq!(rule.expiration.as_ref().unwrap().days, Some(30));
        assert_eq!(rule.abort_multipart_upload.as_ref().unwrap().days, Some(7));
        assert_eq!(rule.transitions.len(), 1);
        assert_eq!(rule.transitions[0].storage_class.as_deref(), Some("IA"));
        assert_eq!(
            rule.noncurrent_version_expiration.as_ref().unwrap().noncurrent_days,
            Some(20)
        );
        assert_eq!(rule.noncurrent_version_transitions.len(), 1);
        assert_eq!(
            rule.noncurrent_version_transitions[0].noncurrent_days,
            Some(15)
        );
        let filter = rule.filter.as_ref().unwrap();
        assert_eq!(filter.nots.len(), 1);
        assert_eq!(filter.nots[0].prefix.as_deref(), Some("logs/tmp/"));
        assert_eq!(filter.object_size_greater_than, Some(100));
        assert_eq!(filter.object_size_less_than, Some(64000000));
        assert_eq!(rule.tags.len(), 1);
        assert_eq!(rule.tags[0].key.as_deref(), Some("k1"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_lifecycle() {
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

        let bucket_name = generate_unique_bucket_name("put-bucket-lifecycle");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .put_bucket_lifecycle(&PutBucketLifecycleRequest {
                bucket: bucket_name.clone(),
                lifecycle_configuration: LifecycleConfiguration {
                    rules: vec![LifecycleRule {
                        id: Some("rule1".to_string()),
                        status: Some("Enabled".to_string()),
                        prefix: Some("logs/".to_string()),
                        expiration: Some(LifecycleRuleExpiration {
                            days: Some(30),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                },
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "put_bucket_lifecycle failed: {:?}", result.err());

        // Clean up
        let _ = client
            .delete_bucket_lifecycle(&crate::api::bucket::DeleteBucketLifecycleRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        let _ = client
            .delete_bucket(&DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
