use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::bucket::Owner;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct ListCloudBoxesRequest {
    /// The name of the bucket from which the list operation begins.
    #[field(type = "query", rename = "marker")]
    pub marker: Option<String>,

    /// The maximum number of buckets that can be returned in the single
    /// query. Valid values: 1 to 1000.
    #[field(type = "query", rename = "max-keys")]
    pub max_keys: Option<i32>,

    /// The prefix that the names of returned buckets must contain.
    #[field(type = "query", rename = "prefix")]
    pub prefix: Option<String>,

    pub common: RequestCommon,
}

/// The properties of a cloud box bucket.
#[derive(Debug, Default, Deserialize)]
pub struct CloudBoxProperties {
    /// The ID of the cloud box.
    #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The name of the cloud box.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The region in which the cloud box is deployed.
    #[serde(rename = "Region", skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// The control endpoint of the cloud box.
    #[serde(rename = "ControlEndpoint", skip_serializing_if = "Option::is_none")]
    pub control_endpoint: Option<String>,

    /// The data endpoint of the cloud box.
    #[serde(rename = "DataEndpoint", skip_serializing_if = "Option::is_none")]
    pub data_endpoint: Option<String>,
}

/// The container that stores the cloud box buckets.
#[derive(Debug, Default, Deserialize)]
pub struct CloudBoxes {
    /// The cloud box buckets.
    #[serde(rename = "CloudBox", default)]
    pub cloud_box: Vec<CloudBoxProperties>,
}

#[derive(Debug, Deserialize, OssResultModel)]
pub struct ListCloudBoxesResult {
    /// The prefix contained in the names of the returned bucket.
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// The name of the bucket after which the ListCloudBoxes operation
    /// starts.
    #[serde(rename = "Marker", skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,

    /// The maximum number of buckets that can be returned for the request.
    #[serde(rename = "MaxKeys", skip_serializing_if = "Option::is_none")]
    pub max_keys: Option<i32>,

    /// Indicates whether all results are returned.
    /// true: Only part of the results are returned for the request.
    /// false: All results are returned for the request.
    #[serde(rename = "IsTruncated", skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,

    /// The marker for the next ListCloudBoxes request, which can be used to
    /// return the remaining results.
    #[serde(rename = "NextMarker", skip_serializing_if = "Option::is_none")]
    pub next_marker: Option<String>,

    /// The container that stores information about the bucket owner.
    #[serde(rename = "Owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<Owner>,

    /// The container that stores information about cloud box buckets.
    #[serde(rename = "CloudBoxes", skip_serializing_if = "Option::is_none")]
    pub cloud_boxes: Option<CloudBoxes>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists cloud box buckets that belong to the current account.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListCloudBoxesRequest` containing the optional
    ///   marker, max-keys and prefix filters.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::service::ListCloudBoxesRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListCloudBoxesRequest::default();
    ///
    /// match client.list_cloud_boxes(&request).await {
    ///     Ok(result) => {
    ///         println!("is truncated: {:?}", result.is_truncated);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to list cloud boxes: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn list_cloud_boxes(
        &self,
        request: &ListCloudBoxesRequest,
    ) -> Result<ListCloudBoxesResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListCloudBoxes".to_string(),
            method: http::Method::GET,
            parameters: [("cloudboxes", "")]
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
            std::rc::Rc::new(vec!["cloudboxes".to_string()]),
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
        let mut result: ListCloudBoxesResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_list_cloud_boxes_result_deserialize() {
        let xml = r#"<ListCloudBoxResult>
            <Prefix>cb</Prefix>
            <Marker></Marker>
            <MaxKeys>100</MaxKeys>
            <IsTruncated>false</IsTruncated>
            <NextMarker></NextMarker>
            <Owner>
                <ID>1234567890</ID>
                <DisplayName>1234567890</DisplayName>
            </Owner>
            <CloudBoxes>
                <CloudBox>
                    <ID>cb-001</ID>
                    <Name>my-cloud-box</Name>
                    <Region>cn-hangzhou</Region>
                    <ControlEndpoint>cb-001.oss-cn-hangzhou.aliyuncs.com</ControlEndpoint>
                    <DataEndpoint>cb-001.oss-cn-hangzhou.aliyuncs.com</DataEndpoint>
                </CloudBox>
            </CloudBoxes>
        </ListCloudBoxResult>"#;

        let result: ListCloudBoxesResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.prefix.as_deref(), Some("cb"));
        assert_eq!(result.max_keys, Some(100));
        assert_eq!(result.is_truncated, Some(false));
        let boxes = result.cloud_boxes.unwrap().cloud_box;
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].id.as_deref(), Some("cb-001"));
        assert_eq!(boxes[0].region.as_deref(), Some("cn-hangzhou"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_cloud_boxes() {
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

        let result = client.list_cloud_boxes(&ListCloudBoxesRequest::default()).await;
        assert!(result.is_ok(), "list_cloud_boxes failed: {:?}", result.err());
    }
}
