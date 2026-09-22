use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{
    BodyContent, OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE,
};

/// The request payment configuration information for the bucket.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RequestPaymentConfiguration {
    /// The payer of the request and traffic fees.
    /// Valid values: BucketOwner, Requester.
    #[serde(rename = "Payer", skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketRequestPaymentRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The request payment configuration information for the bucket.
    pub payment_configuration: RequestPaymentConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketRequestPaymentResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Enables pay-by-requester for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketRequestPaymentRequest` containing the bucket
    ///   name and the request payment configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{PutBucketRequestPaymentRequest, RequestPaymentConfiguration};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketRequestPaymentRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     payment_configuration: RequestPaymentConfiguration {
    ///         payer: Some("Requester".to_string()),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_request_payment(&request).await {
    ///     Ok(result) => {
    ///         println!("Request payment configured: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket request payment: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_request_payment(
        &self,
        request: &PutBucketRequestPaymentRequest,
    ) -> Result<PutBucketRequestPaymentResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketRequestPayment".to_string(),
            method: http::Method::PUT,
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

        let xml_body = quick_xml::se::to_string_with_root(
            "RequestPaymentConfiguration",
            &request.payment_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketRequestPaymentResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::bucket::{
        CreateBucketRequest, DeleteBucketRequest, GetBucketRequestPaymentRequest,
    };
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_request_payment_configuration_serde_round_trip() {
        let cfg = RequestPaymentConfiguration {
            payer: Some("Requester".to_string()),
        };
        let xml = quick_xml::se::to_string_with_root("RequestPaymentConfiguration", &cfg).unwrap();
        assert!(xml.contains("<RequestPaymentConfiguration>"));
        assert!(xml.contains("<Payer>Requester</Payer>"));

        let parsed: RequestPaymentConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.payer.as_deref(), Some("Requester"));

        // None is skipped instead of serialized as an empty element
        let xml = quick_xml::se::to_string_with_root(
            "RequestPaymentConfiguration",
            &RequestPaymentConfiguration::default(),
        )
        .unwrap();
        assert!(!xml.contains("<Payer"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_request_payment() {
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

        let bucket_name = generate_unique_bucket_name("req-payment-test");

        // Prepare a bucket
        let created = client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(created.is_ok(), "create_bucket failed: {:?}", created.err());

        let result = client
            .put_bucket_request_payment(&PutBucketRequestPaymentRequest {
                bucket: bucket_name.clone(),
                payment_configuration: RequestPaymentConfiguration {
                    payer: Some("Requester".to_string()),
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "put_bucket_request_payment failed: {:?}",
            result.err()
        );

        // Verify via get_bucket_request_payment
        let got = client
            .get_bucket_request_payment(&GetBucketRequestPaymentRequest::new(&bucket_name))
            .await;
        assert!(
            got.is_ok(),
            "get_bucket_request_payment failed: {:?}",
            got.err()
        );
        assert_eq!(got.unwrap().payer.as_deref(), Some("Requester"));

        // Clean up: delete the bucket
        let _ = client
            .delete_bucket(&DeleteBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
    }
}
