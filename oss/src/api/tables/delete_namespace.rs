use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE, OP_META_KEY_IS_BUCKET_ARN};

/// Deletes a namespace.
#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteNamespaceRequest {
    /// The ARN of the table bucket.
    pub table_bucket_arn: String,

    /// The namespace to delete.
    pub namespace: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteNamespaceResult {
    pub common: ResultCommon,
}

impl Client {
    /// Deletes a namespace.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn delete_namespace(
        &self,
        request: &DeleteNamespaceRequest,
    ) -> Result<DeleteNamespaceResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteNamespace".to_string(),
            method: http::Method::DELETE,
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

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteNamespaceResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_namespace_request_carries_no_parameters() {
        let request = DeleteNamespaceRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            namespace: "space".to_string(),
            ..Default::default()
        };

        assert!(request.query_map().is_empty());
        assert!(request.header_map().is_empty());
    }
}
