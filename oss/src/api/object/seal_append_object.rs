use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct SealAppendObjectRequest {
    /// Bucket name
    pub bucket: String,

    /// Name of the Appendable Object
    pub key: String,

    /// Used to specify the expected length of the file when the user wants to
    /// seal it.
    #[field(type = "query", rename = "position")]
    pub position: Option<i64>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct SealAppendObjectResult {
    /// The time in GMT format when the SealAppendObject operation was first
    /// performed on the object. This timestamp does not change even if the
    /// operation is performed again.
    #[field(type = "header", rename = "x-oss-sealed-time")]
    pub sealed_time: Option<String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Stops writing to the Appendable Object, after which the user can
    /// configure lifecycle rules to change the storage class of the
    /// corresponding Appendable Object to Cold Archive or Deep Cold Archive.
    ///
    /// # Arguments
    ///
    /// * `request` - The `SealAppendObjectRequest` containing the bucket name,
    ///   object key and the expected object length.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::SealAppendObjectRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = SealAppendObjectRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     position: Some(5),
    ///     ..Default::default()
    /// };
    ///
    /// match client.seal_append_object(&request).await {
    ///     Ok(result) => {
    ///         println!("Sealed time: {:?}", result.sealed_time);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to seal append object: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn seal_append_object(
        &self,
        request: &SealAppendObjectRequest,
    ) -> Result<SealAppendObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "SealAppendObject".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("seal", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input
            .op_metadata
            .set(crate::signer::SUB_RESOURCE, std::rc::Rc::new(vec!["seal".to_string()]));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = SealAppendObjectResult::default();
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
    async fn test_seal_append_object() {
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

        let object_name = crate::test_utils::generate_unique_object_name("seal");

        // Prepare an appendable object with 5 bytes.
        client
            .append_object(crate::api::object::AppendObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                position: Some(0),
                body: Some(BodyContent::from_text("hello".to_string(), None)),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .seal_append_object(&SealAppendObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                position: Some(5),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "seal_append_object failed: {:?}", result.err());

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
