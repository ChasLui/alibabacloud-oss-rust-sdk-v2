use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};
/// The container that stores the access monitor configuration.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AccessMonitorConfiguration {
    /// The access tracking status of the bucket.
    /// Valid values: Enabled, Disabled.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketAccessMonitorRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl GetBucketAccessMonitorRequest {
    pub fn new(bucket: &str) -> Self {
        GetBucketAccessMonitorRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetBucketAccessMonitorResult {
    /// The container that stores the access monitor configuration.
    pub access_monitor_configuration: Option<AccessMonitorConfiguration>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Queries the access tracking status of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketAccessMonitorRequest` containing the bucket
    ///   name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketAccessMonitorRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketAccessMonitorRequest::new("my-bucket");
    ///
    /// match client.get_bucket_access_monitor(&request).await {
    ///     Ok(result) => {
    ///         println!("Access monitor: {:?}", result.access_monitor_configuration);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket access monitor: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_access_monitor(
        &self,
        request: &GetBucketAccessMonitorRequest,
    ) -> Result<GetBucketAccessMonitorResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketAccessMonitor".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("accessmonitor", "")]
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
            vec![update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let access_monitor_configuration: AccessMonitorConfiguration =
            quick_xml::de::from_str(&data_str)?;

        let mut result = GetBucketAccessMonitorResult {
            access_monitor_configuration: Some(access_monitor_configuration),
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
    fn test_get_bucket_access_monitor_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AccessMonitorConfiguration>
  <Status>Enabled</Status>
</AccessMonitorConfiguration>"#;
        let parsed: AccessMonitorConfiguration = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(parsed.status.as_deref(), Some("Enabled"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_access_monitor() {
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
            .get_bucket_access_monitor(&GetBucketAccessMonitorRequest::new(&config.bucket))
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_access_monitor failed: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().common.status, http::StatusCode::OK);
    }
}
