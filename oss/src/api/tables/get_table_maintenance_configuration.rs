use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use super::types::{IcebergCompactionSettingsDetail, IcebergSnapshotManagementSettingsDetail};
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE, OP_META_KEY_IS_BUCKET_ARN};

/// The maintenance configuration of a table.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TableMaintenanceConfiguration {
    /// The Iceberg compaction maintenance job.
    #[serde(rename = "icebergCompaction", skip_serializing_if = "Option::is_none")]
    pub iceberg_compaction: Option<IcebergCompaction>,

    /// The Iceberg snapshot management maintenance job.
    #[serde(
        rename = "icebergSnapshotManagement",
        skip_serializing_if = "Option::is_none"
    )]
    pub iceberg_snapshot_management: Option<IcebergSnapshotManagement>,
}

/// The state and settings of the Iceberg compaction maintenance job.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IcebergCompaction {
    /// The settings of the job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<IcebergCompactionSettings>,

    /// The status of the job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// The settings container of the Iceberg compaction maintenance job.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IcebergCompactionSettings {
    /// The compaction settings.
    #[serde(rename = "icebergCompaction", skip_serializing_if = "Option::is_none")]
    pub iceberg_compaction: Option<IcebergCompactionSettingsDetail>,
}

/// The state and settings of the Iceberg snapshot management maintenance job.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IcebergSnapshotManagement {
    /// The settings of the job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<IcebergSnapshotManagementSettings>,

    /// The status of the job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// The settings container of the Iceberg snapshot management maintenance job.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IcebergSnapshotManagementSettings {
    /// The snapshot management settings.
    #[serde(
        rename = "icebergSnapshotManagement",
        skip_serializing_if = "Option::is_none"
    )]
    pub iceberg_snapshot_management: Option<IcebergSnapshotManagementSettingsDetail>,
}

/// Queries the maintenance configuration of a table.
#[derive(Debug, Default, OssRequestModel)]
pub struct GetTableMaintenanceConfigurationRequest {
    /// The ARN of the table bucket.
    pub table_bucket_arn: String,

    /// The namespace of the table.
    pub namespace: String,

    /// The name of the table.
    pub name: String,

    /// The ARN of the table, sent as a header.
    #[field(type = "header", rename = "x-oss-table-arn")]
    pub table_arn: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct GetTableMaintenanceConfigurationResult {
    /// The container that stores the maintenance configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<TableMaintenanceConfiguration>,

    /// The ARN of the table.
    #[serde(rename = "tableARN", skip_serializing_if = "Option::is_none")]
    pub table_arn: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the maintenance configuration of a table.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn get_table_maintenance_configuration(
        &self,
        request: &GetTableMaintenanceConfigurationRequest,
    ) -> Result<GetTableMaintenanceConfigurationResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "GetTableMaintenanceConfiguration".to_string(),
            method: http::Method::GET,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "tables/{}/{}/{}/maintenance",
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
        let mut result: GetTableMaintenanceConfigurationResult =
            serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_table_maintenance_configuration_request_header() {
        let request = GetTableMaintenanceConfigurationRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            namespace: "space".to_string(),
            name: "table".to_string(),
            table_arn: Some("acs:osstables:cn-hangzhou:123:bucket/demo/table/1".to_string()),
            ..Default::default()
        };

        assert_eq!(
            request.header_map().get("x-oss-table-arn").unwrap(),
            "acs:osstables:cn-hangzhou:123:bucket/demo/table/1"
        );
    }

    #[test]
    fn test_get_table_maintenance_configuration_result_deserialize() {
        let body = r#"{"configuration":{
            "icebergCompaction":{"settings":{"icebergCompaction":{"strategy":"auto","targetFileSizeMB":128}},"status":"enabled"},
            "icebergSnapshotManagement":{"settings":{"icebergSnapshotManagement":{"maxSnapshotAgeHours":24,"minSnapshotsToKeep":2}},"status":"enabled"}},
            "tableARN":"acs:osstables:cn-hangzhou:123:bucket/demo/table/1"}"#;

        let result: GetTableMaintenanceConfigurationResult = serde_json::from_str(body).unwrap();

        let configuration = result.configuration.expect("configuration");
        let compaction = configuration.iceberg_compaction.expect("compaction");
        let compaction_settings = compaction
            .settings
            .expect("settings")
            .iceberg_compaction
            .expect("compaction settings");
        assert_eq!(compaction_settings.strategy.as_deref(), Some("auto"));
        assert_eq!(compaction_settings.target_file_size_mb, Some(128));

        let snapshot = configuration
            .iceberg_snapshot_management
            .expect("snapshot management");
        let snapshot_settings = snapshot
            .settings
            .expect("settings")
            .iceberg_snapshot_management
            .expect("snapshot settings");
        assert_eq!(snapshot_settings.max_snapshot_age_hours, Some(24));
        assert_eq!(snapshot_settings.min_snapshots_to_keep, Some(2));
    }
}
