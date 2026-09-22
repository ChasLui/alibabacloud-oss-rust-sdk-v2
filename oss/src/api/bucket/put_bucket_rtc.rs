use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use super::put_bucket_replication::ReplicationTimeControl;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container of the RTC configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RtcConfiguration {
    /// The container that stores the status of RTC.
    #[serde(rename = "RTC", skip_serializing_if = "Option::is_none")]
    pub rtc: Option<ReplicationTimeControl>,

    /// The ID of the data replication rule for which you want to configure
    /// RTC.
    #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketRtcRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The container of the request body.
    pub rtc_configuration: RtcConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketRtcResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Enables or disables the Replication Time Control (RTC) feature for
    /// existing cross-region replication (CRR) rules.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketRtcRequest` containing the bucket name and
    ///   the RTC configuration to set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    /// #     PutBucketRtcRequest, ReplicationTimeControl, RtcConfiguration,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketRtcRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     rtc_configuration: RtcConfiguration {
    ///         rtc: Some(ReplicationTimeControl {
    ///             status: Some("enabled".to_string()),
    ///         }),
    ///         id: Some("rule-id".to_string()),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_rtc(&request).await {
    ///     Ok(result) => {
    ///         println!("RTC updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket rtc: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_rtc(
        &self,
        request: &PutBucketRtcRequest,
    ) -> Result<PutBucketRtcResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketRtc".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("rtc", "")]
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
            std::rc::Rc::new(vec!["rtc".to_string()]),
        );

        // The Go SDK marshals RtcConfiguration with the "ReplicationRule" root.
        let xml_body =
            quick_xml::se::to_string_with_root("ReplicationRule", &request.rtc_configuration)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketRtcResult::default();
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
    fn test_rtc_configuration_serde_round_trip() {
        let config = RtcConfiguration {
            rtc: Some(ReplicationTimeControl {
                status: Some("enabled".to_string()),
            }),
            id: Some("rule-id".to_string()),
        };

        let xml = quick_xml::se::to_string_with_root("ReplicationRule", &config).unwrap();
        assert!(xml.contains("<ReplicationRule>"));
        assert!(xml.contains("<RTC>"));
        assert!(xml.contains("<Status>enabled</Status>"));
        assert!(xml.contains("<ID>rule-id</ID>"));

        let parsed: RtcConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(
            parsed.rtc.as_ref().unwrap().status.as_deref(),
            Some("enabled")
        );
        assert_eq!(parsed.id.as_deref(), Some("rule-id"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_rtc() {
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

        let bucket_name = generate_unique_bucket_name("rtc-test");

        let created = client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(created.is_ok(), "create_bucket failed: {:?}", created.err());

        // RTC applies to existing CRR rules; a fresh bucket has none, so the
        // server is expected to reject the request.
        let result = client
            .put_bucket_rtc(&PutBucketRtcRequest {
                bucket: bucket_name.clone(),
                rtc_configuration: RtcConfiguration {
                    rtc: Some(ReplicationTimeControl {
                        status: Some("enabled".to_string()),
                    }),
                    id: Some("non-existent-rule".to_string()),
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_err(),
            "put_bucket_rtc without a replication rule should fail"
        );

        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
    }
}
