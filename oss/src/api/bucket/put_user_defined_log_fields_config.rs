use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the configurations of custom request headers.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LoggingHeaderSet {
    /// The list of the custom request headers.
    #[serde(rename = "header", default)]
    pub headers: Vec<String>,
}

/// The container that stores the configurations of custom URL parameters.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LoggingParamSet {
    /// The list of the custom URL parameters.
    #[serde(rename = "parameter", default)]
    pub parameters: Vec<String>,
}

/// The container that stores the custom configurations of the
/// user_defined_log_fields field in real-time logs.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserDefinedLogFieldsConfiguration {
    /// The container that stores the configurations of custom request headers.
    #[serde(rename = "HeaderSet", skip_serializing_if = "Option::is_none")]
    pub header_set: Option<LoggingHeaderSet>,

    /// The container that stores the configurations of custom URL parameters.
    #[serde(rename = "ParamSet", skip_serializing_if = "Option::is_none")]
    pub param_set: Option<LoggingParamSet>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutUserDefinedLogFieldsConfigRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The container that stores the specified log configurations.
    pub user_defined_log_fields_configuration: UserDefinedLogFieldsConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutUserDefinedLogFieldsConfigResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Customizes the user_defined_log_fields field in real-time logs by
    /// adding custom request headers or query parameters to the field for
    /// subsequent analysis of requests.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutUserDefinedLogFieldsConfigRequest` containing the
    ///   bucket name and the user-defined log fields configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    /// #     LoggingHeaderSet, PutUserDefinedLogFieldsConfigRequest,
    /// #     UserDefinedLogFieldsConfiguration,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutUserDefinedLogFieldsConfigRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     user_defined_log_fields_configuration: UserDefinedLogFieldsConfiguration {
    ///         header_set: Some(LoggingHeaderSet {
    ///             headers: vec!["x-oss-test".to_string()],
    ///         }),
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_user_defined_log_fields_config(&request).await {
    ///     Ok(result) => {
    ///         println!(
    ///             "User defined log fields updated: {:?}",
    ///             result.common.status
    ///         );
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put user defined log fields config: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_user_defined_log_fields_config(
        &self,
        request: &PutUserDefinedLogFieldsConfigRequest,
    ) -> Result<PutUserDefinedLogFieldsConfigResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutUserDefinedLogFieldsConfig".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("userDefinedLogFieldsConfig", "")]
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
            std::rc::Rc::new(vec!["userDefinedLogFieldsConfig".to_string()]),
        );

        let xml_body = quick_xml::se::to_string_with_root(
            "UserDefinedLogFieldsConfiguration",
            &request.user_defined_log_fields_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutUserDefinedLogFieldsConfigResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{
        CreateBucketRequest, DeleteBucketRequest, DeleteUserDefinedLogFieldsConfigRequest,
    };
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_user_defined_log_fields_configuration_serde_round_trip() {
        let config = UserDefinedLogFieldsConfiguration {
            header_set: Some(LoggingHeaderSet {
                headers: vec!["x-oss-test".to_string(), "x-oss-meta-a".to_string()],
            }),
            param_set: Some(LoggingParamSet {
                parameters: vec!["versionId".to_string()],
            }),
        };

        let xml = quick_xml::se::to_string_with_root("UserDefinedLogFieldsConfiguration", &config)
            .unwrap();
        assert!(xml.contains("<UserDefinedLogFieldsConfiguration>"));
        assert!(xml.contains("<header>x-oss-test</header>"));
        assert!(xml.contains("<parameter>versionId</parameter>"));

        let parsed: UserDefinedLogFieldsConfiguration = quick_xml::de::from_str(&xml).unwrap();
        let header_set = parsed.header_set.unwrap();
        assert_eq!(
            header_set.headers,
            vec!["x-oss-test".to_string(), "x-oss-meta-a".to_string()]
        );
        assert_eq!(
            parsed.param_set.unwrap().parameters,
            vec!["versionId".to_string()]
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_user_defined_log_fields_config() {
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

        let bucket_name = generate_unique_bucket_name("put-user-defined-log-fields");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .put_user_defined_log_fields_config(&PutUserDefinedLogFieldsConfigRequest {
                bucket: bucket_name.clone(),
                user_defined_log_fields_configuration: UserDefinedLogFieldsConfiguration {
                    header_set: Some(LoggingHeaderSet {
                        headers: vec!["x-oss-test".to_string()],
                    }),
                    param_set: Some(LoggingParamSet {
                        parameters: vec!["versionId".to_string()],
                    }),
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "put_user_defined_log_fields_config failed: {:?}",
            result.err()
        );

        // Clean up
        let _ = client
            .delete_user_defined_log_fields_config(&DeleteUserDefinedLogFieldsConfigRequest {
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
