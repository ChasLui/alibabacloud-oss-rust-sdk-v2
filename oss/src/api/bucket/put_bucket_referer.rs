use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the Referer whitelist.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RefererList {
    /// The addresses in the Referer whitelist.
    #[serde(rename = "Referer", default)]
    pub referers: Vec<String>,
}

/// The container that stores the Referer blacklist.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RefererBlacklist {
    /// The addresses in the Referer blacklist.
    #[serde(rename = "Referer", default)]
    pub referers: Vec<String>,
}

/// The container that stores the hotlink protection configurations.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RefererConfiguration {
    /// Specifies whether to allow a request whose Referer field is empty.
    #[serde(rename = "AllowEmptyReferer", skip_serializing_if = "Option::is_none")]
    pub allow_empty_referer: Option<bool>,

    /// Specifies whether to truncate the query string in the URL when the
    /// Referer is matched.
    #[serde(rename = "AllowTruncateQueryString", skip_serializing_if = "Option::is_none")]
    pub allow_truncate_query_string: Option<bool>,

    /// Specifies whether to truncate the path and parts that follow the path in
    /// the URL when the Referer is matched.
    #[serde(rename = "TruncatePath", skip_serializing_if = "Option::is_none")]
    pub truncate_path: Option<bool>,

    /// The container that stores the Referer whitelist. The PutBucketReferer
    /// operation overwrites the existing Referer whitelist with the Referer
    /// whitelist specified in RefererList. If RefererList is not specified in
    /// the request, which specifies that no Referer elements are included, the
    /// operation clears the existing Referer whitelist.
    #[serde(rename = "RefererList", skip_serializing_if = "Option::is_none")]
    pub referer_list: Option<RefererList>,

    /// The container that stores the Referer blacklist.
    #[serde(rename = "RefererBlacklist", skip_serializing_if = "Option::is_none")]
    pub referer_blacklist: Option<RefererBlacklist>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketRefererRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The request body schema.
    pub referer_configuration: RefererConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketRefererResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Configures a Referer whitelist for an Object Storage Service (OSS)
    /// bucket. You can specify whether to allow the requests whose Referer
    /// field is empty or whose query strings are truncated.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketRefererRequest` containing the bucket name
    ///   and the Referer configuration to set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{PutBucketRefererRequest, RefererConfiguration, RefererList};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketRefererRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     referer_configuration: RefererConfiguration {
    ///         allow_empty_referer: Some(true),
    ///         referer_list: Some(RefererList {
    ///             referers: vec!["https://example.com".to_string()],
    ///         }),
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_referer(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket Referer updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket Referer: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_referer(
        &self,
        request: &PutBucketRefererRequest,
    ) -> Result<PutBucketRefererResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketReferer".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("referer", "")]
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
            std::rc::Rc::new(vec!["referer".to_string()]),
        );

        let xml_body =
            quick_xml::se::to_string_with_root("RefererConfiguration", &request.referer_configuration)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketRefererResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_referer_configuration_serde_round_trip() {
        let config = RefererConfiguration {
            allow_empty_referer: Some(true),
            allow_truncate_query_string: Some(false),
            truncate_path: Some(true),
            referer_list: Some(RefererList {
                referers: vec![
                    "https://example.com".to_string(),
                    "https://*.example.org".to_string(),
                ],
            }),
            referer_blacklist: Some(RefererBlacklist {
                referers: vec!["https://bad.example.com".to_string()],
            }),
        };

        let xml = quick_xml::se::to_string_with_root("RefererConfiguration", &config).unwrap();
        assert!(xml.contains("<RefererConfiguration>"));
        assert!(xml.contains("<AllowEmptyReferer>true</AllowEmptyReferer>"));
        assert!(xml.contains("<AllowTruncateQueryString>false</AllowTruncateQueryString>"));
        assert!(xml.contains("<TruncatePath>true</TruncatePath>"));
        assert!(xml.contains("<Referer>https://example.com</Referer>"));
        assert!(xml.contains("<Referer>https://bad.example.com</Referer>"));

        let parsed: RefererConfiguration = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.allow_empty_referer, Some(true));
        assert_eq!(parsed.allow_truncate_query_string, Some(false));
        assert_eq!(parsed.truncate_path, Some(true));
        assert_eq!(
            parsed.referer_list.unwrap().referers,
            vec![
                "https://example.com".to_string(),
                "https://*.example.org".to_string()
            ]
        );
        assert_eq!(
            parsed.referer_blacklist.unwrap().referers,
            vec!["https://bad.example.com".to_string()]
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_referer() {
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

        let bucket_name = generate_unique_bucket_name("put-bucket-referer");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .put_bucket_referer(&PutBucketRefererRequest {
                bucket: bucket_name.clone(),
                referer_configuration: RefererConfiguration {
                    allow_empty_referer: Some(true),
                    referer_list: Some(RefererList {
                        referers: vec!["https://example.com".to_string()],
                    }),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "put_bucket_referer failed: {:?}", result.err());

        // Clean up
        let _ = client
            .delete_bucket(&DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
