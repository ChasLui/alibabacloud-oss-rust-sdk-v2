use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct UpdateSmartClusterRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the dataset.
    #[field(type = "query", rename = "datasetName")]
    pub dataset_name: String,

    /// The ID of the smart cluster.
    #[field(type = "query", rename = "objectId")]
    pub object_id: String,

    /// The new name of the smart cluster.
    #[field(type = "query", rename = "name")]
    pub name: Option<String>,

    /// The new description of the smart cluster.
    #[field(type = "query", rename = "description")]
    pub description: Option<String>,

    /// The new rules, as produced by
    /// [`super::create_smart_cluster::SmartClusterRules::to_parameter_value`].
    #[field(type = "query", rename = "rules")]
    pub rules: Option<String>,

    /// The new notification configuration, as produced by
    /// [`super::create_smart_cluster::SmartClusterNotification::to_parameter_value`].
    #[field(type = "query", rename = "notification")]
    pub notification: Option<String>,

    pub common: RequestCommon,
}

impl UpdateSmartClusterRequest {
    /// Creates a request that updates the smart cluster with the given ID.
    pub fn new(bucket: &str, dataset_name: &str, object_id: &str) -> Self {
        UpdateSmartClusterRequest {
            bucket: bucket.to_string(),
            dataset_name: dataset_name.to_string(),
            object_id: object_id.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct UpdateSmartClusterResult {
    /// The ID of the smart cluster.
    #[serde(rename = "ObjectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Updates a smart cluster.
    ///
    /// # Arguments
    ///
    /// * `request` - The `UpdateSmartClusterRequest` containing the bucket and
    ///   dataset names, the smart cluster ID and the fields to update.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::UpdateSmartClusterRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = UpdateSmartClusterRequest {
    ///     name: Some("new-name".to_string()),
    ///     ..UpdateSmartClusterRequest::new("my-bucket", "my-dataset", "cluster-abc123")
    /// };
    ///
    /// match client.update_smart_cluster(&request).await {
    ///     Ok(result) => println!("object id: {:?}", result.object_id),
    ///     Err(error) => eprintln!("Failed to update smart cluster: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn update_smart_cluster(
        &self,
        request: &UpdateSmartClusterRequest,
    ) -> Result<UpdateSmartClusterResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "UpdateSmartCluster".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", ""), ("action", "updateSmartCluster")]
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
        let mut result: UpdateSmartClusterResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_update_smart_cluster_request_query_map() {
        let request = UpdateSmartClusterRequest {
            name: Some("face-cluster-alice".to_string()),
            description: Some("this is a demo".to_string()),
            rules: Some(r#"[{"RuleType":"face","Sensitivity":0.7}]"#.to_string()),
            ..UpdateSmartClusterRequest::new("bucket", "your_dataset", "cluster-abc123def456")
        };
        let query = request.query_map();
        assert_eq!(
            query.get("datasetName").map(String::as_str),
            Some("your_dataset")
        );
        assert_eq!(
            query.get("objectId").map(String::as_str),
            Some("cluster-abc123def456")
        );
        assert_eq!(
            query.get("name").map(String::as_str),
            Some("face-cluster-alice")
        );
        assert_eq!(
            query.get("rules").map(String::as_str),
            Some(r#"[{"RuleType":"face","Sensitivity":0.7}]"#)
        );
        assert!(!query.contains_key("notification"));
    }

    #[test]
    fn test_update_smart_cluster_result_deserialize() {
        let xml = r#"<UpdateSmartClusterResponse><ObjectId>cluster-abc123def456</ObjectId></UpdateSmartClusterResponse>"#;
        let result: UpdateSmartClusterResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.object_id.as_deref(), Some("cluster-abc123def456"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_update_smart_cluster() {
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

        let request = UpdateSmartClusterRequest {
            description: Some("sdk integration test".to_string()),
            ..UpdateSmartClusterRequest::new(&config.bucket, "sdk-test-dataset", "sdk-test-cluster")
        };
        let result = client.update_smart_cluster(&request).await;
        if let Err(error) = &result {
            eprintln!("update_smart_cluster rejected: {}", error);
        }
    }
}
