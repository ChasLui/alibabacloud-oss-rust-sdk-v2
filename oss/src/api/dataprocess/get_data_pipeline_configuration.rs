use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::put_data_pipeline_configuration::DataPipelineConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetDataPipelineConfigurationRequest {
    /// The name of the data pipeline.
    #[field(type = "query", rename = "dataPipelineName")]
    pub data_pipeline_name: String,

    pub common: RequestCommon,
}

impl GetDataPipelineConfigurationRequest {
    /// Creates a request that queries the configuration of a data pipeline.
    pub fn new(data_pipeline_name: &str) -> Self {
        GetDataPipelineConfigurationRequest {
            data_pipeline_name: data_pipeline_name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetDataPipelineConfigurationResult {
    /// The configuration of the data pipeline.
    pub data_pipeline_configuration: Option<DataPipelineConfiguration>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the configuration of a data pipeline.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetDataPipelineConfigurationRequest` containing the
    ///   data pipeline name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::GetDataPipelineConfigurationRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetDataPipelineConfigurationRequest::new("my-data-pipeline");
    ///
    /// match client.get_data_pipeline_configuration(&request).await {
    ///     Ok(result) => println!("configuration: {:?}", result.data_pipeline_configuration),
    ///     Err(error) => eprintln!("Failed to get data pipeline: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn get_data_pipeline_configuration(
        &self,
        request: &GetDataPipelineConfigurationRequest,
    ) -> Result<GetDataPipelineConfigurationResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetDataPipelineConfiguration".to_string(),
            method: http::Method::POST,
            parameters: [
                ("dataPipeline", ""),
                ("action", "getDataPipelineConfiguration"),
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
        let configuration: DataPipelineConfiguration = quick_xml::de::from_str(&data_str)?;

        let mut result = GetDataPipelineConfigurationResult {
            data_pipeline_configuration: Some(configuration),
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
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_get_data_pipeline_configuration_request_query_map() {
        let request = GetDataPipelineConfigurationRequest::new("my-data-pipeline");
        let query = request.query_map();
        assert_eq!(
            query.get("dataPipelineName").map(String::as_str),
            Some("my-data-pipeline")
        );
        assert_eq!(query.len(), 1);
    }

    #[test]
    fn test_get_data_pipeline_configuration_result_deserialize() {
        let xml = r#"<DataPipelineConfiguration>
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
            <ObjectMediaTypes>image</ObjectMediaTypes>
            <ObjectMediaTypes>video</ObjectMediaTypes>
        </FilterConfiguration>
    </Sources>
    <DataPipelineEmbeddingConfiguration>
        <EmbeddingProvider>bailian</EmbeddingProvider>
        <ApiKey>sk-12345678901234556</ApiKey>
        <Model>qwen2.5-vl-embedding</Model>
        <FPS>1</FPS>
    </DataPipelineEmbeddingConfiguration>
    <Destination>
        <VectorBucketName>my-vector-bucket</VectorBucketName>
        <VectorIndexNames>my-index</VectorIndexNames>
        <VectorKeyPrefix></VectorKeyPrefix>
        <ObjectTagToMetadata>key1</ObjectTagToMetadata>
        <ObjectTagToMetadata>key2</ObjectTagToMetadata>
        <UsermetaToMetadata>x-oss-meta-key1</UsermetaToMetadata>
    </Destination>
    <DataPipelineError>
        <ErrorMode>ignoreAndRecord</ErrorMode>
        <ErrorBucket>my-error-bucket</ErrorBucket>
        <ErrorPrefix>error-output/</ErrorPrefix>
    </DataPipelineError>
    <CreateTime>2021-06-29T14:50:13.011643661+08:00</CreateTime>
</DataPipelineConfiguration>"#;
        let configuration: DataPipelineConfiguration = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(
            configuration.data_pipeline_name.as_deref(),
            Some("my-data-pipeline")
        );
        assert_eq!(configuration.status.as_deref(), Some("Running"));
        assert_eq!(configuration.phase.as_deref(), Some("IncrementalScanning"));

        let sources = &configuration.sources;
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].input_bucket.as_deref(), Some("my-bucket"));
        assert_eq!(sources[0].ignore_delete, Some(true));
        let filter = sources[0].filter_configuration.as_ref().unwrap();
        assert_eq!(filter.prefix_set, vec!["prefix1/", "prefix2/prefix3/"]);
        assert_eq!(filter.object_media_types.len(), 3);

        let destination = configuration.destination.as_ref().unwrap();
        assert_eq!(
            destination.vector_bucket_name.as_deref(),
            Some("my-vector-bucket")
        );
        assert_eq!(destination.vector_index_names, vec!["my-index"]);
        assert_eq!(destination.object_tag_to_metadata, vec!["key1", "key2"]);

        let error = configuration.data_pipeline_error.as_ref().unwrap();
        assert_eq!(error.error_mode.as_deref(), Some("ignoreAndRecord"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_data_pipeline_configuration() {
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
            .get_data_pipeline_configuration(&GetDataPipelineConfigurationRequest::new(
                "sdk-test-data-pipeline",
            ))
            .await;
        if let Err(error) = &result {
            eprintln!("get_data_pipeline_configuration rejected: {}", error);
        }
    }
}
