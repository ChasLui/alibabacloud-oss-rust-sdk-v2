use crate::HTTP_HEADER_CONTENT_TYPE;
use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::put_bucket_referer::{RefererBlacklist, RefererList};
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetBucketRefererRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "RefererConfiguration")]
pub struct GetBucketRefererResult {
    /// Indicates whether a request whose Referer field is empty is allowed.
    #[serde(rename = "AllowEmptyReferer", skip_serializing_if = "Option::is_none")]
    pub allow_empty_referer: Option<bool>,

    /// Indicates whether the query string in the URL is truncated when the
    /// Referer is matched.
    #[serde(rename = "AllowTruncateQueryString", skip_serializing_if = "Option::is_none")]
    pub allow_truncate_query_string: Option<bool>,

    /// Indicates whether the path and parts that follow the path in the URL are
    /// truncated when the Referer is matched.
    #[serde(rename = "TruncatePath", skip_serializing_if = "Option::is_none")]
    pub truncate_path: Option<bool>,

    /// The container that stores the Referer whitelist.
    #[serde(rename = "RefererList", skip_serializing_if = "Option::is_none")]
    pub referer_list: Option<RefererList>,

    /// The container that stores the Referer blacklist.
    #[serde(rename = "RefererBlacklist", skip_serializing_if = "Option::is_none")]
    pub referer_blacklist: Option<RefererBlacklist>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the hotlink protection configurations for a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetBucketRefererRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketRefererRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetBucketRefererRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_bucket_referer(&request).await {
    ///     Ok(result) => {
    ///         println!("Allow empty Referer: {:?}", result.allow_empty_referer);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get bucket Referer: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_bucket_referer(
        &self,
        request: &GetBucketRefererRequest,
    ) -> Result<GetBucketRefererResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetBucketReferer".to_string(),
            method: http::Method::GET,
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
        let mut result: GetBucketRefererResult = quick_xml::de::from_str(&data_str)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::super::put_bucket_referer::{PutBucketRefererRequest, RefererConfiguration, RefererList};
    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_get_bucket_referer_result_deserialize() {
        let xml = r#"<RefererConfiguration><AllowEmptyReferer>false</AllowEmptyReferer><AllowTruncateQueryString>true</AllowTruncateQueryString><TruncatePath>true</TruncatePath><RefererList><Referer>https://example.com</Referer></RefererList></RefererConfiguration>"#;
        let result: GetBucketRefererResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.allow_empty_referer, Some(false));
        assert_eq!(result.allow_truncate_query_string, Some(true));
        assert_eq!(result.truncate_path, Some(true));
        assert_eq!(
            result.referer_list.unwrap().referers,
            vec!["https://example.com".to_string()]
        );
        assert!(result.referer_blacklist.is_none());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_bucket_referer() {
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

        let bucket_name = generate_unique_bucket_name("get-bucket-referer");

        client
            .create_bucket(&CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        client
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
            .await
            .unwrap();

        let result = client
            .get_bucket_referer(&GetBucketRefererRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "get_bucket_referer failed: {:?}", result.err());
        let result = result.unwrap();
        assert_eq!(result.allow_empty_referer, Some(true));
        assert_eq!(
            result.referer_list.map(|l| l.referers),
            Some(vec!["https://example.com".to_string()])
        );

        // Clean up
        let _ = client
            .delete_bucket(&DeleteBucketRequest {
                bucket: bucket_name,
                ..Default::default()
            })
            .await;
    }
}
