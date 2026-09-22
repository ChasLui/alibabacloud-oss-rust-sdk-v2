use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the information about the VPC.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AccessPointVpcConfiguration {
    /// The ID of the VPC that is required only when the NetworkOrigin
    /// parameter is set to vpc.
    #[serde(rename = "VpcId", skip_serializing_if = "Option::is_none")]
    pub vpc_id: Option<String>,
}

/// The container of the CreateAccessPoint request body.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CreateAccessPointConfiguration {
    /// The name of the access point. The name must be unique in a region of
    /// your Alibaba Cloud account, cannot end with -ossalias, can contain only
    /// lowercase letters, digits, and hyphens (-), and must be 3 to 19
    /// characters in length.
    #[serde(rename = "AccessPointName", skip_serializing_if = "Option::is_none")]
    pub access_point_name: Option<String>,

    /// The network origin of the access point. Valid values: vpc and internet.
    #[serde(rename = "NetworkOrigin", skip_serializing_if = "Option::is_none")]
    pub network_origin: Option<String>,

    /// The container that stores the information about the VPC.
    #[serde(rename = "VpcConfiguration", skip_serializing_if = "Option::is_none")]
    pub vpc_configuration: Option<AccessPointVpcConfiguration>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct CreateAccessPointRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The container of the request body.
    pub create_access_point_configuration: CreateAccessPointConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct CreateAccessPointResult {
    /// The Alibaba Cloud Resource Name (ARN) of the access point.
    #[serde(rename = "AccessPointArn", skip_serializing_if = "Option::is_none")]
    pub access_point_arn: Option<String>,

    /// The alias of the access point.
    #[serde(rename = "Alias", skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Creates an access point.
    ///
    /// # Arguments
    ///
    /// * `request` - The `CreateAccessPointRequest` containing the bucket name
    ///   and the access point configuration to create.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::accesspoint::{CreateAccessPointConfiguration, CreateAccessPointRequest};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = CreateAccessPointRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     create_access_point_configuration: CreateAccessPointConfiguration {
    ///         access_point_name: Some("my-ap".to_string()),
    ///         network_origin: Some("internet".to_string()),
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.create_access_point(&request).await {
    ///     Ok(result) => {
    ///         println!("Access point ARN: {:?}", result.access_point_arn);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to create access point: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn create_access_point(
        &self,
        request: &CreateAccessPointRequest,
    ) -> Result<CreateAccessPointResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "CreateAccessPoint".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("accessPoint", "")]
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
            .set(SUB_RESOURCE, Rc::new(vec!["accessPoint".to_string()]));

        let xml_body = quick_xml::se::to_string_with_root(
            "CreateAccessPointConfiguration",
            &request.create_access_point_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: CreateAccessPointResult = quick_xml::de::from_str(&data_str)?;

        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::service::PublicAccessBlockConfiguration;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    pub(crate) fn generate_access_point_name() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        // Access point names must be 3 to 19 characters long.
        format!("ap-{:013}", millis)
    }

    #[test]
    fn test_create_access_point_configuration_round_trip() {
        let config = CreateAccessPointConfiguration {
            access_point_name: Some("my-ap".to_string()),
            network_origin: Some("vpc".to_string()),
            vpc_configuration: Some(AccessPointVpcConfiguration {
                vpc_id: Some("vpc-123".to_string()),
            }),
        };

        let xml =
            quick_xml::se::to_string_with_root("CreateAccessPointConfiguration", &config).unwrap();
        assert!(xml.contains("<CreateAccessPointConfiguration>"));
        assert!(xml.contains("<AccessPointName>my-ap</AccessPointName>"));
        assert!(xml.contains("<NetworkOrigin>vpc</NetworkOrigin>"));
        assert!(xml.contains("<VpcId>vpc-123</VpcId>"));

        let parsed: CreateAccessPointConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.access_point_name.as_deref(), Some("my-ap"));
        assert_eq!(parsed.network_origin.as_deref(), Some("vpc"));
        assert_eq!(
            parsed.vpc_configuration.unwrap().vpc_id.as_deref(),
            Some("vpc-123")
        );
    }

    #[test]
    fn test_public_access_block_configuration_round_trip() {
        let config = PublicAccessBlockConfiguration {
            block_public_access: Some(true),
        };
        let xml =
            quick_xml::se::to_string_with_root("PublicAccessBlockConfiguration", &config).unwrap();
        assert!(xml.contains("<BlockPublicAccess>true</BlockPublicAccess>"));

        let parsed: PublicAccessBlockConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.block_public_access, Some(true));
    }

    #[test]
    fn test_create_access_point_result_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<CreateAccessPointResult>
  <AccessPointArn>acs:oss:cn-hangzhou:1234567890:accesspoint/my-ap</AccessPointArn>
  <Alias>my-ap-1234567890.oss-cn-hangzhou.oss-accesspoint.aliyuncs.com</Alias>
</CreateAccessPointResult>"#;
        let result: CreateAccessPointResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(
            result.access_point_arn.as_deref(),
            Some("acs:oss:cn-hangzhou:1234567890:accesspoint/my-ap")
        );
        assert!(result.alias.is_some());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_access_point() {
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

        let ap_name = generate_access_point_name();
        let result = client
            .create_access_point(&CreateAccessPointRequest {
                bucket: config.bucket.clone(),
                create_access_point_configuration: CreateAccessPointConfiguration {
                    access_point_name: Some(ap_name.clone()),
                    network_origin: Some("internet".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "create_access_point failed: {:?}",
            result.err()
        );

        // Clean up
        let _ = client
            .delete_access_point(&crate::api::accesspoint::DeleteAccessPointRequest {
                bucket: config.bucket.clone(),
                access_point_name: Some(ap_name),
                ..Default::default()
            })
            .await;
    }
}
