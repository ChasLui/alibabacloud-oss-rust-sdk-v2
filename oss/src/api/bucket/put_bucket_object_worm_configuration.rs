use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::get_bucket_object_worm_configuration::{
    ObjectWormConfiguration,
};
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketObjectWormConfigurationRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The request body schema.
    pub object_worm_configuration: ObjectWormConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketObjectWormConfigurationResult {
    /// Common result fields
    pub common: ResultCommon,
}

/// Validates the object worm configuration, mirroring the Go SDK's
/// checkObjectWormConfiguration.
fn check_object_worm_configuration(
    configuration: &ObjectWormConfiguration,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(rule) = &configuration.rule {
        if let Some(default_retention) = &rule.default_retention {
            if default_retention.days.is_none() && default_retention.years.is_none() {
                return Err(
                    "either DefaultRetention.Days or DefaultRetention.Years must be configured"
                        .into(),
                );
            }
            if let Some(days) = default_retention.days {
                if days <= 0 {
                    return Err("DefaultRetention.Days must be greater than 0".into());
                }
            }
            if let Some(years) = default_retention.years {
                if years <= 0 {
                    return Err("DefaultRetention.Years must be greater than 0".into());
                }
            }
        }
    }
    Ok(())
}

impl Client {
    /// Enable object retention on the bucket and configure a retention policy.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketObjectWormConfigurationRequest` containing
    ///   the bucket name and the object-level retention configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    /// #     ObjectWormConfiguration, ObjectWormDefaultRetention, ObjectWormRule,
    /// #     PutBucketObjectWormConfigurationRequest,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketObjectWormConfigurationRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     object_worm_configuration: ObjectWormConfiguration {
    ///         object_worm_enabled: Some("Enabled".to_string()),
    ///         rule: Some(ObjectWormRule {
    ///             default_retention: Some(ObjectWormDefaultRetention {
    ///                 mode: Some("COMPLIANCE".to_string()),
    ///                 days: Some(10),
    ///                 years: None,
    ///             }),
    ///         }),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_object_worm_configuration(&request).await {
    ///     Ok(result) => {
    ///         println!("Object worm configuration set: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket object worm configuration: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_object_worm_configuration(
        &self,
        request: &PutBucketObjectWormConfigurationRequest,
    ) -> Result<PutBucketObjectWormConfigurationResult, Box<dyn std::error::Error + Send + Sync>> {
        check_object_worm_configuration(&request.object_worm_configuration)?;

        let mut input = OperationInput {
            op_name: "PutBucketObjectWormConfiguration".to_string(),
            method: http::Method::PUT,
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

        let xml_body = quick_xml::se::to_string_with_root(
            "ObjectWormConfiguration",
            &request.object_worm_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketObjectWormConfigurationResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::get_bucket_object_worm_configuration::{
        ObjectWormDefaultRetention, ObjectWormRule,
    };
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::SignatureVersionType;
    use crate::test_utils::load_test_config;

    #[test]
    fn test_check_object_worm_configuration() {
        // No rule: valid
        let config = ObjectWormConfiguration {
            object_worm_enabled: Some("Enabled".to_string()),
            rule: None,
        };
        assert!(check_object_worm_configuration(&config).is_ok());

        // Neither days nor years: invalid
        let config = ObjectWormConfiguration {
            object_worm_enabled: Some("Enabled".to_string()),
            rule: Some(ObjectWormRule {
                default_retention: Some(ObjectWormDefaultRetention {
                    mode: Some("COMPLIANCE".to_string()),
                    days: None,
                    years: None,
                }),
            }),
        };
        assert!(check_object_worm_configuration(&config).is_err());

        // Non-positive days: invalid
        let config = ObjectWormConfiguration {
            object_worm_enabled: Some("Enabled".to_string()),
            rule: Some(ObjectWormRule {
                default_retention: Some(ObjectWormDefaultRetention {
                    mode: Some("COMPLIANCE".to_string()),
                    days: Some(0),
                    years: None,
                }),
            }),
        };
        assert!(check_object_worm_configuration(&config).is_err());

        // Positive days: valid
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
        assert!(check_object_worm_configuration(&config).is_ok());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_object_worm_configuration() {
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

        let bucket_name = crate::test_utils::generate_unique_bucket_name("objectworm-put");

        client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .put_bucket_object_worm_configuration(
                &PutBucketObjectWormConfigurationRequest {
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
            .await;
        assert!(
            result.is_ok(),
            "put_bucket_object_worm_configuration failed: {:?}",
            result.err()
        );

        // Clean up
        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
