use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteDatasetRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the dataset.
    #[field(type = "query", rename = "datasetName")]
    pub dataset_name: String,

    pub common: RequestCommon,
}

impl DeleteDatasetRequest {
    /// Creates a request that deletes the dataset with the given name.
    pub fn new(bucket: &str, dataset_name: &str) -> Self {
        DeleteDatasetRequest {
            bucket: bucket.to_string(),
            dataset_name: dataset_name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteDatasetResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes a dataset.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteDatasetRequest` containing the bucket name and
    ///   the dataset name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::DeleteDatasetRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteDatasetRequest::new("my-bucket", "my-dataset");
    ///
    /// match client.delete_dataset(&request).await {
    ///     Ok(_) => println!("dataset deleted"),
    ///     Err(error) => eprintln!("Failed to delete dataset: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn delete_dataset(
        &self,
        request: &DeleteDatasetRequest,
    ) -> Result<DeleteDatasetResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteDataset".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", ""), ("action", "deleteDataset")]
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

        let mut result = DeleteDatasetResult::default();
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
    fn test_delete_dataset_request_query_map() {
        let request = DeleteDatasetRequest::new("bucket", "test-dataset");
        let query = request.query_map();
        assert_eq!(
            query.get("datasetName").map(String::as_str),
            Some("test-dataset")
        );
        assert_eq!(query.len(), 1);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_dataset() {
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
            .delete_dataset(&DeleteDatasetRequest::new(
                &config.bucket,
                "sdk-test-dataset",
            ))
            .await;
        if let Err(error) = &result {
            eprintln!("delete_dataset rejected: {}", error);
        }
    }
}
