//! Model types shared by more than one table bucket operation.
//!
//! Go declares these in the operation file that uses them first; Rust needs a
//! single home because the modules are separate. Mirrors the Go type
//! definitions field for field.

use serde::{Deserialize, Serialize};

/// The encryption rules of a table bucket or a table.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EncryptionConfiguration {
    /// The ARN of the KMS key used for encryption.
    #[serde(rename = "kmsKeyArn", skip_serializing_if = "Option::is_none")]
    pub kms_key_arn: Option<String>,

    /// The encryption algorithm, e.g. `AES256`.
    #[serde(rename = "sseAlgorithm", skip_serializing_if = "Option::is_none")]
    pub sse_algorithm: Option<String>,
}

/// The retention thresholds of the unreferenced file removal maintenance job.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SettingsDetail {
    /// The number of days after which non-current files are removed.
    #[serde(rename = "nonCurrentDays", skip_serializing_if = "Option::is_none")]
    pub non_current_days: Option<i32>,

    /// The number of days after which unreferenced files are removed.
    #[serde(rename = "unreferencedDays", skip_serializing_if = "Option::is_none")]
    pub unreferenced_days: Option<i32>,
}

/// The settings of the Iceberg compaction maintenance job.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IcebergCompactionSettingsDetail {
    /// The compaction strategy, e.g. `auto`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,

    /// The target size of a compacted file, in MB.
    #[serde(rename = "targetFileSizeMB", skip_serializing_if = "Option::is_none")]
    pub target_file_size_mb: Option<i32>,
}

/// The settings of the Iceberg snapshot management maintenance job.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IcebergSnapshotManagementSettingsDetail {
    /// The maximum age of a snapshot before it is removed, in hours.
    #[serde(
        rename = "maxSnapshotAgeHours",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_snapshot_age_hours: Option<i32>,

    /// The minimum number of snapshots to keep.
    #[serde(rename = "minSnapshotsToKeep", skip_serializing_if = "Option::is_none")]
    pub min_snapshots_to_keep: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_configuration_round_trip() {
        let configuration: EncryptionConfiguration = serde_json::from_str(
            r#"{"kmsKeyArn":"acs:kms:cn-hangzhou:123:key/1","sseAlgorithm":"KMS"}"#,
        )
        .unwrap();

        assert_eq!(configuration.sse_algorithm.as_deref(), Some("KMS"));
        assert_eq!(
            serde_json::to_string(&configuration).unwrap(),
            r#"{"kmsKeyArn":"acs:kms:cn-hangzhou:123:key/1","sseAlgorithm":"KMS"}"#
        );
    }

    #[test]
    fn test_settings_detail_deserialize() {
        let settings: SettingsDetail =
            serde_json::from_str(r#"{"nonCurrentDays":3,"unreferencedDays":5}"#).unwrap();

        assert_eq!(settings.non_current_days, Some(3));
        assert_eq!(settings.unreferenced_days, Some(5));
    }
}
