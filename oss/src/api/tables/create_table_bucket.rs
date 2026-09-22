use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use super::types::EncryptionConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// Creates a table bucket.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct CreateTableBucketRequest {
    /// The name of the table bucket to create.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The encryption of the table bucket.
    #[serde(
        rename = "encryptionConfiguration",
        skip_serializing_if = "Option::is_none"
    )]
    pub encryption_configuration: Option<EncryptionConfiguration>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct CreateTableBucketResult {
    /// The ARN of the bucket that was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arn: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Creates a table bucket.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn create_table_bucket(
        &self,
        request: &CreateTableBucketRequest,
    ) -> Result<CreateTableBucketResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "CreateTableBucket".to_string(),
            method: http::Method::PUT,
            key: Some("buckets".to_string()),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/json")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

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
        let mut result: CreateTableBucketResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_table_bucket_body() {
        let request = CreateTableBucketRequest {
            name: Some("demo-bucket".to_string()),
            encryption_configuration: Some(EncryptionConfiguration {
                sse_algorithm: Some("AES256".to_string()),
                kms_key_arn: None,
            }),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"name":"demo-bucket","encryptionConfiguration":{"sseAlgorithm":"AES256"}}"#
        );
    }

    #[test]
    fn test_create_table_bucket_body_without_encryption() {
        let request = CreateTableBucketRequest {
            name: Some("demo-bucket".to_string()),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"name":"demo-bucket"}"#
        );
    }

    #[test]
    fn test_create_table_bucket_result_deserialize() {
        let result: CreateTableBucketResult =
            serde_json::from_str(r#"{"arn":"acs:osstables:cn-hangzhou:123:bucket/demo"}"#).unwrap();

        assert_eq!(
            result.arn.as_deref(),
            Some("acs:osstables:cn-hangzhou:123:bucket/demo")
        );
    }
}
