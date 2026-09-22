use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::get_bucket_resource_group::BucketResourceGroupConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketResourceGroupRequest {
    /// The bucket for which you want to modify the ID of the resource group.
    pub bucket: String,

    /// The container that stores the ID of the resource group.
    pub bucket_resource_group_configuration: BucketResourceGroupConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketResourceGroupResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Modifies the ID of the resource group to which a bucket belongs.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketResourceGroupRequest` containing the bucket
    ///   name and the ID of the resource group.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{BucketResourceGroupConfiguration, PutBucketResourceGroupRequest};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketResourceGroupRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     bucket_resource_group_configuration: BucketResourceGroupConfiguration {
    ///         resource_group_id: Some("rg-aekz****".to_string()),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_resource_group(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket resource group updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket resource group: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_resource_group(
        &self,
        request: &PutBucketResourceGroupRequest,
    ) -> Result<PutBucketResourceGroupResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketResourceGroup".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("resourceGroup", "")]
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
            std::rc::Rc::new(vec!["resourceGroup".to_string()]),
        );

        let xml_body = quick_xml::se::to_string_with_root(
            "BucketResourceGroupConfiguration",
            &request.bucket_resource_group_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketResourceGroupResult::default();
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

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_resource_group() {
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

        // Query the current resource group and set it back (idempotent)
        let current = client
            .get_bucket_resource_group(&crate::api::bucket::GetBucketResourceGroupRequest {
                bucket: config.bucket.clone(),
                ..Default::default()
            })
            .await
            .unwrap();
        let resource_group_id = match current.resource_group_id {
            Some(id) => id,
            None => {
                eprintln!("No resource group ID returned. Skipping test.");
                return;
            }
        };

        let result = client
            .put_bucket_resource_group(&PutBucketResourceGroupRequest {
                bucket: config.bucket.clone(),
                bucket_resource_group_configuration: BucketResourceGroupConfiguration {
                    resource_group_id: Some(resource_group_id),
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "put_bucket_resource_group failed: {:?}",
            result.err()
        );
    }
}
