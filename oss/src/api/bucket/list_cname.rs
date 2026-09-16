use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::OperationInput;
use crate::OperationOutput;

/// The container in which the certificate information is stored.
#[derive(Debug, Default, Deserialize)]
pub struct CnameCertificate {
    /// The time when the certificate was bound.
    #[serde(rename = "CreationDate", skip_serializing_if = "Option::is_none")]
    pub creation_date: Option<String>,

    /// The signature of the certificate.
    #[serde(rename = "Fingerprint", skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,

    /// The time when the certificate takes effect.
    #[serde(rename = "ValidStartDate", skip_serializing_if = "Option::is_none")]
    pub valid_start_date: Option<String>,

    /// The time when the certificate expires.
    #[serde(rename = "ValidEndDate", skip_serializing_if = "Option::is_none")]
    pub valid_end_date: Option<String>,

    /// The source of the certificate. Valid values: CAS, Upload.
    #[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
    pub certificate_type: Option<String>,

    /// The ID of the certificate.
    #[serde(rename = "CertId", skip_serializing_if = "Option::is_none")]
    pub cert_id: Option<String>,

    /// The status of the certificate. Valid values: Enabled, Disabled.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// The information about a CNAME record.
#[derive(Debug, Default, Deserialize)]
pub struct CnameInfo {
    /// The custom domain name.
    #[serde(rename = "Domain", skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    /// The time when the custom domain name was mapped.
    #[serde(rename = "LastModified", skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,

    /// The status of the domain name. Valid values: Enabled, Disabled.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// The container in which the certificate information is stored.
    #[serde(rename = "Certificate", skip_serializing_if = "Option::is_none")]
    pub certificate: Option<CnameCertificate>,

    /// Specifies whether the domain name is a wildcard domain name.
    #[serde(rename = "IsWildCard", skip_serializing_if = "Option::is_none")]
    pub is_wild_card: Option<bool>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct ListCnameRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct ListCnameResult {
    /// The container that is used to store the information about all CNAME
    /// records.
    #[serde(rename = "Cname", default)]
    pub cnames: Vec<CnameInfo>,

    /// The name of the bucket to which the CNAME records you want to query
    /// are mapped.
    #[serde(rename = "Bucket", skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// The name of the bucket owner.
    #[serde(rename = "Owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries all CNAME records that are mapped to a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListCnameRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::ListCnameRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListCnameRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.list_cname(&request).await {
    ///     Ok(result) => {
    ///         println!("Cname count: {:?}", result.cnames.len());
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to list cname: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn list_cname(
        &self,
        request: &ListCnameRequest,
    ) -> Result<ListCnameResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListCname".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("cname", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["cname".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: ListCnameResult = quick_xml::de::from_str(&data_str)?;
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
    fn test_list_cname_result_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListCnameResult>
  <Bucket>my-bucket</Bucket>
  <Owner>123456789</Owner>
  <Cname>
    <Domain>example.com</Domain>
    <LastModified>2021-09-15T02:03:04.000Z</LastModified>
    <Status>Enabled</Status>
    <Certificate>
      <Type>CAS</Type>
      <CertId>cert-id</CertId>
      <Status>Enabled</Status>
      <CreationDate>Wed, 15 Sep 2021 02:03:04 GMT</CreationDate>
      <Fingerprint>DE:01:CF:EC</Fingerprint>
      <ValidStartDate>Tue, 12 Apr 2022 10:14:51 GMT</ValidStartDate>
      <ValidEndDate>Mon, 4 May 2048 10:14:51 GMT</ValidEndDate>
    </Certificate>
    <IsWildCard>false</IsWildCard>
  </Cname>
</ListCnameResult>"#;

        let result: ListCnameResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.bucket.as_deref(), Some("my-bucket"));
        assert_eq!(result.owner.as_deref(), Some("123456789"));
        assert_eq!(result.cnames.len(), 1);
        let cname = &result.cnames[0];
        assert_eq!(cname.domain.as_deref(), Some("example.com"));
        assert_eq!(cname.status.as_deref(), Some("Enabled"));
        assert_eq!(cname.is_wild_card, Some(false));
        let cert = cname.certificate.as_ref().unwrap();
        assert_eq!(cert.certificate_type.as_deref(), Some("CAS"));
        assert_eq!(cert.cert_id.as_deref(), Some("cert-id"));
        assert_eq!(cert.fingerprint.as_deref(), Some("DE:01:CF:EC"));
        assert_eq!(
            cert.valid_end_date.as_deref(),
            Some("Mon, 4 May 2048 10:14:51 GMT")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_cname() {
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

        let bucket_name = generate_unique_bucket_name("list-cname-test");

        let created = client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(created.is_ok(), "create_bucket failed: {:?}", created.err());

        let result = client
            .list_cname(&ListCnameRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "list_cname failed: {:?}", result.err());
        assert!(result.unwrap().cnames.is_empty());

        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
    }
}
