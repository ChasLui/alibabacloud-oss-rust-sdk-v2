use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketVersioningRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl GetBucketVersioningRequest {
    pub fn new(bucket: &str) -> Self {
        Self {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetBucketVersioningResult {
    /// The versioning state of the bucket. Valid values: Enabled, Suspended.
    /// Absent when versioning has never been configured for the bucket.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub version_status: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the versioning state of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketVersioningRequest` containing the bucket
    ///   name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketVersioningRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketVersioningRequest::new("my-bucket");
    ///
    /// match client.get_bucket_versioning(&request).await {
    ///     Ok(result) => {
    ///         println!("Versioning status: {:?}", result.version_status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket versioning: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_versioning(
        &self,
        request: &GetBucketVersioningRequest,
    ) -> Result<GetBucketVersioningResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketVersioning".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("versioning", "")]
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
        let mut result: GetBucketVersioningResult = quick_xml::de::from_str(&data_str)?;

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
    fn test_get_bucket_versioning_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<VersioningConfiguration>
  <Status>Enabled</Status>
</VersioningConfiguration>"#;
        let result: GetBucketVersioningResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.version_status.as_deref(), Some("Enabled"));

        // Status is absent when versioning has never been configured
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<VersioningConfiguration/>"#;
        let result: GetBucketVersioningResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.version_status, None);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_versioning() {
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
            .get_bucket_versioning(&GetBucketVersioningRequest::new(&config.bucket))
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_versioning failed: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().common.status, http::StatusCode::OK);
    }
}
