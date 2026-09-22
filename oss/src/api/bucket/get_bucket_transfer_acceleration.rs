use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketTransferAccelerationRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "TransferAccelerationConfiguration")]
pub struct GetBucketTransferAccelerationResult {
    /// Indicates whether transfer acceleration is enabled for the bucket.
    #[serde(rename = "Enabled", skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the transfer acceleration configurations of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketTransferAccelerationRequest` containing the
    ///   bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketTransferAccelerationRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketTransferAccelerationRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_transfer_acceleration(&request).await {
    ///     Ok(result) => {
    ///         println!("Transfer acceleration enabled: {:?}", result.enabled);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket transfer acceleration: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_transfer_acceleration(
        &self,
        request: &GetBucketTransferAccelerationRequest,
    ) -> Result<GetBucketTransferAccelerationResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketTransferAcceleration".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("transferAcceleration", "")]
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
            std::rc::Rc::new(vec!["transferAcceleration".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        // Parse the XML response
        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetBucketTransferAccelerationResult = quick_xml::de::from_str(&data_str)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::put_bucket_transfer_acceleration::{
        PutBucketTransferAccelerationRequest, TransferAccelerationConfiguration,
    };
    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_get_bucket_transfer_acceleration_result_deserialize() {
        let xml = r#"<TransferAccelerationConfiguration><Enabled>true</Enabled></TransferAccelerationConfiguration>"#;
        let result: GetBucketTransferAccelerationResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.enabled, Some(true));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_transfer_acceleration() {
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

        let bucket_name = generate_unique_bucket_name("get-bucket-transfer-acceleration");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        client
            .put_bucket_transfer_acceleration(&PutBucketTransferAccelerationRequest {
                bucket: bucket_name.clone(),
                transfer_acceleration_configuration: TransferAccelerationConfiguration {
                    enabled: Some(true),
                },
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .get_bucket_transfer_acceleration(&GetBucketTransferAccelerationRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_transfer_acceleration failed: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().enabled, Some(true));

        // Clean up
        let _ = client
            .delete_bucket(&DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
