use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores CORS rules.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CORSConfiguration {
    /// The CORS rules. Up to 10 rules can be configured for a bucket.
    #[serde(rename = "CORSRule", default)]
    pub cors_rules: Vec<CORSRule>,

    /// Indicates whether the Vary: Origin header was returned. Default value: false.
    #[serde(rename = "ResponseVary", skip_serializing_if = "Option::is_none")]
    pub response_vary: Option<bool>,
}

/// A CORS rule of a bucket.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CORSRule {
    /// The origins from which cross-origin requests are allowed.
    #[serde(rename = "AllowedOrigin", default)]
    pub allowed_origins: Vec<String>,

    /// The methods that you can use in cross-origin requests.
    #[serde(rename = "AllowedMethod", default)]
    pub allowed_methods: Vec<String>,

    /// The headers that are allowed in cross-origin requests.
    #[serde(rename = "AllowedHeader", default)]
    pub allowed_headers: Vec<String>,

    /// The response headers for allowed access requests from applications.
    #[serde(rename = "ExposeHeader", default)]
    pub expose_headers: Vec<String>,

    /// The period of time within which the browser can cache the response to an
    /// OPTIONS preflight request for the specified resource. Unit: seconds.
    #[serde(rename = "MaxAgeSeconds", skip_serializing_if = "Option::is_none")]
    pub max_age_seconds: Option<i64>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketCorsRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The request body schema.
    pub cors_configuration: CORSConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketCorsResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Configures cross-origin resource sharing (CORS) rules for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketCorsRequest` containing the bucket name and
    ///   the CORS configuration to set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{CORSConfiguration, CORSRule, PutBucketCorsRequest};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketCorsRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     cors_configuration: CORSConfiguration {
    ///         cors_rules: vec![CORSRule {
    ///             allowed_origins: vec!["https://example.com".to_string()],
    ///             allowed_methods: vec!["GET".to_string()],
    ///             ..Default::default()
    ///         }],
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_cors(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket CORS updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket CORS: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_cors(
        &self,
        request: &PutBucketCorsRequest,
    ) -> Result<PutBucketCorsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketCors".to_string(),
            method: http::Method::PUT,
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

        let xml_body =
            quick_xml::se::to_string_with_root("CORSConfiguration", &request.cors_configuration)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketCorsResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketCorsRequest, DeleteBucketRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_cors_configuration_serde_round_trip() {
        let config = CORSConfiguration {
            cors_rules: vec![CORSRule {
                allowed_origins: vec!["https://example.com".to_string()],
                allowed_methods: vec!["GET".to_string(), "PUT".to_string()],
                allowed_headers: vec!["Authorization".to_string()],
                expose_headers: vec!["x-oss-request-id".to_string()],
                max_age_seconds: Some(3600),
            }],
            response_vary: Some(true),
        };

        let xml = quick_xml::se::to_string_with_root("CORSConfiguration", &config).unwrap();
        assert!(xml.contains("<CORSConfiguration>"));
        assert!(xml.contains("<AllowedOrigin>https://example.com</AllowedOrigin>"));
        assert!(xml.contains("<AllowedMethod>PUT</AllowedMethod>"));
        assert!(xml.contains("<MaxAgeSeconds>3600</MaxAgeSeconds>"));
        assert!(xml.contains("<ResponseVary>true</ResponseVary>"));

        let parsed: CORSConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.cors_rules.len(), 1);
        let rule = &parsed.cors_rules[0];
        assert_eq!(rule.allowed_origins, vec!["https://example.com".to_string()]);
        assert_eq!(rule.allowed_methods.len(), 2);
        assert_eq!(rule.allowed_headers, vec!["Authorization".to_string()]);
        assert_eq!(rule.expose_headers, vec!["x-oss-request-id".to_string()]);
        assert_eq!(rule.max_age_seconds, Some(3600));
        assert_eq!(parsed.response_vary, Some(true));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_cors() {
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

        let bucket_name = generate_unique_bucket_name("put-bucket-cors");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
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
            .await;
        assert!(result.is_ok(), "put_bucket_cors failed: {:?}", result.err());

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
