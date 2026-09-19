use crate::HTTP_HEADER_CONTENT_TYPE;
use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::delete_style::StyleInfo;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::OperationInput;
use crate::OperationOutput;

#[derive(Debug, Default, OssRequestModel)]
pub struct ListStyleRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl ListStyleRequest {
    pub fn new(bucket: &str) -> Self {
        Self {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct ListStyleResult {
    /// The list of styles.
    #[serde(rename = "Style", default)]
    pub styles: Vec<StyleInfo>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries all image styles that are created for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListStyleRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::ListStyleRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListStyleRequest::new("my-bucket");
    ///
    /// match client.list_style(&request).await {
    ///     Ok(result) => {
    ///         println!("Styles: {:?}", result.styles.len());
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to list styles: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn list_style(
        &self,
        request: &ListStyleRequest,
    ) -> Result<ListStyleResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListStyle".to_string(),
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
            std::rc::Rc::new(vec!["style".to_string()]),
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
        let mut result: ListStyleResult = quick_xml::de::from_str(&data_str)?;

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
    fn test_list_style_result_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<StyleList>
  <Style>
    <CreateTime>2024-01-01T00:00:00.000Z</CreateTime>
    <LastModifyTime>2024-01-02T00:00:00.000Z</LastModifyTime>
    <Category>image</Category>
    <Name>style-1</Name>
    <Content>image/resize,w_100</Content>
  </Style>
  <Style>
    <CreateTime>2024-01-03T00:00:00.000Z</CreateTime>
    <LastModifyTime>2024-01-04T00:00:00.000Z</LastModifyTime>
    <Category>image</Category>
    <Name>style-2</Name>
    <Content>image/crop,w_200</Content>
  </Style>
</StyleList>"#;
        let result: ListStyleResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.styles.len(), 2);
        assert_eq!(result.styles[0].name.as_deref(), Some("style-1"));
        assert_eq!(result.styles[1].content.as_deref(), Some("image/crop,w_200"));

        // Empty list
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<StyleList/>"#;
        let result: ListStyleResult = quick_xml::de::from_str(xml).unwrap();
        assert!(result.styles.is_empty());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_style() {
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

        // Prepare a style to list
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
            .list_style(&ListStyleRequest::new(&config.bucket))
            .await;
        assert!(result.is_ok(), "list_style failed: {:?}", result.err());
        assert_eq!(result.unwrap().common.status, http::StatusCode::OK);

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
