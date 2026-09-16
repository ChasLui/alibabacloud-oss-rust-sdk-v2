use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct OptionObjectRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The full path of the object.
    pub key: String,

    /// The origin of the request. It is used to identify a cross-origin request.
    #[field(type = "header", rename = "Origin")]
    pub origin: Option<String>,

    /// The method to be used in the actual cross-origin request.
    #[field(type = "header", rename = "Access-Control-Request-Method")]
    pub access_control_request_method: Option<String>,

    /// The custom headers to be sent in the actual cross-origin request.
    #[field(type = "header", rename = "Access-Control-Request-Headers")]
    pub access_control_request_headers: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct OptionObjectResult {
    /// The HTTP method of the request. If the request is denied, the response
    /// does not contain the header.
    #[field(type = "header", rename = "Access-Control-Allow-Methods")]
    pub access_control_allow_methods: Option<String>,

    /// The list of headers included in the request.
    #[field(type = "header", rename = "Access-Control-Allow-Headers")]
    pub access_control_allow_headers: Option<String>,

    /// The list of headers that can be accessed by JavaScript applications on a client.
    #[field(type = "header", rename = "Access-Control-Expose-Headers")]
    pub access_control_expose_headers: Option<String>,

    /// The maximum duration for the browser to cache preflight results.
    /// Unit: seconds. The Go SDK types this header as int64; the result macro
    /// only supports string headers, so it is kept as a string here.
    #[field(type = "header", rename = "Access-Control-Max-Age")]
    pub access_control_max_age: Option<String>,

    /// The origin that is included in the request. If the request is denied,
    /// the response does not contain the header.
    #[field(type = "header", rename = "Access-Control-Allow-Origin")]
    pub access_control_allow_origin: Option<String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Determines whether to send a cross-origin request. Before a cross-origin
    /// request is sent, the browser sends a preflight OPTIONS request that
    /// includes a specific origin, HTTP method, and header information to Object
    /// Storage Service (OSS) to determine whether to send the cross-origin request.
    ///
    /// # Arguments
    ///
    /// * `request` - The `OptionObjectRequest` containing the bucket name, object
    ///   key and the preflight origin/method/headers.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::OptionObjectRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = OptionObjectRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     origin: Some("https://example.com".to_string()),
    ///     access_control_request_method: Some("GET".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.option_object(&request).await {
    ///     Ok(result) => {
    ///         println!("Allow origin: {:?}", result.access_control_allow_origin);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to send preflight request: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn option_object(
        &self,
        request: &OptionObjectRequest,
    ) -> Result<OptionObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "OptionObject".to_string(),
            method: http::Method::OPTIONS,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = OptionObjectResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::put_bucket_cors::{CORSConfiguration, CORSRule, PutBucketCorsRequest};
    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_option_object() {
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

        let bucket_name = generate_unique_bucket_name("option-object");

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
            .option_object(&OptionObjectRequest {
                bucket: bucket_name.clone(),
                key: "any-object".to_string(),
                origin: Some("https://example.com".to_string()),
                access_control_request_method: Some("GET".to_string()),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "option_object failed: {:?}", result.err());
        assert_eq!(
            result.unwrap().access_control_allow_origin.as_deref(),
            Some("https://example.com")
        );

        // Clean up
        let _ = client
            .delete_bucket(&DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
