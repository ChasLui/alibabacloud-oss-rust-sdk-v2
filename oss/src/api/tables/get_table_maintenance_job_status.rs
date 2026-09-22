use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE, OP_META_KEY_IS_BUCKET_ARN};

/// The status of a maintenance job.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StatusDetail {
    /// The message reported when the last run failed.
    #[serde(rename = "failureMessage", skip_serializing_if = "Option::is_none")]
    pub failure_message: Option<String>,

    /// The time when the job last ran.
    #[serde(rename = "lastRunTimestamp", skip_serializing_if = "Option::is_none")]
    pub last_run_timestamp: Option<String>,

    /// The status of the job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// The status of every maintenance job of a table.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MaintenanceJobStatus {
    /// The status of the Iceberg compaction job.
    #[serde(rename = "icebergCompaction", skip_serializing_if = "Option::is_none")]
    pub iceberg_compaction: Option<StatusDetail>,

    /// The status of the Iceberg snapshot management job.
    #[serde(
        rename = "icebergSnapshotManagement",
        skip_serializing_if = "Option::is_none"
    )]
    pub iceberg_snapshot_management: Option<StatusDetail>,

    /// The status of the unreferenced file removal job.
    #[serde(
        rename = "icebergUnreferencedFileRemoval",
        skip_serializing_if = "Option::is_none"
    )]
    pub iceberg_unreferenced_file_removal: Option<StatusDetail>,
}

/// Queries the maintenance job status of a table.
#[derive(Debug, Default, OssRequestModel)]
pub struct GetTableMaintenanceJobStatusRequest {
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
pub struct GetTableMaintenanceJobStatusResult {
    /// The status of every maintenance job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MaintenanceJobStatus>,

    /// The ARN of the table.
    #[serde(rename = "tableARN", skip_serializing_if = "Option::is_none")]
    pub table_arn: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the maintenance job status of a table.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn get_table_maintenance_job_status(
        &self,
        request: &GetTableMaintenanceJobStatusRequest,
    ) -> Result<GetTableMaintenanceJobStatusResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetTableMaintenanceJobStatus".to_string(),
            method: http::Method::GET,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "tables/{}/{}/{}/maintenance-job-status",
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
        let mut result: GetTableMaintenanceJobStatusResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_table_maintenance_job_status_result_deserialize() {
        let body = r#"{"status":{
            "icebergCompaction":{"status":"successful","lastRunTimestamp":"2026-01-01"},
            "icebergSnapshotManagement":{"status":"failed","failureMessage":"boom"}},
            "tableARN":"acs:osstables:cn-hangzhou:123:bucket/demo/table/1"}"#;

        let result: GetTableMaintenanceJobStatusResult = serde_json::from_str(body).unwrap();

        let status = result.status.expect("status");
        assert_eq!(
            status
                .iceberg_compaction
                .expect("compaction")
                .status
                .as_deref(),
            Some("successful")
        );
        assert_eq!(
            status
                .iceberg_snapshot_management
                .expect("snapshot management")
                .failure_message
                .as_deref(),
            Some("boom")
        );
        assert!(status.iceberg_unreferenced_file_removal.is_none());
    }

    #[test]
    fn test_get_table_maintenance_job_status_result_without_body() {
        let result: GetTableMaintenanceJobStatusResult = serde_json::from_str("{}").unwrap();

        assert!(result.status.is_none());
    }
}
