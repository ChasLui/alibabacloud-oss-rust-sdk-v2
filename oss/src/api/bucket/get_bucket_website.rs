use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::put_bucket_website::{ErrorDocument, IndexDocument, RoutingRules};
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketWebsiteRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "WebsiteConfiguration")]
pub struct GetBucketWebsiteResult {
    /// The container that stores the default homepage.
    #[serde(rename = "IndexDocument", skip_serializing_if = "Option::is_none")]
    pub index_document: Option<IndexDocument>,

    /// The container that stores the default 404 page.
    #[serde(rename = "ErrorDocument", skip_serializing_if = "Option::is_none")]
    pub error_document: Option<ErrorDocument>,

    /// The container that stores the redirection rules.
    #[serde(rename = "RoutingRules", skip_serializing_if = "Option::is_none")]
    pub routing_rules: Option<RoutingRules>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the static website hosting status and redirection rules
    /// configured for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketWebsiteRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketWebsiteRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketWebsiteRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_website(&request).await {
    ///     Ok(result) => {
    ///         println!("Website configuration: {:?}", result.index_document);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket website: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_website(
        &self,
        request: &GetBucketWebsiteRequest,
    ) -> Result<GetBucketWebsiteResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketWebsite".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("website", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["website".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        // Parse the XML response
        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetBucketWebsiteResult = quick_xml::de::from_str(&data_str)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::put_bucket_website::{
        ErrorDocument, IndexDocument, PutBucketWebsiteRequest, WebsiteConfiguration,
    };
    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest, DeleteBucketWebsiteRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_get_bucket_website_result_deserialize() {
        let xml = r#"<WebsiteConfiguration><IndexDocument><Suffix>index.html</Suffix><SupportSubDir>true</SupportSubDir><Type>0</Type></IndexDocument><ErrorDocument><Key>error.html</Key><HttpStatus>404</HttpStatus></ErrorDocument><RoutingRules><RoutingRule><RuleNumber>1</RuleNumber><Condition><KeyPrefixEquals>abc/</KeyPrefixEquals><HttpErrorCodeReturnedEquals>404</HttpErrorCodeReturnedEquals></Condition><Redirect><RedirectType>Mirror</RedirectType><MirrorURL>http://example.com/</MirrorURL></Redirect></RoutingRule></RoutingRules></WebsiteConfiguration>"#;
        let result: GetBucketWebsiteResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(
            result.index_document.unwrap().suffix.as_deref(),
            Some("index.html")
        );
        assert_eq!(result.error_document.unwrap().key.as_deref(), Some("error.html"));
        let rules = result.routing_rules.unwrap().routing_rules;
        assert_eq!(rules.len(), 1);
        assert_eq!(
            rules[0].redirect.as_ref().unwrap().redirect_type.as_deref(),
            Some("Mirror")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_website() {
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

        let bucket_name = generate_unique_bucket_name("get-bucket-website");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        client
            .put_bucket_website(&PutBucketWebsiteRequest {
                bucket: bucket_name.clone(),
                website_configuration: WebsiteConfiguration {
                    index_document: Some(IndexDocument {
                        suffix: Some("index.html".to_string()),
                        ..Default::default()
                    }),
                    error_document: Some(ErrorDocument {
                        key: Some("error.html".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .get_bucket_website(&GetBucketWebsiteRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "get_bucket_website failed: {:?}", result.err());
        assert_eq!(
            result.unwrap().index_document.unwrap().suffix.as_deref(),
            Some("index.html")
        );

        // Clean up
        let _ = client
            .delete_bucket_website(&DeleteBucketWebsiteRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        let _ = client
            .delete_bucket(&DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
