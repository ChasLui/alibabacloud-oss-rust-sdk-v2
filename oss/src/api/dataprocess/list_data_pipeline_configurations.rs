use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::put_data_pipeline_configuration::DataPipelineConfigurations;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct ListDataPipelineConfigurationsRequest {
    /// The maximum number of data pipelines to return.
    #[field(type = "query", rename = "maxResults")]
    pub max_results: Option<i64>,

    /// The prefix contained in the names of the returned data pipelines.
    #[field(type = "query", rename = "prefix")]
    pub prefix: Option<String>,

    /// The token that is used to retrieve the next page of results.
    #[field(type = "query", rename = "nextToken")]
    pub next_token: Option<String>,

    pub common: RequestCommon,
}

impl ListDataPipelineConfigurationsRequest {
    /// Creates a request that lists the data pipelines of the current account.
    pub fn new() -> Self {
        ListDataPipelineConfigurationsRequest::default()
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct ListDataPipelineConfigurationsResult {
    /// The data pipeline configurations.
    #[serde(
        rename = "DataPipelineConfigurations",
        skip_serializing_if = "Option::is_none"
    )]
    pub data_pipeline_configurations: Option<DataPipelineConfigurations>,

    /// The token that is used to retrieve the next page of results.
    #[serde(rename = "NextToken", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists the data pipelines of the current account.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListDataPipelineConfigurationsRequest` containing the
    ///   optional pagination and prefix filters.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::ListDataPipelineConfigurationsRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListDataPipelineConfigurationsRequest {
    ///     max_results: Some(100),
    ///     ..Default::default()
    /// };
    ///
    /// match client.list_data_pipeline_configurations(&request).await {
    ///     Ok(result) => println!("data pipelines: {:?}", result.data_pipeline_configurations),
    ///     Err(error) => eprintln!("Failed to list data pipelines: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn list_data_pipeline_configurations(
        &self,
        request: &ListDataPipelineConfigurationsRequest,
    ) -> Result<ListDataPipelineConfigurationsResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "ListDataPipelineConfigurations".to_string(),
            method: http::Method::POST,
            parameters: [
                ("dataPipeline", ""),
                ("action", "listDataPipelineConfigurations"),
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

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: ListDataPipelineConfigurationsResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_list_data_pipeline_configurations_request_query_map() {
        let request = ListDataPipelineConfigurationsRequest {
            max_results: Some(100),
            prefix: Some("prefix".to_string()),
            next_token: Some("next-token".to_string()),
            ..Default::default()
        };
        let query = request.query_map();
        assert_eq!(query.get("maxResults").map(String::as_str), Some("100"));
        assert_eq!(query.get("prefix").map(String::as_str), Some("prefix"));
        assert_eq!(
            query.get("nextToken").map(String::as_str),
            Some("next-token")
        );
    }

    #[test]
    fn test_list_data_pipeline_configurations_result_deserialize() {
        let xml = r#"<ListDataPipelineConfigurationsResult>
  <DataPipelineConfigurations>
    <DataPipelineConfiguration>
      <DataPipelineName>my-data-pipeline</DataPipelineName>
      <DataPipelineDescription>使用百炼多模态模型为业务数据向量化</DataPipelineDescription>
      <DataPipelineRole>my-data-pipeline-role</DataPipelineRole>
      <Status>Running</Status>
      <Phase>IncrementalScanning</Phase>
      <Sources>
          <InputBucket>my-bucket</InputBucket>
          <InputDataScope>All</InputDataScope>
          <IgnoreDelete>true</IgnoreDelete>
          <FilterConfiguration>
              <PrefixSet>prefix1/</PrefixSet>
              <PrefixSet>prefix2/prefix3/</PrefixSet>
              <ObjectMediaTypes>text</ObjectMediaTypes>
          </FilterConfiguration>
      </Sources>
      <Destination>
          <VectorBucketName>my-vector-bucket</VectorBucketName>
          <VectorIndexNames>my-index</VectorIndexNames>
      </Destination>
    </DataPipelineConfiguration>
  </DataPipelineConfigurations>
  <NextToken>xxx</NextToken>
</ListDataPipelineConfigurationsResult>"#;
        let result: ListDataPipelineConfigurationsResult = quick_xml::de::from_str(xml).unwrap();
        let configurations = result.data_pipeline_configurations.unwrap();
        assert_eq!(configurations.data_pipeline_configurations.len(), 1);
        let configuration = &configurations.data_pipeline_configurations[0];
        assert_eq!(
            configuration.data_pipeline_name.as_deref(),
            Some("my-data-pipeline")
        );
        assert_eq!(configuration.status.as_deref(), Some("Running"));
        assert_eq!(
            configuration.sources[0].input_bucket.as_deref(),
            Some("my-bucket")
        );
        assert_eq!(result.next_token.as_deref(), Some("xxx"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_data_pipeline_configurations() {
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
            .list_data_pipeline_configurations(&ListDataPipelineConfigurationsRequest::new())
            .await;
        if let Err(error) = &result {
            eprintln!("list_data_pipeline_configurations rejected: {}", error);
        }
    }
}
