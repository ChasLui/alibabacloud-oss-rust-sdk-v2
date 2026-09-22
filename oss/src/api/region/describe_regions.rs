use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The information about a region.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RegionInfo {
    /// The region ID.
    #[serde(rename = "Region", skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// The public endpoint of the region.
    #[serde(rename = "InternetEndpoint", skip_serializing_if = "Option::is_none")]
    pub internet_endpoint: Option<String>,

    /// The internal endpoint of the region.
    #[serde(rename = "InternalEndpoint", skip_serializing_if = "Option::is_none")]
    pub internal_endpoint: Option<String>,

    /// The acceleration endpoint of the region. The value is always
    /// oss-accelerate.aliyuncs.com.
    #[serde(rename = "AccelerateEndpoint", skip_serializing_if = "Option::is_none")]
    pub accelerate_endpoint: Option<String>,
}

/// The container that stores the information about the regions.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RegionInfoList {
    /// The information about the regions.
    #[serde(rename = "RegionInfo", default)]
    pub region_infos: Vec<RegionInfo>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct DescribeRegionsRequest {
    /// The region ID of the request.
    #[field(type = "query", rename = "regions")]
    pub regions: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DescribeRegionsResult {
    /// The information about the regions.
    pub region_info_list: Option<RegionInfoList>,

    pub common: ResultCommon,
}

impl Client {
    /// Queries the endpoints of all supported regions or the endpoints of a
    /// specific region.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DescribeRegionsRequest` containing the optional
    ///   region ID.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::region::DescribeRegionsRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DescribeRegionsRequest::default();
    ///
    /// match client.describe_regions(&request).await {
    ///     Ok(result) => {
    ///         if let Some(list) = result.region_info_list {
    ///             for info in list.region_infos {
    ///                 println!("region: {:?}", info.region);
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to describe regions: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn describe_regions(
        &self,
        request: &DescribeRegionsRequest,
    ) -> Result<DescribeRegionsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DescribeRegions".to_string(),
            method: http::Method::GET,
            parameters: [("regions", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let region_info_list: RegionInfoList = quick_xml::de::from_str(&data_str)?;

        let mut result = DescribeRegionsResult {
            region_info_list: Some(region_info_list),
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
    fn test_region_info_list_deserialize() {
        let xml = r#"<RegionInfoList>
            <RegionInfo>
                <Region>cn-hangzhou</Region>
                <InternetEndpoint>oss-cn-hangzhou.aliyuncs.com</InternetEndpoint>
                <InternalEndpoint>oss-cn-hangzhou-internal.aliyuncs.com</InternalEndpoint>
                <AccelerateEndpoint>oss-accelerate.aliyuncs.com</AccelerateEndpoint>
            </RegionInfo>
            <RegionInfo>
                <Region>cn-beijing</Region>
                <InternetEndpoint>oss-cn-beijing.aliyuncs.com</InternetEndpoint>
                <InternalEndpoint>oss-cn-beijing-internal.aliyuncs.com</InternalEndpoint>
                <AccelerateEndpoint>oss-accelerate.aliyuncs.com</AccelerateEndpoint>
            </RegionInfo>
        </RegionInfoList>"#;

        let list: RegionInfoList = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(list.region_infos.len(), 2);
        assert_eq!(list.region_infos[0].region.as_deref(), Some("cn-hangzhou"));
        assert_eq!(
            list.region_infos[1].internet_endpoint.as_deref(),
            Some("oss-cn-beijing.aliyuncs.com")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_describe_regions() {
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

        let result = client
            .describe_regions(&DescribeRegionsRequest::default())
            .await;
        assert!(
            result.is_ok(),
            "describe_regions failed: {:?}",
            result.err()
        );
        let list = result.unwrap().region_info_list.unwrap();
        assert!(!list.region_infos.is_empty());
    }
}
