use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The headers that are sent to the origin in mirroring-based back-to-origin.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MirrorHeadersSet {
    /// The key of the header. This parameter takes effect only when the value
    /// of RedirectType is Mirror.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The value of the header. This parameter takes effect only when the
    /// value of RedirectType is Mirror.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// The pass-through header configuration for mirroring-based back-to-origin.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MirrorHeaders {
    /// Specifies whether to pass through all request headers to the origin.
    /// This parameter takes effect only when the value of RedirectType is Mirror.
    #[serde(rename = "PassAll", skip_serializing_if = "Option::is_none")]
    pub pass_all: Option<bool>,

    /// The headers to pass through to the origin.
    #[serde(rename = "Pass", default)]
    pub passes: Vec<String>,

    /// The headers that are not allowed to pass through to the origin.
    #[serde(rename = "Remove", default)]
    pub removes: Vec<String>,

    /// The headers that are sent to the origin.
    #[serde(rename = "Set", default)]
    pub sets: Vec<MirrorHeadersSet>,
}

/// The header matching condition of a redirection rule.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RoutingRuleIncludeHeader {
    /// The key of the header. The rule is matched only when the specified
    /// header is included in the request and the header value equals Equals.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The value of the header.
    #[serde(rename = "Equals", skip_serializing_if = "Option::is_none")]
    pub equals: Option<String>,
}

/// The matching condition of a redirection rule.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RoutingRuleCondition {
    /// The prefix of object names. Only objects whose names contain the
    /// specified prefix match the rule.
    #[serde(rename = "KeyPrefixEquals", skip_serializing_if = "Option::is_none")]
    pub key_prefix_equals: Option<String>,

    /// Only objects that match this suffix can match this rule.
    #[serde(rename = "KeySuffixEquals", skip_serializing_if = "Option::is_none")]
    pub key_suffix_equals: Option<String>,

    /// The HTTP status code. The rule is matched only when the specified
    /// object is accessed and the specified HTTP status code is returned.
    #[serde(rename = "HttpErrorCodeReturnedEquals", skip_serializing_if = "Option::is_none")]
    pub http_error_code_returned_equals: Option<i64>,

    /// The headers that must be included in the request for the rule to match.
    #[serde(rename = "IncludeHeader", default)]
    pub include_headers: Vec<RoutingRuleIncludeHeader>,
}

/// The Lua script config of a redirection rule.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RoutingRuleLuaConfig {
    /// The name of the Lua script.
    #[serde(rename = "Script", skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
}

/// The authentication information for the origin server in mirroring-based
/// back-to-origin.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MirrorAuth {
    /// The authentication type.
    #[serde(rename = "AuthType", skip_serializing_if = "Option::is_none")]
    pub auth_type: Option<String>,

    /// The sign region for signature.
    #[serde(rename = "Region", skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// The access key id for signature.
    #[serde(rename = "AccessKeyId", skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,

    /// The access key secret for signature.
    #[serde(rename = "AccessKeySecret", skip_serializing_if = "Option::is_none")]
    pub access_key_secret: Option<String>,
}

/// The rule for setting a tag when saving files during mirroring-based
/// back-to-origin.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MirrorTagging {
    /// The tag key.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The rule for setting tag value for a specific tag key.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// The container of the rules for setting tags when saving files during
/// mirroring-based back-to-origin.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MirrorTaggings {
    /// The rule list for setting tags.
    #[serde(rename = "Taggings", default)]
    pub taggings: Vec<MirrorTagging>,
}

/// The rule for setting a response header in mirroring-based back-to-origin.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ReturnHeader {
    /// The response header.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The rule for setting response header value for a specific header.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// The container of the rules for setting response headers in mirroring-based
/// back-to-origin.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MirrorReturnHeaders {
    /// The rule list for setting response headers.
    #[serde(rename = "ReturnHeader", default)]
    pub return_headers: Vec<ReturnHeader>,
}

/// The configuration of a specific origin in mirroring-based back-to-origin.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MirrorMultiAlternate {
    /// The region for a specific origin.
    #[serde(rename = "MirrorMultiAlternateDstRegion", skip_serializing_if = "Option::is_none")]
    pub mirror_multi_alternate_dst_region: Option<String>,

    /// The distinct number of a specific origin.
    #[serde(rename = "MirrorMultiAlternateNumber", skip_serializing_if = "Option::is_none")]
    pub mirror_multi_alternate_number: Option<i64>,

    /// The URL for a specific origin.
    #[serde(rename = "MirrorMultiAlternateURL", skip_serializing_if = "Option::is_none")]
    pub mirror_multi_alternate_url: Option<String>,

    /// The VPC ID for a specific origin.
    #[serde(rename = "MirrorMultiAlternateVpcId", skip_serializing_if = "Option::is_none")]
    pub mirror_multi_alternate_vpc_id: Option<String>,
}

/// The container to store the configuration for multiple origins in
/// mirroring-based back-to-origin.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MirrorMultiAlternates {
    /// The configuration list for multiple origins.
    #[serde(rename = "MirrorMultiAlternate", default)]
    pub mirror_multi_alternates: Vec<MirrorMultiAlternate>,
}

/// The operation to perform after a redirection rule is matched.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RoutingRuleRedirect {
    /// The origin URL for mirroring-based back-to-origin.
    #[serde(rename = "MirrorURL", skip_serializing_if = "Option::is_none")]
    pub mirror_url: Option<String>,

    /// Specifies whether to redirect the access to the address specified by
    /// Location if the origin returns an HTTP 3xx status code.
    #[serde(rename = "MirrorFollowRedirect", skip_serializing_if = "Option::is_none")]
    pub mirror_follow_redirect: Option<bool>,

    /// Specifies whether the prefix of the object names is replaced with the
    /// value specified by ReplaceKeyPrefixWith.
    #[serde(rename = "EnableReplacePrefix", skip_serializing_if = "Option::is_none")]
    pub enable_replace_prefix: Option<bool>,

    /// The string that is used to replace the requested object name when the
    /// request is redirected.
    #[serde(rename = "ReplaceKeyWith", skip_serializing_if = "Option::is_none")]
    pub replace_key_with: Option<String>,

    /// The domain name used for redirection.
    #[serde(rename = "HostName", skip_serializing_if = "Option::is_none")]
    pub host_name: Option<String>,

    /// Specifies whether to include parameters of the original request in the
    /// redirection request.
    #[serde(rename = "PassQueryString", skip_serializing_if = "Option::is_none")]
    pub pass_query_string: Option<bool>,

    /// The headers contained in the response that is returned when you use
    /// mirroring-based back-to-origin.
    #[serde(rename = "MirrorHeaders", skip_serializing_if = "Option::is_none")]
    pub mirror_headers: Option<MirrorHeaders>,

    /// The string that is used to replace the prefix of the object name
    /// during redirection.
    #[serde(rename = "ReplaceKeyPrefixWith", skip_serializing_if = "Option::is_none")]
    pub replace_key_prefix_with: Option<String>,

    /// The redirection type. Valid values: Mirror, External, and AliCDN.
    #[serde(rename = "RedirectType", skip_serializing_if = "Option::is_none")]
    pub redirect_type: Option<String>,

    /// Is SNI transparent.
    #[serde(rename = "MirrorSNI", skip_serializing_if = "Option::is_none")]
    pub mirror_sni: Option<bool>,

    /// The protocol used for redirection. Valid values: http and https.
    #[serde(rename = "Protocol", skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,

    /// Specifies whether to check the MD5 hash of the body of the response
    /// returned by the origin.
    #[serde(rename = "MirrorCheckMd5", skip_serializing_if = "Option::is_none")]
    pub mirror_check_md5: Option<bool>,

    /// The HTTP redirect code in the response. Valid values: 301, 302, and 307.
    #[serde(rename = "HttpRedirectCode", skip_serializing_if = "Option::is_none")]
    pub http_redirect_code: Option<i64>,

    /// Is it transmitted transparently '/' to the source site.
    #[serde(rename = "MirrorPassOriginalSlashes", skip_serializing_if = "Option::is_none")]
    pub mirror_pass_original_slashes: Option<bool>,

    /// This parameter plays the same role as PassQueryString and has a higher
    /// priority than PassQueryString.
    #[serde(rename = "MirrorPassQueryString", skip_serializing_if = "Option::is_none")]
    pub mirror_pass_query_string: Option<bool>,

    /// The HTTP status codes that trigger the asynchronous pull mode in
    /// mirroring-based back-to-origin.
    #[serde(rename = "MirrorAsyncStatus", skip_serializing_if = "Option::is_none")]
    pub mirror_async_status: Option<i64>,

    /// The authentication information for the origin server in mirroring-based
    /// back-to-origin.
    #[serde(rename = "MirrorAuth", skip_serializing_if = "Option::is_none")]
    pub mirror_auth: Option<MirrorAuth>,

    /// The probe URL for mirroring-based back-to-origin.
    #[serde(rename = "MirrorURLProbe", skip_serializing_if = "Option::is_none")]
    pub mirror_url_probe: Option<String>,

    /// Whether to allow take video snapshot in mirroring-based back-to-origin.
    #[serde(rename = "MirrorAllowVideoSnapshot", skip_serializing_if = "Option::is_none")]
    pub mirror_allow_video_snapshot: Option<bool>,

    /// The slave URL for mirroring-based back-to-origin.
    #[serde(rename = "MirrorURLSlave", skip_serializing_if = "Option::is_none")]
    pub mirror_url_slave: Option<String>,

    /// The VPC ID for mirroring-based back-to-origin express tunnel.
    #[serde(rename = "MirrorDstVpcId", skip_serializing_if = "Option::is_none")]
    pub mirror_dst_vpc_id: Option<String>,

    /// Use LastModifiedTime of the file from origin.
    #[serde(rename = "MirrorUserLastModified", skip_serializing_if = "Option::is_none")]
    pub mirror_user_last_modified: Option<bool>,

    /// Whether to use role for mirroring-based back-to-origin.
    #[serde(rename = "MirrorUsingRole", skip_serializing_if = "Option::is_none")]
    pub mirror_using_role: Option<bool>,

    /// Mirroring-based back-to-origin with express tunnel.
    #[serde(rename = "MirrorIsExpressTunnel", skip_serializing_if = "Option::is_none")]
    pub mirror_is_express_tunnel: Option<bool>,

    /// Not save data in web-based back-to-origin.
    #[serde(rename = "MirrorProxyPass", skip_serializing_if = "Option::is_none")]
    pub mirror_proxy_pass: Option<bool>,

    /// The rules for setting tags when saving files during mirroring-based
    /// back-to-origin.
    #[serde(rename = "MirrorTaggings", skip_serializing_if = "Option::is_none")]
    pub mirror_taggings: Option<MirrorTaggings>,

    /// The slave VPC ID for mirroring-based back-to-origin express tunnel.
    #[serde(rename = "MirrorDstSlaveVpcId", skip_serializing_if = "Option::is_none")]
    pub mirror_dst_slave_vpc_id: Option<String>,

    /// The VPC region for mirroring-based back-to-origin express tunnel.
    #[serde(rename = "MirrorDstRegion", skip_serializing_if = "Option::is_none")]
    pub mirror_dst_region: Option<String>,

    /// Used for determining the state of primary-secondary switching.
    #[serde(rename = "MirrorSwitchAllErrors", skip_serializing_if = "Option::is_none")]
    pub mirror_switch_all_errors: Option<bool>,

    /// The tunnel ID for mirroring-based back-to-origin.
    #[serde(rename = "MirrorTunnelId", skip_serializing_if = "Option::is_none")]
    pub mirror_tunnel_id: Option<String>,

    /// The role name used for mirroring-based back-to-origin.
    #[serde(rename = "MirrorRole", skip_serializing_if = "Option::is_none")]
    pub mirror_role: Option<String>,

    /// Whether to allow get image information in mirroring-based
    /// back-to-origin.
    #[serde(rename = "MirrorAllowGetImageInfo", skip_serializing_if = "Option::is_none")]
    pub mirror_allow_get_image_info: Option<bool>,

    /// Whether to store the user defined metadata in mirroring-based
    /// back-to-origin.
    #[serde(rename = "MirrorSaveOssMeta", skip_serializing_if = "Option::is_none")]
    pub mirror_save_oss_meta: Option<bool>,

    /// Whether to allow take HeadObject in mirroring-based back-to-origin.
    #[serde(rename = "MirrorAllowHeadObject", skip_serializing_if = "Option::is_none")]
    pub mirror_allow_head_object: Option<bool>,

    /// The container to store the configuration for multiple origins in
    /// mirroring-based back-to-origin.
    #[serde(rename = "MirrorMultiAlternates", skip_serializing_if = "Option::is_none")]
    pub mirror_multi_alternates: Option<MirrorMultiAlternates>,

    /// The status codes returned by the origin server that should be passed
    /// through to the client along with the body, separated by commas.
    #[serde(rename = "TransparentMirrorResponseCodes", skip_serializing_if = "Option::is_none")]
    pub transparent_mirror_response_codes: Option<String>,

    /// Container to store the rules for setting response headers in
    /// mirroring-based back-to-origin.
    #[serde(rename = "MirrorReturnHeaders", skip_serializing_if = "Option::is_none")]
    pub mirror_return_headers: Option<MirrorReturnHeaders>,
}

/// A redirection rule or mirroring-based back-to-origin rule.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RoutingRule {
    /// The sequence number that is used to match and run the redirection rules.
    #[serde(rename = "RuleNumber", skip_serializing_if = "Option::is_none")]
    pub rule_number: Option<i64>,

    /// The matching condition. If all of the specified conditions are met, the
    /// rule is run.
    #[serde(rename = "Condition", skip_serializing_if = "Option::is_none")]
    pub condition: Option<RoutingRuleCondition>,

    /// The operation to perform after the rule is matched.
    #[serde(rename = "Redirect", skip_serializing_if = "Option::is_none")]
    pub redirect: Option<RoutingRuleRedirect>,

    /// The Lua script config of this rule.
    #[serde(rename = "LuaConfig", skip_serializing_if = "Option::is_none")]
    pub lua_config: Option<RoutingRuleLuaConfig>,
}

/// The container that stores the redirection rules.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RoutingRules {
    /// The specified redirection rules or mirroring-based back-to-origin
    /// rules. You can specify up to 20 rules.
    #[serde(rename = "RoutingRule", default)]
    pub routing_rules: Vec<RoutingRule>,
}

/// The container that stores the default homepage.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IndexDocument {
    /// The default homepage.
    #[serde(rename = "Suffix", skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,

    /// Specifies whether to redirect the access to the default homepage of the
    /// subdirectory when the subdirectory is accessed.
    #[serde(rename = "SupportSubDir", skip_serializing_if = "Option::is_none")]
    pub support_sub_dir: Option<bool>,

    /// The operation to perform when the default homepage is set, the name of
    /// the accessed object does not end with a forward slash (/), and the
    /// object does not exist.
    #[serde(rename = "Type")]
    pub r#type: Option<i64>,
}

/// The container that stores the default 404 page.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ErrorDocument {
    /// The error page.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The HTTP status code returned with the error page.
    #[serde(rename = "HttpStatus", skip_serializing_if = "Option::is_none")]
    pub http_status: Option<i64>,
}

/// The container that stores the website configuration.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WebsiteConfiguration {
    /// The container that stores the default homepage.
    #[serde(rename = "IndexDocument", skip_serializing_if = "Option::is_none")]
    pub index_document: Option<IndexDocument>,

    /// The container that stores the default 404 page.
    #[serde(rename = "ErrorDocument", skip_serializing_if = "Option::is_none")]
    pub error_document: Option<ErrorDocument>,

    /// The container that stores the redirection rules.
    #[serde(rename = "RoutingRules", skip_serializing_if = "Option::is_none")]
    pub routing_rules: Option<RoutingRules>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketWebsiteRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The request body schema.
    pub website_configuration: WebsiteConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketWebsiteResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Enables the static website hosting mode for a bucket and configures
    /// redirection rules for the bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutBucketWebsiteRequest` containing the bucket name
    ///   and the website configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    /// #     IndexDocument, PutBucketWebsiteRequest, WebsiteConfiguration,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketWebsiteRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     website_configuration: WebsiteConfiguration {
    ///         index_document: Some(IndexDocument {
    ///             suffix: Some("index.html".to_string()),
    ///             ..Default::default()
    ///         }),
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_bucket_website(&request).await {
    ///     Ok(result) => {
    ///         println!("Bucket website updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put bucket website: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_website(
        &self,
        request: &PutBucketWebsiteRequest,
    ) -> Result<PutBucketWebsiteResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketWebsite".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("website", "")]
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
            std::rc::Rc::new(vec!["website".to_string()]),
        );

        let xml_body =
            quick_xml::se::to_string_with_root("WebsiteConfiguration", &request.website_configuration)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketWebsiteResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest, DeleteBucketWebsiteRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_website_configuration_serde_round_trip() {
        let config = WebsiteConfiguration {
            index_document: Some(IndexDocument {
                suffix: Some("index.html".to_string()),
                support_sub_dir: Some(true),
                r#type: Some(0),
            }),
            error_document: Some(ErrorDocument {
                key: Some("error.html".to_string()),
                http_status: Some(404),
            }),
            routing_rules: Some(RoutingRules {
                routing_rules: vec![RoutingRule {
                    rule_number: Some(1),
                    condition: Some(RoutingRuleCondition {
                        key_prefix_equals: Some("abc/".to_string()),
                        http_error_code_returned_equals: Some(404),
                        include_headers: vec![RoutingRuleIncludeHeader {
                            key: Some("host".to_string()),
                            equals: Some("example.com".to_string()),
                        }],
                        ..Default::default()
                    }),
                    redirect: Some(RoutingRuleRedirect {
                        redirect_type: Some("Mirror".to_string()),
                        mirror_url: Some("http://example.com/".to_string()),
                        pass_query_string: Some(true),
                        mirror_headers: Some(MirrorHeaders {
                            pass_all: Some(true),
                            passes: vec!["myheader1".to_string()],
                            removes: vec!["myheader2".to_string()],
                            sets: vec![MirrorHeadersSet {
                                key: Some("myheader3".to_string()),
                                value: Some("value3".to_string()),
                            }],
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
            }),
        };

        let xml = quick_xml::se::to_string_with_root("WebsiteConfiguration", &config).unwrap();
        assert!(xml.contains("<WebsiteConfiguration>"));
        assert!(xml.contains("<Suffix>index.html</Suffix>"));
        assert!(xml.contains("<KeyPrefixEquals>abc/</KeyPrefixEquals>"));
        assert!(xml.contains("<MirrorURL>http://example.com/</MirrorURL>"));
        assert!(xml.contains("<Pass>myheader1</Pass>"));

        let parsed: WebsiteConfiguration = quick_xml::de::from_str(&xml).unwrap();
        let index = parsed.index_document.unwrap();
        assert_eq!(index.suffix.as_deref(), Some("index.html"));
        assert_eq!(index.support_sub_dir, Some(true));
        assert_eq!(index.r#type, Some(0));
        let error = parsed.error_document.unwrap();
        assert_eq!(error.key.as_deref(), Some("error.html"));
        assert_eq!(error.http_status, Some(404));
        let rules = parsed.routing_rules.unwrap().routing_rules;
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].rule_number, Some(1));
        let condition = rules[0].condition.as_ref().unwrap();
        assert_eq!(condition.key_prefix_equals.as_deref(), Some("abc/"));
        assert_eq!(condition.http_error_code_returned_equals, Some(404));
        assert_eq!(condition.include_headers.len(), 1);
        assert_eq!(condition.include_headers[0].key.as_deref(), Some("host"));
        let redirect = rules[0].redirect.as_ref().unwrap();
        assert_eq!(redirect.redirect_type.as_deref(), Some("Mirror"));
        assert_eq!(redirect.mirror_url.as_deref(), Some("http://example.com/"));
        let mirror_headers = redirect.mirror_headers.as_ref().unwrap();
        assert_eq!(mirror_headers.pass_all, Some(true));
        assert_eq!(mirror_headers.passes, vec!["myheader1".to_string()]);
        assert_eq!(mirror_headers.sets.len(), 1);
        assert_eq!(mirror_headers.sets[0].value.as_deref(), Some("value3"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_website() {
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

        let bucket_name = generate_unique_bucket_name("put-bucket-website");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .put_bucket_website(&PutBucketWebsiteRequest {
                bucket: bucket_name.clone(),
                website_configuration: WebsiteConfiguration {
                    index_document: Some(IndexDocument {
                        suffix: Some("index.html".to_string()),
                        support_sub_dir: Some(true),
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
            .await;
        assert!(result.is_ok(), "put_bucket_website failed: {:?}", result.err());

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
