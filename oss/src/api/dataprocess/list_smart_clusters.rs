use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::create_smart_cluster::SmartClusters;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct ListSmartClustersRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the dataset.
    #[field(type = "query", rename = "datasetName")]
    pub dataset_name: String,

    /// The type of the smart clusters to return. Valid values: figure,
    /// knowledge.
    #[field(type = "query", rename = "clusterType")]
    pub cluster_type: Option<String>,

    /// The maximum number of smart clusters to return.
    #[field(type = "query", rename = "maxResults")]
    pub max_results: Option<i64>,

    /// The types of the rules that are used to filter the smart clusters.
    #[field(type = "query", rename = "ruleTypes")]
    pub rule_types: Option<String>,

    /// The token that is used to retrieve the next page of results.
    #[field(type = "query", rename = "nextToken")]
    pub next_token: Option<String>,

    pub common: RequestCommon,
}

impl ListSmartClustersRequest {
    /// Creates a request that lists the smart clusters of a dataset.
    pub fn new(bucket: &str, dataset_name: &str) -> Self {
        ListSmartClustersRequest {
            bucket: bucket.to_string(),
            dataset_name: dataset_name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct ListSmartClustersResult {
    /// The smart clusters.
    #[serde(rename = "SmartClusters", skip_serializing_if = "Option::is_none")]
    pub smart_clusters: Option<SmartClusters>,

    /// The token that is used to retrieve the next page of results.
    #[serde(rename = "NextToken", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists the smart clusters of a dataset.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListSmartClustersRequest` containing the bucket and
    ///   dataset names and the optional filters.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::ListSmartClustersRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListSmartClustersRequest {
    ///     cluster_type: Some("knowledge".to_string()),
    ///     ..ListSmartClustersRequest::new("my-bucket", "my-dataset")
    /// };
    ///
    /// match client.list_smart_clusters(&request).await {
    ///     Ok(result) => println!("smart clusters: {:?}", result.smart_clusters),
    ///     Err(error) => eprintln!("Failed to list smart clusters: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn list_smart_clusters(
        &self,
        request: &ListSmartClustersRequest,
    ) -> Result<ListSmartClustersResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListSmartClusters".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", ""), ("action", "listSmartClusters")]
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
        let mut result: ListSmartClustersResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_list_smart_clusters_request_query_map() {
        let request = ListSmartClustersRequest {
            cluster_type: Some("knowledge".to_string()),
            max_results: Some(10),
            rule_types: Some("keywords".to_string()),
            next_token: Some("next-token".to_string()),
            ..ListSmartClustersRequest::new("bucket", "test-dataset")
        };
        let query = request.query_map();
        assert_eq!(
            query.get("datasetName").map(String::as_str),
            Some("test-dataset")
        );
        assert_eq!(
            query.get("clusterType").map(String::as_str),
            Some("knowledge")
        );
        assert_eq!(query.get("maxResults").map(String::as_str), Some("10"));
        assert_eq!(query.get("ruleTypes").map(String::as_str), Some("keywords"));
        assert_eq!(
            query.get("nextToken").map(String::as_str),
            Some("next-token")
        );
    }

    #[test]
    fn test_list_smart_clusters_result_deserialize() {
        let xml = r#"<ListSmartClustersResponse>
    <SmartClusters>
        <SmartCluster>
            <CreateTime>2026-06-10T09:54:27.484585901+08:00</CreateTime>
            <ObjectId>SmartCluster-cb9f8c95-281f-490b-b677-eca2f6ff0c19</ObjectId>
            <ClusterType>knowledge</ClusterType>
            <Name>demo-2</Name>
            <Rules>
                <Rule>
                    <Keywords>cat</Keywords>
                    <Sensitivity>0.5</Sensitivity>
                    <RuleType>keywords</RuleType>
                </Rule>
            </Rules>
            <Reason></Reason>
        </SmartCluster>
        <SmartCluster>
            <ObjectId>SmartCluster-c30d039b-0b55-4a42-ae19-cae00a53735a</ObjectId>
            <ClusterType>knowledge</ClusterType>
            <Name>new-demo</Name>
            <Description>this is a demo</Description>
            <Rules>
                <Rule>
                    <Keywords>hello</Keywords>
                    <Keywords>world</Keywords>
                    <Sensitivity>0.7</Sensitivity>
                    <RuleType>keywords</RuleType>
                </Rule>
            </Rules>
        </SmartCluster>
    </SmartClusters>
</ListSmartClustersResponse>"#;
        let result: ListSmartClustersResult = quick_xml::de::from_str(xml).unwrap();
        let clusters = result.smart_clusters.unwrap().smart_clusters;
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0].name.as_deref(), Some("demo-2"));
        let first_rules = clusters[0].rules.as_ref().unwrap().rules.clone();
        assert_eq!(first_rules[0].keywords, vec!["cat".to_string()]);
        assert_eq!(clusters[1].description.as_deref(), Some("this is a demo"));
        let second_rules = clusters[1].rules.as_ref().unwrap().rules.clone();
        assert_eq!(
            second_rules[0].keywords,
            vec!["hello".to_string(), "world".to_string()]
        );
        assert!(result.next_token.is_none());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_smart_clusters() {
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
            .list_smart_clusters(&ListSmartClustersRequest::new(
                &config.bucket,
                "sdk-test-dataset",
            ))
            .await;
        if let Err(error) = &result {
            eprintln!("list_smart_clusters rejected: {}", error);
        }
    }
}
