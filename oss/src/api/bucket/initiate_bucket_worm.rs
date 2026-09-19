use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container of the request body.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InitiateWormConfiguration {
    /// The number of days for which objects can be retained.
    #[serde(rename = "RetentionPeriodInDays", skip_serializing_if = "Option::is_none")]
    pub retention_period_in_days: Option<i32>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct InitiateBucketWormRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The container of the request body.
    pub initiate_worm_configuration: InitiateWormConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct InitiateBucketWormResult {
    /// The ID of the retention policy.
    #[field(type = "header", rename = "x-oss-worm-id")]
    pub worm_id: Option<String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Creates a retention policy.
    ///
    /// # Arguments
    ///
    /// * `request` - The `InitiateBucketWormRequest` containing the bucket
    ///   name and the retention configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{InitiateBucketWormRequest, InitiateWormConfiguration};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = InitiateBucketWormRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     initiate_worm_configuration: InitiateWormConfiguration {
    ///         retention_period_in_days: Some(30),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.initiate_bucket_worm(&request).await {
    ///     Ok(result) => {
    ///         println!("Worm ID: {:?}", result.worm_id);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to initiate bucket worm: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn initiate_bucket_worm(
        &self,
        request: &InitiateBucketWormRequest,
    ) -> Result<InitiateBucketWormResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "InitiateBucketWorm".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("worm", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };
        input
            .op_metadata
            .set(SUB_RESOURCE, Rc::new(vec!["worm".to_string()]));

        let xml_body = quick_xml::se::to_string_with_root(
            "InitiateWormConfiguration",
            &request.initiate_worm_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = InitiateBucketWormResult::default();
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
    fn test_initiate_worm_configuration_serde_round_trip() {
        let config = InitiateWormConfiguration {
            retention_period_in_days: Some(30),
        };

        let xml = quick_xml::se::to_string_with_root("InitiateWormConfiguration", &config).unwrap();
        assert!(xml.contains("<InitiateWormConfiguration>"));
        assert!(xml.contains("<RetentionPeriodInDays>30</RetentionPeriodInDays>"));

        let parsed: InitiateWormConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.retention_period_in_days, Some(30));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_initiate_bucket_worm() {
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

        let bucket_name = crate::test_utils::generate_unique_bucket_name("worm-initiate");

        client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .initiate_bucket_worm(&InitiateBucketWormRequest {
                bucket: bucket_name.clone(),
                initiate_worm_configuration: InitiateWormConfiguration {
                    retention_period_in_days: Some(1),
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "initiate_bucket_worm failed: {:?}",
            result.err()
        );
        assert!(result.unwrap().worm_id.is_some());

        // Clean up: abort the unlocked retention policy, then delete the bucket
        let _ = client
            .abort_bucket_worm(&crate::api::bucket::AbortBucketWormRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
