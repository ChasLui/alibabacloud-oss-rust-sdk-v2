use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

use super::create_access_point::AccessPointVpcConfiguration;

/// The information about an access point.
#[derive(Debug, Default, Deserialize)]
pub struct AccessPoint {
    /// The network origin of the access point.
    #[serde(rename = "NetworkOrigin", skip_serializing_if = "Option::is_none")]
    pub network_origin: Option<String>,

    /// The container that stores the information about the VPC.
    #[serde(rename = "VpcConfiguration", skip_serializing_if = "Option::is_none")]
    pub vpc_configuration: Option<AccessPointVpcConfiguration>,

    /// The status of the access point.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// The name of the bucket for which the access point is configured.
    #[serde(rename = "Bucket", skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// The name of the access point.
    #[serde(rename = "AccessPointName", skip_serializing_if = "Option::is_none")]
    pub access_point_name: Option<String>,

    /// The alias of the access point.
    #[serde(rename = "Alias", skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

/// The container that stores the information about all access points.
#[derive(Debug, Default, Deserialize)]
pub struct AccessPoints {
    /// The access points.
    #[serde(rename = "AccessPoint", default)]
    pub access_point: Vec<AccessPoint>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct ListAccessPointsRequest {
    /// The maximum number of access points that can be returned.
    /// Valid values: * For user-level access points: (0,1000].
    /// * For bucket-level access points: (0,100].
    #[field(type = "query", rename = "max-keys")]
    pub max_keys: Option<i64>,

    /// The token from which the listing operation starts. You must specify
    /// the value of NextContinuationToken that is obtained from the previous
    /// query as the value of continuation-token.
    #[field(type = "query", rename = "continuation-token")]
    pub continuation_token: Option<String>,

    /// The name of the bucket. If specified, only bucket-level access points
    /// of this bucket are listed; otherwise user-level access points are
    /// listed.
    pub bucket: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct ListAccessPointsResult {
    /// The maximum number of results set for this enumeration operation.
    #[serde(rename = "MaxKeys", skip_serializing_if = "Option::is_none")]
    pub max_keys: Option<i32>,

    /// Indicates whether the returned list is truncated.
    /// true: indicates that not all results are returned.
    /// false: indicates that all results are returned.
    #[serde(rename = "IsTruncated", skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,

    /// Indicates that this ListAccessPoints request does not return all
    /// results that can be listed. You can use NextContinuationToken to
    /// continue obtaining list results.
    #[serde(rename = "NextContinuationToken", skip_serializing_if = "Option::is_none")]
    pub next_continuation_token: Option<String>,

    /// The ID of the Alibaba Cloud account to which the access point belongs.
    #[serde(rename = "AccountId", skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,

    /// The container that stores the information about all access points.
    #[serde(rename = "AccessPoints", skip_serializing_if = "Option::is_none")]
    pub access_points: Option<AccessPoints>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the information about user-level or bucket-level access points.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListAccessPointsRequest` containing the optional
    ///   bucket name and pagination parameters.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::accesspoint::ListAccessPointsRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListAccessPointsRequest {
    ///     max_keys: Some(100),
    ///     ..Default::default()
    /// };
    ///
    /// match client.list_access_points(&request).await {
    ///     Ok(result) => {
    ///         if let Some(access_points) = result.access_points {
    ///             for ap in access_points.access_point {
    ///                 println!("Access point: {:?}", ap.access_point_name);
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to list access points: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn list_access_points(
        &self,
        request: &ListAccessPointsRequest,
    ) -> Result<ListAccessPointsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListAccessPoints".to_string(),
            method: http::Method::GET,
            bucket: request.bucket.clone(),
            parameters: [("accessPoint", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
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
        let mut result: ListAccessPointsResult = quick_xml::de::from_str(&data_str)?;

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

    #[test]
    fn test_list_access_points_result_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListAccessPointsResult>
  <MaxKeys>100</MaxKeys>
  <IsTruncated>false</IsTruncated>
  <AccountId>1234567890</AccountId>
  <AccessPoints>
    <AccessPoint>
      <NetworkOrigin>internet</NetworkOrigin>
      <Status>enable</Status>
      <Bucket>my-bucket</Bucket>
      <AccessPointName>my-ap</AccessPointName>
      <Alias>my-ap-1234567890.oss-cn-hangzhou.oss-accesspoint.aliyuncs.com</Alias>
    </AccessPoint>
    <AccessPoint>
      <NetworkOrigin>vpc</NetworkOrigin>
      <VpcConfiguration>
        <VpcId>vpc-123</VpcId>
      </VpcConfiguration>
      <Status>enable</Status>
      <Bucket>my-bucket</Bucket>
      <AccessPointName>my-ap-2</AccessPointName>
      <Alias>my-ap-2-1234567890.oss-cn-hangzhou.oss-accesspoint.aliyuncs.com</Alias>
    </AccessPoint>
  </AccessPoints>
</ListAccessPointsResult>"#;
        let result: ListAccessPointsResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.max_keys, Some(100));
        assert_eq!(result.is_truncated, Some(false));
        assert_eq!(result.account_id.as_deref(), Some("1234567890"));
        let aps = result.access_points.unwrap().access_point;
        assert_eq!(aps.len(), 2);
        assert_eq!(aps[0].access_point_name.as_deref(), Some("my-ap"));
        assert_eq!(
            aps[1].vpc_configuration.as_ref().unwrap().vpc_id.as_deref(),
            Some("vpc-123")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_access_points() {
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

        // List bucket-level access points.
        let result = client
            .list_access_points(&ListAccessPointsRequest {
                bucket: Some(config.bucket.clone()),
                max_keys: Some(100),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "list_access_points failed: {:?}",
            result.err()
        );
    }
}
