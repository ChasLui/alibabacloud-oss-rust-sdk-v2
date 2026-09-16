use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::object::Tagging;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketTagsRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The request body schema.
    pub tagging: Tagging,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketTagsResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Adds tags to or modifies the existing tags of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketTagsRequest` containing the bucket name and
    ///   the tags to set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::PutBucketTagsRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::{Tag, TagSet, Tagging};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketTagsRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     tagging: Tagging {
    ///         tag_set: Some(TagSet {
    ///             tags: vec![Tag {
    ///                 key: Some("k1".to_string()),
    ///                 value: Some("v1".to_string()),
    ///             }],
    ///         }),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_tags(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket tags updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket tags: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_tags(
        &self,
        request: &PutBucketTagsRequest,
    ) -> Result<PutBucketTagsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketTags".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("tagging", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["tagging".to_string()]),
        );

        let xml_body = quick_xml::se::to_string_with_root("Tagging", &request.tagging)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketTagsResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest, DeleteBucketTagsRequest};
    use crate::api::object::{Tag, TagSet, Tagging};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_tags() {
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

        let bucket_name = generate_unique_bucket_name("put-bucket-tags");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
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
            .await;
        assert!(result.is_ok(), "put_bucket_tags failed: {:?}", result.err());

        // Clean up
        let _ = client
            .delete_bucket_tags(&DeleteBucketTagsRequest {
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
