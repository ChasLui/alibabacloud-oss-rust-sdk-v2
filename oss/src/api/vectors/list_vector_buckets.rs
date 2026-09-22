use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// A vector bucket as returned by a list operation.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VectorBucketProperties {
    /// The name of the bucket.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The data center in which the bucket is located.
    #[serde(rename = "Location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// The time when the bucket was created.
    #[serde(rename = "CreationDate", skip_serializing_if = "Option::is_none")]
    pub creation_date: Option<String>,

    /// The public endpoint used to access the bucket over the Internet.
    #[serde(rename = "ExtranetEndpoint", skip_serializing_if = "Option::is_none")]
    pub extranet_endpoint: Option<String>,

    /// The internal endpoint used to access the bucket from ECS instances in
    /// the same region.
    #[serde(rename = "IntranetEndpoint", skip_serializing_if = "Option::is_none")]
    pub intranet_endpoint: Option<String>,

    /// The region in which the bucket is located.
    #[serde(rename = "Region", skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// The ID of the resource group to which the bucket belongs.
    #[serde(rename = "ResourceGroupId", skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<String>,
}

/// Lists the vector buckets that belong to the current account.
#[derive(Debug, Default, OssRequestModel)]
pub struct ListVectorBucketsRequest {
    /// The name of the bucket from which the list operation begins.
    #[field(type = "query", rename = "marker")]
    pub marker: Option<String>,

    /// The maximum number of buckets returned in a single query, 1 to 1000.
    #[field(type = "query", rename = "max-keys")]
    pub max_keys: Option<i32>,

    /// The prefix that the names of the returned buckets must contain.
    #[field(type = "query", rename = "prefix")]
    pub prefix: Option<String>,

    /// The ID of the resource group.
    #[field(type = "header", rename = "x-oss-resource-group-id")]
    pub resource_group_id: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct ListVectorBucketsResult {
    /// The prefix contained in the names of the returned buckets.
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// The name of the bucket after which the list operation starts.
    #[serde(rename = "Marker", skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,

    /// The maximum number of buckets returned for the request.
    #[serde(rename = "MaxKeys", default)]
    pub max_keys: i32,

    /// Whether only part of the results are returned.
    #[serde(rename = "IsTruncated", default)]
    pub is_truncated: bool,

    /// The marker to pass to the next request.
    #[serde(rename = "NextMarker", skip_serializing_if = "Option::is_none")]
    pub next_marker: Option<String>,

    /// The container that stores information about buckets.
    #[serde(rename = "Buckets", default)]
    pub buckets: Vec<VectorBucketProperties>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists the vector buckets that belong to the current account.
    ///
    /// Requires a client built with [`Client::new_vectors`].
    pub async fn list_vector_buckets(
        &self,
        request: &ListVectorBucketsRequest,
    ) -> Result<ListVectorBucketsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListVectorBuckets".to_string(),
            method: http::Method::GET,
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/json")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let mut result: ListVectorBucketsResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_vector_buckets_request_query() {
        let request = ListVectorBucketsRequest {
            marker: Some("b-1".to_string()),
            max_keys: Some(10),
            prefix: Some("logs".to_string()),
            ..Default::default()
        };

        let query = request.query_map();
        assert_eq!(query.get("marker").unwrap(), "b-1");
        assert_eq!(query.get("max-keys").unwrap(), "10");
        assert_eq!(query.get("prefix").unwrap(), "logs");
    }

    #[test]
    fn test_list_vector_buckets_result_deserialize() {
        let body = r#"{"Prefix":"","Marker":"","MaxKeys":100,"IsTruncated":true,
            "NextMarker":"b-2","Buckets":[{"Name":"b-1","Region":"cn-hangzhou"}]}"#;

        let result: ListVectorBucketsResult = serde_json::from_str(body).unwrap();

        assert!(result.is_truncated);
        assert_eq!(result.next_marker.as_deref(), Some("b-2"));
        assert_eq!(result.buckets.len(), 1);
        assert_eq!(result.buckets[0].name.as_deref(), Some("b-1"));
    }

    #[test]
    fn test_list_vector_buckets_result_without_buckets() {
        let result: ListVectorBucketsResult = serde_json::from_str("{}").unwrap();
        assert!(result.buckets.is_empty());
    }
}
