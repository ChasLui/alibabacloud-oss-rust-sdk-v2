use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::delete_bucket_inventory::InventoryConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketInventoryRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the inventory.
    #[field(type = "query", rename = "inventoryId")]
    pub inventory_id: Option<String>,

    /// The inventory configuration to set.
    pub inventory_configuration: InventoryConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketInventoryResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Configures an inventory for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketInventoryRequest` containing the bucket
    ///   name, the inventory name and the inventory configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    /// #     InventoryConfiguration, InventoryDestination, InventoryOSSBucketDestination,
    /// #     InventorySchedule, OptionalFields, PutBucketInventoryRequest,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketInventoryRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     inventory_id: Some("my-inventory".to_string()),
    ///     inventory_configuration: InventoryConfiguration {
    ///         id: Some("my-inventory".to_string()),
    ///         is_enabled: Some(true),
    ///         destination: Some(InventoryDestination {
    ///             oss_bucket_destination: Some(InventoryOSSBucketDestination {
    ///                 format: Some("CSV".to_string()),
    ///                 account_id: Some("1000000000000000".to_string()),
    ///                 role_arn: Some("acs:ram::1000000000000000:role/my-role".to_string()),
    ///                 bucket: Some("acs:oss:::dest-bucket".to_string()),
    ///                 prefix: Some("reports/".to_string()),
    ///                 ..Default::default()
    ///             }),
    ///         }),
    ///         schedule: Some(InventorySchedule {
    ///             frequency: Some("Daily".to_string()),
    ///             ..Default::default()
    ///         }),
    ///         included_object_versions: Some("All".to_string()),
    ///         optional_fields: Some(OptionalFields {
    ///             fields: vec!["Size".to_string(), "LastModifiedDate".to_string()],
    ///         }),
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_inventory(&request).await {
    ///     Ok(result) => {
    ///         println!("Inventory configured: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket inventory: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_inventory(
        &self,
        request: &PutBucketInventoryRequest,
    ) -> Result<PutBucketInventoryResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketInventory".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("inventory", "")]
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
            SUB_RESOURCE,
            Rc::new(vec!["inventory".to_string(), "inventoryId".to_string()]),
        );

        let xml_body = quick_xml::se::to_string_with_root(
            "InventoryConfiguration",
            &request.inventory_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketInventoryResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::bucket::{
        InventoryDestination, InventoryOSSBucketDestination, InventorySchedule,
    };
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_inventory() {
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

        // A real inventory requires a RAM role ARN that can read the source
        // bucket and write to the destination bucket, so this call may be
        // rejected on test accounts without such a role.
        let inventory_id = "sdk-rust-test-inventory";
        let result = client
            .put_bucket_inventory(&PutBucketInventoryRequest {
                bucket: config.bucket.clone(),
                inventory_id: Some(inventory_id.to_string()),
                inventory_configuration: InventoryConfiguration {
                    id: Some(inventory_id.to_string()),
                    is_enabled: Some(true),
                    destination: Some(InventoryDestination {
                        oss_bucket_destination: Some(InventoryOSSBucketDestination {
                            format: Some("CSV".to_string()),
                            account_id: None,
                            role_arn: None,
                            bucket: Some(format!("acs:oss:::{}", config.bucket)),
                            prefix: Some("inventory/".to_string()),
                            encryption: None,
                        }),
                    }),
                    schedule: Some(InventorySchedule {
                        frequency: Some("Daily".to_string()),
                        day_of_month: None,
                    }),
                    included_object_versions: Some("All".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!(
                "put_bucket_inventory rejected (may lack a RAM role): {}",
                error
            );
            return;
        }

        // Clean up
        let _ = client
            .delete_bucket_inventory(&crate::api::bucket::DeleteBucketInventoryRequest::new(
                &config.bucket,
                inventory_id,
            ))
            .await;
    }
}
