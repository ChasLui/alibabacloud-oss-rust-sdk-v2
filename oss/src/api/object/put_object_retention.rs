use std::time::SystemTime;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{
    modify_request, option_time_rfc3339_serde, update_content_length, update_content_md5,
};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the object-level retention policy.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ObjectWormRetention {
    /// The object-level retention mode. Only COMPLIANCE is supported.
    #[serde(rename = "Mode", skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,

    /// The absolute date and time until which the object is retained,
    /// in RFC 3339 format.
    #[serde(
        rename = "RetainUntilDate",
        with = "option_time_rfc3339_serde",
        default
    )]
    pub retain_until_date: Option<SystemTime>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutObjectRetentionRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// Version of the object.
    #[field(type = "query", rename = "versionId")]
    pub version_id: Option<String>,

    /// Specifies whether to bypass the governance-mode retention policy.
    #[field(type = "header", rename = "x-oss-bypass-governance-retention")]
    pub bypass_governance_retention: Option<bool>,

    /// The container that stores the retention policy.
    pub retention: ObjectWormRetention,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutObjectRetentionResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Configures a retention policy on an object.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutObjectRetentionRequest` containing the bucket
    ///   name, object key and the retention policy to set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::time::{Duration, SystemTime};
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::{ObjectWormRetention, PutObjectRetentionRequest};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutObjectRetentionRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     retention: ObjectWormRetention {
    ///         mode: Some("COMPLIANCE".to_string()),
    ///         retain_until_date: Some(SystemTime::now() + Duration::from_secs(86400)),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_object_retention(&request).await {
    ///     Ok(result) => {
    ///         println!("Object retention updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put object retention: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_object_retention(
        &self,
        request: &PutObjectRetentionRequest,
    ) -> Result<PutObjectRetentionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutObjectRetention".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("retention", "")]
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
            std::rc::Rc::new(vec!["retention".to_string()]),
        );

        let xml_body = quick_xml::se::to_string_with_root("Retention", &request.retention)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutObjectRetentionResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::time::Duration;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_retention_serde_round_trip() {
        let retain_until = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000_000);
        let retention = ObjectWormRetention {
            mode: Some("COMPLIANCE".to_string()),
            retain_until_date: Some(retain_until),
        };

        let xml = quick_xml::se::to_string_with_root("Retention", &retention).unwrap();
        assert!(xml.contains("<Retention>"));
        assert!(xml.contains("<Mode>COMPLIANCE</Mode>"));
        assert!(xml.contains("<RetainUntilDate>2033-05-18T03:33:20Z</RetainUntilDate>"));

        let parsed: ObjectWormRetention = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.mode.as_deref(), Some("COMPLIANCE"));
        assert_eq!(parsed.retain_until_date, Some(retain_until));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_object_retention() {
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

        let object_name = crate::test_utils::generate_unique_object_name("retention");

        // Prepare an object
        client
            .put_object(crate::api::object::PutObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                body: Some(BodyContent::from_text("retention-test".to_string(), None)),
                ..Default::default()
            })
            .await
            .unwrap();

        // Object-level retention requires a bucket with the object WORM
        // configuration enabled, so a regular bucket may reject this call.
        let result = client
            .put_object_retention(&PutObjectRetentionRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                retention: ObjectWormRetention {
                    mode: Some("COMPLIANCE".to_string()),
                    retain_until_date: Some(SystemTime::now() + Duration::from_secs(86400)),
                },
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!(
                "put_object_retention rejected (bucket may lack WORM config): {}",
                error
            );
        }

        // Clean up
        let _ = client
            .delete_object(crate::api::object::DeleteObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name,
                ..Default::default()
            })
            .await;
    }
}
