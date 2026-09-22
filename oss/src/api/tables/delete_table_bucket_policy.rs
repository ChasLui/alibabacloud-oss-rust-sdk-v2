use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE, OP_META_KEY_IS_BUCKET_ARN};

/// Deletes the policy of a table bucket.
#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteTableBucketPolicyRequest {
    /// The ARN of the table bucket.
    pub table_bucket_arn: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteTableBucketPolicyResult {
    pub common: ResultCommon,
}

impl Client {
    /// Deletes the policy of a table bucket.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn delete_table_bucket_policy(
        &self,
        request: &DeleteTableBucketPolicyRequest,
    ) -> Result<DeleteTableBucketPolicyResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteTableBucketPolicy".to_string(),
            method: http::Method::DELETE,
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

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteTableBucketPolicyResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_table_bucket_policy_request_carries_no_parameters() {
        let request = DeleteTableBucketPolicyRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            ..Default::default()
        };

        assert!(request.query_map().is_empty());
        assert!(request.header_map().is_empty());
    }
}
