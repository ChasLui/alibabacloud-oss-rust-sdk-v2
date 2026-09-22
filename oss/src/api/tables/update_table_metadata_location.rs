use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{
    BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE,
    OP_META_KEY_IS_BUCKET_ARN,
};

/// Updates the metadata location of a table.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct UpdateTableMetadataLocationRequest {
    /// The ARN of the table bucket.
    #[serde(skip)]
    pub table_bucket_arn: String,

    /// The namespace of the table.
    #[serde(skip)]
    pub namespace: String,

    /// The name of the table.
    #[serde(skip)]
    pub name: String,

    /// The new location of the table metadata.
    #[serde(rename = "metadataLocation", skip_serializing_if = "Option::is_none")]
    pub metadata_location: Option<String>,

    /// The version of the table metadata the update is based on.
    #[serde(rename = "versionToken", skip_serializing_if = "Option::is_none")]
    pub version_token: Option<String>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct UpdateTableMetadataLocationResult {
    /// The location of the table metadata.
    #[serde(rename = "metadataLocation", skip_serializing_if = "Option::is_none")]
    pub metadata_location: Option<String>,

    /// The name of the table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The namespace of the table, as its path segments.
    #[serde(default)]
    pub namespace: Vec<String>,

    /// The ARN of the table.
    #[serde(rename = "tableARN", skip_serializing_if = "Option::is_none")]
    pub table_arn: Option<String>,

    /// The version of the table metadata.
    #[serde(rename = "versionToken", skip_serializing_if = "Option::is_none")]
    pub version_token: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Updates the metadata location of a table.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn update_table_metadata_location(
        &self,
        request: &UpdateTableMetadataLocationRequest,
    ) -> Result<UpdateTableMetadataLocationResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "UpdateTableMetadataLocation".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "tables/{}/{}/{}/metadata-location",
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

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let mut result: UpdateTableMetadataLocationResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_table_metadata_location_body() {
        let request = UpdateTableMetadataLocationRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            namespace: "space".to_string(),
            name: "table".to_string(),
            metadata_location: Some("oss://demo/metadata/2.json".to_string()),
            version_token: Some("v-2".to_string()),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"metadataLocation":"oss://demo/metadata/2.json","versionToken":"v-2"}"#
        );
    }

    #[test]
    fn test_update_table_metadata_location_result_deserialize() {
        let body = r#"{"metadataLocation":"oss://demo/metadata/2.json","name":"table",
            "namespace":["space"],"tableARN":"acs:osstables:cn-hangzhou:123:bucket/demo/table/1",
            "versionToken":"v-2"}"#;

        let result: UpdateTableMetadataLocationResult = serde_json::from_str(body).unwrap();

        assert_eq!(result.name.as_deref(), Some("table"));
        assert_eq!(result.namespace, vec!["space".to_string()]);
        assert_eq!(result.version_token.as_deref(), Some("v-2"));
    }
}
