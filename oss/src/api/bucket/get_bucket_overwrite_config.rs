use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::put_bucket_overwrite_config::OverwriteRule;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketOverwriteConfigRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "OverwriteConfiguration")]
pub struct GetBucketOverwriteConfigResult {
    /// The overwrite protection rules configured for the bucket.
    #[serde(rename = "Rule", default)]
    pub rules: Vec<OverwriteRule>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Retrieves the overwrite configuration of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketOverwriteConfigRequest` containing the
    ///   bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketOverwriteConfigRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketOverwriteConfigRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_overwrite_config(&request).await {
    ///     Ok(result) => {
    ///         println!("Overwrite rules: {:?}", result.rules.len());
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket overwrite config: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_overwrite_config(
        &self,
        request: &GetBucketOverwriteConfigRequest,
    ) -> Result<GetBucketOverwriteConfigResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketOverwriteConfig".to_string(),
            method: http::Method::GET,
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
        let mut result: GetBucketOverwriteConfigResult = quick_xml::de::from_str(&data_str)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::put_bucket_overwrite_config::{
        OverwriteConfiguration, PutBucketOverwriteConfigRequest,
    };
    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_get_bucket_overwrite_config_result_deserialize() {
        let xml = r#"<OverwriteConfiguration><Rule><ID>rule-1</ID><Action>forbid</Action><Prefix>protected/</Prefix><Suffix>.txt</Suffix><Principals><Principal>1234567890</Principal></Principals></Rule></OverwriteConfiguration>"#;
        let result: GetBucketOverwriteConfigResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.rules.len(), 1);
        let rule = &result.rules[0];
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
    async fn test_get_bucket_overwrite_config() {
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

        let bucket_name = generate_unique_bucket_name("get-bucket-overwrite-config");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        client
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
            .await
            .unwrap();

        let result = client
            .get_bucket_overwrite_config(&GetBucketOverwriteConfigRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_overwrite_config failed: {:?}",
            result.err()
        );
        let result = result.unwrap();
        assert_eq!(result.rules.len(), 1);
        assert_eq!(result.rules[0].action, Some("forbid".to_string()));
        assert_eq!(result.rules[0].prefix, Some("protected/".to_string()));

        // Clean up
        let _ = client
            .delete_bucket(&DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
