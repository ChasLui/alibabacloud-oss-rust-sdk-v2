use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::delete_bucket_inventory::InventoryConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationOutput, OperationInput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct ListBucketInventoryRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// Specify the start position of the list operation. You can obtain this
    /// token from the `next_continuation_token` field of the previous
    /// `ListBucketInventory` result.
    #[field(type = "query", rename = "continuation-token")]
    pub continuation_token: Option<String>,

    pub common: RequestCommon,
}

impl ListBucketInventoryRequest {
    pub fn new(bucket: &str) -> Self {
        ListBucketInventoryRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct ListBucketInventoryResult {
    /// The container that stores inventory configurations.
    #[serde(rename = "InventoryConfiguration", default)]
    pub inventory_configurations: Vec<InventoryConfiguration>,

    /// Specifies whether to list all inventory tasks configured for the bucket.
    /// The value of true indicates that not all inventory tasks are listed; set
    /// the continuation-token parameter in the next request to the value of
    /// `next_continuation_token`.
    #[serde(rename = "IsTruncated", skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,

    /// If `is_truncated` is true and this field is not empty, set the
    /// continuation-token parameter in the next request to the value of this field.
    #[serde(rename = "NextContinuationToken", skip_serializing_if = "Option::is_none")]
    pub next_continuation_token: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries all inventories in a bucket at a time.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListBucketInventoryRequest` containing the bucket name
    ///   and an optional continuation token.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::ListBucketInventoryRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListBucketInventoryRequest::new("my-bucket");
    ///
    /// match client.list_bucket_inventory(&request).await {
    ///     Ok(result) => {
    ///         println!("Inventories: {:?}", result.inventory_configurations.len());
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to list bucket inventory: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn list_bucket_inventory(
        &self,
        request: &ListBucketInventoryRequest,
    ) -> Result<ListBucketInventoryResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListBucketInventory".to_string(),
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
        input
            .op_metadata
            .set(SUB_RESOURCE, Rc::new(vec!["inventory".to_string()]));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: ListBucketInventoryResult = quick_xml::de::from_str(&data_str)?;

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
    fn test_list_bucket_inventory_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListInventoryConfigurationsResult>
  <InventoryConfiguration>
    <Id>report1</Id>
    <IsEnabled>true</IsEnabled>
    <IncludedObjectVersions>All</IncludedObjectVersions>
  </InventoryConfiguration>
  <InventoryConfiguration>
    <Id>report2</Id>
    <IsEnabled>false</IsEnabled>
    <IncludedObjectVersions>Current</IncludedObjectVersions>
  </InventoryConfiguration>
  <IsTruncated>true</IsTruncated>
  <NextContinuationToken>token-xxx</NextContinuationToken>
</ListInventoryConfigurationsResult>"#;
        let result: ListBucketInventoryResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.inventory_configurations.len(), 2);
        assert_eq!(result.inventory_configurations[0].id.as_deref(), Some("report1"));
        assert_eq!(result.inventory_configurations[1].is_enabled, Some(false));
        assert_eq!(result.is_truncated, Some(true));
        assert_eq!(result.next_continuation_token.as_deref(), Some("token-xxx"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_bucket_inventory() {
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
            .list_bucket_inventory(&ListBucketInventoryRequest::new(&config.bucket))
            .await;
        assert!(
            result.is_ok(),
            "list_bucket_inventory failed: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().common.status, http::StatusCode::OK);
    }
}
