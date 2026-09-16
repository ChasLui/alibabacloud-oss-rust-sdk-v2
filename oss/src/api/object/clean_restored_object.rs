use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct CleanRestoredObjectRequest {
    /// The name of the bucket
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// Version of the object.
    #[field(type = "query", rename = "versionId")]
    pub version_id: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct CleanRestoredObjectResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Cleans an object restored from Archive or Cold Archive state. After
    /// that, the restored object returns to the frozen state.
    ///
    /// # Arguments
    ///
    /// * `request` - The `CleanRestoredObjectRequest` containing the bucket
    ///   name and object key.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::CleanRestoredObjectRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = CleanRestoredObjectRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.clean_restored_object(&request).await {
    ///     Ok(result) => {
    ///         println!("Restored object cleaned: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to clean restored object: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn clean_restored_object(
        &self,
        request: &CleanRestoredObjectRequest,
    ) -> Result<CleanRestoredObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "CleanRestoredObject".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("cleanRestoredObject", "")]
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
            std::rc::Rc::new(vec!["cleanRestoredObject".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = CleanRestoredObjectResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::{BodyContent, SignatureVersionType};

    #[tokio::test]
    #[serial_test::serial]
    async fn test_clean_restored_object() {
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

        let object_name = crate::test_utils::generate_unique_object_name("clean-restored");

        // Prepare an Archive object.
        client
            .put_object(crate::api::object::PutObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                storage_class: "Archive".to_string(),
                body: Some(BodyContent::from_text("clean-restored-test".to_string(), None)),
                ..Default::default()
            })
            .await
            .unwrap();

        // Restore it first; restore is asynchronous on the server side.
        client
            .restore_object(&crate::api::object::RestoreObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                restore_request: Some(crate::api::object::RestoreRequest {
                    days: 1,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .clean_restored_object(&CleanRestoredObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "clean_restored_object failed: {:?}", result.err());

        // Clean up
        let _ = client
            .delete_object(crate::api::object::DeleteObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name,
                ..Default::default()
            })
            .await;
    }
}
