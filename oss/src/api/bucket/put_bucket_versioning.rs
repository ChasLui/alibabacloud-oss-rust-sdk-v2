use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the versioning state of the bucket.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VersioningConfiguration {
    /// The versioning state of the bucket. Valid values: Enabled, Suspended.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketVersioningRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The container that stores the versioning state of the bucket.
    pub versioning_configuration: VersioningConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketVersioningResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Configures the versioning state for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketVersioningRequest` containing the bucket
    ///   name and the versioning configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{PutBucketVersioningRequest, VersioningConfiguration};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketVersioningRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     versioning_configuration: VersioningConfiguration {
    ///         status: Some("Enabled".to_string()),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_versioning(&request).await {
    ///     Ok(result) => {
    ///         println!("Versioning configured: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket versioning: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_versioning(
        &self,
        request: &PutBucketVersioningRequest,
    ) -> Result<PutBucketVersioningResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketVersioning".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("versioning", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        let xml_body = quick_xml::se::to_string_with_root(
            "VersioningConfiguration",
            &request.versioning_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketVersioningResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{
        CreateBucketRequest, DeleteBucketRequest, GetBucketVersioningRequest,
    };
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_versioning_configuration_serde_round_trip() {
        let cfg = VersioningConfiguration {
            status: Some("Enabled".to_string()),
        };
        let xml = quick_xml::se::to_string_with_root("VersioningConfiguration", &cfg).unwrap();
        assert!(xml.contains("<VersioningConfiguration>"));
        assert!(xml.contains("<Status>Enabled</Status>"));

        let parsed: VersioningConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.status.as_deref(), Some("Enabled"));

        // None is skipped instead of serialized as an empty element
        let xml = quick_xml::se::to_string_with_root(
            "VersioningConfiguration",
            &VersioningConfiguration::default(),
        )
        .unwrap();
        assert!(!xml.contains("<Status"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_versioning() {
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

        let bucket_name = generate_unique_bucket_name("versioning-test");

        // Prepare a bucket
        let created = client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(created.is_ok(), "create_bucket failed: {:?}", created.err());

        let result = client
            .put_bucket_versioning(&PutBucketVersioningRequest {
                bucket: bucket_name.clone(),
                versioning_configuration: VersioningConfiguration {
                    status: Some("Enabled".to_string()),
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "put_bucket_versioning failed: {:?}",
            result.err()
        );

        // Verify via get_bucket_versioning
        let got = client
            .get_bucket_versioning(&GetBucketVersioningRequest::new(&bucket_name))
            .await;
        assert!(got.is_ok(), "get_bucket_versioning failed: {:?}", got.err());
        assert_eq!(got.unwrap().version_status.as_deref(), Some("Enabled"));

        // Clean up: delete the bucket
        let _ = client
            .delete_bucket(&DeleteBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
    }
}
