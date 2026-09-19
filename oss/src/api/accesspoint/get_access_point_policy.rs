use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::signer::SUB_RESOURCE;
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetAccessPointPolicyRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the access point.
    #[field(type = "header", rename = "x-oss-access-point-name")]
    pub access_point_name: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetAccessPointPolicyResult {
    /// The configurations of the access point policy, as raw JSON text.
    pub body: Option<String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Queries the configurations of an access point policy.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetAccessPointPolicyRequest` containing the bucket
    ///   name and the access point name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::accesspoint::GetAccessPointPolicyRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetAccessPointPolicyRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     access_point_name: Some("my-ap".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_access_point_policy(&request).await {
    ///     Ok(result) => {
    ///         println!("Access point policy: {:?}", result.body);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get access point policy: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_access_point_policy(
        &self,
        request: &GetAccessPointPolicyRequest,
    ) -> Result<GetAccessPointPolicyResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetAccessPointPolicy".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("accessPointPolicy", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };
        input
            .op_metadata
            .set(SUB_RESOURCE, Rc::new(vec!["accessPointPolicy".to_string()]));

        modify_request(&mut input, request.header_map(), request.query_map(), vec![])?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        // The response body is a plain JSON policy text, not XML.
        let body_data = output.get_all_data().await?;
        let mut result = GetAccessPointPolicyResult::default();
        result.body = Some(String::from_utf8_lossy(&body_data).into_owned());
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
    use super::super::put_access_point_policy::PutAccessPointPolicyRequest;
    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_access_point_policy() {
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

        // Prepare an access point with a policy
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
        client
            .put_access_point_policy(&PutAccessPointPolicyRequest {
                bucket: config.bucket.clone(),
                access_point_name: Some(ap_name.clone()),
                body: Some(policy),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .get_access_point_policy(&GetAccessPointPolicyRequest {
                bucket: config.bucket.clone(),
                access_point_name: Some(ap_name.clone()),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "get_access_point_policy failed: {:?}",
            result.err()
        );
        let body = result.unwrap().body.unwrap_or_default();
        assert!(body.contains("\"Version\""));

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
