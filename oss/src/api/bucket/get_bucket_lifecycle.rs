use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::put_bucket_lifecycle::LifecycleRule;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketLifecycleRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "LifecycleConfiguration")]
pub struct GetBucketLifecycleResult {
    /// The lifecycle rules configured for the bucket.
    #[serde(rename = "Rule", default)]
    pub rules: Vec<LifecycleRule>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the lifecycle rules configured for a bucket. Only the owner of a
    /// bucket has the permissions to query the lifecycle rules configured for
    /// the bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketLifecycleRequest` containing the bucket
    ///   name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketLifecycleRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketLifecycleRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_lifecycle(&request).await {
    ///     Ok(result) => {
    ///         println!("Lifecycle rules: {:?}", result.rules.len());
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket lifecycle: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_lifecycle(
        &self,
        request: &GetBucketLifecycleRequest,
    ) -> Result<GetBucketLifecycleResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketLifecycle".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("lifecycle", "")]
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
            std::rc::Rc::new(vec!["lifecycle".to_string()]),
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
        let mut result: GetBucketLifecycleResult = quick_xml::de::from_str(&data_str)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::put_bucket_lifecycle::{
        LifecycleConfiguration, LifecycleRule, LifecycleRuleExpiration, PutBucketLifecycleRequest,
    };
    use super::*;
    use crate::api::bucket::{
        CreateBucketRequest, DeleteBucketLifecycleRequest, DeleteBucketRequest,
    };
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_get_bucket_lifecycle_result_deserialize() {
        let xml = r#"<LifecycleConfiguration><Rule><ID>rule1</ID><Prefix>logs/</Prefix><Status>Enabled</Status><Expiration><Days>30</Days></Expiration></Rule></LifecycleConfiguration>"#;
        let result: GetBucketLifecycleResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.rules.len(), 1);
        assert_eq!(result.rules[0].id.as_deref(), Some("rule1"));
        assert_eq!(result.rules[0].prefix.as_deref(), Some("logs/"));
        assert_eq!(result.rules[0].expiration.as_ref().unwrap().days, Some(30));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_lifecycle() {
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

        let bucket_name = generate_unique_bucket_name("get-bucket-lifecycle");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        client
            .put_bucket_lifecycle(&PutBucketLifecycleRequest {
                bucket: bucket_name.clone(),
                lifecycle_configuration: LifecycleConfiguration {
                    rules: vec![LifecycleRule {
                        id: Some("rule1".to_string()),
                        status: Some("Enabled".to_string()),
                        prefix: Some("logs/".to_string()),
                        expiration: Some(LifecycleRuleExpiration {
                            days: Some(30),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                },
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .get_bucket_lifecycle(&GetBucketLifecycleRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_lifecycle failed: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().rules.len(), 1);

        // Clean up
        let _ = client
            .delete_bucket_lifecycle(&DeleteBucketLifecycleRequest {
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
