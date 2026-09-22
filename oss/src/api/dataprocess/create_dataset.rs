use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize, Serializer};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// A single workflow parameter of a dataset.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WorkflowParameter {
    /// The name of the workflow parameter.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The value of the workflow parameter.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// The description of the workflow parameter.
    #[serde(rename = "Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// The container that stores workflow parameters.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WorkflowParameters {
    /// The workflow parameters.
    #[serde(rename = "WorkflowParameter", default)]
    pub workflow_parameters: Vec<WorkflowParameter>,
}

impl WorkflowParameters {
    /// Serializes the workflow parameters into the JSON array expected by the
    /// `workflowParameters` query parameter.
    pub fn to_parameter_value(&self) -> String {
        serde_json::to_string(&self.workflow_parameters).unwrap_or_default()
    }
}

/// A label item of an insights rule.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LabelItem {
    /// The name of the label.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The description of the label.
    #[serde(rename = "Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// The container that stores label items.
///
/// The metadata index library nests the items under a `Label` element while the
/// `datasetConfig` query parameter carries them as a plain JSON array, so
/// serialization flattens the wrapper.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct LabelItems {
    /// The label items.
    #[serde(rename = "Label", default)]
    pub labels: Vec<LabelItem>,
}

impl Serialize for LabelItems {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.labels.serialize(serializer)
    }
}

/// The caption configuration of an image.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InsightsImageCaption {
    /// Specifies whether to enable the caption feature.
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,

    /// The custom prompt.
    #[serde(rename = "Prompt", skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

/// The person reference configuration of a video caption.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InsightsPersonReference {
    /// Specifies whether to enable the person reference feature.
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
}

/// The caption configuration of a video.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InsightsVideoCaption {
    /// Specifies whether to enable the caption feature.
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,

    /// The custom prompt.
    #[serde(rename = "Prompt", skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// The person reference configuration.
    #[serde(rename = "PersonReference", skip_serializing_if = "Option::is_none")]
    pub person_reference: Option<InsightsPersonReference>,
}

/// The system label configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InsightsVideoSystem {
    /// Specifies whether to enable system labels.
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
}

/// The custom label configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InsightsVideoUserDefined {
    /// Specifies whether to enable custom labels.
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,

    /// The mode in which the custom labels are used.
    #[serde(rename = "Mode", skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,

    /// The custom labels.
    #[serde(rename = "Labels", skip_serializing_if = "Option::is_none")]
    pub labels: Option<LabelItems>,
}

/// The highlight configuration of a video.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InsightsVideoHighlight {
    /// Specifies whether to enable the highlight feature.
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,

    /// The labels of the highlight.
    #[serde(rename = "Labels", skip_serializing_if = "Option::is_none")]
    pub labels: Option<LabelItems>,
}

/// The label configuration of a video.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InsightsVideoLabel {
    /// The system labels.
    #[serde(rename = "System", skip_serializing_if = "Option::is_none")]
    pub system: Option<InsightsVideoSystem>,

    /// The custom labels.
    #[serde(rename = "UserDefined", skip_serializing_if = "Option::is_none")]
    pub user_defined: Option<InsightsVideoUserDefined>,

    /// The highlight labels.
    #[serde(rename = "Highlight", skip_serializing_if = "Option::is_none")]
    pub highlight: Option<InsightsVideoHighlight>,
}

/// The multi-stream configuration of a video.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InsightsVideoMultiStream {
    /// Specifies whether to enable the multi-stream feature.
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
}

/// The label configuration of an image.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InsightsImageLabel {
    /// The system labels.
    #[serde(rename = "System", skip_serializing_if = "Option::is_none")]
    pub system: Option<InsightsVideoSystem>,

    /// The custom labels.
    #[serde(rename = "UserDefined", skip_serializing_if = "Option::is_none")]
    pub user_defined: Option<InsightsVideoUserDefined>,
}

/// The insights configuration of an image.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InsightsImage {
    /// The caption configuration.
    #[serde(rename = "Caption", skip_serializing_if = "Option::is_none")]
    pub caption: Option<InsightsImageCaption>,

    /// The label configuration.
    #[serde(rename = "Label", skip_serializing_if = "Option::is_none")]
    pub label: Option<InsightsImageLabel>,
}

/// The insights configuration of a video.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InsightsVideo {
    /// The caption configuration.
    #[serde(rename = "Caption", skip_serializing_if = "Option::is_none")]
    pub caption: Option<InsightsVideoCaption>,

    /// The label configuration.
    #[serde(rename = "Label", skip_serializing_if = "Option::is_none")]
    pub label: Option<InsightsVideoLabel>,

    /// The multi-stream configuration.
    #[serde(rename = "MultiStream", skip_serializing_if = "Option::is_none")]
    pub multi_stream: Option<InsightsVideoMultiStream>,
}

/// The insights configuration of a dataset.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InsightsConfig {
    /// The language of the insights.
    #[serde(rename = "Language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// The insights configuration of an image.
    #[serde(rename = "Image", skip_serializing_if = "Option::is_none")]
    pub image: Option<InsightsImage>,

    /// The insights configuration of a video.
    #[serde(rename = "Video", skip_serializing_if = "Option::is_none")]
    pub video: Option<InsightsVideo>,
}

/// The smart clustering configuration of a dataset.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SmartClusterFigure {
    /// Specifies whether to automatically generate clusters.
    #[serde(rename = "AutoGenerate", skip_serializing_if = "Option::is_none")]
    pub auto_generate: Option<bool>,

    /// Specifies whether to automatically cluster the figures.
    #[serde(rename = "AutoClustering", skip_serializing_if = "Option::is_none")]
    pub auto_clustering: Option<bool>,

    /// The minimum number of entities in a cluster.
    #[serde(rename = "MinEntityCount", skip_serializing_if = "Option::is_none")]
    pub min_entity_count: Option<i64>,

    /// The enabled features.
    #[serde(
        rename = "EnabledFeatures",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub enabled_features: Vec<String>,
}

/// The smart clustering configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SmartCluster {
    /// The figure clustering configuration.
    #[serde(rename = "Figure", skip_serializing_if = "Option::is_none")]
    pub figure: Option<SmartClusterFigure>,
}

/// The reverse image configuration of an image.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReverseImageImage {
    /// Specifies whether to enable reverse image search for images.
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
}

/// The reverse image configuration of a video.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReverseImageVideo {
    /// Specifies whether to enable reverse image search for videos.
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
}

/// The reverse image configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReverseImage {
    /// The image configuration.
    #[serde(rename = "Image", skip_serializing_if = "Option::is_none")]
    pub image: Option<ReverseImageImage>,

    /// The video configuration.
    #[serde(rename = "Video", skip_serializing_if = "Option::is_none")]
    pub video: Option<ReverseImageVideo>,
}

/// The configuration of a dataset.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DatasetConfig {
    /// The insights configuration.
    #[serde(rename = "Insights", skip_serializing_if = "Option::is_none")]
    pub insights: Option<InsightsConfig>,

    /// The smart clustering configuration.
    #[serde(rename = "SmartCluster", skip_serializing_if = "Option::is_none")]
    pub smart_cluster: Option<SmartCluster>,

    /// The reverse image configuration.
    #[serde(rename = "ReverseImage", skip_serializing_if = "Option::is_none")]
    pub reverse_image: Option<ReverseImage>,
}

impl DatasetConfig {
    /// Serializes the dataset configuration into the JSON object expected by
    /// the `datasetConfig` query parameter.
    pub fn to_parameter_value(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// A dataset in the metadata index library.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Dataset {
    /// The time when the dataset was created.
    #[serde(rename = "CreateTime", skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,

    /// The maximum number of entities that can be created in the dataset.
    #[serde(
        rename = "DatasetMaxEntityCount",
        skip_serializing_if = "Option::is_none"
    )]
    pub dataset_max_entity_count: Option<i64>,

    /// The maximum number of files that can be created in the dataset.
    #[serde(
        rename = "DatasetMaxFileCount",
        skip_serializing_if = "Option::is_none"
    )]
    pub dataset_max_file_count: Option<i64>,

    /// The maximum number of relationships that can be created in the dataset.
    #[serde(
        rename = "DatasetMaxRelationCount",
        skip_serializing_if = "Option::is_none"
    )]
    pub dataset_max_relation_count: Option<i64>,

    /// The maximum total size of files in the dataset.
    #[serde(
        rename = "DatasetMaxTotalFileSize",
        skip_serializing_if = "Option::is_none"
    )]
    pub dataset_max_total_file_size: Option<i64>,

    /// The name of the dataset.
    #[serde(rename = "DatasetName", skip_serializing_if = "Option::is_none")]
    pub dataset_name: Option<String>,

    /// The description of the dataset.
    #[serde(rename = "Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The number of files in the dataset.
    #[serde(rename = "FileCount", skip_serializing_if = "Option::is_none")]
    pub file_count: Option<i64>,

    /// The total size of files in the dataset.
    #[serde(rename = "TotalFileSize", skip_serializing_if = "Option::is_none")]
    pub total_file_size: Option<i64>,

    /// The time when the dataset was last updated.
    #[serde(rename = "UpdateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,

    /// The workflow parameters of the dataset.
    #[serde(rename = "WorkflowParameters", skip_serializing_if = "Option::is_none")]
    pub workflow_parameters: Option<WorkflowParameters>,

    /// The configuration of the dataset.
    #[serde(rename = "DatasetConfig", skip_serializing_if = "Option::is_none")]
    pub dataset_config: Option<DatasetConfig>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct CreateDatasetRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the dataset.
    #[field(type = "query", rename = "datasetName")]
    pub dataset_name: String,

    /// The description of the dataset.
    #[field(type = "query", rename = "description")]
    pub description: Option<String>,

    /// The workflow parameters, as produced by
    /// [`WorkflowParameters::to_parameter_value`].
    #[field(type = "query", rename = "workflowParameters")]
    pub workflow_parameters: Option<String>,

    /// The dataset configuration, as produced by
    /// [`DatasetConfig::to_parameter_value`].
    #[field(type = "query", rename = "datasetConfig")]
    pub dataset_config: Option<String>,

    pub common: RequestCommon,
}

impl CreateDatasetRequest {
    /// Creates a request that creates a dataset with the given name.
    pub fn new(bucket: &str, dataset_name: &str) -> Self {
        CreateDatasetRequest {
            bucket: bucket.to_string(),
            dataset_name: dataset_name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct CreateDatasetResult {
    /// The information about the dataset.
    #[serde(rename = "Dataset", skip_serializing_if = "Option::is_none")]
    pub dataset: Option<Dataset>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Creates a dataset.
    ///
    /// # Arguments
    ///
    /// * `request` - The `CreateDatasetRequest` containing the bucket name and
    ///   the dataset description and configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::{
    /// #     CreateDatasetRequest, DatasetConfig, InsightsConfig,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let dataset_config = DatasetConfig {
    ///     insights: Some(InsightsConfig {
    ///         language: Some("zh".to_string()),
    ///         ..Default::default()
    ///     }),
    ///     ..Default::default()
    /// };
    /// let request = CreateDatasetRequest {
    ///     dataset_config: Some(dataset_config.to_parameter_value()),
    ///     ..CreateDatasetRequest::new("my-bucket", "my-dataset")
    /// };
    ///
    /// match client.create_dataset(&request).await {
    ///     Ok(result) => println!("dataset: {:?}", result.dataset),
    ///     Err(error) => eprintln!("Failed to create dataset: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn create_dataset(
        &self,
        request: &CreateDatasetRequest,
    ) -> Result<CreateDatasetResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "CreateDataset".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", ""), ("action", "createDataset")]
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
        let mut result: CreateDatasetResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_dataset_config_to_parameter_value() {
        // Mirrors the `datasetConfig` value asserted by the Go SDK mock tests.
        let config = DatasetConfig {
            insights: Some(InsightsConfig {
                language: Some("zh".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(
            config.to_parameter_value(),
            r#"{"Insights":{"Language":"zh"}}"#
        );
    }

    #[test]
    fn test_workflow_parameters_to_parameter_value() {
        // Mirrors the `workflowParameters` value asserted by the Go SDK mock
        // tests.
        let parameters = WorkflowParameters {
            workflow_parameters: vec![
                WorkflowParameter {
                    name: Some("VideoInsightEnable".to_string()),
                    value: Some("True".to_string()),
                    ..Default::default()
                },
                WorkflowParameter {
                    name: Some("ImageInsightEnable".to_string()),
                    value: Some("True".to_string()),
                    ..Default::default()
                },
            ],
        };
        assert_eq!(
            parameters.to_parameter_value(),
            r#"[{"Name":"VideoInsightEnable","Value":"True"},{"Name":"ImageInsightEnable","Value":"True"}]"#
        );
    }

    #[test]
    fn test_dataset_config_labels_flatten_to_json_array() {
        let config = DatasetConfig {
            insights: Some(InsightsConfig {
                language: Some("zh".to_string()),
                video: Some(InsightsVideo {
                    label: Some(InsightsVideoLabel {
                        user_defined: Some(InsightsVideoUserDefined {
                            enable: Some(true),
                            labels: Some(LabelItems {
                                labels: vec![LabelItem {
                                    name: Some("sitting".to_string()),
                                    ..Default::default()
                                }],
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(
            config.to_parameter_value(),
            r#"{"Insights":{"Language":"zh","Video":{"Label":{"UserDefined":{"Enable":true,"Labels":[{"Name":"sitting"}]}}}}}"#
        );
    }

    #[test]
    fn test_create_dataset_request_query_map() {
        let request = CreateDatasetRequest {
            description: Some("this is a demo".to_string()),
            ..CreateDatasetRequest::new("bucket", "test_dataset")
        };
        let query = request.query_map();
        assert_eq!(
            query.get("datasetName").map(String::as_str),
            Some("test_dataset")
        );
        assert_eq!(
            query.get("description").map(String::as_str),
            Some("this is a demo")
        );
        assert!(!query.contains_key("workflowParameters"));
        assert!(!query.contains_key("datasetConfig"));
    }

    #[test]
    fn test_create_dataset_result_deserialize() {
        let xml = r#"<CreateDatasetResponse>
<Dataset>
<DatasetName>test-dataset</DatasetName>
<WorkflowParameters></WorkflowParameters>
<CreateTime>2026-04-22T11:39:28.148283473+08:00</CreateTime>
<UpdateTime>2026-04-22T11:39:28.148283473+08:00</UpdateTime>
<DatasetMaxFileCount>100000000</DatasetMaxFileCount>
<DatasetMaxEntityCount>10000000000</DatasetMaxEntityCount>
<DatasetMaxRelationCount>100000000000</DatasetMaxRelationCount>
<DatasetMaxTotalFileSize>90000000000000000</DatasetMaxTotalFileSize>
<DatasetConfig><Insights><Language>zh</Language></Insights></DatasetConfig>
</Dataset>
</CreateDatasetResponse>"#;
        let result: CreateDatasetResult = quick_xml::de::from_str(xml).unwrap();
        let dataset = result.dataset.unwrap();
        assert_eq!(dataset.dataset_name.as_deref(), Some("test-dataset"));
        assert_eq!(dataset.dataset_max_file_count, Some(100000000));
        assert_eq!(dataset.dataset_max_total_file_size, Some(90000000000000000));
        assert_eq!(
            dataset.create_time.as_deref(),
            Some("2026-04-22T11:39:28.148283473+08:00")
        );
        assert!(dataset
            .workflow_parameters
            .unwrap()
            .workflow_parameters
            .is_empty());
        assert_eq!(
            dataset
                .dataset_config
                .unwrap()
                .insights
                .unwrap()
                .language
                .as_deref(),
            Some("zh")
        );
    }

    #[test]
    fn test_create_dataset_result_deserialize_insights_labels() {
        let xml = r#"<CreateDatasetResponse>
<Dataset>
<DatasetName>test_dataset</DatasetName>
<DatasetConfig>
  <Insights>
    <Language>zh</Language>
    <Video>
      <Label>
        <UserDefined>
          <Enable>true</Enable>
          <Labels><Label><Name>sitting</Name></Label></Labels>
        </UserDefined>
      </Label>
    </Video>
  </Insights>
</DatasetConfig>
</Dataset>
</CreateDatasetResponse>"#;
        let result: CreateDatasetResult = quick_xml::de::from_str(xml).unwrap();
        let labels = result
            .dataset
            .unwrap()
            .dataset_config
            .unwrap()
            .insights
            .unwrap()
            .video
            .unwrap()
            .label
            .unwrap()
            .user_defined
            .unwrap()
            .labels
            .unwrap()
            .labels;
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].name.as_deref(), Some("sitting"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_dataset() {
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

        let request = CreateDatasetRequest {
            description: Some("sdk integration test".to_string()),
            ..CreateDatasetRequest::new(&config.bucket, "sdk-test-dataset")
        };
        let result = client.create_dataset(&request).await;
        if let Err(error) = &result {
            eprintln!("create_dataset rejected: {}", error);
        }
        let _ = client
            .delete_dataset(&super::super::DeleteDatasetRequest::new(
                &config.bucket,
                "sdk-test-dataset",
            ))
            .await;
    }
}
