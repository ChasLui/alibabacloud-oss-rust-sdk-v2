use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE, OP_META_KEY_IS_BUCKET_ARN};

/// Queries information about a namespace.
#[derive(Debug, Default, OssRequestModel)]
pub struct GetNamespaceRequest {
    /// The ARN of the table bucket.
    pub table_bucket_arn: String,

    /// The namespace to query.
    pub namespace: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct GetNamespaceResult {
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

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries information about a namespace.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn get_namespace(
        &self,
        request: &GetNamespaceRequest,
    ) -> Result<GetNamespaceResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetNamespace".to_string(),
            method: http::Method::GET,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "namespaces/{}/{}",
                escape_path(&request.table_bucket_arn, true),
                escape_path(&request.namespace, true)
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
        let mut result: GetNamespaceResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_namespace_result_deserialize() {
        let body = r#"{"createdAt":"2026-01-01","createdBy":"123","namespace":["space"],
            "namespaceId":"ns-1","ownerAccountId":"123","tableBucketId":"tb-1"}"#;

        let result: GetNamespaceResult = serde_json::from_str(body).unwrap();

        assert_eq!(result.namespace, vec!["space".to_string()]);
        assert_eq!(result.namespace_id.as_deref(), Some("ns-1"));
        assert_eq!(result.created_by.as_deref(), Some("123"));
    }

    #[test]
    fn test_get_namespace_result_without_body() {
        let result: GetNamespaceResult = serde_json::from_str("{}").unwrap();

        assert!(result.namespace.is_empty());
        assert!(result.namespace_id.is_none());
    }
}
