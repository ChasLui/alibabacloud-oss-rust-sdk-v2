use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::get_bucket_https_config::HttpsConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketHttpsConfigRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The container that stores HTTPS configurations.
    pub https_configuration: HttpsConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketHttpsConfigResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Enables or disables Transport Layer Security (TLS) version management
    /// for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketHttpsConfigRequest` containing the bucket
    ///   name and the HTTPS configurations to set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{HttpsConfiguration, PutBucketHttpsConfigRequest, Tls};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketHttpsConfigRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     https_configuration: HttpsConfiguration {
    ///         tls: Some(Tls {
    ///             enable: Some(true),
    ///             tls_versions: vec!["TLSv1.2".to_string(), "TLSv1.3".to_string()],
    ///         }),
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_https_config(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket HTTPS config updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket HTTPS config: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_https_config(
        &self,
        request: &PutBucketHttpsConfigRequest,
    ) -> Result<PutBucketHttpsConfigResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketHttpsConfig".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("httpsConfig", "")]
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
            std::rc::Rc::new(vec!["httpsConfig".to_string()]),
        );

        let xml_body =
            quick_xml::se::to_string_with_root("HttpsConfiguration", &request.https_configuration)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketHttpsConfigResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::Tls;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_https_config() {
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
            .put_bucket_https_config(&PutBucketHttpsConfigRequest {
                bucket: config.bucket.clone(),
                https_configuration: HttpsConfiguration {
                    tls: Some(Tls {
                        enable: Some(true),
                        tls_versions: vec!["TLSv1.2".to_string(), "TLSv1.3".to_string()],
                    }),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "put_bucket_https_config failed: {:?}",
            result.err()
        );

        // Clean up: disable TLS version management
        let _ = client
            .put_bucket_https_config(&PutBucketHttpsConfigRequest {
                bucket: config.bucket.clone(),
                https_configuration: HttpsConfiguration {
                    tls: Some(Tls {
                        enable: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;
    }
}
