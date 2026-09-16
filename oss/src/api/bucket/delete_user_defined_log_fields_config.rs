use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteUserDefinedLogFieldsConfigRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteUserDefinedLogFieldsConfigResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes the custom configurations of the user_defined_log_fields field
    /// in the real-time logs of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteUserDefinedLogFieldsConfigRequest` containing
    ///   the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::DeleteUserDefinedLogFieldsConfigRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteUserDefinedLogFieldsConfigRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_user_defined_log_fields_config(&request).await {
    ///     Ok(result) => {
    ///         println!("User defined log fields deleted: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete user defined log fields config: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_user_defined_log_fields_config(
        &self,
        request: &DeleteUserDefinedLogFieldsConfigRequest,
    ) -> Result<DeleteUserDefinedLogFieldsConfigResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "DeleteUserDefinedLogFieldsConfig".to_string(),
            method: http::Method::DELETE,
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

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteUserDefinedLogFieldsConfigResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::put_user_defined_log_fields_config::{
        LoggingHeaderSet, PutUserDefinedLogFieldsConfigRequest, UserDefinedLogFieldsConfiguration,
    };
    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_user_defined_log_fields_config() {
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

        let bucket_name = generate_unique_bucket_name("delete-user-defined-log-fields");

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
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .delete_user_defined_log_fields_config(&DeleteUserDefinedLogFieldsConfigRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "delete_user_defined_log_fields_config failed: {:?}",
            result.err()
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
