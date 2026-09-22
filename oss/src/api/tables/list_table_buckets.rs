use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// A table bucket as returned by a list operation.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TableBucketSummary {
    /// The ARN of the table bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arn: Option<String>,

    /// The time when the table bucket was created.
    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// The name of the table bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The account that owns the table bucket.
    #[serde(rename = "ownerAccountId", skip_serializing_if = "Option::is_none")]
    pub owner_account_id: Option<String>,

    /// The ID of the table bucket.
    #[serde(rename = "tableBucketId", skip_serializing_if = "Option::is_none")]
    pub table_bucket_id: Option<String>,

    /// The type of the table bucket.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

/// Lists the table buckets that belong to the current account.
#[derive(Debug, Default, OssRequestModel)]
pub struct ListTableBucketsRequest {
    /// The token from which the ListTableBuckets operation must start.
    #[field(type = "query", rename = "continuationToken")]
    pub continuation_token: Option<String>,

    /// The maximum number of buckets returned in a single query, 1 to 1000.
    #[field(type = "query", rename = "maxBuckets")]
    pub max_buckets: Option<i32>,

    /// The prefix that the names of the returned buckets must contain.
    #[field(type = "query", rename = "prefix")]
    pub prefix: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct ListTableBucketsResult {
    /// The token to pass to the next ListTableBuckets request.
    #[serde(rename = "continuationToken", skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,

    /// The container that stores information about buckets.
    #[serde(rename = "tableBuckets", default)]
    pub table_buckets: Vec<TableBucketSummary>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists the table buckets that belong to the current account.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn list_table_buckets(
        &self,
        request: &ListTableBucketsRequest,
    ) -> Result<ListTableBucketsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListTableBuckets".to_string(),
            method: http::Method::GET,
            key: Some("buckets".to_string()),
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
        let mut result: ListTableBucketsResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_table_buckets_request_query() {
        let request = ListTableBucketsRequest {
            continuation_token: Some("token-1".to_string()),
            max_buckets: Some(10),
            prefix: Some("demo".to_string()),
            ..Default::default()
        };

        let query = request.query_map();
        assert_eq!(query.get("continuationToken").unwrap(), "token-1");
        assert_eq!(query.get("maxBuckets").unwrap(), "10");
        assert_eq!(query.get("prefix").unwrap(), "demo");
    }

    #[test]
    fn test_list_table_buckets_result_deserialize() {
        let body = r#"{"continuationToken":"token-2","tableBuckets":[
            {"arn":"acs:osstables:cn-hangzhou:123:bucket/demo","name":"demo","type":"customer"}]}"#;

        let result: ListTableBucketsResult = serde_json::from_str(body).unwrap();

        assert_eq!(result.continuation_token.as_deref(), Some("token-2"));
        assert_eq!(result.table_buckets.len(), 1);
        assert_eq!(result.table_buckets[0].name.as_deref(), Some("demo"));
        assert_eq!(result.table_buckets[0].r#type.as_deref(), Some("customer"));
    }

    #[test]
    fn test_list_table_buckets_result_without_buckets() {
        let result: ListTableBucketsResult = serde_json::from_str("{}").unwrap();

        assert!(result.table_buckets.is_empty());
        assert!(result.continuation_token.is_none());
    }
}
