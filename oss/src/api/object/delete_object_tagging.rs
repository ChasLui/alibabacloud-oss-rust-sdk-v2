use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteObjectTaggingRequest {
    /// The name of the bucket.
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
pub struct DeleteObjectTaggingResult {
    /// Version of the object.
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes the tags of a specified object.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteObjectTaggingRequest` containing the bucket
    ///   name and object key.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::DeleteObjectTaggingRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteObjectTaggingRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_object_tagging(&request).await {
    ///     Ok(result) => {
    ///         println!("Object tagging deleted: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete object tagging: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_object_tagging(
        &self,
        request: &DeleteObjectTaggingRequest,
    ) -> Result<DeleteObjectTaggingResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteObjectTagging".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("tagging", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteObjectTaggingResult::default();
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
    async fn test_delete_object_tagging() {
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

        let object_name = crate::test_utils::generate_unique_object_name("tagging");

        // Prepare an object with tags.
        client
            .put_object(crate::api::object::PutObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                body: Some(BodyContent::from_text("tagging-test".to_string(), None)),
                ..Default::default()
            })
            .await
            .unwrap();

        client
            .put_object_tagging(&crate::api::object::PutObjectTaggingRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                tagging: crate::api::object::Tagging {
                    tag_set: Some(crate::api::object::TagSet {
                        tags: vec![crate::api::object::Tag {
                            key: Some("k1".to_string()),
                            value: Some("v1".to_string()),
                        }],
                    }),
                },
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .delete_object_tagging(&DeleteObjectTaggingRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "delete_object_tagging failed: {:?}", result.err());

        // Verify the tags are gone.
        let get_result = client
            .get_object_tagging(&crate::api::object::GetObjectTaggingRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();
        let tags = get_result.tag_set.map(|t| t.tags).unwrap_or_default();
        assert!(tags.is_empty(), "tags should be empty after deletion");

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
