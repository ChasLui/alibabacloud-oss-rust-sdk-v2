use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::get_bucket_access_monitor::AccessMonitorConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketAccessMonitorRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The request body schema.
    pub access_monitor_configuration: AccessMonitorConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketAccessMonitorResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Modifies the access tracking status of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketAccessMonitorRequest` containing the bucket
    ///   name and the access monitor configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{AccessMonitorConfiguration, PutBucketAccessMonitorRequest};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketAccessMonitorRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     access_monitor_configuration: AccessMonitorConfiguration {
    ///         status: Some("Enabled".to_string()),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_access_monitor(&request).await {
    ///     Ok(result) => {
    ///         println!("Access monitor configured: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket access monitor: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_access_monitor(
        &self,
        request: &PutBucketAccessMonitorRequest,
    ) -> Result<PutBucketAccessMonitorResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketAccessMonitor".to_string(),
            method: http::Method::PUT,
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

        let xml_body = quick_xml::se::to_string_with_root(
            "AccessMonitorConfiguration",
            &request.access_monitor_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketAccessMonitorResult::default();
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
    fn test_access_monitor_configuration_serde() {
        let configuration = AccessMonitorConfiguration {
            status: Some("Enabled".to_string()),
        };
        let xml =
            quick_xml::se::to_string_with_root("AccessMonitorConfiguration", &configuration)
                .unwrap();
        assert!(xml.contains("<AccessMonitorConfiguration>"));
        assert!(xml.contains("<Status>Enabled</Status>"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_access_monitor() {
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
            .put_bucket_access_monitor(&PutBucketAccessMonitorRequest {
                bucket: config.bucket.clone(),
                access_monitor_configuration: AccessMonitorConfiguration {
                    status: Some("Enabled".to_string()),
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "put_bucket_access_monitor failed: {:?}",
            result.err()
        );

        let get_result = client
            .get_bucket_access_monitor(&crate::api::bucket::GetBucketAccessMonitorRequest::new(
                &config.bucket,
            ))
            .await;
        assert!(
            get_result.is_ok(),
            "get_bucket_access_monitor failed: {:?}",
            get_result.err()
        );
        assert_eq!(
            get_result
                .unwrap()
                .access_monitor_configuration
                .and_then(|c| c.status)
                .as_deref(),
            Some("Enabled")
        );

        // Restore the default status
        let _ = client
            .put_bucket_access_monitor(&PutBucketAccessMonitorRequest {
                bucket: config.bucket.clone(),
                access_monitor_configuration: AccessMonitorConfiguration {
                    status: Some("Disabled".to_string()),
                },
                ..Default::default()
            })
            .await;
    }
}
