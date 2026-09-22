use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::create_smart_cluster::SmartClusterInfo;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetSmartClusterRequest {
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

impl GetSmartClusterRequest {
    /// Creates a request that queries a smart cluster by ID.
    pub fn new(bucket: &str, dataset_name: &str, object_id: &str) -> Self {
        GetSmartClusterRequest {
            bucket: bucket.to_string(),
            dataset_name: dataset_name.to_string(),
            object_id: object_id.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetSmartClusterResult {
    /// The information about the smart cluster.
    #[serde(rename = "SmartCluster", skip_serializing_if = "Option::is_none")]
    pub smart_cluster: Option<SmartClusterInfo>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the information about a smart cluster.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetSmartClusterRequest` containing the bucket and
    ///   dataset names and the smart cluster ID.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::GetSmartClusterRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetSmartClusterRequest::new("my-bucket", "my-dataset", "cluster-abc123");
    ///
    /// match client.get_smart_cluster(&request).await {
    ///     Ok(result) => println!("smart cluster: {:?}", result.smart_cluster),
    ///     Err(error) => eprintln!("Failed to get smart cluster: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn get_smart_cluster(
        &self,
        request: &GetSmartClusterRequest,
    ) -> Result<GetSmartClusterResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetSmartCluster".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", ""), ("action", "getSmartCluster")]
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
        let mut result: GetSmartClusterResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_get_smart_cluster_request_query_map() {
        let request = GetSmartClusterRequest::new("bucket", "test-dataset", "cluster-abc123def456");
        let query = request.query_map();
        assert_eq!(
            query.get("datasetName").map(String::as_str),
            Some("test-dataset")
        );
        assert_eq!(
            query.get("objectId").map(String::as_str),
            Some("cluster-abc123def456")
        );
    }

    #[test]
    fn test_get_smart_cluster_result_deserialize() {
        let xml = r#"<GetSmartClusterResponse>
  <SmartCluster>
    <ObjectId>cluster-abc123def456</ObjectId>
    <ClusterType>figure</ClusterType>
    <Name>face-cluster-alice</Name>
    <Description>this is a demo</Description>
    <Rules>
      <Rule>
        <RuleType>face</RuleType>
        <BaseURIs>oss://examplebucket/refs/alice.jpg</BaseURIs>
        <Sensitivity>0.7</Sensitivity>
      </Rule>
    </Rules>
    <Reason></Reason>
    <Notification>
      <MNS><TopicName>imm-cluster-notification</TopicName></MNS>
    </Notification>
    <CreateTime>2026-05-20T11:00:00.000+08:00</CreateTime>
    <UpdateTime>2026-05-20T11:08:00.000+08:00</UpdateTime>
  </SmartCluster>
</GetSmartClusterResponse>"#;
        let result: GetSmartClusterResult = quick_xml::de::from_str(xml).unwrap();
        let cluster = result.smart_cluster.unwrap();
        assert_eq!(cluster.object_id.as_deref(), Some("cluster-abc123def456"));
        assert_eq!(cluster.cluster_type.as_deref(), Some("figure"));
        assert_eq!(cluster.name.as_deref(), Some("face-cluster-alice"));
        let rules = cluster.rules.unwrap().rules;
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].rule_type.as_deref(), Some("face"));
        assert_eq!(
            rules[0].base_uris,
            vec!["oss://examplebucket/refs/alice.jpg".to_string()]
        );
        assert_eq!(rules[0].sensitivity, Some(0.7));
        assert_eq!(
            cluster
                .notification
                .unwrap()
                .mns
                .unwrap()
                .topic_name
                .as_deref(),
            Some("imm-cluster-notification")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_smart_cluster() {
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
            GetSmartClusterRequest::new(&config.bucket, "sdk-test-dataset", "sdk-test-cluster");
        let result = client.get_smart_cluster(&request).await;
        if let Err(error) = &result {
            eprintln!("get_smart_cluster rejected: {}", error);
        }
    }
}
