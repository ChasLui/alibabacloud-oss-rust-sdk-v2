use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE, OP_META_KEY_IS_BUCKET_ARN};

/// Deletes a table.
#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteTableRequest {
    /// The ARN of the table bucket.
    pub table_bucket_arn: String,

    /// The namespace of the table.
    pub namespace: String,

    /// The name of the table.
    pub name: String,

    /// The version of the table metadata the deletion is based on.
    #[field(type = "query", rename = "versionToken")]
    pub version_token: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteTableResult {
    pub common: ResultCommon,
}

impl Client {
    /// Deletes a table.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn delete_table(
        &self,
        request: &DeleteTableRequest,
    ) -> Result<DeleteTableResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteTable".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "tables/{}/{}/{}",
                escape_path(&request.table_bucket_arn, true),
                escape_path(&request.namespace, true),
                escape_path(&request.name, true)
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

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteTableResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_table_request_query() {
        let request = DeleteTableRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            namespace: "space".to_string(),
            name: "table".to_string(),
            version_token: Some("v-1".to_string()),
            ..Default::default()
        };

        assert_eq!(request.query_map().get("versionToken").unwrap(), "v-1");
    }

    #[test]
    fn test_delete_table_request_without_a_version_token() {
        let request = DeleteTableRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            namespace: "space".to_string(),
            name: "table".to_string(),
            ..Default::default()
        };

        assert!(request.query_map().is_empty());
    }
}
