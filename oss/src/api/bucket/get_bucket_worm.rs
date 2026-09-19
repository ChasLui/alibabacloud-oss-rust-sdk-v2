use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the information about retention policies of the
/// bucket.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WormConfiguration {
    /// The ID of the retention policy. Note: If the specified retention policy
    /// ID that is used to query the retention policy configurations of the
    /// bucket does not exist, OSS returns the 404 error code.
    #[serde(rename = "WormId", skip_serializing_if = "Option::is_none")]
    pub worm_id: Option<String>,

    /// The status of the retention policy. Valid values:
    /// - InProgress: indicates that the retention policy is in the InProgress
    ///   state. By default, a retention policy is in the InProgress state
    ///   after it is created. The policy remains in this state for 24 hours.
    /// - Locked: indicates that the retention policy is in the Locked state.
    #[serde(rename = "State", skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// The number of days for which objects can be retained.
    #[serde(rename = "RetentionPeriodInDays", skip_serializing_if = "Option::is_none")]
    pub retention_period_in_days: Option<i32>,

    /// The time at which the retention policy was created.
    #[serde(rename = "CreationDate", skip_serializing_if = "Option::is_none")]
    pub creation_date: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketWormRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl GetBucketWormRequest {
    pub fn new(bucket: &str) -> Self {
        GetBucketWormRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetBucketWormResult {
    /// The container that stores the information about retention policies of
    /// the bucket.
    pub worm_configuration: Option<WormConfiguration>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Queries the retention policy configured for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketWormRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketWormRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketWormRequest::new("my-bucket");
    ///
    /// match client.get_bucket_worm(&request).await {
    ///     Ok(result) => {
    ///         println!("Worm configuration: {:?}", result.worm_configuration);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket worm: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_worm(
        &self,
        request: &GetBucketWormRequest,
    ) -> Result<GetBucketWormResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketWorm".to_string(),
            method: http::Method::GET,
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

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let worm_configuration: WormConfiguration = quick_xml::de::from_str(&data_str)?;

        let mut result = GetBucketWormResult {
            worm_configuration: Some(worm_configuration),
            ..Default::default()
        };
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
    fn test_worm_configuration_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<WormConfiguration>
  <WormId>1666E2CFB2B****</WormId>
  <State>Locked</State>
  <RetentionPeriodInDays>1</RetentionPeriodInDays>
  <CreationDate>2021-08-10T02:26:25.000Z</CreationDate>
</WormConfiguration>"#;
        let config: WormConfiguration = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(config.worm_id.as_deref(), Some("1666E2CFB2B****"));
        assert_eq!(config.state.as_deref(), Some("Locked"));
        assert_eq!(config.retention_period_in_days, Some(1));
        assert_eq!(
            config.creation_date.as_deref(),
            Some("2021-08-10T02:26:25.000Z")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_worm() {
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

        let bucket_name = crate::test_utils::generate_unique_bucket_name("worm-get");

        client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        client
            .initiate_bucket_worm(&crate::api::bucket::InitiateBucketWormRequest {
                bucket: bucket_name.clone(),
                initiate_worm_configuration:
                    crate::api::bucket::InitiateWormConfiguration {
                        retention_period_in_days: Some(1),
                    },
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .get_bucket_worm(&GetBucketWormRequest::new(&bucket_name))
            .await;
        assert!(result.is_ok(), "get_bucket_worm failed: {:?}", result.err());
        let result = result.unwrap();
        assert!(result.worm_configuration.is_some());
        let worm = result.worm_configuration.unwrap();
        assert!(worm.worm_id.is_some());
        assert_eq!(worm.retention_period_in_days, Some(1));

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
