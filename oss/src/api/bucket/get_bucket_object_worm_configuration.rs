use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores object worm config.
///
/// This type is shared by the put and get bucket object worm configuration
/// operations.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ObjectWormConfiguration {
    /// Whether to enable object-level retention policy.
    #[serde(rename = "ObjectWormEnabled", skip_serializing_if = "Option::is_none")]
    pub object_worm_enabled: Option<String>,

    /// Container with object-level retention policy.
    #[serde(rename = "Rule", skip_serializing_if = "Option::is_none")]
    pub rule: Option<ObjectWormRule>,
}

/// Container with object-level retention policy.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ObjectWormRule {
    /// The default retention configuration.
    #[serde(rename = "DefaultRetention", skip_serializing_if = "Option::is_none")]
    pub default_retention: Option<ObjectWormDefaultRetention>,
}

/// The default retention configuration.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ObjectWormDefaultRetention {
    /// Object-level retention strategy pattern. valid value: GOVERNANCE,
    /// COMPLIANCE.
    #[serde(rename = "Mode", skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,

    /// Object-level retention policy days (max 36500).
    #[serde(rename = "Days", skip_serializing_if = "Option::is_none")]
    pub days: Option<i32>,

    /// Bucket object level retention policy years (max 100).
    #[serde(rename = "Years", skip_serializing_if = "Option::is_none")]
    pub years: Option<i32>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketObjectWormConfigurationRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl GetBucketObjectWormConfigurationRequest {
    pub fn new(bucket: &str) -> Self {
        GetBucketObjectWormConfigurationRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetBucketObjectWormConfigurationResult {
    /// The container that stores object worm config.
    pub object_worm_configuration: Option<ObjectWormConfiguration>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Queries the object-level retention policy of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketObjectWormConfigurationRequest` containing
    ///   the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketObjectWormConfigurationRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketObjectWormConfigurationRequest::new("my-bucket");
    ///
    /// match client.get_bucket_object_worm_configuration(&request).await {
    ///     Ok(result) => {
    ///         println!("Object worm config: {:?}", result.object_worm_configuration);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket object worm configuration: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_object_worm_configuration(
        &self,
        request: &GetBucketObjectWormConfigurationRequest,
    ) -> Result<GetBucketObjectWormConfigurationResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketObjectWormConfiguration".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("objectWorm", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };
        input
            .op_metadata
            .set(SUB_RESOURCE, Rc::new(vec!["objectWorm".to_string()]));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let object_worm_configuration: ObjectWormConfiguration =
            quick_xml::de::from_str(&data_str)?;

        let mut result = GetBucketObjectWormConfigurationResult {
            object_worm_configuration: Some(object_worm_configuration),
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
    fn test_object_worm_configuration_serde_round_trip() {
        let config = ObjectWormConfiguration {
            object_worm_enabled: Some("Enabled".to_string()),
            rule: Some(ObjectWormRule {
                default_retention: Some(ObjectWormDefaultRetention {
                    mode: Some("COMPLIANCE".to_string()),
                    days: Some(10),
                    years: None,
                }),
            }),
        };

        let xml = quick_xml::se::to_string_with_root("ObjectWormConfiguration", &config).unwrap();
        assert!(xml.contains("<ObjectWormConfiguration>"));
        assert!(xml.contains("<ObjectWormEnabled>Enabled</ObjectWormEnabled>"));
        assert!(xml.contains("<Mode>COMPLIANCE</Mode>"));
        assert!(xml.contains("<Days>10</Days>"));

        let parsed: ObjectWormConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.object_worm_enabled.as_deref(), Some("Enabled"));
        let retention = parsed
            .rule
            .and_then(|r| r.default_retention)
            .expect("default retention missing");
        assert_eq!(retention.mode.as_deref(), Some("COMPLIANCE"));
        assert_eq!(retention.days, Some(10));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_object_worm_configuration() {
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

        let bucket_name = crate::test_utils::generate_unique_bucket_name("objectworm-get");

        client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        client
            .put_bucket_object_worm_configuration(
                &crate::api::bucket::PutBucketObjectWormConfigurationRequest {
                    bucket: bucket_name.clone(),
                    object_worm_configuration: ObjectWormConfiguration {
                        object_worm_enabled: Some("Enabled".to_string()),
                        rule: Some(ObjectWormRule {
                            default_retention: Some(ObjectWormDefaultRetention {
                                mode: Some("COMPLIANCE".to_string()),
                                days: Some(10),
                                years: None,
                            }),
                        }),
                    },
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let result = client
            .get_bucket_object_worm_configuration(
                &GetBucketObjectWormConfigurationRequest::new(&bucket_name),
            )
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_object_worm_configuration failed: {:?}",
            result.err()
        );
        assert!(result.unwrap().object_worm_configuration.is_some());

        // Clean up
        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
