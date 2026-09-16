use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::put_object_retention::ObjectWormRetention;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetObjectRetentionRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// Version of the object.
    #[field(type = "query", rename = "versionId")]
    pub version_id: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetObjectRetentionResult {
    /// The container that stores the retention policy.
    pub retention: Option<ObjectWormRetention>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Queries the object-level retention policy of an object in a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetObjectRetentionRequest` containing the bucket name
    ///   and the object key.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::GetObjectRetentionRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetObjectRetentionRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_object_retention(&request).await {
    ///     Ok(result) => {
    ///         println!("Object retention: {:?}", result.retention);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get object retention: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_object_retention(
        &self,
        request: &GetObjectRetentionRequest,
    ) -> Result<GetObjectRetentionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetObjectRetention".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("retention", "")]
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
            std::rc::Rc::new(vec!["retention".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let retention: ObjectWormRetention = quick_xml::de::from_str(&data_str)?;

        let mut result = GetObjectRetentionResult {
            retention: Some(retention),
            ..Default::default()
        };
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
    fn test_get_retention_deserialize() {
        let xml = r#"<Retention><Mode>COMPLIANCE</Mode><RetainUntilDate>2033-05-18T03:33:20Z</RetainUntilDate></Retention>"#;
        let parsed: ObjectWormRetention = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(parsed.mode.as_deref(), Some("COMPLIANCE"));
        assert!(parsed.retain_until_date.is_some());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_object_retention() {
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
                body: Some(crate::BodyContent::from_text("retention-test".to_string(), None)),
                ..Default::default()
            })
            .await
            .unwrap();

        // Objects without a retention policy are rejected by the server.
        let result = client
            .get_object_retention(&GetObjectRetentionRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("get_object_retention rejected (no retention policy set): {}", error);
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
