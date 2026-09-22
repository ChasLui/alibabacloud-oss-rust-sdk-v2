use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Serialize;

use super::types::EncryptionConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{
    BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE,
    OP_META_KEY_IS_BUCKET_ARN,
};

/// Configures the encryption rules of a table bucket.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct PutTableBucketEncryptionRequest {
    /// The ARN of the table bucket.
    #[serde(skip)]
    pub table_bucket_arn: String,

    /// The encryption of the table bucket.
    #[serde(
        rename = "encryptionConfiguration",
        skip_serializing_if = "Option::is_none"
    )]
    pub encryption_configuration: Option<EncryptionConfiguration>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutTableBucketEncryptionResult {
    pub common: ResultCommon,
}

impl Client {
    /// Configures the encryption rules of a table bucket.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn put_table_bucket_encryption(
        &self,
        request: &PutTableBucketEncryptionRequest,
    ) -> Result<PutTableBucketEncryptionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutTableBucketEncryption".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "buckets/{}/encryption",
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

        let mut result = PutTableBucketEncryptionResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_table_bucket_encryption_body() {
        let request = PutTableBucketEncryptionRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            encryption_configuration: Some(EncryptionConfiguration {
                kms_key_arn: Some("acs:kms:cn-hangzhou:123:key/1".to_string()),
                sse_algorithm: Some("KMS".to_string()),
            }),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"encryptionConfiguration":{"kmsKeyArn":"acs:kms:cn-hangzhou:123:key/1","sseAlgorithm":"KMS"}}"#
        );
    }

    #[test]
    fn test_put_table_bucket_encryption_body_excludes_the_arn() {
        let request = PutTableBucketEncryptionRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            ..Default::default()
        };

        assert!(!serde_json::to_string(&request)
            .unwrap()
            .contains("acs:osstables"));
    }
}
