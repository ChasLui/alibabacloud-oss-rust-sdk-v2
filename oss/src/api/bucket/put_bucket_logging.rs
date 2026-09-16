use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

/// The container used to store access logging information.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LoggingEnabled {
    /// The bucket that stores access logs.
    #[serde(rename = "TargetBucket", skip_serializing_if = "Option::is_none")]
    pub target_bucket: Option<String>,

    /// The prefix of the log objects. This parameter can be left empty.
    #[serde(rename = "TargetPrefix", skip_serializing_if = "Option::is_none")]
    pub target_prefix: Option<String>,

    /// Log transfer authorization role.
    #[serde(rename = "LoggingRole", skip_serializing_if = "Option::is_none")]
    pub logging_role: Option<String>,
}

/// The container that stores the access logging configuration of a bucket.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BucketLoggingStatus {
    /// Indicates the container used to store access logging information. This
    /// element is returned if it is enabled and is not returned if it is disabled.
    #[serde(rename = "LoggingEnabled", skip_serializing_if = "Option::is_none")]
    pub logging_enabled: Option<LoggingEnabled>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketLoggingRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The request body schema.
    pub bucket_logging_status: BucketLoggingStatus,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketLoggingResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Enables logging for a bucket. After you enable logging for a bucket,
    /// Object Storage Service (OSS) generates logs every hour based on the
    /// defined naming rule and stores the logs as objects in the specified
    /// destination bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketLoggingRequest` containing the bucket name
    ///   and the logging configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    /// #     BucketLoggingStatus, LoggingEnabled, PutBucketLoggingRequest,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketLoggingRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     bucket_logging_status: BucketLoggingStatus {
    ///         logging_enabled: Some(LoggingEnabled {
    ///             target_bucket: Some("my-log-bucket".to_string()),
    ///             target_prefix: Some("logs/".to_string()),
    ///             ..Default::default()
    ///         }),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_logging(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket logging updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket logging: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_logging(
        &self,
        request: &PutBucketLoggingRequest,
    ) -> Result<PutBucketLoggingResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketLogging".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("logging", "")]
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
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["logging".to_string()]),
        );

        let xml_body =
            quick_xml::se::to_string_with_root("BucketLoggingStatus", &request.bucket_logging_status)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketLoggingResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketLoggingRequest, DeleteBucketRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_bucket_logging_status_serde_round_trip() {
        let status = BucketLoggingStatus {
            logging_enabled: Some(LoggingEnabled {
                target_bucket: Some("log-bucket".to_string()),
                target_prefix: Some("logs/".to_string()),
                logging_role: None,
            }),
        };

        let xml = quick_xml::se::to_string_with_root("BucketLoggingStatus", &status).unwrap();
        assert!(xml.contains("<BucketLoggingStatus>"));
        assert!(xml.contains("<TargetBucket>log-bucket</TargetBucket>"));
        assert!(xml.contains("<TargetPrefix>logs/</TargetPrefix>"));

        let parsed: BucketLoggingStatus = quick_xml::de::from_str(&xml).unwrap();
        let enabled = parsed.logging_enabled.unwrap();
        assert_eq!(enabled.target_bucket.as_deref(), Some("log-bucket"));
        assert_eq!(enabled.target_prefix.as_deref(), Some("logs/"));
        // None fields are skipped during serialization and read back as None
        assert_eq!(enabled.logging_role, None);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_logging() {
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

        let bucket_name = generate_unique_bucket_name("put-bucket-logging");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .put_bucket_logging(&PutBucketLoggingRequest {
                bucket: bucket_name.clone(),
                bucket_logging_status: BucketLoggingStatus {
                    logging_enabled: Some(LoggingEnabled {
                        target_bucket: Some(config.bucket.clone()),
                        target_prefix: Some("logging-test/".to_string()),
                        ..Default::default()
                    }),
                },
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "put_bucket_logging failed: {:?}", result.err());

        // Clean up
        let _ = client
            .delete_bucket_logging(&DeleteBucketLoggingRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        let _ = client
            .delete_bucket(&DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
