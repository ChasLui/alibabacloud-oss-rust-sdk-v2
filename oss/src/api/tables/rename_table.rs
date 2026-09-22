use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Serialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{
    BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE,
    OP_META_KEY_IS_BUCKET_ARN,
};

/// Renames a table, moves it to another namespace, or both.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct RenameTableRequest {
    /// The ARN of the table bucket.
    #[serde(skip)]
    pub table_bucket_arn: String,

    /// The namespace the table currently lives in.
    #[serde(skip)]
    pub namespace: String,

    /// The current name of the table.
    #[serde(skip)]
    pub name: String,

    /// The namespace to move the table to.
    #[serde(rename = "newNamespaceName", skip_serializing_if = "Option::is_none")]
    pub new_namespace_name: Option<String>,

    /// The new name of the table.
    #[serde(rename = "newName", skip_serializing_if = "Option::is_none")]
    pub new_name: Option<String>,

    /// The version of the table metadata the rename is based on.
    #[serde(rename = "versionToken", skip_serializing_if = "Option::is_none")]
    pub version_token: Option<String>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct RenameTableResult {
    pub common: ResultCommon,
}

impl Client {
    /// Renames a table, moves it to another namespace, or both.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn rename_table(
        &self,
        request: &RenameTableRequest,
    ) -> Result<RenameTableResult, Box<dyn std::error::Error + Send + Sync>> {
        if request.new_name.is_none() && request.new_namespace_name.is_none() {
            return Err("either NewTable or NewNamespace must be provided".into());
        }

        let mut input = OperationInput {
            op_name: "RenameTable".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "tables/{}/{}/{}/rename",
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

        let mut result = RenameTableResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rename_table_body() {
        let request = RenameTableRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            namespace: "space".to_string(),
            name: "table".to_string(),
            new_name: Some("renamed".to_string()),
            version_token: Some("v-1".to_string()),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"newName":"renamed","versionToken":"v-1"}"#
        );
    }

    #[test]
    fn test_rename_table_body_excludes_the_nop_fields() {
        let request = RenameTableRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            namespace: "space".to_string(),
            name: "table".to_string(),
            new_namespace_name: Some("other".to_string()),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"newNamespaceName":"other"}"#
        );
    }
}
