use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores information about the RestoreObject request.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RestoreRequest {
    /// The duration within which the restored object remains in the restored
    /// state.
    #[serde(rename = "Days")]
    pub days: i32,

    /// The restoration priority of Cold Archive or Deep Cold Archive objects.
    /// Valid values: Expedited, Standard, Bulk.
    /// Deprecated: use `job_parameters.tier` instead. If both exist, the value
    /// of `job_parameters` takes precedence.
    #[serde(skip)]
    pub tier: Option<String>,

    /// The container that stores the restoration priority configuration.
    /// This configuration takes effect only when the request is sent to
    /// restore Cold Archive objects. If you do not specify the JobParameters
    /// parameter, the default restoration priority Standard is used.
    #[serde(rename = "JobParameters", skip_serializing_if = "Option::is_none")]
    pub job_parameters: Option<JobParameters>,
}

/// The container that stores the restoration priority configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct JobParameters {
    /// The restoration priority of Cold Archive or Deep Cold Archive objects.
    /// Valid values: Expedited, Standard, Bulk.
    #[serde(rename = "Tier", skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
}

/// Builds the RestoreObject XML body, mirroring the Go SDK's
/// marshalRestoreObject: the deprecated top-level `tier` is mapped into
/// `JobParameters.Tier` only when `job_parameters` is not set.
fn build_restore_xml_body(
    restore_request: &RestoreRequest,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut effective = restore_request.clone();
    if effective.job_parameters.is_none() {
        if let Some(tier) = effective.tier.take() {
            effective.job_parameters = Some(JobParameters { tier: Some(tier) });
        }
    }
    let xml_body = quick_xml::se::to_string_with_root("RestoreRequest", &effective)?;
    Ok(xml_body)
}

#[derive(Debug, Default, OssRequestModel)]
pub struct RestoreObjectRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// The version ID of the source object.
    #[field(type = "query", rename = "versionId")]
    pub version_id: Option<String>,

    /// The container that stores information about the RestoreObject request.
    pub restore_request: Option<RestoreRequest>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct RestoreObjectResult {
    /// Version of the object.
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    /// The restoration priority. This header is displayed only for the
    /// Cold Archive or Deep Cold Archive object in the restored state.
    #[field(type = "header", rename = "x-oss-object-restore-priority")]
    pub restore_priority: Option<String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Restores Archive, Cold Archive, or Deep Cold Archive objects.
    /// You are charged based on tier (restoration priority) when you restore
    /// an object. For more information, see the OSS documentation.
    ///
    /// # Arguments
    ///
    /// * `request` - The `RestoreObjectRequest` containing the bucket name,
    ///   object key and the restore configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::{RestoreObjectRequest, RestoreRequest};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = RestoreObjectRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     restore_request: Some(RestoreRequest {
    ///         days: 2,
    ///         ..Default::default()
    ///     }),
    ///     ..Default::default()
    /// };
    ///
    /// match client.restore_object(&request).await {
    ///     Ok(result) => {
    ///         println!("Restore priority: {:?}", result.restore_priority);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to restore object: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn restore_object(
        &self,
        request: &RestoreObjectRequest,
    ) -> Result<RestoreObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "RestoreObject".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("restore", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        if let Some(restore_request) = &request.restore_request {
            let xml_body = build_restore_xml_body(restore_request)?;
            input.body = Some(BodyContent::from_text(xml_body, None));
        }

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = RestoreObjectResult::default();
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
    fn test_restore_request_xml_body() {
        // Days only.
        let xml = build_restore_xml_body(&RestoreRequest {
            days: 2,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(xml, "<RestoreRequest><Days>2</Days></RestoreRequest>");

        // Deprecated top-level tier is mapped into JobParameters.
        let xml = build_restore_xml_body(&RestoreRequest {
            days: 2,
            tier: Some("Expedited".to_string()),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            xml,
            "<RestoreRequest><Days>2</Days><JobParameters><Tier>Expedited</Tier></JobParameters></\
             RestoreRequest>"
        );

        // JobParameters takes precedence over the deprecated top-level tier.
        let xml = build_restore_xml_body(&RestoreRequest {
            days: 2,
            tier: Some("Bulk".to_string()),
            job_parameters: Some(JobParameters {
                tier: Some("Standard".to_string()),
            }),
        })
        .unwrap();
        assert_eq!(
            xml,
            "<RestoreRequest><Days>2</Days><JobParameters><Tier>Standard</Tier></JobParameters></\
             RestoreRequest>"
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_restore_object() {
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

        let object_name = crate::test_utils::generate_unique_object_name("restore");

        // Prepare an Archive object.
        client
            .put_object(crate::api::object::PutObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                storage_class: "Archive".to_string(),
                body: Some(BodyContent::from_text("restore-test".to_string(), None)),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .restore_object(&RestoreObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                restore_request: Some(RestoreRequest {
                    days: 1,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "restore_object failed: {:?}", result.err());

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
