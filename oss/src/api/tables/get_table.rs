use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::arn::Arn;
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{
    ClientError, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE,
    OP_META_KEY_IS_BUCKET_ARN,
};

/// Queries information about a table.
///
/// A table is addressed either by `table_arn` alone or by the table bucket
/// ARN together with `namespace` and `name`.
#[derive(Debug, Default, OssRequestModel)]
pub struct GetTableRequest {
    /// The ARN of the table bucket.
    #[field(type = "query", rename = "tableBucketARN")]
    pub table_bucket_arn: Option<String>,

    /// The name of the table.
    #[field(type = "query", rename = "name")]
    pub name: Option<String>,

    /// The namespace of the table.
    #[field(type = "query", rename = "namespace")]
    pub namespace: Option<String>,

    /// The ARN of the table.
    #[field(type = "query", rename = "tableArn")]
    pub table_arn: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct GetTableResult {
    /// The time when the table was created.
    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// The account that created the table.
    #[serde(rename = "createdBy", skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,

    /// The format of the table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// The location of the table metadata.
    #[serde(rename = "metadataLocation", skip_serializing_if = "Option::is_none")]
    pub metadata_location: Option<String>,

    /// The time when the table was last modified.
    #[serde(rename = "modifiedAt", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,

    /// The account that last modified the table.
    #[serde(rename = "modifiedBy", skip_serializing_if = "Option::is_none")]
    pub modified_by: Option<String>,

    /// The name of the table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The namespace of the table, as its path segments.
    #[serde(default)]
    pub namespace: Vec<String>,

    /// The ID of the namespace.
    #[serde(rename = "namespaceId", skip_serializing_if = "Option::is_none")]
    pub namespace_id: Option<String>,

    /// The account that owns the table.
    #[serde(rename = "ownerAccountId", skip_serializing_if = "Option::is_none")]
    pub owner_account_id: Option<String>,

    /// The ARN of the table.
    #[serde(rename = "tableARN", skip_serializing_if = "Option::is_none")]
    pub table_arn: Option<String>,

    /// The ID of the table bucket.
    #[serde(rename = "tableBucketId", skip_serializing_if = "Option::is_none")]
    pub table_bucket_id: Option<String>,

    /// The type of the table.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// The version of the table metadata.
    #[serde(rename = "versionToken", skip_serializing_if = "Option::is_none")]
    pub version_token: Option<String>,

    /// The location of the table warehouse.
    #[serde(rename = "warehouseLocation", skip_serializing_if = "Option::is_none")]
    pub warehouse_location: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

/// Builds the `missing required field, {field}.` error Go raises from
/// `NewErrParamRequired`.
fn param_required(field: &str) -> Box<dyn std::error::Error + Send + Sync> {
    let message = format!("missing required field, {}.", field);
    Box::new(ClientError {
        code: "InvalidArgument".to_string(),
        message: message.clone(),
        err: Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            message,
        )),
    })
}

/// Rejects a request that addresses a table twice or not at all.
///
/// Mirrors Go's `checkGetTableRequest`.
fn check_get_table_request(
    request: &GetTableRequest,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match (&request.table_bucket_arn, &request.table_arn) {
        (Some(_), Some(_)) => Err(
            "must provide either table arn alone OR all of (table bucket arn, namespace, table \
             name) together"
                .into(),
        ),
        (Some(_), None) => {
            if request.namespace.is_none() {
                return Err(param_required("Namespace"));
            }
            if request.name.is_none() {
                return Err(param_required("Name"));
            }
            Ok(())
        }
        (None, Some(_)) => Ok(()),
        (None, None) => Err(param_required("TableBucketARN")),
    }
}

/// The ARN of the table bucket the request addresses.
///
/// A table ARN names its bucket in the segments before `/table`, so it is
/// rewritten into the bucket ARN the rest of the operation expects. Mirrors
/// Go's `parseBucketArn`.
fn parse_bucket_arn(
    request: &GetTableRequest,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(table_arn) = &request.table_arn {
        let parsed = Arn::parse(table_arn)?;
        let is_table = parsed.arn_resource.resource_type.as_deref() == Some("bucket")
            && parsed
                .arn_resource
                .qualifier
                .as_deref()
                .map_or(false, |qualifier| qualifier.starts_with("table/"));
        if !is_table {
            return Err("malformed table arn".into());
        }
        return Ok(table_arn.split("/table").next().map(|arn| arn.to_string()));
    }

    Ok(request.table_bucket_arn.clone())
}

impl Client {
    /// Queries information about a table.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn get_table(
        &self,
        request: &GetTableRequest,
    ) -> Result<GetTableResult, Box<dyn std::error::Error + Send + Sync>> {
        check_get_table_request(request)?;

        let mut input = OperationInput {
            op_name: "GetTable".to_string(),
            method: http::Method::GET,
            bucket: parse_bucket_arn(request)?,
            key: Some("get-table".to_string()),
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
        let mut result: GetTableResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUCKET_ARN: &str = "acs:osstables:cn-hangzhou:123:bucket/demo";
    const TABLE_ARN: &str = "acs:osstables:cn-hangzhou:123:bucket/demo/table/f13de3a6";

    #[test]
    fn test_get_table_request_query_names() {
        let request = GetTableRequest {
            table_bucket_arn: Some(BUCKET_ARN.to_string()),
            namespace: Some("space".to_string()),
            name: Some("table".to_string()),
            ..Default::default()
        };

        let query = request.query_map();
        assert_eq!(query.get("tableBucketARN").unwrap(), BUCKET_ARN);
        assert_eq!(query.get("namespace").unwrap(), "space");
        assert_eq!(query.get("name").unwrap(), "table");
        assert!(!query.contains_key("tableArn"));
    }

    #[test]
    fn test_parse_bucket_arn_from_a_table_arn() {
        let request = GetTableRequest {
            table_arn: Some(TABLE_ARN.to_string()),
            ..Default::default()
        };

        assert_eq!(
            parse_bucket_arn(&request).unwrap().as_deref(),
            Some(BUCKET_ARN)
        );
    }

    #[test]
    fn test_parse_bucket_arn_from_a_bucket_arn() {
        let request = GetTableRequest {
            table_bucket_arn: Some(BUCKET_ARN.to_string()),
            ..Default::default()
        };

        assert_eq!(
            parse_bucket_arn(&request).unwrap().as_deref(),
            Some(BUCKET_ARN)
        );
    }

    #[test]
    fn test_parse_bucket_arn_rejects_a_bucket_without_a_table_qualifier() {
        let request = GetTableRequest {
            table_arn: Some(BUCKET_ARN.to_string()),
            ..Default::default()
        };

        assert!(parse_bucket_arn(&request).is_err());
    }

    #[test]
    fn test_check_get_table_request() {
        let both = GetTableRequest {
            table_bucket_arn: Some(BUCKET_ARN.to_string()),
            table_arn: Some(TABLE_ARN.to_string()),
            ..Default::default()
        };
        assert!(check_get_table_request(&both).is_err());

        let missing_name = GetTableRequest {
            table_bucket_arn: Some(BUCKET_ARN.to_string()),
            namespace: Some("space".to_string()),
            ..Default::default()
        };
        assert!(check_get_table_request(&missing_name).is_err());

        let neither = GetTableRequest::default();
        assert!(check_get_table_request(&neither).is_err());

        let by_arn = GetTableRequest {
            table_arn: Some(TABLE_ARN.to_string()),
            ..Default::default()
        };
        assert!(check_get_table_request(&by_arn).is_ok());

        let by_name = GetTableRequest {
            table_bucket_arn: Some(BUCKET_ARN.to_string()),
            namespace: Some("space".to_string()),
            name: Some("table".to_string()),
            ..Default::default()
        };
        assert!(check_get_table_request(&by_name).is_ok());
    }

    #[test]
    fn test_get_table_result_deserialize() {
        let body = r#"{"name":"table","namespace":["space"],"format":"iceberg",
            "metadataLocation":"oss://demo/metadata/1.json","tableARN":"acs:osstables:cn-hangzhou:123:bucket/demo/table/1",
            "versionToken":"v-1","warehouseLocation":"oss://demo/warehouse"}"#;

        let result: GetTableResult = serde_json::from_str(body).unwrap();

        assert_eq!(result.name.as_deref(), Some("table"));
        assert_eq!(result.namespace, vec!["space".to_string()]);
        assert_eq!(result.format.as_deref(), Some("iceberg"));
        assert_eq!(result.version_token.as_deref(), Some("v-1"));
    }
}
