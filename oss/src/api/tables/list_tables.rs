use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE, OP_META_KEY_IS_BUCKET_ARN};

/// A table as returned by a list operation.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TableSummary {
    /// The time when the table was created.
    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// The time when the table was last modified.
    #[serde(rename = "modifiedAt", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,

    /// The name of the table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The namespace of the table, as its path segments.
    #[serde(default)]
    pub namespace: Vec<String>,

    /// The ARN of the table.
    #[serde(rename = "tableARN", skip_serializing_if = "Option::is_none")]
    pub table_arn: Option<String>,

    /// The type of the table.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

/// Lists the tables of a namespace.
#[derive(Debug, Default, OssRequestModel)]
pub struct ListTablesRequest {
    /// The ARN of the table bucket.
    pub table_bucket_arn: String,

    /// The namespace whose tables are listed.
    #[field(type = "query", rename = "namespace")]
    pub namespace: Option<String>,

    /// The token from which the ListTables operation must start.
    #[field(type = "query", rename = "continuationToken")]
    pub continuation_token: Option<String>,

    /// The maximum number of tables returned in a single query, 1 to 1000.
    #[field(type = "query", rename = "maxTables")]
    pub max_tables: Option<i32>,

    /// The prefix that the names of the returned tables must contain.
    #[field(type = "query", rename = "prefix")]
    pub prefix: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct ListTablesResult {
    /// The token to pass to the next ListTables request.
    #[serde(rename = "continuationToken", skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,

    /// The container that stores information about tables.
    #[serde(default)]
    pub tables: Vec<TableSummary>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists the tables of a namespace.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn list_tables(
        &self,
        request: &ListTablesRequest,
    ) -> Result<ListTablesResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListTables".to_string(),
            method: http::Method::GET,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "tables/{}",
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
        let mut result: ListTablesResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tables_request_query() {
        let request = ListTablesRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            namespace: Some("space".to_string()),
            continuation_token: Some("token-1".to_string()),
            max_tables: Some(10),
            prefix: Some("table".to_string()),
            ..Default::default()
        };

        let query = request.query_map();
        assert_eq!(query.get("namespace").unwrap(), "space");
        assert_eq!(query.get("continuationToken").unwrap(), "token-1");
        assert_eq!(query.get("maxTables").unwrap(), "10");
        assert_eq!(query.get("prefix").unwrap(), "table");
    }

    #[test]
    fn test_list_tables_result_deserialize() {
        let body = r#"{"continuationToken":"token-2","tables":[
            {"name":"table","namespace":["space"],"tableARN":"acs:osstables:cn-hangzhou:123:bucket/demo/table/1","type":"customer"}]}"#;

        let result: ListTablesResult = serde_json::from_str(body).unwrap();

        assert_eq!(result.continuation_token.as_deref(), Some("token-2"));
        assert_eq!(result.tables.len(), 1);
        assert_eq!(result.tables[0].name.as_deref(), Some("table"));
        assert_eq!(result.tables[0].namespace, vec!["space".to_string()]);
    }

    #[test]
    fn test_list_tables_result_without_tables() {
        let result: ListTablesResult = serde_json::from_str("{}").unwrap();

        assert!(result.tables.is_empty());
        assert!(result.continuation_token.is_none());
    }
}
