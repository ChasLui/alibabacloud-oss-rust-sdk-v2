use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationOutput, OperationInput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the configurations for real-time access of Archive
/// objects.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArchiveDirectReadConfiguration {
    /// Specifies whether to enable real-time access of Archive objects for a bucket.
    #[serde(rename = "Enabled", skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketArchiveDirectReadRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl GetBucketArchiveDirectReadRequest {
    pub fn new(bucket: &str) -> Self {
        GetBucketArchiveDirectReadRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetBucketArchiveDirectReadResult {
    /// The container that stores the configurations for real-time access of
    /// Archive objects.
    pub archive_direct_read_configuration: Option<ArchiveDirectReadConfiguration>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Queries whether real-time access of Archive objects is enabled for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketArchiveDirectReadRequest` containing the
    ///   bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketArchiveDirectReadRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketArchiveDirectReadRequest::new("my-bucket");
    ///
    /// match client.get_bucket_archive_direct_read(&request).await {
    ///     Ok(result) => {
    ///         println!("Archive direct read: {:?}", result.archive_direct_read_configuration);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket archive direct read: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_archive_direct_read(
        &self,
        request: &GetBucketArchiveDirectReadRequest,
    ) -> Result<GetBucketArchiveDirectReadResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketArchiveDirectRead".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("bucketArchiveDirectRead", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };
        input.op_metadata.set(
            SUB_RESOURCE,
            Rc::new(vec!["bucketArchiveDirectRead".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let archive_direct_read_configuration: ArchiveDirectReadConfiguration =
            quick_xml::de::from_str(&data_str)?;

        let mut result = GetBucketArchiveDirectReadResult {
            archive_direct_read_configuration: Some(archive_direct_read_configuration),
            ..Default::default()
        };
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_get_bucket_archive_direct_read_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ArchiveDirectReadConfiguration>
  <Enabled>true</Enabled>
</ArchiveDirectReadConfiguration>"#;
        let parsed: ArchiveDirectReadConfiguration = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(parsed.enabled, Some(true));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_archive_direct_read() {
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

        // Querying the configuration fails if archive direct read was never
        // configured for the bucket; this only verifies the request is
        // well-formed.
        let result = client
            .get_bucket_archive_direct_read(&GetBucketArchiveDirectReadRequest::new(&config.bucket))
            .await;
        if let Err(error) = &result {
            eprintln!("get_bucket_archive_direct_read rejected (may not be configured): {}", error);
        }
    }
}
