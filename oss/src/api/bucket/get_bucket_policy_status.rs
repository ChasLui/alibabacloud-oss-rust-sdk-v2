use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketPolicyStatusRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetBucketPolicyStatusResult {
    /// Indicates whether the current bucket policy allows public access.
    #[serde(rename = "IsPublic", skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Checks whether the current bucket policy allows public access.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketPolicyStatusRequest` containing the bucket
    ///   name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketPolicyStatusRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketPolicyStatusRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_policy_status(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket policy is public: {:?}", result.is_public);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket policy status: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_policy_status(
        &self,
        request: &GetBucketPolicyStatusRequest,
    ) -> Result<GetBucketPolicyStatusResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketPolicyStatus".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("policyStatus", "")]
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
            std::rc::Rc::new(vec!["policyStatus".to_string()]),
        );

        modify_request(&mut input, request.header_map(), request.query_map(), vec![])?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetBucketPolicyStatusResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_policy_status_serde_round_trip() {
        let xml = "<PolicyStatus><IsPublic>false</IsPublic></PolicyStatus>";
        let result: GetBucketPolicyStatusResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.is_public, Some(false));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_policy_status() {
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
            .get_bucket_policy_status(&GetBucketPolicyStatusRequest {
                bucket: config.bucket.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_policy_status failed: {:?}",
            result.err()
        );
    }
}
