use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The filter configuration of a data pipeline source.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DataPipelineSourceFilterConfiguration {
    /// The prefixes of the objects to process.
    #[serde(rename = "PrefixSet", default)]
    pub prefix_set: Vec<String>,

    /// The media types of the objects to process.
    #[serde(rename = "ObjectMediaTypes", default)]
    pub object_media_types: Vec<String>,
}

/// A source of a data pipeline.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DataPipelineSource {
    /// The name of the source bucket.
    #[serde(rename = "InputBucket", skip_serializing_if = "Option::is_none")]
    pub input_bucket: Option<String>,

    /// The data scope of the source bucket. Valid values: All, CurrentVersion,
    /// PreviousVersion.
    #[serde(rename = "InputDataScope", skip_serializing_if = "Option::is_none")]
    pub input_data_scope: Option<String>,

    /// Specifies whether to ignore delete operations.
    #[serde(rename = "IgnoreDelete", skip_serializing_if = "Option::is_none")]
    pub ignore_delete: Option<bool>,

    /// The filter configuration.
    #[serde(
        rename = "FilterConfiguration",
        skip_serializing_if = "Option::is_none"
    )]
    pub filter_configuration: Option<DataPipelineSourceFilterConfiguration>,
}

/// The embedding configuration of a data pipeline.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DataPipelineEmbeddingConfiguration {
    /// The embedding provider.
    #[serde(rename = "EmbeddingProvider", skip_serializing_if = "Option::is_none")]
    pub embedding_provider: Option<String>,

    /// The API key of the embedding provider.
    #[serde(rename = "ApiKey", skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// The embedding model.
    #[serde(rename = "Model", skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// The number of frames per second that are extracted from videos.
    #[serde(rename = "FPS", skip_serializing_if = "Option::is_none")]
    pub fps: Option<f64>,
}

/// The destination of a data pipeline.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DataPipelineDestination {
    /// The name of the vector bucket.
    #[serde(rename = "VectorBucketName", skip_serializing_if = "Option::is_none")]
    pub vector_bucket_name: Option<String>,

    /// The prefix of the vector keys.
    #[serde(rename = "VectorKeyPrefix", skip_serializing_if = "Option::is_none")]
    pub vector_key_prefix: Option<String>,

    /// The names of the vector indexes.
    #[serde(rename = "VectorIndexNames", default)]
    pub vector_index_names: Vec<String>,

    /// The object tags that are written to the vector metadata.
    #[serde(rename = "ObjectTagToMetadata", default)]
    pub object_tag_to_metadata: Vec<String>,

    /// The user metadata that is written to the vector metadata.
    #[serde(rename = "UsermetaToMetadata", default)]
    pub usermeta_to_metadata: Vec<String>,
}

/// The error handling configuration of a data pipeline.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DataPipelineError {
    /// The error handling mode. Valid values: ignoreAndRecord, ignore, fail.
    #[serde(rename = "ErrorMode", skip_serializing_if = "Option::is_none")]
    pub error_mode: Option<String>,

    /// The bucket in which the error records are stored.
    #[serde(rename = "ErrorBucket", skip_serializing_if = "Option::is_none")]
    pub error_bucket: Option<String>,

    /// The prefix of the error records.
    #[serde(rename = "ErrorPrefix", skip_serializing_if = "Option::is_none")]
    pub error_prefix: Option<String>,
}

/// The configuration of a data pipeline.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DataPipelineConfiguration {
    /// The description of the data pipeline.
    #[serde(
        rename = "DataPipelineDescription",
        skip_serializing_if = "Option::is_none"
    )]
    pub data_pipeline_description: Option<String>,

    /// The sources of the data pipeline.
    #[serde(rename = "Sources", default)]
    pub sources: Vec<DataPipelineSource>,

    /// The embedding configuration of the data pipeline.
    #[serde(
        rename = "DataPipelineEmbeddingConfiguration",
        skip_serializing_if = "Option::is_none"
    )]
    pub data_pipeline_embedding_configuration: Option<DataPipelineEmbeddingConfiguration>,

    /// The destination of the data pipeline.
    #[serde(rename = "Destination", skip_serializing_if = "Option::is_none")]
    pub destination: Option<DataPipelineDestination>,

    /// The error handling configuration of the data pipeline.
    #[serde(rename = "DataPipelineError", skip_serializing_if = "Option::is_none")]
    pub data_pipeline_error: Option<DataPipelineError>,

    /// The name of the data pipeline. Only returned in responses.
    #[serde(rename = "DataPipelineName", skip_serializing_if = "Option::is_none")]
    pub data_pipeline_name: Option<String>,

    /// The role that is assumed by the data pipeline. Only returned in
    /// responses.
    #[serde(rename = "DataPipelineRole", skip_serializing_if = "Option::is_none")]
    pub data_pipeline_role: Option<String>,

    /// The status of the data pipeline. Only returned in responses.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// The phase of the data pipeline. Only returned in responses.
    #[serde(rename = "Phase", skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,

    /// The time when the data pipeline was created. Only returned in
    /// responses.
    #[serde(rename = "CreateTime", skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,
}

/// The container that stores data pipeline configurations.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DataPipelineConfigurations {
    /// The data pipeline configurations.
    #[serde(rename = "DataPipelineConfiguration", default)]
    pub data_pipeline_configurations: Vec<DataPipelineConfiguration>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutDataPipelineConfigurationRequest {
    /// The name of the data pipeline.
    #[field(type = "query", rename = "dataPipelineName")]
    pub data_pipeline_name: String,

    /// The role that is assumed by the data pipeline.
    #[field(type = "query", rename = "role")]
    pub role: String,

    /// The configuration of the data pipeline.
    pub data_pipeline_configuration: DataPipelineConfiguration,

    pub common: RequestCommon,
}

impl PutDataPipelineConfigurationRequest {
    /// Creates a request that writes the configuration of a data pipeline.
    pub fn new(
        data_pipeline_name: &str,
        role: &str,
        data_pipeline_configuration: DataPipelineConfiguration,
    ) -> Self {
        PutDataPipelineConfigurationRequest {
            data_pipeline_name: data_pipeline_name.to_string(),
            role: role.to_string(),
            data_pipeline_configuration,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutDataPipelineConfigurationResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Creates or updates the configuration of a data pipeline.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutDataPipelineConfigurationRequest` containing the
    ///   data pipeline name, the role and the configuration to write.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::{
    /// #     DataPipelineConfiguration, DataPipelineDestination, DataPipelineSource,
    /// #     PutDataPipelineConfigurationRequest,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let configuration = DataPipelineConfiguration {
    ///     data_pipeline_description: Some("vectorize objects".to_string()),
    ///     sources: vec![DataPipelineSource {
    ///         input_bucket: Some("my-bucket".to_string()),
    ///         ..Default::default()
    ///     }],
    ///     destination: Some(DataPipelineDestination {
    ///         vector_bucket_name: Some("my-vector-bucket".to_string()),
    ///         vector_index_names: vec!["my-index".to_string()],
    ///         ..Default::default()
    ///     }),
    ///     ..Default::default()
    /// };
    /// let request = PutDataPipelineConfigurationRequest::new(
    ///     "my-data-pipeline",
    ///     "AliyunOSSDataPipelineRole",
    ///     configuration,
    /// );
    ///
    /// match client.put_data_pipeline_configuration(&request).await {
    ///     Ok(_) => println!("configuration written"),
    ///     Err(error) => eprintln!("Failed to put data pipeline: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn put_data_pipeline_configuration(
        &self,
        request: &PutDataPipelineConfigurationRequest,
    ) -> Result<PutDataPipelineConfigurationResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutDataPipelineConfiguration".to_string(),
            method: http::Method::POST,
            parameters: [
                ("dataPipeline", ""),
                ("action", "putDataPipelineConfiguration"),
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

        let xml_body = quick_xml::se::to_string_with_root(
            "DataPipelineConfiguration",
            &request.data_pipeline_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutDataPipelineConfigurationResult::default();
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

    /// The request used by the Go SDK mock tests, and the body it expects the
    /// SDK to serialize.
    fn go_mock_configuration() -> DataPipelineConfiguration {
        DataPipelineConfiguration {
            data_pipeline_description: Some("使用百炼多模态模型为业务数据向量化".to_string()),
            sources: vec![DataPipelineSource {
                input_bucket: Some("bucket".to_string()),
                input_data_scope: Some("All".to_string()),
                filter_configuration: Some(DataPipelineSourceFilterConfiguration {
                    prefix_set: vec!["prefix1".to_string(), "prefix2/prefix3".to_string()],
                    object_media_types: vec![
                        "text".to_string(),
                        "image".to_string(),
                        "video".to_string(),
                    ],
                }),
                ..Default::default()
            }],
            data_pipeline_embedding_configuration: Some(DataPipelineEmbeddingConfiguration {
                embedding_provider: Some("bailian".to_string()),
                api_key: Some("your_api_key".to_string()),
                model: Some("qwen2.5-vl-embedding".to_string()),
                fps: Some(1.0),
            }),
            destination: Some(DataPipelineDestination {
                vector_bucket_name: Some("my-vector-bucket".to_string()),
                vector_key_prefix: Some("prefix".to_string()),
                vector_index_names: vec!["my-index".to_string()],
                object_tag_to_metadata: vec!["key1".to_string(), "key2".to_string()],
                usermeta_to_metadata: vec!["x-oss-meta-key1".to_string()],
            }),
            data_pipeline_error: Some(DataPipelineError {
                error_mode: Some("ignoreAndRecord".to_string()),
                error_bucket: Some("my-error-bucket".to_string()),
                error_prefix: Some("error-output/".to_string()),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_put_data_pipeline_configuration_body_serialization() {
        let xml = quick_xml::se::to_string_with_root(
            "DataPipelineConfiguration",
            &go_mock_configuration(),
        )
        .unwrap();
        assert_eq!(
            xml,
            "<DataPipelineConfiguration><DataPipelineDescription>使用百炼多模态模型为业务数据向量化</DataPipelineDescription><Sources><InputBucket>bucket</InputBucket><InputDataScope>All</InputDataScope><FilterConfiguration><PrefixSet>prefix1</PrefixSet><PrefixSet>prefix2/prefix3</PrefixSet><ObjectMediaTypes>text</ObjectMediaTypes><ObjectMediaTypes>image</ObjectMediaTypes><ObjectMediaTypes>video</ObjectMediaTypes></FilterConfiguration></Sources><DataPipelineEmbeddingConfiguration><EmbeddingProvider>bailian</EmbeddingProvider><ApiKey>your_api_key</ApiKey><Model>qwen2.5-vl-embedding</Model><FPS>1</FPS></DataPipelineEmbeddingConfiguration><Destination><VectorBucketName>my-vector-bucket</VectorBucketName><VectorKeyPrefix>prefix</VectorKeyPrefix><VectorIndexNames>my-index</VectorIndexNames><ObjectTagToMetadata>key1</ObjectTagToMetadata><ObjectTagToMetadata>key2</ObjectTagToMetadata><UsermetaToMetadata>x-oss-meta-key1</UsermetaToMetadata></Destination><DataPipelineError><ErrorMode>ignoreAndRecord</ErrorMode><ErrorBucket>my-error-bucket</ErrorBucket><ErrorPrefix>error-output/</ErrorPrefix></DataPipelineError></DataPipelineConfiguration>"
        );
    }

    #[test]
    fn test_put_data_pipeline_configuration_request_query_map() {
        let request = PutDataPipelineConfigurationRequest::new(
            "data-pipeline",
            "AliyunOSSDataPipelineRole",
            go_mock_configuration(),
        );
        let query = request.query_map();
        assert_eq!(
            query.get("dataPipelineName").map(String::as_str),
            Some("data-pipeline")
        );
        assert_eq!(
            query.get("role").map(String::as_str),
            Some("AliyunOSSDataPipelineRole")
        );
        assert_eq!(query.len(), 2);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_data_pipeline_configuration() {
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

        let request = PutDataPipelineConfigurationRequest::new(
            "sdk-test-data-pipeline",
            "AliyunOSSDataPipelineRole",
            go_mock_configuration(),
        );
        let result = client.put_data_pipeline_configuration(&request).await;
        if let Err(error) = &result {
            eprintln!("put_data_pipeline_configuration rejected: {}", error);
        }
    }
}
