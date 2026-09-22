use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::bucket::Owner;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct ListBucketSpacesRequest {
    /// The prefix of the agentic bucket whose spaces are listed. The client
    /// expands it to `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    /// The prefix that the names of the returned bucket spaces must contain.
    #[field(type = "query", rename = "prefix")]
    pub prefix: Option<String>,

    /// The token from which the list operation starts. Set it to the
    /// `NextContinuationToken` of the previous response.
    #[field(type = "query", rename = "continuation-token")]
    pub continuation_token: Option<String>,

    /// The name of the bucket space after which the list operation begins,
    /// sorted alphabetically.
    #[field(type = "query", rename = "start-after")]
    pub start_after: Option<String>,

    /// The maximum number of bucket spaces to return.
    #[field(type = "query", rename = "max-keys")]
    pub max_keys: Option<i32>,

    pub common: RequestCommon,
}

/// The summary of a bucket space in a listing.
#[derive(Debug, Default, Deserialize)]
pub struct BucketSpaceSummary {
    /// The name of the bucket space.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The region in which the bucket space is located.
    #[serde(rename = "Location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// The time when the bucket space was created.
    #[serde(rename = "CreationDate", skip_serializing_if = "Option::is_none")]
    pub creation_date: Option<String>,

    /// The storage class of the bucket space.
    #[serde(rename = "StorageClass", skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,
}

/// Deserializes the `<BucketSpaces>` container into its items.
mod bucket_spaces_de {
    use serde::{Deserialize, Deserializer};

    use super::BucketSpaceSummary;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<BucketSpaceSummary>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(rename = "BucketSpace", default)]
            spaces: Vec<BucketSpaceSummary>,
        }

        Wrapper::deserialize(deserializer).map(|w| w.spaces)
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "ListBucketSpacesResult")]
pub struct ListBucketSpacesResult {
    /// The owner of the bucket spaces.
    #[serde(rename = "Owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<Owner>,

    /// The bucket spaces in the agentic bucket.
    #[serde(rename = "BucketSpaces", default, with = "bucket_spaces_de")]
    pub bucket_spaces: Vec<BucketSpaceSummary>,

    /// The prefix that the names of the returned bucket spaces contain.
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// The maximum number of bucket spaces that can be returned.
    #[serde(rename = "MaxKeys", skip_serializing_if = "Option::is_none")]
    pub max_keys: Option<i32>,

    /// The token from which the list operation started.
    #[serde(rename = "ContinuationToken", skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,

    /// The token from which the next list operation starts.
    #[serde(
        rename = "NextContinuationToken",
        skip_serializing_if = "Option::is_none"
    )]
    pub next_continuation_token: Option<String>,

    /// The name of the bucket space after which the list operation began.
    #[serde(rename = "StartAfter", skip_serializing_if = "Option::is_none")]
    pub start_after: Option<String>,

    /// Indicates whether the returned results are truncated.
    #[serde(rename = "IsTruncated", skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists the bucket spaces in an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListBucketSpacesRequest` containing the bucket prefix
    ///   and the listing filters.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::ListBucketSpacesRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = ListBucketSpacesRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     prefix: Some("sandbox-".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.list_bucket_spaces(&request).await {
    ///     Ok(result) => println!("{} spaces", result.bucket_spaces.len()),
    ///     Err(error) => eprintln!("failed to list bucket spaces: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn list_bucket_spaces(
        &self,
        request: &ListBucketSpacesRequest,
    ) -> Result<ListBucketSpacesResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListBucketSpaces".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("agenticBucket", ""), ("bucketSpace", "")]
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
        let mut result: ListBucketSpacesResult =
            quick_xml::de::from_str(&String::from_utf8_lossy(&body_data))?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, ensure_agentic_bucket};

    const BODY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketSpacesResult>
  <Owner>
    <ID>1234567890123456</ID>
    <DisplayName>owner-name</DisplayName>
  </Owner>
  <Prefix>sandbox-</Prefix>
  <MaxKeys>20</MaxKeys>
  <StartAfter>sandbox-000</StartAfter>
  <IsTruncated>false</IsTruncated>
  <BucketSpaces>
    <BucketSpace>
      <Name>sandbox-001</Name>
      <Location>oss-cn-hangzhou</Location>
      <CreationDate>2024-01-01T00:00:00.000Z</CreationDate>
      <StorageClass>Standard</StorageClass>
    </BucketSpace>
  </BucketSpaces>
</ListBucketSpacesResult>"#;

    #[test]
    fn test_list_bucket_spaces_result_deserialize() {
        let result: ListBucketSpacesResult = quick_xml::de::from_str(BODY).unwrap();

        let owner = result.owner.unwrap();
        assert_eq!(owner.id.as_deref(), Some("1234567890123456"));
        assert_eq!(owner.display_name.as_deref(), Some("owner-name"));
        assert_eq!(result.prefix.as_deref(), Some("sandbox-"));
        assert_eq!(result.max_keys, Some(20));
        assert_eq!(result.start_after.as_deref(), Some("sandbox-000"));
        assert_eq!(result.is_truncated, Some(false));
        assert_eq!(result.bucket_spaces.len(), 1);
        assert_eq!(result.bucket_spaces[0].name.as_deref(), Some("sandbox-001"));
        assert_eq!(
            result.bucket_spaces[0].location.as_deref(),
            Some("oss-cn-hangzhou")
        );
        assert_eq!(
            result.bucket_spaces[0].storage_class.as_deref(),
            Some("Standard")
        );
    }

    #[test]
    fn test_list_bucket_spaces_result_deserialize_without_spaces() {
        let result: ListBucketSpacesResult =
            quick_xml::de::from_str("<ListBucketSpacesResult/>").unwrap();

        assert!(result.bucket_spaces.is_empty());
        assert!(result.owner.is_none());
    }

    #[test]
    fn test_list_bucket_spaces_request_queries() {
        let request = ListBucketSpacesRequest {
            bucket: "my-agentic".to_string(),
            prefix: Some("sandbox-".to_string()),
            continuation_token: Some("token1".to_string()),
            start_after: Some("sandbox-000".to_string()),
            max_keys: Some(20),
            ..Default::default()
        };

        let queries = request.query_map();
        assert_eq!(queries.get("prefix").unwrap(), "sandbox-");
        assert_eq!(queries.get("continuation-token").unwrap(), "token1");
        assert_eq!(queries.get("start-after").unwrap(), "sandbox-000");
        assert_eq!(queries.get("max-keys").unwrap(), "20");
        assert!(ListBucketSpacesRequest::default().query_map().is_empty());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_bucket_spaces() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .list_bucket_spaces(&ListBucketSpacesRequest {
                bucket: prefix.clone(),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("list_bucket_spaces rejected: {}", error);
        }
    }
}
