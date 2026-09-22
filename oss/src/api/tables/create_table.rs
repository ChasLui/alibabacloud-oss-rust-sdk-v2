use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use super::types::EncryptionConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{
    BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE,
    OP_META_KEY_IS_BUCKET_ARN,
};

/// The metadata of a table.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TableMetadata {
    /// The Iceberg metadata of the table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iceberg: Option<IcebergMetadata>,
}

/// The Iceberg metadata of a table.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IcebergMetadata {
    /// The Iceberg schema, as a JSON object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}

/// Creates a table.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct CreateTableRequest {
    /// The ARN of the table bucket.
    #[serde(skip)]
    pub table_bucket_arn: String,

    /// The namespace of the table.
    #[serde(skip)]
    pub namespace: String,

    /// The name of the table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The format of the table, e.g. `iceberg`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// The metadata of the table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TableMetadata>,

    /// The encryption of the table.
    #[serde(
        rename = "encryptionConfiguration",
        skip_serializing_if = "Option::is_none"
    )]
    pub encryption_configuration: Option<EncryptionConfiguration>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct CreateTableResult {
    /// The ARN of the table that was created.
    #[serde(rename = "tableARN", skip_serializing_if = "Option::is_none")]
    pub table_arn: Option<String>,

    /// The version of the table metadata.
    #[serde(rename = "versionToken", skip_serializing_if = "Option::is_none")]
    pub version_token: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Creates a table.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn create_table(
        &self,
        request: &CreateTableRequest,
    ) -> Result<CreateTableResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "CreateTable".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "tables/{}/{}",
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
        let mut result: CreateTableResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_table_body() {
        let request = CreateTableRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            namespace: "space".to_string(),
            name: Some("table".to_string()),
            format: Some("iceberg".to_string()),
            metadata: Some(TableMetadata {
                iceberg: Some(IcebergMetadata {
                    schema: Some(serde_json::json!({"type": "struct"})),
                }),
            }),
            encryption_configuration: Some(EncryptionConfiguration {
                sse_algorithm: Some("AES256".to_string()),
                kms_key_arn: None,
            }),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"name":"table","format":"iceberg","metadata":{"iceberg":{"schema":{"type":"struct"}}},"encryptionConfiguration":{"sseAlgorithm":"AES256"}}"#
        );
    }

    #[test]
    fn test_create_table_body_excludes_the_nop_fields() {
        let request = CreateTableRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            namespace: "space".to_string(),
            name: Some("table".to_string()),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"name":"table"}"#
        );
    }

    #[test]
    fn test_create_table_result_deserialize() {
        let body = r#"{"tableARN":"acs:osstables:cn-hangzhou:123:bucket/demo/table/1","versionToken":"v-1"}"#;

        let result: CreateTableResult = serde_json::from_str(body).unwrap();

        assert!(result.table_arn.is_some());
        assert_eq!(result.version_token.as_deref(), Some("v-1"));
    }
}
