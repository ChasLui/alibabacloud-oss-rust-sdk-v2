use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

/// The container of the request body.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExtendWormConfiguration {
    /// The number of days for which objects can be retained.
    #[serde(rename = "RetentionPeriodInDays", skip_serializing_if = "Option::is_none")]
    pub retention_period_in_days: Option<i32>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct ExtendBucketWormRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The ID of the retention policy. If the ID of the retention policy that
    /// specifies the number of days for which objects can be retained does not
    /// exist, the HTTP status code 404 is returned.
    #[field(type = "query", rename = "wormId")]
    pub worm_id: Option<String>,

    /// The container of the request body.
    pub extend_worm_configuration: ExtendWormConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct ExtendBucketWormResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Extends the retention period of objects in a bucket for which a
    /// retention policy is locked.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ExtendBucketWormRequest` containing the bucket name,
    ///   the ID of the retention policy and the extended retention period.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{ExtendBucketWormRequest, ExtendWormConfiguration};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ExtendBucketWormRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     worm_id: Some("worm-id".to_string()),
    ///     extend_worm_configuration: ExtendWormConfiguration {
    ///         retention_period_in_days: Some(60),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.extend_bucket_worm(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket worm extended: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to extend bucket worm: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn extend_bucket_worm(
        &self,
        request: &ExtendBucketWormRequest,
    ) -> Result<ExtendBucketWormResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ExtendBucketWorm".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("wormExtend", "")]
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
            Rc::new(vec!["wormExtend".to_string(), "wormId".to_string()]),
        );

        let xml_body = quick_xml::se::to_string_with_root(
            "ExtendWormConfiguration",
            &request.extend_worm_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = ExtendBucketWormResult::default();
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
    fn test_extend_worm_configuration_serde_round_trip() {
        let config = ExtendWormConfiguration {
            retention_period_in_days: Some(60),
        };

        let xml = quick_xml::se::to_string_with_root("ExtendWormConfiguration", &config).unwrap();
        assert!(xml.contains("<ExtendWormConfiguration>"));
        assert!(xml.contains("<RetentionPeriodInDays>60</RetentionPeriodInDays>"));

        let parsed: ExtendWormConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.retention_period_in_days, Some(60));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_extend_bucket_worm() {
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

        let bucket_name = crate::test_utils::generate_unique_bucket_name("worm-extend");

        client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let worm_id = client
            .initiate_bucket_worm(&crate::api::bucket::InitiateBucketWormRequest {
                bucket: bucket_name.clone(),
                initiate_worm_configuration:
                    crate::api::bucket::InitiateWormConfiguration {
                        retention_period_in_days: Some(1),
                    },
                ..Default::default()
            })
            .await
            .unwrap()
            .worm_id
            .unwrap();

        client
            .complete_bucket_worm(&crate::api::bucket::CompleteBucketWormRequest {
                bucket: bucket_name.clone(),
                worm_id: Some(worm_id.clone()),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .extend_bucket_worm(&ExtendBucketWormRequest {
                bucket: bucket_name.clone(),
                worm_id: Some(worm_id),
                extend_worm_configuration: ExtendWormConfiguration {
                    retention_period_in_days: Some(2),
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "extend_bucket_worm failed: {:?}",
            result.err()
        );

        // Clean up: the empty bucket can be deleted even with a locked policy
        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
