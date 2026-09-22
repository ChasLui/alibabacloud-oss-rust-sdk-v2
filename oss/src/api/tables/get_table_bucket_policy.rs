use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE, OP_META_KEY_IS_BUCKET_ARN};

/// Queries the policy configured for a table bucket.
#[derive(Debug, Default, OssRequestModel)]
pub struct GetTableBucketPolicyRequest {
    /// The ARN of the table bucket.
    pub table_bucket_arn: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct GetTableBucketPolicyResult {
    /// The policy document, serialized as a JSON string.
    #[serde(rename = "resourcePolicy", skip_serializing_if = "Option::is_none")]
    pub resource_policy: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the policy configured for a table bucket.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn get_table_bucket_policy(
        &self,
        request: &GetTableBucketPolicyRequest,
    ) -> Result<GetTableBucketPolicyResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetTableBucketPolicy".to_string(),
            method: http::Method::GET,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "buckets/{}/policy",
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
        let mut result: GetTableBucketPolicyResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_table_bucket_policy_result_deserialize() {
        let body = r#"{"resourcePolicy":"{\"Version\":\"1\"}"}"#;

        let result: GetTableBucketPolicyResult = serde_json::from_str(body).unwrap();

        assert_eq!(
            result.resource_policy.as_deref(),
            Some(r#"{"Version":"1"}"#)
        );
    }

    #[test]
    fn test_get_table_bucket_policy_result_without_body() {
        let result: GetTableBucketPolicyResult = serde_json::from_str("{}").unwrap();

        assert!(result.resource_policy.is_none());
    }
}
