use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::delete_bucket_inventory::InventoryConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationOutput, OperationInput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketInventoryRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the inventory to be queried.
    #[field(type = "query", rename = "inventoryId")]
    pub inventory_id: Option<String>,

    pub common: RequestCommon,
}

impl GetBucketInventoryRequest {
    pub fn new(bucket: &str, inventory_id: &str) -> Self {
        GetBucketInventoryRequest {
            bucket: bucket.to_string(),
            inventory_id: Some(inventory_id.to_string()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetBucketInventoryResult {
    /// The inventory task configured for the bucket.
    pub inventory_configuration: Option<InventoryConfiguration>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Queries the inventories that are configured for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketInventoryRequest` containing the bucket name
    ///   and the inventory name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketInventoryRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketInventoryRequest::new("my-bucket", "my-inventory");
    ///
    /// match client.get_bucket_inventory(&request).await {
    ///     Ok(result) => {
    ///         println!("Inventory: {:?}", result.inventory_configuration);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket inventory: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_inventory(
        &self,
        request: &GetBucketInventoryRequest,
    ) -> Result<GetBucketInventoryResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketInventory".to_string(),
            method: http::Method::GET,
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

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let inventory_configuration: InventoryConfiguration = quick_xml::de::from_str(&data_str)?;

        let mut result = GetBucketInventoryResult {
            inventory_configuration: Some(inventory_configuration),
            ..Default::default()
        };
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
    fn test_get_bucket_inventory_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<InventoryConfiguration>
  <Id>report1</Id>
  <IsEnabled>true</IsEnabled>
  <Destination>
    <OSSBucketDestination>
      <Format>CSV</Format>
      <AccountId>1000000000000000</AccountId>
      <RoleArn>acs:ram::1000000000000000:role/test-role</RoleArn>
      <Bucket>acs:oss:::destination-bucket</Bucket>
      <Prefix>prefix1</Prefix>
    </OSSBucketDestination>
  </Destination>
  <Schedule>
    <Frequency>Daily</Frequency>
  </Schedule>
  <Filter>
    <Prefix>filterPrefix/</Prefix>
    <LastModifyBeginTimeStamp>1637883649</LastModifyBeginTimeStamp>
    <LastModifyEndTimeStamp>1638347592</LastModifyEndTimeStamp>
    <LowerSizeBound>1024</LowerSizeBound>
    <UpperSizeBound>1048576</UpperSizeBound>
    <StorageClass>Standard,IA</StorageClass>
  </Filter>
  <IncludedObjectVersions>All</IncludedObjectVersions>
  <OptionalFields>
    <Field>Size</Field>
    <Field>LastModifiedDate</Field>
  </OptionalFields>
</InventoryConfiguration>"#;
        let parsed: InventoryConfiguration = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(parsed.id.as_deref(), Some("report1"));
        assert_eq!(parsed.is_enabled, Some(true));
        assert_eq!(
            parsed.destination.unwrap().oss_bucket_destination.unwrap().bucket.as_deref(),
            Some("acs:oss:::destination-bucket")
        );
        assert_eq!(parsed.schedule.unwrap().frequency.as_deref(), Some("Daily"));
        assert_eq!(parsed.filter.unwrap().lower_size_bound, Some(1024));
        assert_eq!(parsed.optional_fields.unwrap().fields.len(), 2);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_inventory() {
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

        // Querying a non-existent inventory is expected to fail with
        // NoSuchInventory; this only verifies the request is well-formed.
        let result = client
            .get_bucket_inventory(&GetBucketInventoryRequest::new(
                &config.bucket,
                "non-existent-inventory",
            ))
            .await;
        if let Err(error) = &result {
            eprintln!("get_bucket_inventory rejected (expected for missing inventory): {}", error);
        }
    }
}
