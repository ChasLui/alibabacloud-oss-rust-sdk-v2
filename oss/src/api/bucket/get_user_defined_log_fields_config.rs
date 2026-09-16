use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::put_user_defined_log_fields_config::{LoggingHeaderSet, LoggingParamSet};
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetUserDefinedLogFieldsConfigRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "UserDefinedLogFieldsConfiguration")]
pub struct GetUserDefinedLogFieldsConfigResult {
    /// The container that stores the configurations of custom request headers.
    #[serde(rename = "HeaderSet", skip_serializing_if = "Option::is_none")]
    pub header_set: Option<LoggingHeaderSet>,

    /// The container that stores the configurations of custom URL parameters.
    #[serde(rename = "ParamSet", skip_serializing_if = "Option::is_none")]
    pub param_set: Option<LoggingParamSet>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the custom configurations of the user_defined_log_fields field
    /// in the real-time logs of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetUserDefinedLogFieldsConfigRequest` containing the
    ///   bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetUserDefinedLogFieldsConfigRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetUserDefinedLogFieldsConfigRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_user_defined_log_fields_config(&request).await {
    ///     Ok(result) => {
    ///         println!("User defined log fields: {:?}", result.header_set);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get user defined log fields config: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_user_defined_log_fields_config(
        &self,
        request: &GetUserDefinedLogFieldsConfigRequest,
    ) -> Result<GetUserDefinedLogFieldsConfigResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetUserDefinedLogFieldsConfig".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("userDefinedLogFieldsConfig", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["userDefinedLogFieldsConfig".to_string()]),
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
        let mut result: GetUserDefinedLogFieldsConfigResult = quick_xml::de::from_str(&data_str)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::put_user_defined_log_fields_config::{
        LoggingHeaderSet, LoggingParamSet, PutUserDefinedLogFieldsConfigRequest,
        UserDefinedLogFieldsConfiguration,
    };
    use super::*;
    use crate::api::bucket::{
        CreateBucketRequest, DeleteBucketRequest, DeleteUserDefinedLogFieldsConfigRequest,
    };
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_get_user_defined_log_fields_config_result_deserialize() {
        let xml = r#"<UserDefinedLogFieldsConfiguration><HeaderSet><header>x-oss-test</header></HeaderSet><ParamSet><parameter>versionId</parameter></ParamSet></UserDefinedLogFieldsConfiguration>"#;
        let result: GetUserDefinedLogFieldsConfigResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(
            result.header_set.unwrap().headers,
            vec!["x-oss-test".to_string()]
        );
        assert_eq!(
            result.param_set.unwrap().parameters,
            vec!["versionId".to_string()]
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_user_defined_log_fields_config() {
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

        let bucket_name = generate_unique_bucket_name("get-user-defined-log-fields");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        client
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
            .await
            .unwrap();

        let result = client
            .get_user_defined_log_fields_config(&GetUserDefinedLogFieldsConfigRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "get_user_defined_log_fields_config failed: {:?}",
            result.err()
        );
        assert_eq!(
            result.unwrap().header_set.unwrap().headers,
            vec!["x-oss-test".to_string()]
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
