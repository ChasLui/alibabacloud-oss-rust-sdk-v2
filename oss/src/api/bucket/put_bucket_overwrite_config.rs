use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// An overwrite protection rule.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OverwriteRule {
    /// The unique identifier of the rule. If you do not specify this element, a
    /// UUID is randomly generated. If you specify this element, the value must
    /// be unique. Different rules cannot have the same ID.
    #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The operation type. Currently, only `forbid` (prohibit overwrites) is
    /// supported.
    #[serde(rename = "Action", skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,

    /// The prefix of object names to filter the objects that you want to
    /// process. The maximum length is 1,023 characters. Each rule can have at
    /// most one prefix. Prefixes and suffixes do not support regular
    /// expressions.
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// The suffix of object names to filter the objects that you want to
    /// process. The maximum length is 1,023 characters. Each rule can have at
    /// most one suffix. Prefixes and suffixes do not support regular
    /// expressions.
    #[serde(rename = "Suffix", skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,

    /// A collection of authorized entities. The usage is similar to the
    /// `Principal` element in a bucket policy. You can specify an Alibaba Cloud
    /// account, a RAM user, or a RAM role. If this element is empty or not
    /// configured, overwrites are prohibited for all objects that match the
    /// prefix and suffix conditions.
    #[serde(rename = "Principals", skip_serializing_if = "Option::is_none")]
    pub principals: Option<OverwritePrincipals>,
}

/// A collection of authorized entities.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OverwritePrincipals {
    /// A collection of authorized entities. The usage is similar to the
    /// `Principal` element in a bucket policy. You can specify an Alibaba Cloud
    /// account, a RAM user, or a RAM role.
    #[serde(rename = "Principal", default)]
    pub principals: Vec<String>,
}

/// The container that stores the overwrite protection rules.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OverwriteConfiguration {
    /// List of overwrite protection rules. A bucket can have a maximum of 100
    /// rules.
    #[serde(rename = "Rule", default)]
    pub rules: Vec<OverwriteRule>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketOverwriteConfigRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The request body schema.
    pub overwrite_configuration: OverwriteConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketOverwriteConfigResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Configures overwrite protection for a bucket. This prevents specified
    /// objects from being overwritten.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketOverwriteConfigRequest` containing the
    ///   bucket name and the overwrite configuration to set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{OverwriteConfiguration, OverwriteRule, PutBucketOverwriteConfigRequest};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketOverwriteConfigRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     overwrite_configuration: OverwriteConfiguration {
    ///         rules: vec![OverwriteRule {
    ///             action: Some("forbid".to_string()),
    ///             prefix: Some("protected/".to_string()),
    ///             ..Default::default()
    ///         }],
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_overwrite_config(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket overwrite config updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket overwrite config: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_overwrite_config(
        &self,
        request: &PutBucketOverwriteConfigRequest,
    ) -> Result<PutBucketOverwriteConfigResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketOverwriteConfig".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("overwriteConfig", "")]
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
            std::rc::Rc::new(vec!["overwriteConfig".to_string()]),
        );

        let xml_body = quick_xml::se::to_string_with_root(
            "OverwriteConfiguration",
            &request.overwrite_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketOverwriteConfigResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_overwrite_configuration_serde_round_trip() {
        let config = OverwriteConfiguration {
            rules: vec![OverwriteRule {
                id: Some("rule-1".to_string()),
                action: Some("forbid".to_string()),
                prefix: Some("protected/".to_string()),
                suffix: Some(".txt".to_string()),
                principals: Some(OverwritePrincipals {
                    principals: vec!["1234567890".to_string()],
                }),
            }],
        };

        let xml = quick_xml::se::to_string_with_root("OverwriteConfiguration", &config).unwrap();
        assert!(xml.contains("<OverwriteConfiguration>"));
        assert!(xml.contains("<ID>rule-1</ID>"));
        assert!(xml.contains("<Action>forbid</Action>"));
        assert!(xml.contains("<Prefix>protected/</Prefix>"));
        assert!(xml.contains("<Suffix>.txt</Suffix>"));
        assert!(xml.contains("<Principal>1234567890</Principal>"));

        let parsed: OverwriteConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.rules.len(), 1);
        let rule = &parsed.rules[0];
        assert_eq!(rule.id, Some("rule-1".to_string()));
        assert_eq!(rule.action, Some("forbid".to_string()));
        assert_eq!(rule.prefix, Some("protected/".to_string()));
        assert_eq!(rule.suffix, Some(".txt".to_string()));
        assert_eq!(
            rule.principals.as_ref().unwrap().principals,
            vec!["1234567890".to_string()]
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_overwrite_config() {
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

        let bucket_name = generate_unique_bucket_name("put-bucket-overwrite-config");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .put_bucket_overwrite_config(&PutBucketOverwriteConfigRequest {
                bucket: bucket_name.clone(),
                overwrite_configuration: OverwriteConfiguration {
                    rules: vec![OverwriteRule {
                        action: Some("forbid".to_string()),
                        prefix: Some("protected/".to_string()),
                        ..Default::default()
                    }],
                },
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "put_bucket_overwrite_config failed: {:?}", result.err());

        // Clean up
        let _ = client
            .delete_bucket(&DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
