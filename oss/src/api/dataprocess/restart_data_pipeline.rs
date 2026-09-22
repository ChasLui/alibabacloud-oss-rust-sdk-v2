use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct RestartDataPipelineRequest {
    /// The name of the data pipeline.
    #[field(type = "query", rename = "dataPipelineName")]
    pub data_pipeline_name: String,

    pub common: RequestCommon,
}

impl RestartDataPipelineRequest {
    /// Creates a request that restarts a data pipeline.
    pub fn new(data_pipeline_name: &str) -> Self {
        RestartDataPipelineRequest {
            data_pipeline_name: data_pipeline_name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct RestartDataPipelineResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Restarts a data pipeline.
    ///
    /// # Arguments
    ///
    /// * `request` - The `RestartDataPipelineRequest` containing the data
    ///   pipeline name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::RestartDataPipelineRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = RestartDataPipelineRequest::new("my-data-pipeline");
    ///
    /// match client.restart_data_pipeline(&request).await {
    ///     Ok(_) => println!("data pipeline restarted"),
    ///     Err(error) => eprintln!("Failed to restart data pipeline: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn restart_data_pipeline(
        &self,
        request: &RestartDataPipelineRequest,
    ) -> Result<RestartDataPipelineResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "RestartDataPipeline".to_string(),
            method: http::Method::POST,
            parameters: [("dataPipeline", ""), ("action", "restartDataPipeline")]
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

        let mut result = RestartDataPipelineResult::default();
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
    fn test_restart_data_pipeline_request_query_map() {
        let request = RestartDataPipelineRequest::new("my-data-pipeline");
        let query = request.query_map();
        assert_eq!(
            query.get("dataPipelineName").map(String::as_str),
            Some("my-data-pipeline")
        );
        assert_eq!(query.len(), 1);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_restart_data_pipeline() {
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
            .restart_data_pipeline(&RestartDataPipelineRequest::new("sdk-test-data-pipeline"))
            .await;
        if let Err(error) = &result {
            eprintln!("restart_data_pipeline rejected: {}", error);
        }
    }
}
