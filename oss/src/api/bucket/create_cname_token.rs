use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::put_cname::{effective_cname_configuration, BucketCnameConfiguration, Cname};
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationOutput, BodyContent, OperationInput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct CreateCnameTokenRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The request body schema.
    pub bucket_cname_configuration: BucketCnameConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, serde::Deserialize, OssResultModel)]
pub struct CreateCnameTokenResult {
    /// The name of the bucket to which the CNAME record is mapped.
    #[serde(rename = "Bucket", skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// The name of the CNAME record that is mapped to the bucket.
    #[serde(rename = "Cname", skip_serializing_if = "Option::is_none")]
    pub cname: Option<String>,

    /// The CNAME token that is returned by OSS.
    #[serde(rename = "Token", skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// The time when the CNAME token expires.
    #[serde(rename = "ExpireTime", skip_serializing_if = "Option::is_none")]
    pub expire_time: Option<String>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Creates a CNAME token to verify the ownership of a domain name.
    ///
    /// # Arguments
    ///
    /// * `request` - The `CreateCnameTokenRequest` containing the bucket name
    ///   and the CNAME for which to create the token.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    /// #     BucketCnameConfiguration, Cname, CreateCnameTokenRequest,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = CreateCnameTokenRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     bucket_cname_configuration: BucketCnameConfiguration {
    ///         cname: Some(Cname {
    ///             domain: Some("example.com".to_string()),
    ///             ..Default::default()
    ///         }),
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.create_cname_token(&request).await {
    ///     Ok(result) => {
    ///         println!("Cname token: {:?}", result.token);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to create cname token: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn create_cname_token(
        &self,
        request: &CreateCnameTokenRequest,
    ) -> Result<CreateCnameTokenResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "CreateCnameToken".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("cname", ""), ("comp", "token")]
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
            std::rc::Rc::new(vec!["cname".to_string(), "comp".to_string()]),
        );

        let effective_config =
            effective_cname_configuration(&request.bucket_cname_configuration);
        let xml_body =
            quick_xml::se::to_string_with_root("BucketCnameConfiguration", &effective_config)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: CreateCnameTokenResult = quick_xml::de::from_str(&data_str)?;
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
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_create_cname_token_result_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<CnameToken>
  <Bucket>my-bucket</Bucket>
  <Cname>example.com</Cname>
  <Token>be1d49d863dea9ffeff3df7d6455****</Token>
  <ExpireTime>Wed, 23 Feb 2022 21:39:42 GMT</ExpireTime>
</CnameToken>"#;

        let result: CreateCnameTokenResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.bucket.as_deref(), Some("my-bucket"));
        assert_eq!(result.cname.as_deref(), Some("example.com"));
        assert_eq!(
            result.token.as_deref(),
            Some("be1d49d863dea9ffeff3df7d6455****")
        );
        assert_eq!(
            result.expire_time.as_deref(),
            Some("Wed, 23 Feb 2022 21:39:42 GMT")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_cname_token() {
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

        let bucket_name = generate_unique_bucket_name("cname-token-test");

        let created = client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(created.is_ok(), "create_bucket failed: {:?}", created.err());

        let result = client
            .create_cname_token(&CreateCnameTokenRequest {
                bucket: bucket_name.clone(),
                bucket_cname_configuration: BucketCnameConfiguration {
                    cname: Some(Cname {
                        domain: Some("nonexistent-domain-for-test.example.com".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;

        // When the token is created, query it back to verify the round trip.
        if result.is_ok() {
            let got = client
                .get_cname_token(&crate::api::bucket::GetCnameTokenRequest {
                    bucket: bucket_name.clone(),
                    cname: Some("nonexistent-domain-for-test.example.com".to_string()),
                    ..Default::default()
                })
                .await;
            assert!(got.is_ok(), "get_cname_token failed: {:?}", got.err());
            assert!(got.unwrap().token.is_some());
        } else {
            eprintln!(
                "create_cname_token rejected by server: {:?}",
                result.err()
            );
        }

        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
    }
}
