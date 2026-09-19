use crate::HTTP_HEADER_CONTENT_TYPE;
use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::delete_style::StyleInfo;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::OperationInput;
use crate::OperationOutput;

#[derive(Debug, Default, OssRequestModel)]
pub struct GetStyleRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the image style.
    #[field(type = "query", rename = "styleName")]
    pub style_name: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetStyleResult {
    /// The container that stores the information about the image style.
    pub style: Option<StyleInfo>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Queries the information about an image style of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetStyleRequest` containing the bucket name and the
    ///   name of the image style.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetStyleRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetStyleRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     style_name: Some("my-style".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_style(&request).await {
    ///     Ok(result) => {
    ///         println!("Style: {:?}", result.style);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get style: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_style(
        &self,
        request: &GetStyleRequest,
    ) -> Result<GetStyleResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetStyle".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("style", "")]
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
            std::rc::Rc::new(vec!["style".to_string(), "styleName".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let style: StyleInfo = quick_xml::de::from_str(&data_str)?;

        let mut result = GetStyleResult {
            style: Some(style),
            ..Default::default()
        };
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{DeleteStyleRequest, PutStyleRequest, StyleContent};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_get_style_body_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Style>
  <CreateTime>2024-01-01T00:00:00.000Z</CreateTime>
  <LastModifyTime>2024-01-02T00:00:00.000Z</LastModifyTime>
  <Category>image</Category>
  <Name>style-1</Name>
  <Content>image/resize,w_100</Content>
</Style>"#;
        let style: StyleInfo = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(style.name.as_deref(), Some("style-1"));
        assert_eq!(style.content.as_deref(), Some("image/resize,w_100"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_style() {
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

        // Prepare a style to get
        let put_result = client
            .put_style(&PutStyleRequest {
                bucket: config.bucket.clone(),
                style_name: Some(style_name.clone()),
                style: StyleContent {
                    content: Some("image/resize,w_100".to_string()),
                },
                ..Default::default()
            })
            .await;
        assert!(put_result.is_ok(), "put_style failed: {:?}", put_result.err());

        let result = client
            .get_style(&GetStyleRequest {
                bucket: config.bucket.clone(),
                style_name: Some(style_name.clone()),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "get_style failed: {:?}", result.err());
        let style = result.unwrap().style.unwrap();
        assert_eq!(style.name.as_deref(), Some(style_name.as_str()));
        assert_eq!(style.content.as_deref(), Some("image/resize,w_100"));

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
