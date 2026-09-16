use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container in which the Block Public Access configurations are stored.
///
/// This type is shared by the account-level, bucket-level and
/// access-point-level Block Public Access operations.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PublicAccessBlockConfiguration {
    /// Specifies whether to enable Block Public Access.
    /// true: enables Block Public Access.
    /// false (default): disables Block Public Access.
    #[serde(rename = "BlockPublicAccess", skip_serializing_if = "Option::is_none")]
    pub block_public_access: Option<bool>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutPublicAccessBlockRequest {
    /// The container in which the Block Public Access configurations are
    /// stored.
    pub public_access_block_configuration: PublicAccessBlockConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutPublicAccessBlockResult {
    pub common: ResultCommon,
}

impl Client {
    /// Enables or disables Block Public Access for Object Storage Service
    /// (OSS) resources of the current account.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutPublicAccessBlockRequest` containing the Block
    ///   Public Access configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::service::{PutPublicAccessBlockRequest, PublicAccessBlockConfiguration};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutPublicAccessBlockRequest {
    ///     public_access_block_configuration: PublicAccessBlockConfiguration {
    ///         block_public_access: Some(true),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_public_access_block(&request).await {
    ///     Ok(result) => {
    ///         println!("status: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put public access block: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_public_access_block(
        &self,
        request: &PutPublicAccessBlockRequest,
    ) -> Result<PutPublicAccessBlockResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutPublicAccessBlock".to_string(),
            method: http::Method::PUT,
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

        let mut result = PutPublicAccessBlockResult::default();
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
    fn test_public_access_block_configuration_serde_round_trip() {
        let config = PublicAccessBlockConfiguration {
            block_public_access: Some(true),
        };

        let xml =
            quick_xml::se::to_string_with_root("PublicAccessBlockConfiguration", &config).unwrap();
        assert!(xml.contains("<PublicAccessBlockConfiguration>"));
        assert!(xml.contains("<BlockPublicAccess>true</BlockPublicAccess>"));

        let parsed: PublicAccessBlockConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.block_public_access, Some(true));

        let parsed_false: PublicAccessBlockConfiguration = quick_xml::de::from_str(
            "<PublicAccessBlockConfiguration><BlockPublicAccess>false</BlockPublicAccess></PublicAccessBlockConfiguration>",
        )
        .unwrap();
        assert_eq!(parsed_false.block_public_access, Some(false));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_public_access_block() {
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

        // Enable Block Public Access for the account.
        let put_result = client
            .put_public_access_block(&PutPublicAccessBlockRequest {
                public_access_block_configuration: PublicAccessBlockConfiguration {
                    block_public_access: Some(true),
                },
                ..Default::default()
            })
            .await;
        assert!(
            put_result.is_ok(),
            "put_public_access_block failed: {:?}",
            put_result.err()
        );

        // Query the configuration.
        let get_result = client
            .get_public_access_block(&crate::api::service::GetPublicAccessBlockRequest::default())
            .await;
        assert!(
            get_result.is_ok(),
            "get_public_access_block failed: {:?}",
            get_result.err()
        );
        let got = get_result
            .unwrap()
            .public_access_block_configuration
            .unwrap();
        assert_eq!(got.block_public_access, Some(true));

        // Clean up: delete the account-level configuration.
        let del_result = client
            .delete_public_access_block(
                &crate::api::service::DeletePublicAccessBlockRequest::default(),
            )
            .await;
        assert!(
            del_result.is_ok(),
            "delete_public_access_block failed: {:?}",
            del_result.err()
        );
    }
}
