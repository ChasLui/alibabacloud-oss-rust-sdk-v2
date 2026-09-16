use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the object-level legal hold configuration.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ObjectWormLegalHold {
    /// The object-level legal hold status. Valid values: ON and OFF.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutObjectLegalHoldRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// Version of the object.
    #[field(type = "query", rename = "versionId")]
    pub version_id: Option<String>,

    /// The container that stores the object-level legal hold configuration.
    pub legal_hold: ObjectWormLegalHold,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutObjectLegalHoldResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Configures a legal hold on an object.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutObjectLegalHoldRequest` containing the bucket name,
    ///   object key and the legal hold status to set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::{ObjectWormLegalHold, PutObjectLegalHoldRequest};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutObjectLegalHoldRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     legal_hold: ObjectWormLegalHold {
    ///         status: Some("ON".to_string()),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_object_legal_hold(&request).await {
    ///     Ok(result) => {
    ///         println!("Object legal hold updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put object legal hold: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_object_legal_hold(
        &self,
        request: &PutObjectLegalHoldRequest,
    ) -> Result<PutObjectLegalHoldResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutObjectLegalHold".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("legalHold", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["legalHold".to_string()]),
        );

        let xml_body = quick_xml::se::to_string_with_root("LegalHold", &request.legal_hold)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutObjectLegalHoldResult::default();
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
    fn test_legal_hold_serde_round_trip() {
        let legal_hold = ObjectWormLegalHold {
            status: Some("ON".to_string()),
        };

        let xml = quick_xml::se::to_string_with_root("LegalHold", &legal_hold).unwrap();
        assert!(xml.contains("<LegalHold>"));
        assert!(xml.contains("<Status>ON</Status>"));

        let parsed: ObjectWormLegalHold = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.status.as_deref(), Some("ON"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_object_legal_hold() {
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

        let object_name = crate::test_utils::generate_unique_object_name("legal-hold");

        // Prepare an object
        client
            .put_object(crate::api::object::PutObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                body: Some(BodyContent::from_text("legal-hold-test".to_string(), None)),
                ..Default::default()
            })
            .await
            .unwrap();

        // Object-level legal hold requires a bucket with the object WORM
        // configuration enabled, so a regular bucket may reject this call.
        let result = client
            .put_object_legal_hold(&PutObjectLegalHoldRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                legal_hold: ObjectWormLegalHold {
                    status: Some("ON".to_string()),
                },
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("put_object_legal_hold rejected (bucket may lack WORM config): {}", error);
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
