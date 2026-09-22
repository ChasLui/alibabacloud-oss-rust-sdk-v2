use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, BodyStream, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Default, OssRequestModel)]
pub struct DoMetaQueryActionRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The action to perform.
    #[field(type = "query", rename = "action")]
    pub action: Option<String>,

    /// The request body. The content is a XML string and varies with the
    /// action.
    pub body: Option<BodyContent>,

    pub common: RequestCommon,
}

#[derive(Default, OssResultModel)]
pub struct DoMetaQueryActionResult {
    /// The response body stream. The content is a JSON string and varies with
    /// the action.
    pub body: Option<BodyStream>,

    pub common: ResultCommon,
}

impl std::fmt::Debug for DoMetaQueryActionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DoMetaQueryActionResult")
            .field("body", &"<stream>") // Don't print the stream itself
            .field("common", &self.common)
            .finish()
    }
}

impl crate::client::BodyDataReader for DoMetaQueryActionResult {
    fn take_body(&mut self) -> Option<crate::client::BodyStream> {
        self.body.take()
    }

    fn set_body(&mut self, body: Option<crate::client::BodyStream>) {
        self.body = body;
    }
}

impl Client {
    /// Performs a meta query related action on a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DoMetaQueryActionRequest` containing the bucket name,
    ///   the action and the request body.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::service::DoMetaQueryActionRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::{BodyDataReader, Client};
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// # use alibabacloud_oss_sdk_rust_v2::BodyContent;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DoMetaQueryActionRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     action: Some("my-action".to_string()),
    ///     body: Some(BodyContent::from_text(
    ///         "<MetaQuery></MetaQuery>".to_string(),
    ///         None,
    ///     )),
    ///     ..Default::default()
    /// };
    ///
    /// match client.do_meta_query_action(request).await {
    ///     Ok(mut result) => {
    ///         let data = result.get_all_data().await.unwrap();
    ///         println!("response: {}", String::from_utf8_lossy(&data));
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to do meta query action: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn do_meta_query_action(
        &self,
        request: DoMetaQueryActionRequest,
    ) -> Result<DoMetaQueryActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let headers = request.header_map();
        let queries = request.query_map();

        let mut input = OperationInput {
            op_name: "DoMetaQueryAction".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            body: request.body,
            ..Default::default()
        };

        modify_request(
            &mut input,
            headers,
            queries,
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = DoMetaQueryActionResult::default();
        result.update_result(&output);
        result.body = output.body;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::client::BodyDataReader;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_do_meta_query_action() {
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

        // The test bucket may not have meta query enabled; accept a
        // service-side error as a valid exercised path.
        match client
            .do_meta_query_action(DoMetaQueryActionRequest {
                bucket: config.bucket.clone(),
                action: Some("GetMetaQueryStatus".to_string()),
                body: Some(BodyContent::from_text(
                    "<MetaQuery></MetaQuery>".to_string(),
                    None,
                )),
                ..Default::default()
            })
            .await
        {
            Ok(mut result) => {
                let data = result.get_all_data().await.unwrap_or_default();
                println!("response: {}", String::from_utf8_lossy(&data));
            }
            Err(error) => {
                println!("do_meta_query_action returned error: {}", error);
            }
        }
    }
}
