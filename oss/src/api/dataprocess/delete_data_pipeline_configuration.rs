use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteDataPipelineConfigurationRequest {
    /// The name of the data pipeline.
    #[field(type = "query", rename = "dataPipelineName")]
    pub data_pipeline_name: String,

    pub common: RequestCommon,
}

impl DeleteDataPipelineConfigurationRequest {
    /// Creates a request that deletes the configuration of a data pipeline.
    pub fn new(data_pipeline_name: &str) -> Self {
        DeleteDataPipelineConfigurationRequest {
            data_pipeline_name: data_pipeline_name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteDataPipelineConfigurationResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes the configuration of a data pipeline.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteDataPipelineConfigurationRequest` containing
    ///   the data pipeline name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::DeleteDataPipelineConfigurationRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteDataPipelineConfigurationRequest::new("my-data-pipeline");
    ///
    /// match client.delete_data_pipeline_configuration(&request).await {
    ///     Ok(_) => println!("configuration deleted"),
    ///     Err(error) => eprintln!("Failed to delete data pipeline: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn delete_data_pipeline_configuration(
        &self,
        request: &DeleteDataPipelineConfigurationRequest,
    ) -> Result<DeleteDataPipelineConfigurationResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "DeleteDataPipelineConfiguration".to_string(),
            method: http::Method::POST,
            parameters: [
                ("dataPipeline", ""),
                ("action", "deleteDataPipelineConfiguration"),
            ]
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

        let mut result = DeleteDataPipelineConfigurationResult::default();
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
    fn test_delete_data_pipeline_configuration_request_query_map() {
        let request = DeleteDataPipelineConfigurationRequest::new("my-data-pipeline");
        let query = request.query_map();
        assert_eq!(
            query.get("dataPipelineName").map(String::as_str),
            Some("my-data-pipeline")
        );
        assert_eq!(query.len(), 1);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_data_pipeline_configuration() {
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
            .delete_data_pipeline_configuration(&DeleteDataPipelineConfigurationRequest::new(
                "sdk-test-data-pipeline",
            ))
            .await;
        if let Err(error) = &result {
            eprintln!("delete_data_pipeline_configuration rejected: {}", error);
        }
    }
}
