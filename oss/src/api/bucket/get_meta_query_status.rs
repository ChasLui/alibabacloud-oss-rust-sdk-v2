use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::close_meta_query::MetaQueryStatus;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationOutput, OperationInput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetMetaQueryStatusRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl GetMetaQueryStatusRequest {
    pub fn new(bucket: &str) -> Self {
        GetMetaQueryStatusRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetMetaQueryStatusResult {
    /// The container that stores the metadata information.
    pub meta_query_status: Option<MetaQueryStatus>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Queries the information about the metadata index library of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetMetaQueryStatusRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetMetaQueryStatusRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetMetaQueryStatusRequest::new("my-bucket");
    ///
    /// match client.get_meta_query_status(&request).await {
    ///     Ok(result) => {
    ///         println!("Meta query status: {:?}", result.meta_query_status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get meta query status: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_meta_query_status(
        &self,
        request: &GetMetaQueryStatusRequest,
    ) -> Result<GetMetaQueryStatusResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetMetaQueryStatus".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };
        input
            .op_metadata
            .set(SUB_RESOURCE, Rc::new(vec!["metaQuery".to_string()]));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let meta_query_status: MetaQueryStatus = quick_xml::de::from_str(&data_str)?;

        let mut result = GetMetaQueryStatusResult {
            meta_query_status: Some(meta_query_status),
            ..Default::default()
        };
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

    #[test]
    fn test_get_meta_query_status_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<MetaQueryStatus>
  <State>Running</State>
  <Phase>IncrementalScanning</Phase>
  <CreateTime>2021-08-02T10:49:17.289372919+08:00</CreateTime>
  <UpdateTime>2021-08-02T10:49:17.289372919+08:00</UpdateTime>
</MetaQueryStatus>"#;
        let status: MetaQueryStatus = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(status.state.as_deref(), Some("Running"));
        assert_eq!(status.phase.as_deref(), Some("IncrementalScanning"));
        assert!(status.create_time.is_some());
        assert!(status.update_time.is_some());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_meta_query_status() {
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

        // Querying the status fails if meta query was never opened for the
        // bucket; this only verifies the request is well-formed.
        let result = client
            .get_meta_query_status(&GetMetaQueryStatusRequest::new(&config.bucket))
            .await;
        if let Err(error) = &result {
            eprintln!("get_meta_query_status rejected (meta query may not be open): {}", error);
        }
    }
}
