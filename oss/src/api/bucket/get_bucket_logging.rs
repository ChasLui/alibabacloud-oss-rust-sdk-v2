use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::put_bucket_logging::LoggingEnabled;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketLoggingRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "BucketLoggingStatus")]
pub struct GetBucketLoggingResult {
    /// Indicates the container used to store access logging configuration of a
    /// bucket.
    #[serde(rename = "LoggingEnabled", skip_serializing_if = "Option::is_none")]
    pub logging_enabled: Option<LoggingEnabled>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the configurations of access log collection of a bucket. Only
    /// the owner of a bucket can query the configurations of access log
    /// collection of the bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketLoggingRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketLoggingRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketLoggingRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_logging(&request).await {
    ///     Ok(result) => {
    ///         println!("Logging configuration: {:?}", result.logging_enabled);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket logging: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_logging(
        &self,
        request: &GetBucketLoggingRequest,
    ) -> Result<GetBucketLoggingResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketLogging".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("logging", "")]
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
            std::rc::Rc::new(vec!["logging".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        // Parse the XML response
        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetBucketLoggingResult = quick_xml::de::from_str(&data_str)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::put_bucket_logging::{
        BucketLoggingStatus, LoggingEnabled, PutBucketLoggingRequest,
    };
    use super::*;
    use crate::api::bucket::{
        CreateBucketRequest, DeleteBucketLoggingRequest, DeleteBucketRequest,
    };
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_get_bucket_logging_result_deserialize() {
        let xml = r#"<BucketLoggingStatus><LoggingEnabled><TargetBucket>log-bucket</TargetBucket><TargetPrefix>logs/</TargetPrefix></LoggingEnabled></BucketLoggingStatus>"#;
        let result: GetBucketLoggingResult = quick_xml::de::from_str(xml).unwrap();
        let enabled = result.logging_enabled.unwrap();
        assert_eq!(enabled.target_bucket.as_deref(), Some("log-bucket"));
        assert_eq!(enabled.target_prefix.as_deref(), Some("logs/"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_logging() {
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

        let bucket_name = generate_unique_bucket_name("get-bucket-logging");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        client
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
            .await
            .unwrap();

        let result = client
            .get_bucket_logging(&GetBucketLoggingRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_logging failed: {:?}",
            result.err()
        );
        assert_eq!(
            result
                .unwrap()
                .logging_enabled
                .unwrap()
                .target_bucket
                .as_deref(),
            Some(config.bucket.as_str())
        );

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
