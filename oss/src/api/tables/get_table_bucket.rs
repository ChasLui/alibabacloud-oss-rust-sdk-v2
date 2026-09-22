use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE, OP_META_KEY_IS_BUCKET_ARN};

/// Queries information about a table bucket.
#[derive(Debug, Default, OssRequestModel)]
pub struct GetTableBucketRequest {
    /// The ARN of the table bucket to query.
    pub table_bucket_arn: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct GetTableBucketResult {
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

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries information about a table bucket.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn get_table_bucket(
        &self,
        request: &GetTableBucketRequest,
    ) -> Result<GetTableBucketResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetTableBucket".to_string(),
            method: http::Method::GET,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "buckets/{}",
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
        let mut result: GetTableBucketResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_table_bucket_result_deserialize() {
        let body = r#"{"arn":"acs:osstables:cn-hangzhou:123:bucket/demo","createdAt":"2026-01-01",
            "name":"demo","ownerAccountId":"123","tableBucketId":"tb-1","type":"customer"}"#;

        let result: GetTableBucketResult = serde_json::from_str(body).unwrap();

        assert_eq!(result.name.as_deref(), Some("demo"));
        assert_eq!(result.owner_account_id.as_deref(), Some("123"));
        assert_eq!(result.table_bucket_id.as_deref(), Some("tb-1"));
        assert_eq!(result.r#type.as_deref(), Some("customer"));
    }

    #[test]
    fn test_get_table_bucket_result_without_body() {
        let result: GetTableBucketResult = serde_json::from_str("{}").unwrap();

        assert!(result.arn.is_none());
        assert!(result.r#type.is_none());
    }
}
