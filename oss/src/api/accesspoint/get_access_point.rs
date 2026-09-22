use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::create_access_point::AccessPointVpcConfiguration;
use crate::api::service::PublicAccessBlockConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the endpoints of the access point.
#[derive(Debug, Default, Deserialize)]
pub struct AccessPointEndpoints {
    /// The public endpoint of the access point.
    #[serde(rename = "PublicEndpoint", skip_serializing_if = "Option::is_none")]
    pub public_endpoint: Option<String>,

    /// The internal endpoint of the access point.
    #[serde(rename = "InternalEndpoint", skip_serializing_if = "Option::is_none")]
    pub internal_endpoint: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct GetAccessPointRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the access point.
    #[field(type = "header", rename = "x-oss-access-point-name")]
    pub access_point_name: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetAccessPointResult {
    /// The ARN of the access point.
    #[serde(rename = "AccessPointArn", skip_serializing_if = "Option::is_none")]
    pub access_point_arn: Option<String>,

    /// The alias of the access point.
    #[serde(rename = "Alias", skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,

    /// The container that stores the endpoints of the access point.
    #[serde(rename = "Endpoints", skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<AccessPointEndpoints>,

    /// The time when the access point was created.
    #[serde(rename = "CreationDate", skip_serializing_if = "Option::is_none")]
    pub creation_date: Option<String>,

    /// The name of the access point.
    #[serde(rename = "AccessPointName", skip_serializing_if = "Option::is_none")]
    pub access_point_name: Option<String>,

    /// The name of the bucket for which the access point is configured.
    #[serde(rename = "Bucket", skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// The ID of the Alibaba Cloud account for which the access point is
    /// configured.
    #[serde(rename = "AccountId", skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,

    /// The network origin of the access point. Valid values: vpc and internet.
    /// vpc: You can only use the specified VPC ID to access the access point.
    /// internet: You can use public endpoints and internal endpoints to access
    /// the access point.
    #[serde(rename = "NetworkOrigin", skip_serializing_if = "Option::is_none")]
    pub network_origin: Option<String>,

    /// The container that stores the information about the VPC.
    #[serde(rename = "VpcConfiguration", skip_serializing_if = "Option::is_none")]
    pub vpc_configuration: Option<AccessPointVpcConfiguration>,

    /// The status of the access point.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub access_point_status: Option<String>,

    /// The container that stores the Block Public Access configurations.
    #[serde(
        rename = "PublicAccessBlockConfiguration",
        skip_serializing_if = "Option::is_none"
    )]
    pub public_access_block_configuration: Option<PublicAccessBlockConfiguration>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the information about an access point.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetAccessPointRequest` containing the bucket name and
    ///   the access point name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::accesspoint::GetAccessPointRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetAccessPointRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     access_point_name: Some("my-ap".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_access_point(&request).await {
    ///     Ok(result) => {
    ///         println!("Access point status: {:?}", result.access_point_status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get access point: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_access_point(
        &self,
        request: &GetAccessPointRequest,
    ) -> Result<GetAccessPointResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetAccessPoint".to_string(),
            method: http::Method::GET,
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

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetAccessPointResult = quick_xml::de::from_str(&data_str)?;

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
    fn test_get_access_point_result_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<GetAccessPointResult>
  <AccessPointArn>acs:oss:cn-hangzhou:1234567890:accesspoint/my-ap</AccessPointArn>
  <Alias>my-ap-1234567890.oss-cn-hangzhou.oss-accesspoint.aliyuncs.com</Alias>
  <Endpoints>
    <PublicEndpoint>my-ap-1234567890.oss-cn-hangzhou.oss-accesspoint.aliyuncs.com</PublicEndpoint>
    <InternalEndpoint>my-ap-1234567890.oss-cn-hangzhou-internal.oss-accesspoint.aliyuncs.com</InternalEndpoint>
  </Endpoints>
  <CreationDate>2026-01-01T00:00:00.000Z</CreationDate>
  <AccessPointName>my-ap</AccessPointName>
  <Bucket>my-bucket</Bucket>
  <AccountId>1234567890</AccountId>
  <NetworkOrigin>internet</NetworkOrigin>
  <VpcConfiguration>
    <VpcId>vpc-123</VpcId>
  </VpcConfiguration>
  <Status>enable</Status>
  <PublicAccessBlockConfiguration>
    <BlockPublicAccess>true</BlockPublicAccess>
  </PublicAccessBlockConfiguration>
</GetAccessPointResult>"#;
        let result: GetAccessPointResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.access_point_name.as_deref(), Some("my-ap"));
        assert_eq!(result.bucket.as_deref(), Some("my-bucket"));
        assert_eq!(result.account_id.as_deref(), Some("1234567890"));
        assert_eq!(result.network_origin.as_deref(), Some("internet"));
        assert_eq!(result.access_point_status.as_deref(), Some("enable"));
        let endpoints = result.endpoints.unwrap();
        assert!(endpoints.public_endpoint.is_some());
        assert!(endpoints.internal_endpoint.is_some());
        assert_eq!(
            result.vpc_configuration.unwrap().vpc_id.as_deref(),
            Some("vpc-123")
        );
        assert_eq!(
            result
                .public_access_block_configuration
                .unwrap()
                .block_public_access,
            Some(true)
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_access_point() {
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
            .get_access_point(&GetAccessPointRequest {
                bucket: config.bucket.clone(),
                access_point_name: Some(ap_name.clone()),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "get_access_point failed: {:?}",
            result.err()
        );
        assert_eq!(
            result.unwrap().access_point_name.as_deref(),
            Some(ap_name.as_str())
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
