use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::service::PublicAccessBlockConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutAccessPointPublicAccessBlockRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the access point.
    #[field(type = "query", rename = "x-oss-access-point-name")]
    pub access_point_name: Option<String>,

    /// The request body.
    pub public_access_block_configuration: PublicAccessBlockConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutAccessPointPublicAccessBlockResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Enables or disables Block Public Access for an access point.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutAccessPointPublicAccessBlockRequest` containing
    ///   the bucket name, the access point name and the Block Public Access
    ///   configurations.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::accesspoint::PutAccessPointPublicAccessBlockRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::api::service::PublicAccessBlockConfiguration;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutAccessPointPublicAccessBlockRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     access_point_name: Some("my-ap".to_string()),
    ///     public_access_block_configuration: PublicAccessBlockConfiguration {
    ///         block_public_access: Some(true),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_access_point_public_access_block(&request).await {
    ///     Ok(result) => {
    ///         println!("Block Public Access configured: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put access point public access block: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_access_point_public_access_block(
        &self,
        request: &PutAccessPointPublicAccessBlockRequest,
    ) -> Result<PutAccessPointPublicAccessBlockResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "PutAccessPointPublicAccessBlock".to_string(),
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
        input
            .op_metadata
            .set(SUB_RESOURCE, Rc::new(vec!["publicAccessBlock".to_string()]));

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

        let mut result = PutAccessPointPublicAccessBlockResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::create_access_point::tests::generate_access_point_name;
    use super::super::create_access_point::{
        CreateAccessPointConfiguration, CreateAccessPointRequest,
    };
    use super::super::delete_access_point::DeleteAccessPointRequest;
    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_access_point_public_access_block_serde_round_trip() {
        let config = PublicAccessBlockConfiguration {
            block_public_access: Some(true),
        };
        let xml =
            quick_xml::se::to_string_with_root("PublicAccessBlockConfiguration", &config).unwrap();
        let parsed: PublicAccessBlockConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.block_public_access, Some(true));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_access_point_public_access_block() {
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

        // Prepare an access point
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

        let result = client
            .put_access_point_public_access_block(&PutAccessPointPublicAccessBlockRequest {
                bucket: config.bucket.clone(),
                access_point_name: Some(ap_name.clone()),
                public_access_block_configuration: PublicAccessBlockConfiguration {
                    block_public_access: Some(true),
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "put_access_point_public_access_block failed: {:?}",
            result.err()
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
