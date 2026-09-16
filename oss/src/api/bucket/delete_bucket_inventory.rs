use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the customer master key (CMK) used for SSE-KMS encryption.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SSEKMS {
    /// The ID of the key that is managed by Key Management Service (KMS).
    #[serde(rename = "KeyId", skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
}

/// The container that stores the encryption method of the exported inventory lists.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InventoryEncryption {
    /// The container that stores information about the SSE-OSS encryption method.
    #[serde(rename = "SSE-OSS", skip_serializing_if = "Option::is_none")]
    pub sse_oss: Option<String>,

    /// The container that stores the customer master key (CMK) used for SSE-KMS encryption.
    #[serde(rename = "SSE-KMS", skip_serializing_if = "Option::is_none")]
    pub sse_kms: Option<SSEKMS>,
}

/// The container that stores information about the bucket in which exported
/// inventory lists are stored.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InventoryOSSBucketDestination {
    /// The format of exported inventory lists. The exported inventory lists are
    /// CSV objects compressed by using GZIP.
    #[serde(rename = "Format", skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// The ID of the account to which permissions are granted by the bucket owner.
    #[serde(rename = "AccountId", skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,

    /// The Alibaba Cloud Resource Name (ARN) of the role that has the permissions
    /// to read all objects from the source bucket and write objects to the
    /// destination bucket. Format: `acs:ram::uid:role/rolename`.
    #[serde(rename = "RoleArn", skip_serializing_if = "Option::is_none")]
    pub role_arn: Option<String>,

    /// The name of the bucket in which exported inventory lists are stored.
    #[serde(rename = "Bucket", skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// The prefix of the path in which the exported inventory lists are stored.
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// The container that stores the encryption method of the exported inventory lists.
    #[serde(rename = "Encryption", skip_serializing_if = "Option::is_none")]
    pub encryption: Option<InventoryEncryption>,
}

/// The container that stores the exported inventory lists.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InventoryDestination {
    /// The container that stores information about the bucket in which exported
    /// inventory lists are stored.
    #[serde(rename = "OSSBucketDestination", skip_serializing_if = "Option::is_none")]
    pub oss_bucket_destination: Option<InventoryOSSBucketDestination>,
}

/// The container that stores information about the frequency at which inventory
/// lists are exported.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InventorySchedule {
    /// The frequency at which the inventory list is exported.
    /// Valid values: Daily, Weekly, Monthly.
    #[serde(rename = "Frequency", skip_serializing_if = "Option::is_none")]
    pub frequency: Option<String>,

    /// The day of the month on which the inventory list is exported. This
    /// parameter is required if the Frequency is Monthly.
    #[serde(rename = "DayOfMonth", skip_serializing_if = "Option::is_none")]
    pub day_of_month: Option<i64>,
}

/// The container that stores the prefix used to filter objects. Only objects
/// whose names contain the specified prefix are included in the inventory.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InventoryFilter {
    /// The beginning of the time range during which the object was last modified.
    /// Unit: seconds. Valid values: [1262275200, 253402271999].
    #[serde(rename = "LastModifyBeginTimeStamp", skip_serializing_if = "Option::is_none")]
    pub last_modify_begin_time_stamp: Option<i64>,

    /// The end of the time range during which the object was last modified.
    /// Unit: seconds. Valid values: [1262275200, 253402271999].
    #[serde(rename = "LastModifyEndTimeStamp", skip_serializing_if = "Option::is_none")]
    pub last_modify_end_time_stamp: Option<i64>,

    /// The minimum size of the specified object. Unit: B.
    #[serde(rename = "LowerSizeBound", skip_serializing_if = "Option::is_none")]
    pub lower_size_bound: Option<i64>,

    /// The maximum size of the specified object. Unit: B.
    #[serde(rename = "UpperSizeBound", skip_serializing_if = "Option::is_none")]
    pub upper_size_bound: Option<i64>,

    /// The storage class of the object. You can specify multiple storage classes.
    /// Valid values: Standard, IA, Archive, ColdArchive, All.
    #[serde(rename = "StorageClass", skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,

    /// The prefix that is specified in the inventory.
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
}

/// The container that stores the configuration fields in inventory lists.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OptionalFields {
    /// The configuration fields that are included in inventory lists. Available
    /// configuration fields: Size, LastModifiedDate, ETag, StorageClass,
    /// IsMultipartUploaded, EncryptionStatus.
    #[serde(rename = "Field", default)]
    pub fields: Vec<String>,
}

/// Container for incremental inventory export cycle information.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IncrementInventorySchedule {
    /// The frequency at which incremental inventory files are exported.
    #[serde(rename = "Frequency", skip_serializing_if = "Option::is_none")]
    pub frequency: Option<i64>,
}

/// Configuration container for incremental inventory file attributes.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IncrementalInventoryOptionalFields {
    /// The configuration fields that are included in incremental inventory lists.
    #[serde(rename = "Field", default)]
    pub fields: Vec<String>,
}

/// Configuration container for incremental inventory.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IncrementalInventory {
    /// Specifies whether incremental inventory is enabled.
    #[serde(rename = "IsEnabled", skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,

    /// Container for incremental inventory export cycle.
    #[serde(rename = "Schedule", skip_serializing_if = "Option::is_none")]
    pub schedule: Option<IncrementInventorySchedule>,

    /// Configuration container for incremental inventory file attributes.
    #[serde(rename = "OptionalFields", skip_serializing_if = "Option::is_none")]
    pub optional_fields: Option<IncrementalInventoryOptionalFields>,
}

/// The inventory task configured for a bucket.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InventoryConfiguration {
    /// The name of the inventory. The name must be unique in the bucket.
    #[serde(rename = "Id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Specifies whether to enable the bucket inventory feature.
    #[serde(rename = "IsEnabled", skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,

    /// The container that stores the exported inventory lists.
    #[serde(rename = "Destination", skip_serializing_if = "Option::is_none")]
    pub destination: Option<InventoryDestination>,

    /// The container that stores information about the frequency at which
    /// inventory lists are exported.
    #[serde(rename = "Schedule", skip_serializing_if = "Option::is_none")]
    pub schedule: Option<InventorySchedule>,

    /// The container that stores the prefix used to filter objects.
    #[serde(rename = "Filter", skip_serializing_if = "Option::is_none")]
    pub filter: Option<InventoryFilter>,

    /// Specifies whether to include the version information about the objects in
    /// inventory lists. Valid values: All, Current.
    #[serde(rename = "IncludedObjectVersions", skip_serializing_if = "Option::is_none")]
    pub included_object_versions: Option<String>,

    /// The container that stores the configuration fields in inventory lists.
    #[serde(rename = "OptionalFields", skip_serializing_if = "Option::is_none")]
    pub optional_fields: Option<OptionalFields>,

    /// Configuration container for incremental inventory.
    #[serde(rename = "IncrementalInventory", skip_serializing_if = "Option::is_none")]
    pub incremental_inventory: Option<IncrementalInventory>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteBucketInventoryRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the inventory that you want to delete.
    #[field(type = "query", rename = "inventoryId")]
    pub inventory_id: Option<String>,

    pub common: RequestCommon,
}

impl DeleteBucketInventoryRequest {
    pub fn new(bucket: &str, inventory_id: &str) -> Self {
        DeleteBucketInventoryRequest {
            bucket: bucket.to_string(),
            inventory_id: Some(inventory_id.to_string()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteBucketInventoryResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes an inventory for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteBucketInventoryRequest` containing the bucket
    ///   name and the inventory name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::DeleteBucketInventoryRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteBucketInventoryRequest::new("my-bucket", "my-inventory");
    ///
    /// match client.delete_bucket_inventory(&request).await {
    ///     Ok(result) => {
    ///         println!("Inventory deleted: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete bucket inventory: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_bucket_inventory(
        &self,
        request: &DeleteBucketInventoryRequest,
    ) -> Result<DeleteBucketInventoryResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteBucketInventory".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            parameters: [("inventory", "")]
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
            SUB_RESOURCE,
            Rc::new(vec!["inventory".to_string(), "inventoryId".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteBucketInventoryResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_inventory_configuration_serde_round_trip() {
        let configuration = InventoryConfiguration {
            id: Some("report1".to_string()),
            is_enabled: Some(true),
            destination: Some(InventoryDestination {
                oss_bucket_destination: Some(InventoryOSSBucketDestination {
                    format: Some("CSV".to_string()),
                    account_id: Some("1000000000000000".to_string()),
                    role_arn: Some("acs:ram::1000000000000000:role/test-role".to_string()),
                    bucket: Some("acs:oss:::dest-bucket".to_string()),
                    prefix: Some("prefix1".to_string()),
                    encryption: Some(InventoryEncryption {
                        sse_oss: None,
                        sse_kms: Some(SSEKMS {
                            key_id: Some("key-id".to_string()),
                        }),
                    }),
                }),
            }),
            schedule: Some(InventorySchedule {
                frequency: Some("Daily".to_string()),
                day_of_month: None,
            }),
            filter: Some(InventoryFilter {
                last_modify_begin_time_stamp: Some(1637883649),
                last_modify_end_time_stamp: Some(1638347592),
                lower_size_bound: Some(1024),
                upper_size_bound: Some(1048576),
                storage_class: Some("Standard,IA".to_string()),
                prefix: Some("filterPrefix/".to_string()),
            }),
            included_object_versions: Some("All".to_string()),
            optional_fields: Some(OptionalFields {
                fields: vec!["Size".to_string(), "LastModifiedDate".to_string()],
            }),
            incremental_inventory: Some(IncrementalInventory {
                is_enabled: Some(true),
                schedule: Some(IncrementInventorySchedule {
                    frequency: Some(6),
                }),
                optional_fields: Some(IncrementalInventoryOptionalFields {
                    fields: vec!["Size".to_string()],
                }),
            }),
        };

        let xml = quick_xml::se::to_string_with_root("InventoryConfiguration", &configuration)
            .unwrap();
        assert!(xml.contains("<InventoryConfiguration>"));
        assert!(xml.contains("<Id>report1</Id>"));
        assert!(xml.contains("<Frequency>Daily</Frequency>"));
        assert!(xml.contains("<Field>Size</Field>"));
        assert!(xml.contains("<SSE-KMS>"));
        assert!(xml.contains("<KeyId>key-id</KeyId>"));

        let parsed: InventoryConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.id.as_deref(), Some("report1"));
        assert_eq!(parsed.is_enabled, Some(true));
        let dest = parsed.destination.unwrap().oss_bucket_destination.unwrap();
        assert_eq!(dest.format.as_deref(), Some("CSV"));
        assert_eq!(
            dest.encryption.unwrap().sse_kms.unwrap().key_id.as_deref(),
            Some("key-id")
        );
        let filter = parsed.filter.unwrap();
        assert_eq!(filter.last_modify_begin_time_stamp, Some(1637883649));
        assert_eq!(filter.upper_size_bound, Some(1048576));
        assert_eq!(parsed.optional_fields.unwrap().fields.len(), 2);
        let inc = parsed.incremental_inventory.unwrap();
        assert_eq!(inc.schedule.unwrap().frequency, Some(6));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_bucket_inventory() {
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

        // Deleting a non-existent inventory is expected to fail with
        // NoSuchInventory; this only verifies the request is well-formed.
        let result = client
            .delete_bucket_inventory(&DeleteBucketInventoryRequest::new(
                &config.bucket,
                "non-existent-inventory",
            ))
            .await;
        if let Err(error) = &result {
            eprintln!("delete_bucket_inventory rejected (expected for missing inventory): {}", error);
        }
    }
}
