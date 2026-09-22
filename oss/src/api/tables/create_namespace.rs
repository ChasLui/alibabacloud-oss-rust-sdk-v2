use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{
    BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE,
    OP_META_KEY_IS_BUCKET_ARN,
};

/// Creates a namespace.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct CreateNamespaceRequest {
    /// The ARN of the table bucket.
    #[serde(skip)]
    pub table_bucket_arn: String,

    /// The namespace, as its path segments.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub namespace: Vec<String>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct CreateNamespaceResult {
    /// The namespace that was created.
    #[serde(default)]
    pub namespace: Vec<String>,

    /// The ARN of the table bucket.
    #[serde(rename = "tableBucketARN", skip_serializing_if = "Option::is_none")]
    pub table_bucket_arn: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Creates a namespace.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn create_namespace(
        &self,
        request: &CreateNamespaceRequest,
    ) -> Result<CreateNamespaceResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "CreateNamespace".to_string(),
            method: http::Method::PUT,
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

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let mut result: CreateNamespaceResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_namespace_body() {
        let request = CreateNamespaceRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            namespace: vec!["space".to_string(), "nested".to_string()],
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"namespace":["space","nested"]}"#
        );
    }

    #[test]
    fn test_create_namespace_result_deserialize() {
        let body = r#"{"namespace":["space"],"tableBucketARN":"acs:osstables:cn-hangzhou:123:bucket/demo"}"#;

        let result: CreateNamespaceResult = serde_json::from_str(body).unwrap();

        assert_eq!(result.namespace, vec!["space".to_string()]);
        assert!(result.table_bucket_arn.is_some());
    }
}
