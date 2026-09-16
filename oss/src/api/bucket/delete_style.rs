use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::OperationInput;
use crate::OperationOutput;

/// The container that stores the content information about the image style.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StyleContent {
    /// The content of the style.
    #[serde(rename = "Content", skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// The container that stores the information about an image style.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StyleInfo {
    /// The time when the style was created.
    #[serde(rename = "CreateTime", skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,

    /// The time when the style was last modified.
    #[serde(rename = "LastModifyTime", skip_serializing_if = "Option::is_none")]
    pub last_modify_time: Option<String>,

    /// The category of this style. Valid values: image, document, video.
    #[serde(rename = "Category", skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// The style name.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The content of the style.
    #[serde(rename = "Content", skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteStyleRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the image style.
    #[field(type = "query", rename = "styleName")]
    pub style_name: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteStyleResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes an image style from a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteStyleRequest` containing the bucket name and
    ///   the name of the image style to delete.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::DeleteStyleRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteStyleRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     style_name: Some("my-style".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_style(&request).await {
    ///     Ok(result) => {
    ///         println!("Style deleted: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete style: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_style(
        &self,
        request: &DeleteStyleRequest,
    ) -> Result<DeleteStyleResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteStyle".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            parameters: [("style", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["style".to_string(), "styleName".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteStyleResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{PutStyleRequest, StyleContent as PutStyleContent};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_style_info_serde_round_trip() {
        let info = StyleInfo {
            create_time: Some("2024-01-01T00:00:00.000Z".to_string()),
            last_modify_time: Some("2024-01-02T00:00:00.000Z".to_string()),
            category: Some("image".to_string()),
            name: Some("style-1".to_string()),
            content: Some("image/resize,w_100".to_string()),
        };

        let xml = quick_xml::se::to_string_with_root("Style", &info).unwrap();
        assert!(xml.contains("<Name>style-1</Name>"));
        assert!(xml.contains("<Category>image</Category>"));

        let parsed: StyleInfo = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.name.as_deref(), Some("style-1"));
        assert_eq!(parsed.category.as_deref(), Some("image"));
        assert_eq!(parsed.content.as_deref(), Some("image/resize,w_100"));
        assert_eq!(
            parsed.create_time.as_deref(),
            Some("2024-01-01T00:00:00.000Z")
        );
        assert_eq!(
            parsed.last_modify_time.as_deref(),
            Some("2024-01-02T00:00:00.000Z")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_style() {
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

        let style_name = crate::test_utils::generate_unique_object_name("style");

        // Prepare a style to delete
        let put_result = client
            .put_style(&PutStyleRequest {
                bucket: config.bucket.clone(),
                style_name: Some(style_name.clone()),
                style: PutStyleContent {
                    content: Some("image/resize,w_100".to_string()),
                },
                ..Default::default()
            })
            .await;
        assert!(put_result.is_ok(), "put_style failed: {:?}", put_result.err());

        let result = client
            .delete_style(&DeleteStyleRequest {
                bucket: config.bucket.clone(),
                style_name: Some(style_name),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "delete_style failed: {:?}", result.err());
    }
}
