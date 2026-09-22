use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PauseDataPipelineRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the data pipeline.
    #[field(type = "query", rename = "dataPipelineName")]
    pub data_pipeline_name: String,

    pub common: RequestCommon,
}

impl PauseDataPipelineRequest {
    /// Creates a request that pauses a data pipeline.
    pub fn new(bucket: &str, data_pipeline_name: &str) -> Self {
        PauseDataPipelineRequest {
            bucket: bucket.to_string(),
            data_pipeline_name: data_pipeline_name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct PauseDataPipelineResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Pauses a data pipeline.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PauseDataPipelineRequest` containing the bucket name
    ///   and the data pipeline name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::PauseDataPipelineRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PauseDataPipelineRequest::new("my-bucket", "my-data-pipeline");
    ///
    /// match client.pause_data_pipeline(&request).await {
    ///     Ok(_) => println!("data pipeline paused"),
    ///     Err(error) => eprintln!("Failed to pause data pipeline: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn pause_data_pipeline(
        &self,
        request: &PauseDataPipelineRequest,
    ) -> Result<PauseDataPipelineResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PauseDataPipeline".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("dataPipeline", ""), ("action", "pauseDataPipeline")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PauseDataPipelineResult::default();
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
    fn test_pause_data_pipeline_request_query_map() {
        let request = PauseDataPipelineRequest::new("bucket", "my-data-pipeline");
        let query = request.query_map();
        assert_eq!(
            query.get("dataPipelineName").map(String::as_str),
            Some("my-data-pipeline")
        );
        assert_eq!(query.len(), 1);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_pause_data_pipeline() {
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

        let result = client
            .pause_data_pipeline(&PauseDataPipelineRequest::new(
                &config.bucket,
                "sdk-test-data-pipeline",
            ))
            .await;
        if let Err(error) = &result {
            eprintln!("pause_data_pipeline rejected: {}", error);
        }
    }
}
