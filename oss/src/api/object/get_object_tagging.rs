use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::put_object_tagging::TagSet;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{modify_request, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetObjectTaggingRequest {
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

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "Tagging")]
pub struct GetObjectTaggingResult {
    /// The container used to store the collection of tags.
    #[serde(rename = "TagSet", skip_serializing_if = "Option::is_none")]
    pub tag_set: Option<TagSet>,

    /// Version of the object.
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the tags of an object.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetObjectTaggingRequest` containing the bucket name
    ///   and object key.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::GetObjectTaggingRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetObjectTaggingRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_object_tagging(&request).await {
    ///     Ok(result) => {
    ///         println!("Object tags: {:?}", result.tag_set);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get object tagging: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_object_tagging(
        &self,
        request: &GetObjectTaggingRequest,
    ) -> Result<GetObjectTaggingResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetObjectTagging".to_string(),
            method: http::Method::GET,
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

        let mut output = self.invoke_operation(input, vec![]).await?;

        // Parse the XML response
        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetObjectTaggingResult = quick_xml::de::from_str(&data_str)?;
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

    #[test]
    fn test_get_object_tagging_result_serde_round_trip() {
        let xml = "<Tagging><TagSet><Tag><Key>k1</Key><Value>v1</Value></Tag>\
            <Tag><Key>k2</Key><Value></Value></Tag></TagSet></Tagging>";
        let result: GetObjectTaggingResult = quick_xml::de::from_str(xml).unwrap();
        let tags = result.tag_set.unwrap().tags;
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].key.as_deref(), Some("k1"));
        assert_eq!(tags[0].value.as_deref(), Some("v1"));
        assert_eq!(tags[1].key.as_deref(), Some("k2"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_object_tagging() {
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
            .get_object_tagging(&GetObjectTaggingRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "get_object_tagging failed: {:?}", result.err());
        let tags = result.unwrap().tag_set.unwrap().tags;
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].key.as_deref(), Some("k1"));
        assert_eq!(tags[0].value.as_deref(), Some("v1"));

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
