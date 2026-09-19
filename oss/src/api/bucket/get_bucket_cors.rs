use crate::HTTP_HEADER_CONTENT_TYPE;
use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::put_bucket_cors::CORSRule;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketCorsRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "CORSConfiguration")]
pub struct GetBucketCorsResult {
    /// The CORS rules configured for the bucket.
    #[serde(rename = "CORSRule", default)]
    pub cors_rules: Vec<CORSRule>,

    /// Indicates whether the Vary: Origin header was returned.
    #[serde(rename = "ResponseVary", skip_serializing_if = "Option::is_none")]
    pub response_vary: Option<bool>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the cross-origin resource sharing (CORS) rules that are configured
    /// for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketCorsRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketCorsRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketCorsRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_cors(&request).await {
    ///     Ok(result) => {
    ///         println!("CORS rules: {:?}", result.cors_rules.len());
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket CORS: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_cors(
        &self,
        request: &GetBucketCorsRequest,
    ) -> Result<GetBucketCorsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketCors".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("cors", "")]
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
            std::rc::Rc::new(vec!["cors".to_string()]),
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
        let mut result: GetBucketCorsResult = quick_xml::de::from_str(&data_str)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::put_bucket_cors::{CORSConfiguration, CORSRule, PutBucketCorsRequest};
    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketCorsRequest, DeleteBucketRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_get_bucket_cors_result_deserialize() {
        let xml = r#"<CORSConfiguration><CORSRule><AllowedOrigin>https://example.com</AllowedOrigin><AllowedMethod>GET</AllowedMethod><MaxAgeSeconds>3600</MaxAgeSeconds></CORSRule><ResponseVary>false</ResponseVary></CORSConfiguration>"#;
        let result: GetBucketCorsResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.cors_rules.len(), 1);
        assert_eq!(result.cors_rules[0].allowed_origins, vec!["https://example.com".to_string()]);
        assert_eq!(result.cors_rules[0].max_age_seconds, Some(3600));
        assert_eq!(result.response_vary, Some(false));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_cors() {
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

        let bucket_name = generate_unique_bucket_name("get-bucket-cors");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        client
            .put_bucket_cors(&PutBucketCorsRequest {
                bucket: bucket_name.clone(),
                cors_configuration: CORSConfiguration {
                    cors_rules: vec![CORSRule {
                        allowed_origins: vec!["https://example.com".to_string()],
                        allowed_methods: vec!["GET".to_string()],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .get_bucket_cors(&GetBucketCorsRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "get_bucket_cors failed: {:?}", result.err());
        assert_eq!(result.unwrap().cors_rules.len(), 1);

        // Clean up
        let _ = client
            .delete_bucket_cors(&DeleteBucketCorsRequest {
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
