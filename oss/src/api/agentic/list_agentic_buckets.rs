use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct ListAgenticBucketsRequest {
    /// The token from which the list operation starts. Set it to the
    /// `NextContinuationToken` of the previous response.
    #[field(type = "query", rename = "continuation-token")]
    pub continuation_token: Option<String>,

    /// The maximum number of agentic buckets to return.
    #[field(type = "query", rename = "max-keys")]
    pub max_keys: Option<i32>,

    pub common: RequestCommon,
}

/// The summary of an agentic bucket in a listing.
#[derive(Debug, Default, Deserialize)]
pub struct AgenticBucketSummary {
    /// The physical name of the agentic bucket.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The storage class of the agentic bucket.
    #[serde(rename = "StorageClass", skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,

    /// The data redundancy type of the agentic bucket.
    #[serde(rename = "DataRedundancyType", skip_serializing_if = "Option::is_none")]
    pub data_redundancy_type: Option<String>,

    /// The time when the agentic bucket was created.
    #[serde(rename = "CreateTime", skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,
}

/// Deserializes the `<AgenticBuckets>` container into its items.
mod agentic_buckets_de {
    use serde::{Deserialize, Deserializer};

    use super::AgenticBucketSummary;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<AgenticBucketSummary>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(rename = "AgenticBucket", default)]
            buckets: Vec<AgenticBucketSummary>,
        }

        Wrapper::deserialize(deserializer).map(|w| w.buckets)
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "ListAgenticBucketsResult")]
pub struct ListAgenticBucketsResult {
    /// The region in which the agentic buckets are located.
    #[serde(rename = "Region", skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// The owner of the agentic buckets.
    #[serde(rename = "Owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    /// The token from which the list operation started.
    #[serde(rename = "ContinuationToken", skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,

    /// The token from which the next list operation starts.
    #[serde(
        rename = "NextContinuationToken",
        skip_serializing_if = "Option::is_none"
    )]
    pub next_continuation_token: Option<String>,

    /// Indicates whether the returned results are truncated.
    #[serde(rename = "IsTruncated", skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,

    /// The agentic buckets that belong to the account.
    #[serde(rename = "AgenticBuckets", default, with = "agentic_buckets_de")]
    pub agentic_buckets: Vec<AgenticBucketSummary>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists the agentic buckets that belong to the current account.
    ///
    /// This is an account-level operation: no bucket is addressed, so the
    /// request goes to the endpoint host itself.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListAgenticBucketsRequest` containing the paging
    ///   token and page size.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::ListAgenticBucketsRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    ///
    /// match client
    ///     .list_agentic_buckets(&ListAgenticBucketsRequest::default())
    ///     .await
    /// {
    ///     Ok(result) => println!("{} buckets", result.agentic_buckets.len()),
    ///     Err(error) => eprintln!("failed to list agentic buckets: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn list_agentic_buckets(
        &self,
        request: &ListAgenticBucketsRequest,
    ) -> Result<ListAgenticBucketsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListAgenticBuckets".to_string(),
            method: http::Method::GET,
            parameters: [("agenticBucket", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let mut result: ListAgenticBucketsResult =
            quick_xml::de::from_str(&String::from_utf8_lossy(&body_data))?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::agentic_test_client;

    const BODY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListAgenticBucketsResult>
  <Region>cn-hangzhou</Region>
  <Owner>1234567890123456</Owner>
  <IsTruncated>true</IsTruncated>
  <ContinuationToken>token1</ContinuationToken>
  <NextContinuationToken>token2</NextContinuationToken>
  <AgenticBuckets>
    <AgenticBucket>
      <Name>agentic-1</Name>
      <StorageClass>Standard</StorageClass>
      <DataRedundancyType>LRS</DataRedundancyType>
      <CreateTime>2024-01-01T00:00:00.000Z</CreateTime>
    </AgenticBucket>
    <AgenticBucket>
      <Name>agentic-2</Name>
      <StorageClass>IA</StorageClass>
      <DataRedundancyType>ZRS</DataRedundancyType>
      <CreateTime>2024-02-01T00:00:00.000Z</CreateTime>
    </AgenticBucket>
  </AgenticBuckets>
</ListAgenticBucketsResult>"#;

    #[test]
    fn test_list_agentic_buckets_result_deserialize() {
        let result: ListAgenticBucketsResult = quick_xml::de::from_str(BODY).unwrap();

        assert_eq!(result.region.as_deref(), Some("cn-hangzhou"));
        assert_eq!(result.owner.as_deref(), Some("1234567890123456"));
        assert_eq!(result.is_truncated, Some(true));
        assert_eq!(result.continuation_token.as_deref(), Some("token1"));
        assert_eq!(result.next_continuation_token.as_deref(), Some("token2"));
        assert_eq!(result.agentic_buckets.len(), 2);
        assert_eq!(result.agentic_buckets[0].name.as_deref(), Some("agentic-1"));
        assert_eq!(
            result.agentic_buckets[0].storage_class.as_deref(),
            Some("Standard")
        );
        assert_eq!(result.agentic_buckets[1].name.as_deref(), Some("agentic-2"));
        assert_eq!(
            result.agentic_buckets[1].data_redundancy_type.as_deref(),
            Some("ZRS")
        );
    }

    #[test]
    fn test_list_agentic_buckets_result_deserialize_empty() {
        let result: ListAgenticBucketsResult = quick_xml::de::from_str(
            "<ListAgenticBucketsResult><IsTruncated>false</IsTruncated></ListAgenticBucketsResult>",
        )
        .unwrap();

        assert_eq!(result.is_truncated, Some(false));
        assert!(result.agentic_buckets.is_empty());
    }

    #[test]
    fn test_list_agentic_buckets_request_queries() {
        let request = ListAgenticBucketsRequest {
            continuation_token: Some("token123".to_string()),
            max_keys: Some(10),
            ..Default::default()
        };

        let queries = request.query_map();
        assert_eq!(queries.get("continuation-token").unwrap(), "token123");
        assert_eq!(queries.get("max-keys").unwrap(), "10");
        // An unset page size must not reach the wire.
        assert!(ListAgenticBucketsRequest::default().query_map().is_empty());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_agentic_buckets() {
        let Some((client, _prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        let result = client
            .list_agentic_buckets(&ListAgenticBucketsRequest {
                max_keys: Some(10),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("list_agentic_buckets rejected: {}", error);
        }
    }
}
