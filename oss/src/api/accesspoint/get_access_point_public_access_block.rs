use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::service::PublicAccessBlockConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::signer::SUB_RESOURCE;
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetAccessPointPublicAccessBlockRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the access point.
    #[field(type = "query", rename = "x-oss-access-point-name")]
    pub access_point_name: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetAccessPointPublicAccessBlockResult {
    /// The container in which the Block Public Access configurations are
    /// stored.
    pub public_access_block_configuration: Option<PublicAccessBlockConfiguration>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Queries the Block Public Access configurations of an access point.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetAccessPointPublicAccessBlockRequest` containing
    ///   the bucket name and the access point name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::accesspoint::GetAccessPointPublicAccessBlockRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetAccessPointPublicAccessBlockRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     access_point_name: Some("my-ap".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_access_point_public_access_block(&request).await {
    ///     Ok(result) => {
    ///         if let Some(config) = result.public_access_block_configuration {
    ///             println!("block public access: {:?}", config.block_public_access);
    ///         }
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get access point public access block: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_access_point_public_access_block(
        &self,
        request: &GetAccessPointPublicAccessBlockRequest,
    ) -> Result<GetAccessPointPublicAccessBlockResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetAccessPointPublicAccessBlock".to_string(),
            method: http::Method::GET,
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
        input
            .op_metadata
            .set(SUB_RESOURCE, Rc::new(vec!["publicAccessBlock".to_string()]));

        modify_request(&mut input, request.header_map(), request.query_map(), vec![])?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let config: PublicAccessBlockConfiguration = quick_xml::de::from_str(&data_str)?;

        let mut result = GetAccessPointPublicAccessBlockResult {
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

    use super::super::create_access_point::tests::generate_access_point_name;
    use super::super::create_access_point::{CreateAccessPointConfiguration, CreateAccessPointRequest};
    use super::super::delete_access_point::DeleteAccessPointRequest;
    use super::super::put_access_point_public_access_block::PutAccessPointPublicAccessBlockRequest;
    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_get_access_point_public_access_block_result_deserialize() {
        let xml = "<PublicAccessBlockConfiguration><BlockPublicAccess>true</BlockPublicAccess></PublicAccessBlockConfiguration>";
        let config: PublicAccessBlockConfiguration = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(config.block_public_access, Some(true));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_access_point_public_access_block() {
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

        // Prepare an access point with Block Public Access enabled
        let ap_name = generate_access_point_name();
        client
            .create_access_point(&CreateAccessPointRequest {
                bucket: config.bucket.clone(),
                create_access_point_configuration: CreateAccessPointConfiguration {
                    access_point_name: Some(ap_name.clone()),
                    network_origin: Some("internet".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .unwrap();

        client
            .put_access_point_public_access_block(&PutAccessPointPublicAccessBlockRequest {
                bucket: config.bucket.clone(),
                access_point_name: Some(ap_name.clone()),
                public_access_block_configuration: PublicAccessBlockConfiguration {
                    block_public_access: Some(true),
                },
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .get_access_point_public_access_block(&GetAccessPointPublicAccessBlockRequest {
                bucket: config.bucket.clone(),
                access_point_name: Some(ap_name.clone()),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "get_access_point_public_access_block failed: {:?}",
            result.err()
        );
        assert_eq!(
            result
                .unwrap()
                .public_access_block_configuration
                .and_then(|c| c.block_public_access),
            Some(true)
        );

        // Clean up
        let _ = client
            .delete_access_point(&DeleteAccessPointRequest {
                bucket: config.bucket.clone(),
                access_point_name: Some(ap_name),
                ..Default::default()
            })
            .await;
    }
}
