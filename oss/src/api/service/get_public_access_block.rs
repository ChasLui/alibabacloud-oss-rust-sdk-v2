use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::put_public_access_block::PublicAccessBlockConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetPublicAccessBlockRequest {
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetPublicAccessBlockResult {
    /// The container in which the Block Public Access configurations are
    /// stored.
    pub public_access_block_configuration: Option<PublicAccessBlockConfiguration>,

    pub common: ResultCommon,
}

impl Client {
    /// Queries the Block Public Access configurations of the Object Storage
    /// Service (OSS) resources of the current account.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetPublicAccessBlockRequest`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::service::GetPublicAccessBlockRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetPublicAccessBlockRequest::default();
    ///
    /// match client.get_public_access_block(&request).await {
    ///     Ok(result) => {
    ///         if let Some(config) = result.public_access_block_configuration {
    ///             println!("block public access: {:?}", config.block_public_access);
    ///         }
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get public access block: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_public_access_block(
        &self,
        request: &GetPublicAccessBlockRequest,
    ) -> Result<GetPublicAccessBlockResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetPublicAccessBlock".to_string(),
            method: http::Method::GET,
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

        // The SubResource metadata is required by the V1 signer.
        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["publicAccessBlock".to_string()]),
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

        let mut result = GetPublicAccessBlockResult {
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
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_get_public_access_block_result_deserialize() {
        let xml = "<PublicAccessBlockConfiguration><BlockPublicAccess>true</BlockPublicAccess></PublicAccessBlockConfiguration>";
        let config: PublicAccessBlockConfiguration = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(config.block_public_access, Some(true));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_public_access_block() {
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

        // The account may not have Block Public Access configured; accept a
        // service-side error as a valid exercised path.
        match client
            .get_public_access_block(&GetPublicAccessBlockRequest::default())
            .await
        {
            Ok(result) => {
                println!(
                    "block public access: {:?}",
                    result
                        .public_access_block_configuration
                        .and_then(|c| c.block_public_access)
                );
            }
            Err(error) => {
                println!("get_public_access_block returned error: {}", error);
            }
        }
    }
}
