use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use super::types::SettingsDetail;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE, OP_META_KEY_IS_BUCKET_ARN};

/// The maintenance configuration of a table bucket.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MaintenanceConfiguration {
    /// The unreferenced file removal maintenance job.
    #[serde(
        rename = "icebergUnreferencedFileRemoval",
        skip_serializing_if = "Option::is_none"
    )]
    pub iceberg_unreferenced_file_removal: Option<IcebergUnreferencedFileRemoval>,
}

/// The state and settings of the unreferenced file removal maintenance job.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IcebergUnreferencedFileRemoval {
    /// The settings of the job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<IcebergSettings>,

    /// The status of the job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// The settings container of the unreferenced file removal maintenance job.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IcebergSettings {
    /// The retention thresholds of the job.
    #[serde(
        rename = "icebergUnreferencedFileRemoval",
        skip_serializing_if = "Option::is_none"
    )]
    pub iceberg_unreferenced_file_removal: Option<SettingsDetail>,
}

/// Queries the maintenance configuration of a table bucket.
#[derive(Debug, Default, OssRequestModel)]
pub struct GetTableBucketMaintenanceConfigurationRequest {
    /// The ARN of the table bucket.
    pub table_bucket_arn: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct GetTableBucketMaintenanceConfigurationResult {
    /// The container that stores the maintenance configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<MaintenanceConfiguration>,

    /// The ARN of the table bucket.
    #[serde(rename = "tableBucketARN", skip_serializing_if = "Option::is_none")]
    pub table_bucket_arn: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the maintenance configuration of a table bucket.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn get_table_bucket_maintenance_configuration(
        &self,
        request: &GetTableBucketMaintenanceConfigurationRequest,
    ) -> Result<
        GetTableBucketMaintenanceConfigurationResult,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let mut input = OperationInput {
            op_name: "GetTableBucketMaintenanceConfiguration".to_string(),
            method: http::Method::GET,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "buckets/{}/maintenance",
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

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let mut result: GetTableBucketMaintenanceConfigurationResult =
            serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_table_bucket_maintenance_configuration_result_deserialize() {
        let body = r#"{"configuration":{"icebergUnreferencedFileRemoval":{
            "settings":{"icebergUnreferencedFileRemoval":{"nonCurrentDays":3,"unreferencedDays":5}},
            "status":"enabled"}},"tableBucketARN":"acs:osstables:cn-hangzhou:123:bucket/demo"}"#;

        let result: GetTableBucketMaintenanceConfigurationResult =
            serde_json::from_str(body).unwrap();

        let configuration = result.configuration.expect("configuration");
        let removal = configuration
            .iceberg_unreferenced_file_removal
            .expect("unreferenced file removal");
        assert_eq!(removal.status.as_deref(), Some("enabled"));
        let detail = removal
            .settings
            .expect("settings")
            .iceberg_unreferenced_file_removal
            .expect("settings detail");
        assert_eq!(detail.non_current_days, Some(3));
        assert_eq!(detail.unreferenced_days, Some(5));
        assert!(result.table_bucket_arn.is_some());
    }

    #[test]
    fn test_get_table_bucket_maintenance_configuration_result_without_body() {
        let result: GetTableBucketMaintenanceConfigurationResult =
            serde_json::from_str("{}").unwrap();

        assert!(result.configuration.is_none());
    }
}
