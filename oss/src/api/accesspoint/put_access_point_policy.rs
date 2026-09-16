use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutAccessPointPolicyRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the access point.
    #[field(type = "header", rename = "x-oss-access-point-name")]
    pub access_point_name: Option<String>,

    /// The configurations of the access point policy, as raw JSON text.
    pub body: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutAccessPointPolicyResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Configures an access point policy.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutAccessPointPolicyRequest` containing the bucket
    ///   name, the access point name and the policy JSON text.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::accesspoint::PutAccessPointPolicyRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutAccessPointPolicyRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     access_point_name: Some("my-ap".to_string()),
    ///     body: Some("{\"Version\":\"1\",\"Statement\":[]}".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_access_point_policy(&request).await {
    ///     Ok(result) => {
    ///         println!("Access point policy updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put access point policy: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_access_point_policy(
        &self,
        request: &PutAccessPointPolicyRequest,
    ) -> Result<PutAccessPointPolicyResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutAccessPointPolicy".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("accessPointPolicy", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };
        input
            .op_metadata
            .set(SUB_RESOURCE, Rc::new(vec!["accessPointPolicy".to_string()]));

        if let Some(body) = &request.body {
            input.body = Some(BodyContent::from_text(body.clone(), None));
        }

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutAccessPointPolicyResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::create_access_point::tests::generate_access_point_name;
    use super::super::create_access_point::{CreateAccessPointConfiguration, CreateAccessPointRequest};
    use super::super::delete_access_point::DeleteAccessPointRequest;
    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::SignatureVersionType;
    use crate::test_utils::load_test_config;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_access_point_policy() {
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

        // Prepare an access point
        let ap_name = generate_access_point_name();
        client
            .create_access_point(&CreateAccessPointRequest {
                bucket: config.bucket.clone(),
                create_access_point_configuration: CreateAccessPointConfiguration {
                    access_point_name: Some(ap_name.clone()),
                    network_origin: Some("internet".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .unwrap();

        let policy = format!(
            "{{\"Version\":\"1\",\"Statement\":[{{\"Action\":[\"oss:PutObject\"],\"Effect\":\"Allow\",\"Principal\":[\"*\"],\"Resource\":[\"acs:oss:{}:*:accesspoint/{}/object/*\"]}}]}}",
            config.region, ap_name
        );
        let result = client
            .put_access_point_policy(&PutAccessPointPolicyRequest {
                bucket: config.bucket.clone(),
                access_point_name: Some(ap_name.clone()),
                body: Some(policy),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "put_access_point_policy failed: {:?}",
            result.err()
        );

        // Clean up
        let _ = client
            .delete_access_point(&DeleteAccessPointRequest {
                bucket: config.bucket.clone(),
                access_point_name: Some(ap_name),
                ..Default::default()
            })
            .await;
    }
}
