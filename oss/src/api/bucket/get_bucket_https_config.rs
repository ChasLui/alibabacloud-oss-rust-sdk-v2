use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores TLS version configurations.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Tls {
    /// Specifies whether to enable TLS version management for the bucket.
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,

    /// The TLS versions.
    #[serde(rename = "TLSVersion", default)]
    pub tls_versions: Vec<String>,
}

/// The container that stores cipher suite configurations.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CipherSuite {
    /// Specifies whether to enable cipher suite management.
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,

    /// Specifies whether to only use strong cipher suites.
    #[serde(rename = "StrongCipherSuite", skip_serializing_if = "Option::is_none")]
    pub strong_cipher_suite: Option<bool>,

    /// The custom cipher suites for TLS 1.2 and earlier.
    #[serde(rename = "CustomCipherSuite", default)]
    pub custom_cipher_suites: Vec<String>,

    /// The custom cipher suites for TLS 1.3.
    #[serde(rename = "TLS13CustomCipherSuite", default)]
    pub tls13_custom_cipher_suites: Vec<String>,
}

/// The container that stores HTTPS configurations.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HttpsConfiguration {
    /// The container that stores TLS version configurations.
    #[serde(rename = "TLS", skip_serializing_if = "Option::is_none")]
    pub tls: Option<Tls>,

    /// The container that stores cipher suite configurations.
    #[serde(rename = "CipherSuite", skip_serializing_if = "Option::is_none")]
    pub cipher_suite: Option<CipherSuite>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketHttpsConfigRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetBucketHttpsConfigResult {
    /// The container that stores TLS version configurations.
    #[serde(rename = "TLS", skip_serializing_if = "Option::is_none")]
    pub tls: Option<Tls>,

    /// The container that stores cipher suite configurations.
    #[serde(rename = "CipherSuite", skip_serializing_if = "Option::is_none")]
    pub cipher_suite: Option<CipherSuite>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the Transport Layer Security (TLS) version configurations of a
    /// bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketHttpsConfigRequest` containing the bucket
    ///   name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketHttpsConfigRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketHttpsConfigRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_https_config(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket HTTPS config: {:?}", result.tls);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket HTTPS config: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_https_config(
        &self,
        request: &GetBucketHttpsConfigRequest,
    ) -> Result<GetBucketHttpsConfigResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketHttpsConfig".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("httpsConfig", "")]
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
            std::rc::Rc::new(vec!["httpsConfig".to_string()]),
        );

        modify_request(&mut input, request.header_map(), request.query_map(), vec![])?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetBucketHttpsConfigResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_https_configuration_serde_round_trip() {
        let config = HttpsConfiguration {
            tls: Some(Tls {
                enable: Some(true),
                tls_versions: vec!["TLSv1.2".to_string(), "TLSv1.3".to_string()],
            }),
            cipher_suite: Some(CipherSuite {
                enable: Some(true),
                strong_cipher_suite: Some(true),
                custom_cipher_suites: vec!["ECDHE-RSA-AES128-GCM-SHA256".to_string()],
                tls13_custom_cipher_suites: vec!["TLS_AES_128_GCM_SHA256".to_string()],
            }),
        };

        let xml = quick_xml::se::to_string_with_root("HttpsConfiguration", &config).unwrap();
        assert!(xml.contains("<HttpsConfiguration>"));
        assert!(xml.contains("<TLS>"));
        assert!(xml.contains("<Enable>true</Enable>"));
        assert!(xml.contains("<TLSVersion>TLSv1.2</TLSVersion>"));
        assert!(xml.contains("<TLSVersion>TLSv1.3</TLSVersion>"));
        assert!(xml.contains("<StrongCipherSuite>true</StrongCipherSuite>"));
        assert!(xml.contains("<CustomCipherSuite>ECDHE-RSA-AES128-GCM-SHA256</CustomCipherSuite>"));
        assert!(xml.contains("<TLS13CustomCipherSuite>TLS_AES_128_GCM_SHA256</TLS13CustomCipherSuite>"));

        let parsed: HttpsConfiguration = quick_xml::de::from_str(&xml).unwrap();
        let tls = parsed.tls.unwrap();
        assert_eq!(tls.enable, Some(true));
        assert_eq!(tls.tls_versions, vec!["TLSv1.2".to_string(), "TLSv1.3".to_string()]);
        let cipher_suite = parsed.cipher_suite.unwrap();
        assert_eq!(cipher_suite.strong_cipher_suite, Some(true));
        assert_eq!(
            cipher_suite.custom_cipher_suites,
            vec!["ECDHE-RSA-AES128-GCM-SHA256".to_string()]
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_https_config() {
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

        // Prepare an HTTPS config
        client
            .put_bucket_https_config(&crate::api::bucket::PutBucketHttpsConfigRequest {
                bucket: config.bucket.clone(),
                https_configuration: HttpsConfiguration {
                    tls: Some(Tls {
                        enable: Some(true),
                        tls_versions: vec!["TLSv1.2".to_string(), "TLSv1.3".to_string()],
                    }),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .get_bucket_https_config(&GetBucketHttpsConfigRequest {
                bucket: config.bucket.clone(),
                ..Default::default()
            })
            .await;
        assert!(
            result.is_ok(),
            "get_bucket_https_config failed: {:?}",
            result.err()
        );
        let tls = result.unwrap().tls.unwrap();
        assert_eq!(tls.enable, Some(true));

        // Clean up: disable TLS version management
        let _ = client
            .put_bucket_https_config(&crate::api::bucket::PutBucketHttpsConfigRequest {
                bucket: config.bucket.clone(),
                https_configuration: HttpsConfiguration {
                    tls: Some(Tls {
                        enable: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;
    }
}
