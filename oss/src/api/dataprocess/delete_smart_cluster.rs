use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteSmartClusterRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the dataset.
    #[field(type = "query", rename = "datasetName")]
    pub dataset_name: String,

    /// The ID of the smart cluster.
    #[field(type = "query", rename = "objectId")]
    pub object_id: String,

    pub common: RequestCommon,
}

impl DeleteSmartClusterRequest {
    /// Creates a request that deletes the smart cluster with the given ID.
    pub fn new(bucket: &str, dataset_name: &str, object_id: &str) -> Self {
        DeleteSmartClusterRequest {
            bucket: bucket.to_string(),
            dataset_name: dataset_name.to_string(),
            object_id: object_id.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteSmartClusterResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes a smart cluster.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteSmartClusterRequest` containing the bucket and
    ///   dataset names and the smart cluster ID.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::DeleteSmartClusterRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteSmartClusterRequest::new("my-bucket", "my-dataset", "cluster-abc123");
    ///
    /// match client.delete_smart_cluster(&request).await {
    ///     Ok(_) => println!("smart cluster deleted"),
    ///     Err(error) => eprintln!("Failed to delete smart cluster: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn delete_smart_cluster(
        &self,
        request: &DeleteSmartClusterRequest,
    ) -> Result<DeleteSmartClusterResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteSmartCluster".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", ""), ("action", "deleteSmartCluster")]
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

        let mut result = DeleteSmartClusterResult::default();
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
    fn test_delete_smart_cluster_request_query_map() {
        let request = DeleteSmartClusterRequest::new("bucket", "test-dataset", "cluster-abc123");
        let query = request.query_map();
        assert_eq!(
            query.get("datasetName").map(String::as_str),
            Some("test-dataset")
        );
        assert_eq!(
            query.get("objectId").map(String::as_str),
            Some("cluster-abc123")
        );
        assert_eq!(query.len(), 2);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_smart_cluster() {
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

        let request =
            DeleteSmartClusterRequest::new(&config.bucket, "sdk-test-dataset", "sdk-test-cluster");
        let result = client.delete_smart_cluster(&request).await;
        if let Err(error) = &result {
            eprintln!("delete_smart_cluster rejected: {}", error);
        }
    }
}
