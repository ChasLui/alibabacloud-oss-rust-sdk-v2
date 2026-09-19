use crate::HTTP_HEADER_CONTENT_TYPE;
use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::OperationInput;
use crate::OperationOutput;

/// The container that stores regions in which the destination bucket can be
/// located with the TransferType information.
#[derive(Debug, Default, Deserialize)]
pub struct LocationTransferTypeConstraint {
    /// The container that stores regions in which the destination bucket can
    /// be located with the TransferType information.
    #[serde(rename = "LocationTransferType", default)]
    pub location_transfer_types: Vec<LocationTransferType>,
}

/// The region in which the destination bucket can be located, with the
/// transfer type information.
#[derive(Debug, Default, Deserialize)]
pub struct LocationTransferType {
    /// The region in which the destination bucket can be located.
    #[serde(rename = "Location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// The container that stores the transfer type.
    #[serde(rename = "TransferTypes", skip_serializing_if = "Option::is_none")]
    pub transfer_types: Option<TransferTypes>,
}

/// The container that stores the transfer type.
#[derive(Debug, Default, Deserialize)]
pub struct TransferTypes {
    /// The data transfer type that is used to transfer data in data
    /// replication. Valid values: internal (default), oss_acc.
    #[serde(rename = "Type", default)]
    pub types: Vec<String>,
}

/// The container that stores regions in which RTC can be enabled.
#[derive(Debug, Default, Deserialize)]
pub struct LocationRTCConstraint {
    /// The regions where RTC is supported.
    #[serde(rename = "Location", default)]
    pub locations: Vec<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketReplicationLocationRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetBucketReplicationLocationResult {
    /// The regions in which the destination bucket can be located.
    #[serde(rename = "Location", default)]
    pub locations: Vec<String>,

    /// The container that stores regions in which the destination bucket can
    /// be located with TransferType specified.
    #[serde(rename = "LocationTransferTypeConstraint", skip_serializing_if = "Option::is_none")]
    pub location_transfer_type_constraint: Option<LocationTransferTypeConstraint>,

    /// The container that stores regions in which the RTC can be enabled.
    #[serde(rename = "LocationRTCConstraint", skip_serializing_if = "Option::is_none")]
    pub location_rtc_constraint: Option<LocationRTCConstraint>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the regions in which available destination buckets reside. You
    /// can determine the region of the destination bucket to which the data
    /// in the source bucket are replicated based on the returned response.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketReplicationLocationRequest` containing the
    ///   bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketReplicationLocationRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketReplicationLocationRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_replication_location(&request).await {
    ///     Ok(result) => {
    ///         println!("Replication locations: {:?}", result.locations);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket replication location: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_replication_location(
        &self,
        request: &GetBucketReplicationLocationRequest,
    ) -> Result<GetBucketReplicationLocationResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketReplicationLocation".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("replicationLocation", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["replicationLocation".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetBucketReplicationLocationResult = quick_xml::de::from_str(&data_str)?;
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
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_get_bucket_replication_location_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ReplicationLocation>
  <Location>oss-cn-beijing</Location>
  <Location>oss-cn-shanghai</Location>
  <LocationTransferTypeConstraint>
    <LocationTransferType>
      <Location>oss-cn-beijing</Location>
      <TransferTypes>
        <Type>internal</Type>
        <Type>oss_acc</Type>
      </TransferTypes>
    </LocationTransferType>
  </LocationTransferTypeConstraint>
  <LocationRTCConstraint>
    <Location>oss-cn-beijing</Location>
  </LocationRTCConstraint>
</ReplicationLocation>"#;

        let result: GetBucketReplicationLocationResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(
            result.locations,
            vec!["oss-cn-beijing".to_string(), "oss-cn-shanghai".to_string()]
        );
        let lttc = result.location_transfer_type_constraint.as_ref().unwrap();
        assert_eq!(lttc.location_transfer_types.len(), 1);
        let ltt = &lttc.location_transfer_types[0];
        assert_eq!(ltt.location.as_deref(), Some("oss-cn-beijing"));
        assert_eq!(
            ltt.transfer_types.as_ref().unwrap().types,
            vec!["internal".to_string(), "oss_acc".to_string()]
        );
        assert_eq!(
            result.location_rtc_constraint.as_ref().unwrap().locations,
            vec!["oss-cn-beijing".to_string()]
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_replication_location() {
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

        let bucket_name = generate_unique_bucket_name("replication-location-test");

        let created = client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(created.is_ok(), "create_bucket failed: {:?}", created.err());

        let result = client
            .get_bucket_replication_location(&GetBucketReplicationLocationRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_replication_location failed: {:?}",
            result.err()
        );
        assert!(!result.unwrap().locations.is_empty());

        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
    }
}
