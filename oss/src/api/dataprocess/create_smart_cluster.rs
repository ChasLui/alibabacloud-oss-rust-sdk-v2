use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// A rule of a smart cluster.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SmartClusterRule {
    /// The type of the rule. Valid values: keywords, face.
    #[serde(rename = "RuleType", skip_serializing_if = "Option::is_none")]
    pub rule_type: Option<String>,

    /// The base URIs of the reference objects.
    #[serde(rename = "BaseURIs", default, skip_serializing_if = "Vec::is_empty")]
    pub base_uris: Vec<String>,

    /// The keywords of the rule.
    #[serde(rename = "Keywords", default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,

    /// The sensitivity of the rule.
    #[serde(rename = "Sensitivity", skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<f64>,
}

/// The container that stores the rules of a smart cluster.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SmartClusterRules {
    /// The rules.
    #[serde(rename = "Rule", default)]
    pub rules: Vec<SmartClusterRule>,
}

impl SmartClusterRules {
    /// Serializes the rules into the JSON array expected by the `rules` query
    /// parameter.
    pub fn to_parameter_value(&self) -> String {
        serde_json::to_string(&self.rules).unwrap_or_default()
    }
}

/// The topic of a smart cluster notification.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SmartClusterTopicName {
    /// The name of the topic.
    #[serde(rename = "TopicName", skip_serializing_if = "Option::is_none")]
    pub topic_name: Option<String>,
}

/// The notification configuration of a smart cluster.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SmartClusterNotification {
    /// The MNS topic.
    #[serde(rename = "MNS", skip_serializing_if = "Option::is_none")]
    pub mns: Option<SmartClusterTopicName>,
}

impl SmartClusterNotification {
    /// Serializes the notification into the JSON object expected by the
    /// `notification` query parameter.
    pub fn to_parameter_value(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// The information about a smart cluster.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SmartClusterInfo {
    /// The ID of the smart cluster.
    #[serde(rename = "ObjectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,

    /// The type of the smart cluster. Valid values: figure, knowledge.
    #[serde(rename = "ClusterType", skip_serializing_if = "Option::is_none")]
    pub cluster_type: Option<String>,

    /// The name of the smart cluster.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The description of the smart cluster.
    #[serde(rename = "Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The rules of the smart cluster.
    #[serde(rename = "Rules", skip_serializing_if = "Option::is_none")]
    pub rules: Option<SmartClusterRules>,

    /// The reason why the smart cluster is in its current state.
    #[serde(rename = "Reason", skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// The notification configuration.
    #[serde(rename = "Notification", skip_serializing_if = "Option::is_none")]
    pub notification: Option<SmartClusterNotification>,

    /// The time when the smart cluster was created.
    #[serde(rename = "CreateTime", skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,

    /// The time when the smart cluster was last updated.
    #[serde(rename = "UpdateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,
}

/// The container that stores the smart clusters of a dataset.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SmartClusters {
    /// The smart clusters.
    #[serde(rename = "SmartCluster", default)]
    pub smart_clusters: Vec<SmartClusterInfo>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct CreateSmartClusterRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the dataset.
    #[field(type = "query", rename = "datasetName")]
    pub dataset_name: String,

    /// The name of the smart cluster.
    #[field(type = "query", rename = "name")]
    pub name: String,

    /// The type of the smart cluster. Valid values: figure, knowledge.
    #[field(type = "query", rename = "clusterType")]
    pub cluster_type: String,

    /// The rules, as produced by
    /// [`SmartClusterRules::to_parameter_value`].
    #[field(type = "query", rename = "rules")]
    pub rules: String,

    /// The description of the smart cluster.
    #[field(type = "query", rename = "description")]
    pub description: Option<String>,

    /// The notification configuration, as produced by
    /// [`SmartClusterNotification::to_parameter_value`].
    #[field(type = "query", rename = "notification")]
    pub notification: Option<String>,

    pub common: RequestCommon,
}

impl CreateSmartClusterRequest {
    /// Creates a request that creates a smart cluster with the given rules.
    pub fn new(
        bucket: &str,
        dataset_name: &str,
        name: &str,
        cluster_type: &str,
        rules: &SmartClusterRules,
    ) -> Self {
        CreateSmartClusterRequest {
            bucket: bucket.to_string(),
            dataset_name: dataset_name.to_string(),
            name: name.to_string(),
            cluster_type: cluster_type.to_string(),
            rules: rules.to_parameter_value(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct CreateSmartClusterResult {
    /// The ID of the smart cluster.
    #[serde(rename = "ObjectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Creates a smart cluster.
    ///
    /// # Arguments
    ///
    /// * `request` - The `CreateSmartClusterRequest` containing the bucket and
    ///   dataset names, the cluster name and type, and the cluster rules.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::{
    /// #     CreateSmartClusterRequest, SmartClusterRule, SmartClusterRules,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let rules = SmartClusterRules {
    ///     rules: vec![SmartClusterRule {
    ///         rule_type: Some("keywords".to_string()),
    ///         keywords: vec!["car".to_string()],
    ///         ..Default::default()
    ///     }],
    /// };
    /// let request = CreateSmartClusterRequest::new(
    ///     "my-bucket",
    ///     "my-dataset",
    ///     "my-cluster",
    ///     "knowledge",
    ///     &rules,
    /// );
    ///
    /// match client.create_smart_cluster(&request).await {
    ///     Ok(result) => println!("object id: {:?}", result.object_id),
    ///     Err(error) => eprintln!("Failed to create smart cluster: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn create_smart_cluster(
        &self,
        request: &CreateSmartClusterRequest,
    ) -> Result<CreateSmartClusterResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "CreateSmartCluster".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", ""), ("action", "createSmartCluster")]
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
        let mut result: CreateSmartClusterResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_smart_cluster_rules_to_parameter_value() {
        // Mirrors the `rules` value asserted by the Go SDK mock tests.
        let rules = SmartClusterRules {
            rules: vec![SmartClusterRule {
                rule_type: Some("keywords".to_string()),
                keywords: vec!["car".to_string()],
                ..Default::default()
            }],
        };
        assert_eq!(
            rules.to_parameter_value(),
            r#"[{"RuleType":"keywords","Keywords":["car"]}]"#
        );
    }

    #[test]
    fn test_smart_cluster_notification_to_parameter_value() {
        // Mirrors the `notification` value asserted by the Go SDK mock tests.
        let notification = SmartClusterNotification {
            mns: Some(SmartClusterTopicName {
                topic_name: Some("imm-cluster-notification".to_string()),
            }),
        };
        assert_eq!(
            notification.to_parameter_value(),
            r#"{"MNS":{"TopicName":"imm-cluster-notification"}}"#
        );
    }

    #[test]
    fn test_create_smart_cluster_request_query_map() {
        let rules = SmartClusterRules {
            rules: vec![SmartClusterRule {
                rule_type: Some("face".to_string()),
                base_uris: vec!["oss://examplebucket/refs/alice.jpg".to_string()],
                sensitivity: Some(0.7),
                ..Default::default()
            }],
        };
        let request = CreateSmartClusterRequest::new(
            "bucket",
            "test-dataset",
            "your_name",
            "knowledge",
            &rules,
        );
        let query = request.query_map();
        assert_eq!(
            query.get("datasetName").map(String::as_str),
            Some("test-dataset")
        );
        assert_eq!(query.get("name").map(String::as_str), Some("your_name"));
        assert_eq!(
            query.get("clusterType").map(String::as_str),
            Some("knowledge")
        );
        assert_eq!(
            query.get("rules").map(String::as_str),
            Some(
                r#"[{"RuleType":"face","BaseURIs":["oss://examplebucket/refs/alice.jpg"],"Sensitivity":0.7}]"#
            )
        );
        assert!(!query.contains_key("notification"));
    }

    #[test]
    fn test_create_smart_cluster_result_deserialize() {
        let xml = r#"<CreateSmartClusterResponse>
<ObjectId>cluster-abc123def456</ObjectId>
</CreateSmartClusterResponse>"#;
        let result: CreateSmartClusterResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.object_id.as_deref(), Some("cluster-abc123def456"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_smart_cluster() {
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

        let rules = SmartClusterRules {
            rules: vec![SmartClusterRule {
                rule_type: Some("keywords".to_string()),
                keywords: vec!["car".to_string()],
                ..Default::default()
            }],
        };
        let request = CreateSmartClusterRequest::new(
            &config.bucket,
            "sdk-test-dataset",
            "sdk-test-cluster",
            "knowledge",
            &rules,
        );
        let result = client.create_smart_cluster(&request).await;
        if let Err(error) = &result {
            eprintln!("create_smart_cluster rejected: {}", error);
        }
    }
}
