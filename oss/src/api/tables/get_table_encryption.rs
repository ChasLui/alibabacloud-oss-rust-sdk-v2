use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use super::types::EncryptionConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE, OP_META_KEY_IS_BUCKET_ARN};

/// Queries the encryption rules configured for a table.
#[derive(Debug, Default, OssRequestModel)]
pub struct GetTableEncryptionRequest {
    /// The ARN of the table bucket.
    pub table_bucket_arn: String,

    /// The namespace of the table.
    pub namespace: String,

    /// The name of the table.
    pub name: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct GetTableEncryptionResult {
    /// The container that stores the encryption rules.
    #[serde(
        rename = "encryptionConfiguration",
        skip_serializing_if = "Option::is_none"
    )]
    pub encryption_configuration: Option<EncryptionConfiguration>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the encryption rules configured for a table.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn get_table_encryption(
        &self,
        request: &GetTableEncryptionRequest,
    ) -> Result<GetTableEncryptionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetTableEncryption".to_string(),
            method: http::Method::GET,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "tables/{}/{}/{}/encryption",
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

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let mut result: GetTableEncryptionResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_table_encryption_result_deserialize() {
        let body = r#"{"encryptionConfiguration":{"sseAlgorithm":"KMS","kmsKeyArn":"kms-1"}}"#;

        let result: GetTableEncryptionResult = serde_json::from_str(body).unwrap();

        let configuration = result
            .encryption_configuration
            .expect("encryption configuration");
        assert_eq!(configuration.sse_algorithm.as_deref(), Some("KMS"));
        assert_eq!(configuration.kms_key_arn.as_deref(), Some("kms-1"));
    }

    #[test]
    fn test_get_table_encryption_result_without_body() {
        let result: GetTableEncryptionResult = serde_json::from_str("{}").unwrap();

        assert!(result.encryption_configuration.is_none());
    }
}
