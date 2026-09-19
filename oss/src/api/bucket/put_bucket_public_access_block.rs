use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::service::PublicAccessBlockConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketPublicAccessBlockRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// Request body.
    pub public_access_block_configuration: PublicAccessBlockConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketPublicAccessBlockResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Enables or disables Block Public Access for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketPublicAccessBlockRequest` containing the
    ///   bucket name and the Block Public Access configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::PutBucketPublicAccessBlockRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::api::service::PublicAccessBlockConfiguration;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketPublicAccessBlockRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     public_access_block_configuration: PublicAccessBlockConfiguration {
    ///         block_public_access: Some(true),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_public_access_block(&request).await {
    ///     Ok(result) => {
    ///         println!("Block Public Access set: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket public access block: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_public_access_block(
        &self,
        request: &PutBucketPublicAccessBlockRequest,
    ) -> Result<PutBucketPublicAccessBlockResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketPublicAccessBlock".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("publicAccessBlock", "")]
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
            Rc::new(vec!["publicAccessBlock".to_string()]),
        );

        let xml_body = quick_xml::se::to_string_with_root(
            "PublicAccessBlockConfiguration",
            &request.public_access_block_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketPublicAccessBlockResult::default();
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

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_public_access_block() {
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
            .put_bucket_public_access_block(&PutBucketPublicAccessBlockRequest {
                bucket: config.bucket.clone(),
                public_access_block_configuration: PublicAccessBlockConfiguration {
                    block_public_access: Some(true),
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "put_bucket_public_access_block failed: {:?}",
            result.err()
        );

        // Clean up: remove the Block Public Access configuration
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
