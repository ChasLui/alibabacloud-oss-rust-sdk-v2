use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::service::PublicAccessBlockConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketPublicAccessBlockRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl GetBucketPublicAccessBlockRequest {
    pub fn new(bucket: &str) -> Self {
        GetBucketPublicAccessBlockRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetBucketPublicAccessBlockResult {
    /// The container in which the Block Public Access configurations are
    /// stored.
    pub public_access_block_configuration: Option<PublicAccessBlockConfiguration>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Queries the Block Public Access configurations of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketPublicAccessBlockRequest` containing the
    ///   bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketPublicAccessBlockRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketPublicAccessBlockRequest::new("my-bucket");
    ///
    /// match client.get_bucket_public_access_block(&request).await {
    ///     Ok(result) => {
    ///         println!(
    ///             "Block Public Access config: {:?}",
    ///             result.public_access_block_configuration
    ///         );
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket public access block: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_public_access_block(
        &self,
        request: &GetBucketPublicAccessBlockRequest,
    ) -> Result<GetBucketPublicAccessBlockResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketPublicAccessBlock".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("publicAccessBlock", "")]
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
            Rc::new(vec!["publicAccessBlock".to_string()]),
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
        let config: PublicAccessBlockConfiguration = quick_xml::de::from_str(&data_str)?;

        let mut result = GetBucketPublicAccessBlockResult {
            public_access_block_configuration: Some(config),
            ..Default::default()
        };
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
    use crate::SignatureVersionType;
    use crate::test_utils::load_test_config;

    #[test]
    fn test_public_access_block_configuration_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PublicAccessBlockConfiguration>
  <BlockPublicAccess>true</BlockPublicAccess>
</PublicAccessBlockConfiguration>"#;
        let config: PublicAccessBlockConfiguration = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(config.block_public_access, Some(true));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_public_access_block() {
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

        client
            .put_bucket_public_access_block(
                &crate::api::bucket::PutBucketPublicAccessBlockRequest {
                    bucket: config.bucket.clone(),
                    public_access_block_configuration: PublicAccessBlockConfiguration {
                        block_public_access: Some(true),
                    },
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let result = client
            .get_bucket_public_access_block(&GetBucketPublicAccessBlockRequest::new(&config.bucket))
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_public_access_block failed: {:?}",
            result.err()
        );
        let result = result.unwrap();
        assert!(result.public_access_block_configuration.is_some());
        assert_eq!(
            result
                .public_access_block_configuration
                .unwrap()
                .block_public_access,
            Some(true)
        );

        // Clean up
        let _ = client
            .delete_bucket_public_access_block(
                &crate::api::bucket::DeleteBucketPublicAccessBlockRequest {
                    bucket: config.bucket.clone(),
                    ..Default::default()
                },
            )
            .await;
    }
}
