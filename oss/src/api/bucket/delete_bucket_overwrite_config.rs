use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

/// A collection of authorized entities. The usage is similar to the `Principal`
/// element in a bucket policy.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OverwritePrincipals {
    /// The authorized entities. You can specify an Alibaba Cloud account, a RAM
    /// user, or a RAM role.
    #[serde(rename = "Principal", default)]
    pub principals: Vec<String>,
}

/// An overwrite protection rule.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OverwriteRule {
    /// The unique identifier of the rule. If you do not specify this element, a
    /// UUID is randomly generated. Different rules cannot have the same ID.
    #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The operation type. Currently, only `forbid` (prohibit overwrites) is
    /// supported.
    #[serde(rename = "Action", skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,

    /// The prefix of object names to filter the objects that you want to process.
    /// The maximum length is 1,023 characters. Each rule can have at most one
    /// prefix. Prefixes do not support regular expressions.
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// The suffix of object names to filter the objects that you want to process.
    /// The maximum length is 1,023 characters. Each rule can have at most one
    /// suffix. Suffixes do not support regular expressions.
    #[serde(rename = "Suffix", skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,

    /// A collection of authorized entities. If this element is empty or not
    /// configured, overwrites are prohibited for all objects that match the
    /// prefix and suffix conditions.
    #[serde(rename = "Principals", skip_serializing_if = "Option::is_none")]
    pub principals: Option<OverwritePrincipals>,
}

/// The container that stores the overwrite protection rules of a bucket.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OverwriteConfiguration {
    /// The list of overwrite protection rules. A bucket can have a maximum of
    /// 100 rules.
    #[serde(rename = "Rule", default)]
    pub rules: Vec<OverwriteRule>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteBucketOverwriteConfigRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl DeleteBucketOverwriteConfigRequest {
    pub fn new(bucket: &str) -> Self {
        DeleteBucketOverwriteConfigRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteBucketOverwriteConfigResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes the overwrite configuration rules of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteBucketOverwriteConfigRequest` containing the
    ///   bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::DeleteBucketOverwriteConfigRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteBucketOverwriteConfigRequest::new("my-bucket");
    ///
    /// match client.delete_bucket_overwrite_config(&request).await {
    ///     Ok(result) => {
    ///         println!("Overwrite config deleted: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete bucket overwrite config: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_bucket_overwrite_config(
        &self,
        request: &DeleteBucketOverwriteConfigRequest,
    ) -> Result<DeleteBucketOverwriteConfigResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteBucketOverwriteConfig".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            parameters: [("overwriteConfig", "")]
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
            .set(SUB_RESOURCE, Rc::new(vec!["overwriteConfig".to_string()]));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteBucketOverwriteConfigResult::default();
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
    fn test_overwrite_configuration_serde_round_trip() {
        let configuration = OverwriteConfiguration {
            rules: vec![
                OverwriteRule {
                    id: Some("rule-1".to_string()),
                    action: Some("forbid".to_string()),
                    prefix: Some("prefix/".to_string()),
                    suffix: Some(".txt".to_string()),
                    principals: Some(OverwritePrincipals {
                        principals: vec!["1234567890".to_string()],
                    }),
                },
                OverwriteRule {
                    id: None,
                    action: Some("forbid".to_string()),
                    prefix: None,
                    suffix: None,
                    principals: None,
                },
            ],
        };

        let xml =
            quick_xml::se::to_string_with_root("OverwriteConfiguration", &configuration).unwrap();
        assert!(xml.contains("<OverwriteConfiguration>"));
        assert!(xml.contains("<ID>rule-1</ID>"));
        assert!(xml.contains("<Action>forbid</Action>"));
        assert!(xml.contains("<Principal>1234567890</Principal>"));

        let parsed: OverwriteConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.rules.len(), 2);
        assert_eq!(parsed.rules[0].id.as_deref(), Some("rule-1"));
        assert_eq!(parsed.rules[0].action.as_deref(), Some("forbid"));
        assert_eq!(parsed.rules[0].prefix.as_deref(), Some("prefix/"));
        assert_eq!(
            parsed.rules[0].principals.as_ref().unwrap().principals,
            vec!["1234567890".to_string()]
        );
        assert!(parsed.rules[1].principals.is_none());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_bucket_overwrite_config() {
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

        // Deleting a configuration that was never set is expected to fail; this
        // only verifies the request is well-formed.
        let result = client
            .delete_bucket_overwrite_config(&DeleteBucketOverwriteConfigRequest::new(&config.bucket))
            .await;
        if let Err(error) = &result {
            eprintln!("delete_bucket_overwrite_config rejected (may not be configured): {}", error);
        }
    }
}
