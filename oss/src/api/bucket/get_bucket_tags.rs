use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::object::TagSet;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketTagsRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "Tagging")]
pub struct GetBucketTagsResult {
    /// The container that stores the returned tags of the bucket.
    #[serde(rename = "TagSet", skip_serializing_if = "Option::is_none")]
    pub tag_set: Option<TagSet>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the tags of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketTagsRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketTagsRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketTagsRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_tags(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket tags: {:?}", result.tag_set);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket tags: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_tags(
        &self,
        request: &GetBucketTagsRequest,
    ) -> Result<GetBucketTagsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketTags".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("tagging", "")]
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
            std::rc::Rc::new(vec!["tagging".to_string()]),
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
        let mut result: GetBucketTagsResult = quick_xml::de::from_str(&data_str)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::put_bucket_tags::PutBucketTagsRequest;
    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest, DeleteBucketTagsRequest};
    use crate::api::object::{Tag, Tagging};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_get_bucket_tags_result_deserialize() {
        let xml = "<Tagging><TagSet><Tag><Key>k1</Key><Value>v1</Value></Tag></TagSet></Tagging>";
        let result: GetBucketTagsResult = quick_xml::de::from_str(xml).unwrap();
        let tags = result.tag_set.unwrap().tags;
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].key.as_deref(), Some("k1"));
        assert_eq!(tags[0].value.as_deref(), Some("v1"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_tags() {
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

        let bucket_name = generate_unique_bucket_name("get-bucket-tags");

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
            .get_bucket_tags(&GetBucketTagsRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "get_bucket_tags failed: {:?}", result.err());
        let tags = result.unwrap().tag_set.unwrap().tags;
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].key.as_deref(), Some("k1"));

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
