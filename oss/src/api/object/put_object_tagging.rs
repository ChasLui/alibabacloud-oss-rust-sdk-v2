use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput};
/// The container that stores the tag set.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Tagging {
    /// The container used to store a set of Tags.
    #[serde(rename = "TagSet", skip_serializing_if = "Option::is_none")]
    pub tag_set: Option<TagSet>,
}

/// The container used to store a set of Tags.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TagSet {
    /// The tags.
    #[serde(rename = "Tag", default)]
    pub tags: Vec<Tag>,
}

/// A key-value tag.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Tag {
    /// The key of a tag.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The value of the tag.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutObjectTaggingRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// Version of the object.
    #[field(type = "query", rename = "versionId")]
    pub version_id: Option<String>,

    /// The container that stores the tags to add to the object.
    pub tagging: Tagging,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutObjectTaggingResult {
    /// Version of the object.
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Adds tags to an object or updates the tags added to the object. Each tag
    /// added to an object is a key-value pair.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutObjectTaggingRequest` containing the bucket name,
    ///   object key and the tags to set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::{PutObjectTaggingRequest, Tagging, TagSet, Tag};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutObjectTaggingRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
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
    /// match client.put_object_tagging(&request).await {
    ///     Ok(result) => {
    ///         println!("Object tagging updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put object tagging: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_object_tagging(
        &self,
        request: &PutObjectTaggingRequest,
    ) -> Result<PutObjectTaggingResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutObjectTagging".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("tagging", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        let xml_body = quick_xml::se::to_string_with_root("Tagging", &request.tagging)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutObjectTaggingResult::default();
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
    use crate::SignatureVersionType;

    #[test]
    fn test_tagging_serde_round_trip() {
        let tagging = Tagging {
            tag_set: Some(TagSet {
                tags: vec![
                    Tag {
                        key: Some("k1".to_string()),
                        value: Some("v1".to_string()),
                    },
                    Tag {
                        key: Some("k2".to_string()),
                        value: None,
                    },
                ],
            }),
        };

        let xml = quick_xml::se::to_string_with_root("Tagging", &tagging).unwrap();
        assert!(xml.contains("<Tagging>"));
        assert!(xml.contains("<Key>k1</Key>"));
        assert!(xml.contains("<Value>v1</Value>"));

        let parsed: Tagging = quick_xml::de::from_str(&xml).unwrap();
        let tags = parsed.tag_set.unwrap().tags;
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].key.as_deref(), Some("k1"));
        assert_eq!(tags[0].value.as_deref(), Some("v1"));
        assert_eq!(tags[1].key.as_deref(), Some("k2"));
        // None fields are skipped during serialization and read back as None
        assert_eq!(tags[1].value, None);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_object_tagging() {
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

        // Prepare an object
        client
            .put_object(crate::api::object::PutObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                body: Some(BodyContent::from_text("tagging-test".to_string(), None)),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .put_object_tagging(&PutObjectTaggingRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
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
        assert!(
            result.is_ok(),
            "put_object_tagging failed: {:?}",
            result.err()
        );

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
