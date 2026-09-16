use std::collections::HashMap;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;
use urlencoding;

use crate::api::bucket::{BucketProperties, Owner};
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{bucket_properties_de, modify_request, update_content_md5};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};
use crate::client::BodyDataReader;


#[derive(Debug, Default, Clone, OssRequestModel)]
pub struct ListBucketsRequest {
    /// The name of the bucket from which the list operation begins.
    #[field(type = "query")]
    pub marker: Option<String>,

    /// The maximum number of buckets that can be returned in the single query.
    /// Valid values: 1 to 1000.
    #[field(type = "query", rename = "max-keys")]
    pub max_keys: Option<i32>,

    /// The prefix that the names of returned buckets must contain.
    /// Limits the response to keys that begin with the specified prefix
    #[field(type = "query")]
    pub prefix: Option<String>,

    /// The encoding type of the content in the response. Valid value: url.
    #[field(type = "query", rename = "encoding-type")]
    pub encoding_type: Option<String>,

    /// The ID of the resource group.
    #[field(type = "header", rename = "x-oss-resource-group-id")]
    pub resource_group_id: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Deserialize, OssResultModel)]
pub struct ListBucketsResult {
    /// The prefix contained in the names of the returned bucket.
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// The name of the bucket after which the ListBuckets operation starts.
    #[serde(rename = "Marker", skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    /// The marker filter.

    /// The maximum number of buckets that can be returned for the request.
    #[serde(rename = "MaxKeys", skip_serializing_if = "Option::is_none")]
    pub max_keys: Option<i32>,

    /// Indicates whether all results are returned.
    /// true: Only part of the results are returned for the request.
    /// false: All results are returned for the request.
    #[serde(rename = "IsTruncated", skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,

    /// The marker for the next ListBuckets request, which can be used to return
    /// the remaining results.
    #[serde(rename = "NextMarker", skip_serializing_if = "Option::is_none")]
    pub next_marker: Option<String>,

    /// The container that stores information about the bucket owner.
    #[serde(rename = "Owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<Owner>,

    /// The container that stores information about buckets.
    #[serde(rename = "Buckets", with = "bucket_properties_de")]
    pub buckets: Vec<BucketProperties>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists the buckets in the OSS service.
    ///
    /// # Arguments
    ///
    /// * `request` - The list buckets request.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the list buckets result or an error.
    ///
    /// # Errors
    ///
    /// This function can return any error that implements the
    /// `std::error::Error` trait.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::service::ListBucketsRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListBucketsRequest::default();
    ///
    /// match client.list_buckets(&request).await {
    ///     Ok(list_buckets_result) => {
    ///         // Handle the list buckets result
    ///     }
    ///     Err(error) => {
    ///         // Handle the error
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn list_buckets(
        &self,
        request: &ListBucketsRequest,
    ) -> Result<ListBucketsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListBuckets".to_string(),
            method: http::Method::GET,
            parameters: HashMap::new(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        // Set encoding-type=url by default
        input.parameters.insert("encoding-type".to_string(), "url".to_string());

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_bytes = output.get_all_data().await?;
        let body_data = String::from_utf8_lossy(&body_bytes).into_owned();
        let mut result: ListBucketsResult =
            quick_xml::de::from_str(&body_data)?;

        result.update_result(&output);

        // Decode prefix after XML deserialization
        if let Some(ref mut prefix) = result.prefix {
            *prefix = urlencoding::decode(prefix)
                .unwrap_or_else(|_| std::borrow::Cow::Borrowed(prefix))
                .to_string();
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::SignatureVersionType;
    use crate::test_utils::{load_test_config, TestConfig};

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_buckets() {
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
                .with_signature_version(SignatureVersionType::V4)
                .with_log_level(LogLevel::Debug),
        );

        match client.list_buckets(&ListBucketsRequest::default()).await {
            Ok(output) => {
                println!("{:?}", output);
                
                // Print a human-friendly summary of the buckets
                println!("\n--- Human-Friendly Bucket Summary ---");
                println!("Total buckets found: {}", output.buckets.len());
                
                for (index, bucket) in output.buckets.iter().enumerate() {
                    println!(
                        "{}. Bucket Name: {}, Location: {}, Creation Date: {:?}",
                        index + 1,
                        bucket.name.as_ref().unwrap_or(&"<unknown>".to_string()),
                        bucket.location.as_ref().unwrap_or(&"<unknown>".to_string()),
                        bucket.creation_date
                    );
                }
                
                if let Some(owner) = &output.owner {
                    println!("Owner: {} ({})", 
                        owner.display_name.as_ref().unwrap_or(&"<unknown>".to_string()),
                        owner.id.as_ref().unwrap_or(&"<unknown>".to_string()));
                }
            },
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }
    }
}
