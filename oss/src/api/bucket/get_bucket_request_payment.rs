use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketRequestPaymentRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl GetBucketRequestPaymentRequest {
    pub fn new(bucket: &str) -> Self {
        Self {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetBucketRequestPaymentResult {
    /// Indicates who pays the download and request fees.
    /// Valid values: BucketOwner, Requester.
    #[serde(rename = "Payer", skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries pay-by-requester configurations for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketRequestPaymentRequest` containing the
    ///   bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketRequestPaymentRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketRequestPaymentRequest::new("my-bucket");
    ///
    /// match client.get_bucket_request_payment(&request).await {
    ///     Ok(result) => {
    ///         println!("Payer: {:?}", result.payer);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket request payment: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_request_payment(
        &self,
        request: &GetBucketRequestPaymentRequest,
    ) -> Result<GetBucketRequestPaymentResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketRequestPayment".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("requestPayment", "")]
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
            .set(SUB_RESOURCE, Rc::new(vec!["requestPayment".to_string()]));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetBucketRequestPaymentResult = quick_xml::de::from_str(&data_str)?;

        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::SignatureVersionType;
    use crate::test_utils::load_test_config;

    #[test]
    fn test_get_bucket_request_payment_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<RequestPaymentConfiguration>
  <Payer>Requester</Payer>
</RequestPaymentConfiguration>"#;
        let result: GetBucketRequestPaymentResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.payer.as_deref(), Some("Requester"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_request_payment() {
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
            .get_bucket_request_payment(&GetBucketRequestPaymentRequest::new(&config.bucket))
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_request_payment failed: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().common.status, http::StatusCode::OK);
    }
}
