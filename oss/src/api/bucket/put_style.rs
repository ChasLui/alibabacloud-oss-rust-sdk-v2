use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::delete_style::StyleContent;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationOutput, BodyContent, OperationInput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutStyleRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the image style.
    #[field(type = "query", rename = "styleName")]
    pub style_name: Option<String>,

    /// The category of the style.
    #[field(type = "query", rename = "category")]
    pub category: Option<String>,

    /// The container that stores the content information about the image style.
    pub style: StyleContent,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutStyleResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Adds an image style to a bucket. An image style contains one or more
    /// image processing parameters.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutStyleRequest` containing the bucket name, the
    ///   style name and the style content.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{PutStyleRequest, StyleContent};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutStyleRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     style_name: Some("my-style".to_string()),
    ///     category: Some("image".to_string()),
    ///     style: StyleContent {
    ///         content: Some("image/resize,w_100".to_string()),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_style(&request).await {
    ///     Ok(result) => {
    ///         println!("Style added: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put style: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_style(
        &self,
        request: &PutStyleRequest,
    ) -> Result<PutStyleResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutStyle".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("style", "")]
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
            std::rc::Rc::new(vec!["style".to_string(), "styleName".to_string()]),
        );

        let xml_body = quick_xml::se::to_string_with_root("Style", &request.style)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutStyleResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::DeleteStyleRequest;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_style_content_serialize() {
        let style = StyleContent {
            content: Some("image/resize,w_100".to_string()),
        };

        let xml = quick_xml::se::to_string_with_root("Style", &style).unwrap();
        assert!(xml.contains("<Style>"));
        assert!(xml.contains("<Content>image/resize,w_100</Content>"));

        let parsed: StyleContent = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.content.as_deref(), Some("image/resize,w_100"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_style() {
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

        let result = client
            .put_style(&PutStyleRequest {
                bucket: config.bucket.clone(),
                style_name: Some(style_name.clone()),
                category: Some("image".to_string()),
                style: StyleContent {
                    content: Some("image/resize,w_100".to_string()),
                },
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "put_style failed: {:?}", result.err());

        // Clean up
        let _ = client
            .delete_style(&DeleteStyleRequest {
                bucket: config.bucket.clone(),
                style_name: Some(style_name),
                ..Default::default()
            })
            .await;
    }
}
