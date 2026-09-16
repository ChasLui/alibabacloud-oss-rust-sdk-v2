use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the transfer acceleration configurations.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TransferAccelerationConfiguration {
    /// Whether the transfer acceleration is enabled for this bucket.
    #[serde(rename = "Enabled", skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketTransferAccelerationRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The container of the request body.
    pub transfer_acceleration_configuration: TransferAccelerationConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketTransferAccelerationResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Configures transfer acceleration for a bucket. After you enable transfer
    /// acceleration for a bucket, the object access speed is accelerated for
    /// users worldwide. The transfer acceleration feature is applicable to
    /// scenarios where data needs to be transferred over long geographical
    /// distances. This feature can also be used to download or upload objects
    /// that are gigabytes or terabytes in size.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketTransferAccelerationRequest` containing the
    ///   bucket name and the transfer acceleration configuration to set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{PutBucketTransferAccelerationRequest, TransferAccelerationConfiguration};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketTransferAccelerationRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     transfer_acceleration_configuration: TransferAccelerationConfiguration {
    ///         enabled: Some(true),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_transfer_acceleration(&request).await {
    ///     Ok(result) => {
    ///         println!("Transfer acceleration updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket transfer acceleration: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_transfer_acceleration(
        &self,
        request: &PutBucketTransferAccelerationRequest,
    ) -> Result<PutBucketTransferAccelerationResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketTransferAcceleration".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("transferAcceleration", "")]
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
            std::rc::Rc::new(vec!["transferAcceleration".to_string()]),
        );

        let xml_body = quick_xml::se::to_string_with_root(
            "TransferAccelerationConfiguration",
            &request.transfer_acceleration_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketTransferAccelerationResult::default();
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
    fn test_transfer_acceleration_configuration_serde_round_trip() {
        let config = TransferAccelerationConfiguration {
            enabled: Some(true),
        };

        let xml = quick_xml::se::to_string_with_root("TransferAccelerationConfiguration", &config)
            .unwrap();
        assert!(xml.contains("<TransferAccelerationConfiguration>"));
        assert!(xml.contains("<Enabled>true</Enabled>"));

        let parsed: TransferAccelerationConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.enabled, Some(true));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_transfer_acceleration() {
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

        let bucket_name = generate_unique_bucket_name("put-bucket-transfer-acceleration");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .put_bucket_transfer_acceleration(&PutBucketTransferAccelerationRequest {
                bucket: bucket_name.clone(),
                transfer_acceleration_configuration: TransferAccelerationConfiguration {
                    enabled: Some(true),
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "put_bucket_transfer_acceleration failed: {:?}",
            result.err()
        );

        // Clean up
        let _ = client
            .delete_bucket(&DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
