use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteBucketTagsRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The tag key to delete. If you do not specify this parameter, all tags
    /// of the bucket are deleted.
    #[field(type = "query", rename = "tagging")]
    pub tagging: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteBucketTagsResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes tags configured for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteBucketTagsRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::DeleteBucketTagsRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteBucketTagsRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_bucket_tags(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket tags deleted: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete bucket tags: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_bucket_tags(
        &self,
        request: &DeleteBucketTagsRequest,
    ) -> Result<DeleteBucketTagsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteBucketTags".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            parameters: [("tagging", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["tagging".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteBucketTagsResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::put_bucket_tags::PutBucketTagsRequest;
    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest};
    use crate::api::object::{Tag, TagSet, Tagging};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_bucket_tags() {
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

        let bucket_name = generate_unique_bucket_name("delete-bucket-tags");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        client
            .put_bucket_tags(&PutBucketTagsRequest {
                bucket: bucket_name.clone(),
                tagging: Tagging {
                    tag_set: Some(TagSet {
                        tags: vec![Tag {
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
            .delete_bucket_tags(&DeleteBucketTagsRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "delete_bucket_tags failed: {:?}", result.err());

        // Clean up
        let _ = client
            .delete_bucket(&DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
