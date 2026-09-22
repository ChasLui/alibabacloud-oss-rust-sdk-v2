use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::create_access_point_for_object_process::AccessPointEndpoints;
use crate::api::service::PublicAccessBlockConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetAccessPointForObjectProcessRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the Object FC Access Point. The name of an Object FC Access
    /// Point cannot exceed 63 characters in length, can contain only lowercase
    /// letters, digits, and hyphens (-), cannot start or end with a hyphen (-),
    /// and must be unique in the current region.
    #[field(type = "header", rename = "x-oss-access-point-for-object-process-name")]
    pub access_point_for_object_process_name: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "GetAccessPointForObjectProcessResult")]
pub struct GetAccessPointForObjectProcessResult {
    /// Whether allow anonymous user to access this FC Access Point.
    #[serde(
        rename = "AllowAnonymousAccessForObjectProcess",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_anonymous_access_for_object_process: Option<String>,

    /// The container in which the Block Public Access configurations are
    /// stored.
    #[serde(
        rename = "PublicAccessBlockConfiguration",
        skip_serializing_if = "Option::is_none"
    )]
    pub public_access_block_configuration: Option<PublicAccessBlockConfiguration>,

    /// The name of the Object FC Access Point.
    #[serde(
        rename = "AccessPointNameForObjectProcess",
        skip_serializing_if = "Option::is_none"
    )]
    pub access_point_name_for_object_process: Option<String>,

    /// The ARN of the Object FC Access Point.
    #[serde(
        rename = "AccessPointForObjectProcessArn",
        skip_serializing_if = "Option::is_none"
    )]
    pub access_point_for_object_process_arn: Option<String>,

    /// The time when the Object FC Access Point was created. The value is a
    /// timestamp.
    #[serde(rename = "CreationDate", skip_serializing_if = "Option::is_none")]
    pub creation_date: Option<String>,

    /// The status of the Object FC Access Point. Valid values: enable, disable,
    /// creating and deleting.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub access_point_for_object_process_status: Option<String>,

    /// The container that stores the endpoints of the Object FC Access Point.
    #[serde(rename = "Endpoints", skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<AccessPointEndpoints>,

    /// The alias of the Object FC Access Point.
    #[serde(
        rename = "AccessPointForObjectProcessAlias",
        skip_serializing_if = "Option::is_none"
    )]
    pub access_point_for_object_process_alias: Option<String>,

    /// The name of the access point.
    #[serde(rename = "AccessPointName", skip_serializing_if = "Option::is_none")]
    pub access_point_name: Option<String>,

    /// The UID of the Alibaba Cloud account.
    #[serde(rename = "AccountId", skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries basic information about an Object FC Access Point.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetAccessPointForObjectProcessRequest` containing the
    ///   bucket name and the Object FC Access Point name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetAccessPointForObjectProcessRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetAccessPointForObjectProcessRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     access_point_for_object_process_name: Some("fc-ap-01".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_access_point_for_object_process(&request).await {
    ///     Ok(result) => {
    ///         println!(
    ///             "Access point status: {:?}",
    ///             result.access_point_for_object_process_status
    ///         );
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get access point for object process: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_access_point_for_object_process(
        &self,
        request: &GetAccessPointForObjectProcessRequest,
    ) -> Result<GetAccessPointForObjectProcessResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "GetAccessPointForObjectProcess".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("accessPointForObjectProcess", "")]
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
            std::rc::Rc::new(vec!["accessPointForObjectProcess".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetAccessPointForObjectProcessResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_get_result_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<GetAccessPointForObjectProcessResult>
  <AccessPointNameForObjectProcess>fc-ap-01</AccessPointNameForObjectProcess>
  <AccessPointForObjectProcessAlias>fc-ap-01-alias</AccessPointForObjectProcessAlias>
  <AccessPointName>ap-01</AccessPointName>
  <AccountId>1119</AccountId>
  <AccessPointForObjectProcessArn>acs:oss:cn-qingdao:1119:accesspointforobjectprocess/fc-ap-01</AccessPointForObjectProcessArn>
  <CreationDate>1626769503</CreationDate>
  <Status>enable</Status>
  <Endpoints>
    <PublicEndpoint>fc-ap-01.oss-cn-qingdao.oss-object-process.aliyuncs.com</PublicEndpoint>
    <InternalEndpoint>fc-ap-01.oss-cn-qingdao-internal.oss-object-process.aliyuncs.com</InternalEndpoint>
  </Endpoints>
  <PublicAccessBlockConfiguration>
    <BlockPublicAccess>true</BlockPublicAccess>
  </PublicAccessBlockConfiguration>
</GetAccessPointForObjectProcessResult>"#;
        let result: GetAccessPointForObjectProcessResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(
            result.access_point_name_for_object_process.as_deref(),
            Some("fc-ap-01")
        );
        assert_eq!(result.access_point_name.as_deref(), Some("ap-01"));
        assert_eq!(result.account_id.as_deref(), Some("1119"));
        assert_eq!(result.creation_date.as_deref(), Some("1626769503"));
        assert_eq!(
            result.access_point_for_object_process_status.as_deref(),
            Some("enable")
        );
        let endpoints = result.endpoints.unwrap();
        assert_eq!(
            endpoints.public_endpoint.as_deref(),
            Some("fc-ap-01.oss-cn-qingdao.oss-object-process.aliyuncs.com")
        );
        assert_eq!(
            endpoints.internal_endpoint.as_deref(),
            Some("fc-ap-01.oss-cn-qingdao-internal.oss-object-process.aliyuncs.com")
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
    async fn test_get_access_point_for_object_process() {
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

        // Querying a non-existent Object FC Access Point is expected to be
        // rejected by the server; the call exercises the request path.
        let result = client
            .get_access_point_for_object_process(&GetAccessPointForObjectProcessRequest {
                bucket: config.bucket.clone(),
                access_point_for_object_process_name: Some("fc-ap-nonexistent".to_string()),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!(
                "get_access_point_for_object_process rejected (access point may not exist): {}",
                error
            );
        }
    }
}
