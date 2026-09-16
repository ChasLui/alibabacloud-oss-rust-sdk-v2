use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::get_bucket_archive_direct_read::ArchiveDirectReadConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketArchiveDirectReadRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The request body.
    pub archive_direct_read_configuration: ArchiveDirectReadConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketArchiveDirectReadResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Enables or disables real-time access of Archive objects for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketArchiveDirectReadRequest` containing the
    ///   bucket name and the archive direct read configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{ArchiveDirectReadConfiguration, PutBucketArchiveDirectReadRequest};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketArchiveDirectReadRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     archive_direct_read_configuration: ArchiveDirectReadConfiguration {
    ///         enabled: Some(true),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_archive_direct_read(&request).await {
    ///     Ok(result) => {
    ///         println!("Archive direct read configured: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket archive direct read: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_archive_direct_read(
        &self,
        request: &PutBucketArchiveDirectReadRequest,
    ) -> Result<PutBucketArchiveDirectReadResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketArchiveDirectRead".to_string(),
            method: http::Method::PUT,
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

        let xml_body = quick_xml::se::to_string_with_root(
            "ArchiveDirectReadConfiguration",
            &request.archive_direct_read_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketArchiveDirectReadResult::default();
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
    fn test_archive_direct_read_configuration_serde() {
        let configuration = ArchiveDirectReadConfiguration { enabled: Some(true) };
        let xml = quick_xml::se::to_string_with_root(
            "ArchiveDirectReadConfiguration",
            &configuration,
        )
        .unwrap();
        assert!(xml.contains("<ArchiveDirectReadConfiguration>"));
        assert!(xml.contains("<Enabled>true</Enabled>"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_archive_direct_read() {
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
            .put_bucket_archive_direct_read(&PutBucketArchiveDirectReadRequest {
                bucket: config.bucket.clone(),
                archive_direct_read_configuration: ArchiveDirectReadConfiguration {
                    enabled: Some(true),
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "put_bucket_archive_direct_read failed: {:?}",
            result.err()
        );

        let get_result = client
            .get_bucket_archive_direct_read(
                &crate::api::bucket::GetBucketArchiveDirectReadRequest::new(&config.bucket),
            )
            .await;
        assert!(
            get_result.is_ok(),
            "get_bucket_archive_direct_read failed: {:?}",
            get_result.err()
        );
        assert_eq!(
            get_result
                .unwrap()
                .archive_direct_read_configuration
                .and_then(|c| c.enabled),
            Some(true)
        );

        // Restore the default configuration
        let _ = client
            .put_bucket_archive_direct_read(&PutBucketArchiveDirectReadRequest {
                bucket: config.bucket.clone(),
                archive_direct_read_configuration: ArchiveDirectReadConfiguration {
                    enabled: Some(false),
                },
                ..Default::default()
            })
            .await;
    }
}
