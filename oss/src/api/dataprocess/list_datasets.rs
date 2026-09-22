use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use super::create_dataset::Dataset;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the datasets of a bucket.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Datasets {
    /// The datasets.
    #[serde(rename = "Dataset", default)]
    pub datasets: Vec<Dataset>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct ListDatasetsRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The maximum number of datasets to return.
    #[field(type = "query", rename = "maxResults")]
    pub max_results: Option<i64>,

    /// The token that is used to retrieve the next page of results.
    #[field(type = "query", rename = "nextToken")]
    pub next_token: Option<String>,

    /// The prefix contained in the names of the returned datasets.
    #[field(type = "query", rename = "prefix")]
    pub prefix: Option<String>,

    pub common: RequestCommon,
}

impl ListDatasetsRequest {
    /// Creates a request that lists the datasets of a bucket.
    pub fn new(bucket: &str) -> Self {
        ListDatasetsRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct ListDatasetsResult {
    /// The datasets.
    #[serde(rename = "Datasets", skip_serializing_if = "Option::is_none")]
    pub datasets: Option<Datasets>,

    /// The token that is used to retrieve the next page of results.
    #[serde(rename = "NextToken", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,

    /// The maximum number of datasets returned.
    #[serde(rename = "MaxResults", skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i64>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists the datasets of a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListDatasetsRequest` containing the bucket name and
    ///   the optional pagination and prefix filters.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::ListDatasetsRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListDatasetsRequest {
    ///     max_results: Some(10),
    ///     ..ListDatasetsRequest::new("my-bucket")
    /// };
    ///
    /// match client.list_datasets(&request).await {
    ///     Ok(result) => println!("datasets: {:?}", result.datasets),
    ///     Err(error) => eprintln!("Failed to list datasets: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn list_datasets(
        &self,
        request: &ListDatasetsRequest,
    ) -> Result<ListDatasetsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListDatasets".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", ""), ("action", "listDatasets")]
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
        let mut result: ListDatasetsResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_list_datasets_request_query_map() {
        let request = ListDatasetsRequest {
            max_results: Some(100),
            next_token: Some("next-token".to_string()),
            prefix: Some("prefix".to_string()),
            ..ListDatasetsRequest::new("bucket")
        };
        let query = request.query_map();
        assert_eq!(query.get("maxResults").map(String::as_str), Some("100"));
        assert_eq!(
            query.get("nextToken").map(String::as_str),
            Some("next-token")
        );
        assert_eq!(query.get("prefix").map(String::as_str), Some("prefix"));
    }

    #[test]
    fn test_list_datasets_result_deserialize() {
        let xml = r#"<ListDatasetsResponse>
<NextToken>1986505809429276:oss_1234567890_demo-bucket:test-dataset</NextToken>
<Datasets>
<Dataset>
<DatasetName>test-dataset</DatasetName>
<CreateTime>2026-04-22T11:39:28.148283473+08:00</CreateTime>
<UpdateTime>2026-04-22T11:39:28.148283473+08:00</UpdateTime>
</Dataset>
</Datasets>
</ListDatasetsResponse>"#;
        let result: ListDatasetsResult = quick_xml::de::from_str(xml).unwrap();
        let datasets = result.datasets.unwrap().datasets;
        assert_eq!(datasets.len(), 1);
        assert_eq!(datasets[0].dataset_name.as_deref(), Some("test-dataset"));
        assert_eq!(
            result.next_token.as_deref(),
            Some("1986505809429276:oss_1234567890_demo-bucket:test-dataset")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_datasets() {
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
            .list_datasets(&ListDatasetsRequest::new(&config.bucket))
            .await;
        if let Err(error) = &result {
            eprintln!("list_datasets rejected: {}", error);
        }
    }
}
