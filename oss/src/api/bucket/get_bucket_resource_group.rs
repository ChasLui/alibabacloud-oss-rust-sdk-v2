use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the ID of the resource group.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BucketResourceGroupConfiguration {
    /// The ID of the resource group to which the bucket belongs.
    #[serde(rename = "ResourceGroupId", skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketResourceGroupRequest {
    /// The name of the bucket that you want to query.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetBucketResourceGroupResult {
    /// The ID of the resource group to which the bucket belongs.
    #[serde(rename = "ResourceGroupId", skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<String>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the ID of the resource group to which a bucket belongs.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketResourceGroupRequest` containing the bucket
    ///   name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketResourceGroupRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketResourceGroupRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_resource_group(&request).await {
    ///     Ok(result) => {
    ///         println!("Resource group ID: {:?}", result.resource_group_id);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket resource group: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_resource_group(
        &self,
        request: &GetBucketResourceGroupRequest,
    ) -> Result<GetBucketResourceGroupResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketResourceGroup".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("resourceGroup", "")]
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
            std::rc::Rc::new(vec!["resourceGroup".to_string()]),
        );

        modify_request(&mut input, request.header_map(), request.query_map(), vec![])?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetBucketResourceGroupResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_resource_group_configuration_serde_round_trip() {
        let config = BucketResourceGroupConfiguration {
            resource_group_id: Some("rg-aekz****".to_string()),
        };

        let xml =
            quick_xml::se::to_string_with_root("BucketResourceGroupConfiguration", &config)
                .unwrap();
        assert!(xml.contains("<BucketResourceGroupConfiguration>"));
        assert!(xml.contains("<ResourceGroupId>rg-aekz****</ResourceGroupId>"));

        let parsed: BucketResourceGroupConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.resource_group_id.as_deref(), Some("rg-aekz****"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_resource_group() {
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
            .get_bucket_resource_group(&GetBucketResourceGroupRequest {
                bucket: config.bucket.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_resource_group failed: {:?}",
            result.err()
        );
    }
}
