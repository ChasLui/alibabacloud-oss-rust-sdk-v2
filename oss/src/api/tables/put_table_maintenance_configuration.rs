use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Serialize;

use super::types::{IcebergCompactionSettingsDetail, IcebergSnapshotManagementSettingsDetail};
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{
    BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE,
    OP_META_KEY_IS_BUCKET_ARN,
};

/// The value of a table maintenance job configuration.
#[derive(Debug, Default, Serialize)]
pub struct TableMaintenanceValue {
    /// The settings of the job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<TableMaintenanceSettings>,

    /// The status of the job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// The settings of the maintenance jobs of a table.
#[derive(Debug, Default, Serialize)]
pub struct TableMaintenanceSettings {
    /// The settings of the Iceberg compaction job.
    #[serde(rename = "icebergCompaction", skip_serializing_if = "Option::is_none")]
    pub iceberg_compaction: Option<IcebergCompactionSettingsDetail>,

    /// The settings of the Iceberg snapshot management job.
    #[serde(
        rename = "icebergSnapshotManagement",
        skip_serializing_if = "Option::is_none"
    )]
    pub iceberg_snapshot_management: Option<IcebergSnapshotManagementSettingsDetail>,
}

/// Configures a maintenance job of a table.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct PutTableMaintenanceConfigurationRequest {
    /// The ARN of the table bucket.
    #[serde(skip)]
    pub table_bucket_arn: String,

    /// The namespace of the table.
    #[serde(skip)]
    pub namespace: String,

    /// The name of the table.
    #[serde(skip)]
    pub name: String,

    /// The type of the maintenance job, e.g. `icebergCompaction`.
    #[serde(skip)]
    pub r#type: String,

    /// The container that stores the maintenance configuration.
    #[serde(rename = "value", skip_serializing_if = "Option::is_none")]
    pub value: Option<TableMaintenanceValue>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutTableMaintenanceConfigurationResult {
    pub common: ResultCommon,
}

impl Client {
    /// Configures a maintenance job of a table.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn put_table_maintenance_configuration(
        &self,
        request: &PutTableMaintenanceConfigurationRequest,
    ) -> Result<PutTableMaintenanceConfigurationResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "PutTableMaintenanceConfiguration".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "tables/{}/{}/{}/maintenance/{}",
                escape_path(&request.table_bucket_arn, true),
                escape_path(&request.namespace, true),
                escape_path(&request.name, true),
                escape_path(&request.r#type, true)
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

        let mut result = PutTableMaintenanceConfigurationResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_table_maintenance_configuration_body() {
        let request = PutTableMaintenanceConfigurationRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            namespace: "space".to_string(),
            name: "table".to_string(),
            r#type: "icebergCompaction".to_string(),
            value: Some(TableMaintenanceValue {
                settings: Some(TableMaintenanceSettings {
                    iceberg_compaction: Some(IcebergCompactionSettingsDetail {
                        strategy: Some("auto".to_string()),
                        target_file_size_mb: Some(128),
                    }),
                    iceberg_snapshot_management: None,
                }),
                status: Some("enabled".to_string()),
            }),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"value":{"settings":{"icebergCompaction":{"strategy":"auto","targetFileSizeMB":128}},"status":"enabled"}}"#
        );
    }

    #[test]
    fn test_put_table_maintenance_configuration_body_excludes_the_nop_fields() {
        let request = PutTableMaintenanceConfigurationRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            namespace: "space".to_string(),
            name: "table".to_string(),
            r#type: "icebergCompaction".to_string(),
            ..Default::default()
        };

        assert_eq!(serde_json::to_string(&request).unwrap(), "{}");
    }
}
