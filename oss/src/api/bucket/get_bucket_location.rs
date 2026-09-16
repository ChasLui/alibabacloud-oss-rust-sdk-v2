use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketLocationRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl GetBucketLocationRequest {
    pub fn new(bucket: &str) -> Self {
        Self {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetBucketLocationResult {
    /// The region in which the bucket is located.
    /// The response body is the text node of the LocationConstraint element
    /// (Go: `xml:",chardata"`).
    #[serde(rename = "$value", skip_serializing_if = "Option::is_none")]
    pub location_constraint: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the region of an Object Storage Service (OSS) bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketLocationRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketLocationRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketLocationRequest::new("my-bucket");
    ///
    /// match client.get_bucket_location(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket location: {:?}", result.location_constraint);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket location: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_location(
        &self,
        request: &GetBucketLocationRequest,
    ) -> Result<GetBucketLocationResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketLocation".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("location", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
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
        let mut result: GetBucketLocationResult = quick_xml::de::from_str(&data_str)?;

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
    fn test_location_chardata_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<LocationConstraint>oss-cn-hangzhou</LocationConstraint>"#;
        let result: GetBucketLocationResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(
            result.location_constraint.as_deref(),
            Some("oss-cn-hangzhou")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_location() {
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
            .get_bucket_location(&GetBucketLocationRequest::new(&config.bucket))
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_location failed: {:?}",
            result.err()
        );
        let result = result.unwrap();
        assert_eq!(result.common.status, http::StatusCode::OK);
        let location = result.location_constraint.unwrap_or_default();
        // LocationConstraint uses the "oss-<region>" form, e.g. "oss-cn-hangzhou"
        assert!(
            location.contains(&config.region),
            "unexpected location: {}",
            location
        );
    }
}
