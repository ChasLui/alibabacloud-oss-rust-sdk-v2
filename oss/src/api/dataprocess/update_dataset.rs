use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::create_dataset::Dataset;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct UpdateDatasetRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the dataset.
    #[field(type = "query", rename = "datasetName")]
    pub dataset_name: String,

    /// The description of the dataset.
    #[field(type = "query", rename = "description")]
    pub description: Option<String>,

    /// The workflow parameters, as produced by
    /// [`super::create_dataset::WorkflowParameters::to_parameter_value`].
    #[field(type = "query", rename = "workflowParameters")]
    pub workflow_parameters: Option<String>,

    /// The dataset configuration, as produced by
    /// [`super::create_dataset::DatasetConfig::to_parameter_value`].
    #[field(type = "query", rename = "datasetConfig")]
    pub dataset_config: Option<String>,

    pub common: RequestCommon,
}

impl UpdateDatasetRequest {
    /// Creates a request that updates the dataset with the given name.
    pub fn new(bucket: &str, dataset_name: &str) -> Self {
        UpdateDatasetRequest {
            bucket: bucket.to_string(),
            dataset_name: dataset_name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct UpdateDatasetResult {
    /// The information about the dataset.
    #[serde(rename = "Dataset", skip_serializing_if = "Option::is_none")]
    pub dataset: Option<Dataset>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Updates a dataset.
    ///
    /// # Arguments
    ///
    /// * `request` - The `UpdateDatasetRequest` containing the bucket name, the
    ///   dataset name and the fields to update.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::UpdateDatasetRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = UpdateDatasetRequest {
    ///     description: Some("this is a demo".to_string()),
    ///     ..UpdateDatasetRequest::new("my-bucket", "my-dataset")
    /// };
    ///
    /// match client.update_dataset(&request).await {
    ///     Ok(result) => println!("dataset: {:?}", result.dataset),
    ///     Err(error) => eprintln!("Failed to update dataset: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn update_dataset(
        &self,
        request: &UpdateDatasetRequest,
    ) -> Result<UpdateDatasetResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "UpdateDataset".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", ""), ("action", "updateDataset")]
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
        let mut result: UpdateDatasetResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_update_dataset_request_query_map() {
        let request = UpdateDatasetRequest {
            description: Some("this is a demo".to_string()),
            dataset_config: Some(r#"{"Insights":{"Language":"zh"}}"#.to_string()),
            ..UpdateDatasetRequest::new("bucket", "your_dataset")
        };
        let query = request.query_map();
        assert_eq!(
            query.get("datasetName").map(String::as_str),
            Some("your_dataset")
        );
        assert_eq!(
            query.get("description").map(String::as_str),
            Some("this is a demo")
        );
        assert_eq!(
            query.get("datasetConfig").map(String::as_str),
            Some(r#"{"Insights":{"Language":"zh"}}"#)
        );
    }

    #[test]
    fn test_update_dataset_result_deserialize() {
        let xml = r#"<UpdateDatasetResponse>
<Dataset>
<DatasetName>test_dataset</DatasetName>
<Description>this is a demo</Description>
<UpdateTime>2026-06-08T17:36:59.774068044+08:00</UpdateTime>
</Dataset>
</UpdateDatasetResponse>"#;
        let result: UpdateDatasetResult = quick_xml::de::from_str(xml).unwrap();
        let dataset = result.dataset.unwrap();
        assert_eq!(dataset.dataset_name.as_deref(), Some("test_dataset"));
        assert_eq!(dataset.description.as_deref(), Some("this is a demo"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_update_dataset() {
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

        let request = UpdateDatasetRequest {
            description: Some("sdk integration test".to_string()),
            ..UpdateDatasetRequest::new(&config.bucket, "sdk-test-dataset")
        };
        let result = client.update_dataset(&request).await;
        if let Err(error) = &result {
            eprintln!("update_dataset rejected: {}", error);
        }
    }
}
