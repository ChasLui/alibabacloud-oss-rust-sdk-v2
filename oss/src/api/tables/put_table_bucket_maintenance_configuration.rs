use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Serialize;

use super::types::SettingsDetail;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{escape_path, modify_request, update_content_length, update_content_md5};
use crate::{
    BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE,
    OP_META_KEY_IS_BUCKET_ARN,
};

/// The value of a table bucket maintenance job configuration.
#[derive(Debug, Default, Serialize)]
pub struct MaintenanceValue {
    /// The settings of the job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<MaintenanceSettings>,

    /// The status of the job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// The settings of the maintenance jobs of a table bucket.
#[derive(Debug, Default, Serialize)]
pub struct MaintenanceSettings {
    /// The retention thresholds of the unreferenced file removal job.
    #[serde(
        rename = "icebergUnreferencedFileRemoval",
        skip_serializing_if = "Option::is_none"
    )]
    pub iceberg_unreferenced_file_removal: Option<SettingsDetail>,
}

/// Configures a maintenance job of a table bucket.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct PutTableBucketMaintenanceConfigurationRequest {
    /// The ARN of the table bucket.
    #[serde(skip)]
    pub table_bucket_arn: String,

    /// The type of the maintenance job, e.g. `icebergUnreferencedFileRemoval`.
    #[serde(skip)]
    pub r#type: String,

    /// The container that stores the maintenance configuration.
    #[serde(rename = "value", skip_serializing_if = "Option::is_none")]
    pub value: Option<MaintenanceValue>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutTableBucketMaintenanceConfigurationResult {
    pub common: ResultCommon,
}

impl Client {
    /// Configures a maintenance job of a table bucket.
    ///
    /// Requires a client built with [`Client::new_tables`].
    pub async fn put_table_bucket_maintenance_configuration(
        &self,
        request: &PutTableBucketMaintenanceConfigurationRequest,
    ) -> Result<
        PutTableBucketMaintenanceConfigurationResult,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let mut input = OperationInput {
            op_name: "PutTableBucketMaintenanceConfiguration".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.table_bucket_arn.clone()),
            key: Some(format!(
                "buckets/{}/maintenance/{}",
                escape_path(&request.table_bucket_arn, true),
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

        let mut result = PutTableBucketMaintenanceConfigurationResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_table_bucket_maintenance_configuration_body() {
        let request = PutTableBucketMaintenanceConfigurationRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            r#type: "icebergUnreferencedFileRemoval".to_string(),
            value: Some(MaintenanceValue {
                settings: Some(MaintenanceSettings {
                    iceberg_unreferenced_file_removal: Some(SettingsDetail {
                        non_current_days: Some(3),
                        unreferenced_days: None,
                    }),
                }),
                status: Some("enabled".to_string()),
            }),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"value":{"settings":{"icebergUnreferencedFileRemoval":{"nonCurrentDays":3}},"status":"enabled"}}"#
        );
    }

    #[test]
    fn test_put_table_bucket_maintenance_configuration_body_excludes_the_arn_and_type() {
        let request = PutTableBucketMaintenanceConfigurationRequest {
            table_bucket_arn: "acs:osstables:cn-hangzhou:123:bucket/demo".to_string(),
            r#type: "icebergUnreferencedFileRemoval".to_string(),
            ..Default::default()
        };

        assert_eq!(serde_json::to_string(&request).unwrap(), "{}");
    }
}
