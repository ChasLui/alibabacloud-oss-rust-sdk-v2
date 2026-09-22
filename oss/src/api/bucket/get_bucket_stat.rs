use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketStatRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl GetBucketStatRequest {
    pub fn new(bucket: &str) -> Self {
        Self {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(default)]
pub struct GetBucketStatResult {
    /// The storage capacity of the bucket. Unit: bytes.
    #[serde(rename = "Storage")]
    pub storage: i64,

    /// The total number of objects that are stored in the bucket.
    #[serde(rename = "ObjectCount")]
    pub object_count: i64,

    /// The number of multipart upload tasks that have been initiated but are
    /// not completed or canceled.
    #[serde(rename = "MultipartUploadCount")]
    pub multipart_upload_count: i64,

    /// The number of LiveChannels in the bucket.
    #[serde(rename = "LiveChannelCount")]
    pub live_channel_count: i64,

    /// The time when the obtained information is last modified. The value of
    /// this element is a UNIX timestamp. Unit: seconds.
    #[serde(rename = "LastModifiedTime")]
    pub last_modified_time: i64,

    /// The storage usage of Standard objects in the bucket. Unit: bytes.
    #[serde(rename = "StandardStorage")]
    pub standard_storage: i64,

    /// The number of Standard objects in the bucket.
    #[serde(rename = "StandardObjectCount")]
    pub standard_object_count: i64,

    /// The billed storage usage of Infrequent Access (IA) objects in the
    /// bucket. Unit: bytes.
    #[serde(rename = "InfrequentAccessStorage")]
    pub infrequent_access_storage: i64,

    /// The actual storage usage of IA objects in the bucket. Unit: bytes.
    #[serde(rename = "InfrequentAccessRealStorage")]
    pub infrequent_access_real_storage: i64,

    /// The number of IA objects in the bucket.
    #[serde(rename = "InfrequentAccessObjectCount")]
    pub infrequent_access_object_count: i64,

    /// The billed storage usage of Archive objects in the bucket. Unit: bytes.
    #[serde(rename = "ArchiveStorage")]
    pub archive_storage: i64,

    /// The actual storage usage of Archive objects in the bucket. Unit: bytes.
    #[serde(rename = "ArchiveRealStorage")]
    pub archive_real_storage: i64,

    /// The number of Archive objects in the bucket.
    #[serde(rename = "ArchiveObjectCount")]
    pub archive_object_count: i64,

    /// The billed storage usage of Cold Archive objects in the bucket.
    /// Unit: bytes.
    #[serde(rename = "ColdArchiveStorage")]
    pub cold_archive_storage: i64,

    /// The actual storage usage of Cold Archive objects in the bucket.
    /// Unit: bytes.
    #[serde(rename = "ColdArchiveRealStorage")]
    pub cold_archive_real_storage: i64,

    /// The number of Cold Archive objects in the bucket.
    #[serde(rename = "ColdArchiveObjectCount")]
    pub cold_archive_object_count: i64,

    /// The number of Deep Cold Archive objects in the bucket.
    #[serde(rename = "DeepColdArchiveObjectCount")]
    pub deep_cold_archive_object_count: i64,

    /// The billed storage usage of Deep Cold Archive objects in the bucket.
    /// Unit: bytes.
    #[serde(rename = "DeepColdArchiveStorage")]
    pub deep_cold_archive_storage: i64,

    /// The actual storage usage of Deep Cold Archive objects in the bucket.
    /// Unit: bytes.
    #[serde(rename = "DeepColdArchiveRealStorage")]
    pub deep_cold_archive_real_storage: i64,

    /// The number of multipart parts in the bucket.
    #[serde(rename = "MultipartPartCount")]
    pub multipart_part_count: i64,

    /// The number of delete markers in the bucket.
    #[serde(rename = "DeleteMarkerCount")]
    pub delete_marker_count: i64,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the storage capacity of a specified bucket and the number of
    /// objects that are stored in the bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketStatRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketStatRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketStatRequest::new("my-bucket");
    ///
    /// match client.get_bucket_stat(&request).await {
    ///     Ok(result) => {
    ///         println!(
    ///             "Storage: {} bytes, objects: {}",
    ///             result.storage, result.object_count
    ///         );
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket stat: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_stat(
        &self,
        request: &GetBucketStatRequest,
    ) -> Result<GetBucketStatResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketStat".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("stat", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetBucketStatResult = quick_xml::de::from_str(&data_str)?;

        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_bucket_stat_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<BucketStat>
  <Storage>1600</Storage>
  <ObjectCount>230</ObjectCount>
  <MultipartUploadCount>6</MultipartUploadCount>
  <LiveChannelCount>4</LiveChannelCount>
  <LastModifiedTime>1660191839</LastModifiedTime>
  <StandardStorage>1000</StandardStorage>
  <StandardObjectCount>100</StandardObjectCount>
  <InfrequentAccessStorage>200</InfrequentAccessStorage>
  <InfrequentAccessRealStorage>180</InfrequentAccessRealStorage>
  <InfrequentAccessObjectCount>50</InfrequentAccessObjectCount>
  <ArchiveStorage>300</ArchiveStorage>
  <ArchiveRealStorage>280</ArchiveRealStorage>
  <ArchiveObjectCount>60</ArchiveObjectCount>
  <ColdArchiveStorage>100</ColdArchiveStorage>
  <ColdArchiveRealStorage>90</ColdArchiveRealStorage>
  <ColdArchiveObjectCount>10</ColdArchiveObjectCount>
  <DeepColdArchiveObjectCount>5</DeepColdArchiveObjectCount>
  <DeepColdArchiveStorage>50</DeepColdArchiveStorage>
  <DeepColdArchiveRealStorage>45</DeepColdArchiveRealStorage>
  <MultipartPartCount>7</MultipartPartCount>
  <DeleteMarkerCount>9</DeleteMarkerCount>
</BucketStat>"#;
        let result: GetBucketStatResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.storage, 1600);
        assert_eq!(result.object_count, 230);
        assert_eq!(result.multipart_upload_count, 6);
        assert_eq!(result.live_channel_count, 4);
        assert_eq!(result.last_modified_time, 1660191839);
        assert_eq!(result.standard_storage, 1000);
        assert_eq!(result.standard_object_count, 100);
        assert_eq!(result.infrequent_access_storage, 200);
        assert_eq!(result.infrequent_access_real_storage, 180);
        assert_eq!(result.infrequent_access_object_count, 50);
        assert_eq!(result.archive_storage, 300);
        assert_eq!(result.archive_real_storage, 280);
        assert_eq!(result.archive_object_count, 60);
        assert_eq!(result.cold_archive_storage, 100);
        assert_eq!(result.cold_archive_real_storage, 90);
        assert_eq!(result.cold_archive_object_count, 10);
        assert_eq!(result.deep_cold_archive_object_count, 5);
        assert_eq!(result.deep_cold_archive_storage, 50);
        assert_eq!(result.deep_cold_archive_real_storage, 45);
        assert_eq!(result.multipart_part_count, 7);
        assert_eq!(result.delete_marker_count, 9);
    }

    #[test]
    fn test_bucket_stat_deserialize_partial() {
        // Missing elements fall back to zero, matching Go's non-pointer int64
        // fields.
        let xml = r#"<BucketStat><Storage>10</Storage><ObjectCount>1</ObjectCount></BucketStat>"#;
        let result: GetBucketStatResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.storage, 10);
        assert_eq!(result.object_count, 1);
        assert_eq!(result.multipart_upload_count, 0);
        assert_eq!(result.delete_marker_count, 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_stat() {
        let config = match load_test_config() {
            Some(cfg) => cfg,
            None => {
                eprintln!("Test configuration not found. Skipping test.");
                return;
            }
        };

        let client = Client::new(
            &Config::default()
                .with_region(&config.region)
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    &config.access_key_id,
                    &config.access_key_secret,
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V4),
        );

        let result = client
            .get_bucket_stat(&GetBucketStatRequest::new(&config.bucket))
            .await;
        assert!(result.is_ok(), "get_bucket_stat failed: {:?}", result.err());
        let result = result.unwrap();
        assert_eq!(result.common.status, http::StatusCode::OK);
        assert!(result.storage >= 0);
    }
}
