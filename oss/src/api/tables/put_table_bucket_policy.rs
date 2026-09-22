use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Serialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{
    BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE,
    OP_META_KEY_IS_BUCKET_ARN,
};

/// Configures a policy for a table bucket.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct PutTableBucketPolicyRequest {
    /// The ARN of the table bucket.
    #[serde(skip)]
    pub table_bucket_arn: String,

    /// The policy document, serialized as a JSON string.
    #[serde(rename = "resourcePolicy", skip_serializing_if = "Option::is_none")]
    pub resource_policy: Option<String>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutTableBucketPolicyResult {
    pub common: ResultCommon,
}

impl Client {
    /// Configures a policy for a table bucket.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn put_table_bucket_policy(
        &self,
        request: &PutTableBucketPolicyRequest,
    ) -> Result<PutTableBucketPolicyResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutTableBucketPolicy".to_string(),
            method: http::Method::PUT,
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

        input.body = Some(BodyContent::from_text(
            serde_json::to_string(request)?,
            None,
        ));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutTableBucketPolicyResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_table_bucket_policy_body() {
        let request = PutTableBucketPolicyRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            resource_policy: Some(r#"{"Version":"1"}"#.to_string()),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"resourcePolicy":"{\"Version\":\"1\"}"}"#
        );
    }

    #[test]
    fn test_put_table_bucket_policy_body_excludes_the_arn() {
        let request = PutTableBucketPolicyRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            resource_policy: Some(r#"{"Version":"1"}"#.to_string()),
            ..Default::default()
        };

        assert!(!serde_json::to_string(&request)
            .unwrap()
            .contains("acs:osstables"));
    }
}
