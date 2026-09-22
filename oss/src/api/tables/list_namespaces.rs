use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE, OP_META_KEY_IS_BUCKET_ARN};

/// A namespace as returned by a list operation.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NamespaceSummary {
    /// The time when the namespace was created.
    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// The account that created the namespace.
    #[serde(rename = "createdBy", skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,

    /// The namespace, as its path segments.
    #[serde(default)]
    pub namespace: Vec<String>,

    /// The ID of the namespace.
    #[serde(rename = "namespaceId", skip_serializing_if = "Option::is_none")]
    pub namespace_id: Option<String>,

    /// The account that owns the namespace.
    #[serde(rename = "ownerAccountId", skip_serializing_if = "Option::is_none")]
    pub owner_account_id: Option<String>,

    /// The ID of the table bucket.
    #[serde(rename = "tableBucketId", skip_serializing_if = "Option::is_none")]
    pub table_bucket_id: Option<String>,
}

/// Lists the namespaces of a table bucket.
#[derive(Debug, Default, OssRequestModel)]
pub struct ListNamespacesRequest {
    /// The ARN of the table bucket.
    pub table_bucket_arn: String,

    /// The token from which the ListNamespaces operation must start.
    #[field(type = "query", rename = "continuationToken")]
    pub continuation_token: Option<String>,

    /// The maximum number of namespaces returned in a single query, 1 to 1000.
    #[field(type = "query", rename = "maxNamespaces")]
    pub max_namespaces: Option<i32>,

    /// The prefix that the names of the returned namespaces must contain.
    #[field(type = "query", rename = "prefix")]
    pub prefix: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct ListNamespacesResult {
    /// The token to pass to the next ListNamespaces request.
    #[serde(rename = "continuationToken", skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,

    /// The container that stores information about namespaces.
    #[serde(default)]
    pub namespaces: Vec<NamespaceSummary>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists the namespaces of a table bucket.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn list_namespaces(
        &self,
        request: &ListNamespacesRequest,
    ) -> Result<ListNamespacesResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListNamespaces".to_string(),
            method: http::Method::GET,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "namespaces/{}",
                escape_path(&request.table_bucket_arn, true)
            )),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/json")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };
        input
            .op_metadata
            .set(OP_META_KEY_IS_BUCKET_ARN, std::rc::Rc::new(true));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let mut result: ListNamespacesResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_namespaces_request_query() {
        let request = ListNamespacesRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            continuation_token: Some("token-1".to_string()),
            max_namespaces: Some(10),
            prefix: Some("space".to_string()),
            ..Default::default()
        };

        let query = request.query_map();
        assert_eq!(query.get("continuationToken").unwrap(), "token-1");
        assert_eq!(query.get("maxNamespaces").unwrap(), "10");
        assert_eq!(query.get("prefix").unwrap(), "space");
    }

    #[test]
    fn test_list_namespaces_result_deserialize() {
        let body = r#"{"continuationToken":"token-2","namespaces":[
            {"namespace":["space"],"namespaceId":"ns-1","ownerAccountId":"123"}]}"#;

        let result: ListNamespacesResult = serde_json::from_str(body).unwrap();

        assert_eq!(result.continuation_token.as_deref(), Some("token-2"));
        assert_eq!(result.namespaces.len(), 1);
        assert_eq!(result.namespaces[0].namespace, vec!["space".to_string()]);
        assert_eq!(result.namespaces[0].namespace_id.as_deref(), Some("ns-1"));
    }
}
