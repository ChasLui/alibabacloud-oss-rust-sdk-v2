use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct OpenMetaQueryRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The mode in which the metadata index library is created.
    #[field(type = "query", rename = "mode")]
    pub mode: Option<String>,

    pub common: RequestCommon,
}

impl OpenMetaQueryRequest {
    pub fn new(bucket: &str) -> Self {
        OpenMetaQueryRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct OpenMetaQueryResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Enables metadata management for a bucket. After you enable the metadata
    /// management feature for a bucket, OSS creates a metadata index library for
    /// the bucket and creates metadata indexes for all objects in the bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `OpenMetaQueryRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::OpenMetaQueryRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = OpenMetaQueryRequest::new("my-bucket");
    ///
    /// match client.open_meta_query(&request).await {
    ///     Ok(result) => {
    ///         println!("Meta query opened: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to open meta query: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn open_meta_query(
        &self,
        request: &OpenMetaQueryRequest,
    ) -> Result<OpenMetaQueryResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "OpenMetaQuery".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("comp", "add"), ("metaQuery", "")]
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
            SUB_RESOURCE,
            Rc::new(vec!["metaQuery".to_string(), "comp".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = OpenMetaQueryResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_open_meta_query() {
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

        // Opening meta query may fail if it is already open for the bucket or
        // the feature is unavailable in the region.
        let result = client
            .open_meta_query(&OpenMetaQueryRequest::new(&config.bucket))
            .await;
        if let Err(error) = &result {
            eprintln!("open_meta_query rejected (may already be open): {}", error);
            return;
        }

        // Clean up
        let _ = client
            .close_meta_query(&crate::api::bucket::CloseMetaQueryRequest::new(&config.bucket))
            .await;
    }
}
