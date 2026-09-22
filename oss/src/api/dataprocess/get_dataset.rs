use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::create_dataset::Dataset;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetDatasetRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the dataset.
    #[field(type = "query", rename = "datasetName")]
    pub dataset_name: String,

    /// Specifies whether to return the statistics of the dataset.
    #[field(type = "query", rename = "withStatistics")]
    pub with_statistics: Option<bool>,

    pub common: RequestCommon,
}

impl GetDatasetRequest {
    /// Creates a request that queries the information about a dataset.
    pub fn new(bucket: &str, dataset_name: &str) -> Self {
        GetDatasetRequest {
            bucket: bucket.to_string(),
            dataset_name: dataset_name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetDatasetResult {
    /// The information about the dataset.
    #[serde(rename = "Dataset", skip_serializing_if = "Option::is_none")]
    pub dataset: Option<Dataset>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the information about a dataset.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetDatasetRequest` containing the bucket name, the
    ///   dataset name and whether to return the dataset statistics.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::GetDatasetRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetDatasetRequest::new("my-bucket", "my-dataset");
    ///
    /// match client.get_dataset(&request).await {
    ///     Ok(result) => println!("dataset: {:?}", result.dataset),
    ///     Err(error) => eprintln!("Failed to get dataset: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn get_dataset(
        &self,
        request: &GetDatasetRequest,
    ) -> Result<GetDatasetResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetDataset".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", ""), ("action", "getDataset")]
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
        let mut result: GetDatasetResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_get_dataset_request_query_map() {
        let request = GetDatasetRequest {
            with_statistics: Some(true),
            ..GetDatasetRequest::new("bucket", "your_dataset")
        };
        let query = request.query_map();
        assert_eq!(
            query.get("datasetName").map(String::as_str),
            Some("your_dataset")
        );
        assert_eq!(
            query.get("withStatistics").map(String::as_str),
            Some("true")
        );
    }

    #[test]
    fn test_get_dataset_result_deserialize() {
        let xml = r#"<GetDatasetResponse>
<Dataset>
<DatasetName>test-dataset</DatasetName>
<Description>this is a demo</Description>
<CreateTime>2026-04-22T11:39:28.148283473+08:00</CreateTime>
<FileCount>12</FileCount>
<TotalFileSize>1024</TotalFileSize>
</Dataset>
</GetDatasetResponse>"#;
        let result: GetDatasetResult = quick_xml::de::from_str(xml).unwrap();
        let dataset = result.dataset.unwrap();
        assert_eq!(dataset.dataset_name.as_deref(), Some("test-dataset"));
        assert_eq!(dataset.description.as_deref(), Some("this is a demo"));
        assert_eq!(dataset.file_count, Some(12));
        assert_eq!(dataset.total_file_size, Some(1024));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_dataset() {
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
            .get_dataset(&GetDatasetRequest::new(&config.bucket, "sdk-test-dataset"))
            .await;
        if let Err(error) = &result {
            eprintln!("get_dataset rejected: {}", error);
        }
    }
}
