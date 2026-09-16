use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationOutput, BodyContent, OperationInput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

/// The container for which the certificate is configured.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CertificateConfiguration {
    /// The ID of the certificate.
    #[serde(rename = "CertId", skip_serializing_if = "Option::is_none")]
    pub cert_id: Option<String>,

    /// The public key of the certificate.
    #[serde(rename = "Certificate", skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,

    /// The private key of the certificate.
    #[serde(rename = "PrivateKey", skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,

    /// The ID of the current certificate. If Force is not set to true, the OSS
    /// server checks whether the value matches the current certificate ID.
    #[serde(rename = "PreviousCertId", skip_serializing_if = "Option::is_none")]
    pub previous_cert_id: Option<String>,

    /// Specifies whether to overwrite the certificate.
    #[serde(rename = "Force", skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,

    /// Specifies whether to delete the certificate.
    #[serde(rename = "DeleteCertificate", skip_serializing_if = "Option::is_none")]
    pub delete_certificate: Option<bool>,
}

/// The container for the custom domain name.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Cname {
    /// The custom domain name.
    #[serde(rename = "Domain", skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    /// The container for which the certificate is configured.
    #[serde(rename = "CertificateConfiguration", skip_serializing_if = "Option::is_none")]
    pub certificate_configuration: Option<CertificateConfiguration>,

    /// Specifies whether the domain name is a wildcard domain name.
    #[serde(rename = "IsWildCard", skip_serializing_if = "Option::is_none")]
    pub is_wild_card: Option<bool>,
}

/// The request body schema that stores the CNAME configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BucketCnameConfiguration {
    /// The custom domain name.
    /// Deprecated: use `cname.domain` instead. If both exist simultaneously,
    /// the value of `cname` takes precedence.
    #[serde(rename = "Domain", skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    /// The container for which the certificate is configured.
    /// Deprecated: use `cname.certificate_configuration` instead. If both
    /// exist simultaneously, the value of `cname` takes precedence.
    #[serde(rename = "CertificateConfiguration", skip_serializing_if = "Option::is_none")]
    pub certificate_configuration: Option<CertificateConfiguration>,

    /// The container for the custom domain name.
    #[serde(rename = "Cname", skip_serializing_if = "Option::is_none")]
    pub cname: Option<Cname>,
}

/// Builds the effective body, mirroring the Go SDK: when `cname` is not set,
/// the deprecated top-level `domain`/`certificate_configuration` fields are
/// wrapped into a `Cname` container.
pub(crate) fn effective_cname_configuration(
    config: &BucketCnameConfiguration,
) -> BucketCnameConfiguration {
    if config.cname.is_none()
        && (config.domain.is_some() || config.certificate_configuration.is_some())
    {
        BucketCnameConfiguration {
            cname: Some(Cname {
                domain: config.domain.clone(),
                certificate_configuration: config.certificate_configuration.clone(),
                is_wild_card: None,
            }),
            ..Default::default()
        }
    } else {
        config.clone()
    }
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutCnameRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The request body schema.
    pub bucket_cname_configuration: BucketCnameConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutCnameResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Maps a CNAME record to a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutCnameRequest` containing the bucket name and the
    ///   CNAME configuration to set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    /// #     BucketCnameConfiguration, Cname, PutCnameRequest,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutCnameRequest {
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
    /// match client.put_cname(&request).await {
    ///     Ok(result) => {
    ///         println!("Cname mapped: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put cname: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_cname(
        &self,
        request: &PutCnameRequest,
    ) -> Result<PutCnameResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutCname".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("cname", ""), ("comp", "add")]
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
            std::rc::Rc::new(vec!["comp".to_string(), "cname".to_string()]),
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

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutCnameResult::default();
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
    fn test_bucket_cname_configuration_serde_round_trip() {
        let config = BucketCnameConfiguration {
            cname: Some(Cname {
                domain: Some("example.com".to_string()),
                certificate_configuration: Some(CertificateConfiguration {
                    cert_id: Some("cert-id".to_string()),
                    certificate: Some("-----BEGIN CERTIFICATE-----".to_string()),
                    private_key: Some("-----BEGIN PRIVATE KEY-----".to_string()),
                    previous_cert_id: Some("old-cert-id".to_string()),
                    force: Some(true),
                    delete_certificate: Some(false),
                }),
                is_wild_card: Some(false),
            }),
            ..Default::default()
        };

        let xml =
            quick_xml::se::to_string_with_root("BucketCnameConfiguration", &config).unwrap();
        assert!(xml.contains("<BucketCnameConfiguration>"));
        assert!(xml.contains("<Cname>"));
        assert!(xml.contains("<Domain>example.com</Domain>"));
        assert!(xml.contains("<CertId>cert-id</CertId>"));
        assert!(xml.contains("<PreviousCertId>old-cert-id</PreviousCertId>"));
        assert!(xml.contains("<Force>true</Force>"));
        assert!(xml.contains("<DeleteCertificate>false</DeleteCertificate>"));
        assert!(xml.contains("<IsWildCard>false</IsWildCard>"));

        let parsed: BucketCnameConfiguration = quick_xml::de::from_str(&xml).unwrap();
        let cname = parsed.cname.as_ref().unwrap();
        assert_eq!(cname.domain.as_deref(), Some("example.com"));
        let cert = cname.certificate_configuration.as_ref().unwrap();
        assert_eq!(cert.cert_id.as_deref(), Some("cert-id"));
        assert_eq!(cert.previous_cert_id.as_deref(), Some("old-cert-id"));
        assert_eq!(cert.force, Some(true));
        assert_eq!(cert.delete_certificate, Some(false));
        assert_eq!(cname.is_wild_card, Some(false));
    }

    #[test]
    fn test_effective_cname_configuration_fallback() {
        // Deprecated top-level fields are wrapped into a Cname container.
        let config = BucketCnameConfiguration {
            domain: Some("example.com".to_string()),
            ..Default::default()
        };
        let effective = effective_cname_configuration(&config);
        let cname = effective.cname.as_ref().unwrap();
        assert_eq!(cname.domain.as_deref(), Some("example.com"));
        assert!(effective.domain.is_none());

        // An explicit Cname takes precedence over the deprecated fields.
        let config = BucketCnameConfiguration {
            domain: Some("deprecated.com".to_string()),
            cname: Some(Cname {
                domain: Some("preferred.com".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let effective = effective_cname_configuration(&config);
        assert_eq!(
            effective.cname.as_ref().unwrap().domain.as_deref(),
            Some("preferred.com")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_cname() {
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

        let bucket_name = generate_unique_bucket_name("cname-test");

        let created = client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(created.is_ok(), "create_bucket failed: {:?}", created.err());

        // Mapping a CNAME requires a verified domain; the server is expected
        // to reject the request for an unverified domain in test accounts.
        let result = client
            .put_cname(&PutCnameRequest {
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
        if let Err(error) = &result {
            eprintln!("put_cname rejected by server: {}", error);
        }

        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
    }
}
