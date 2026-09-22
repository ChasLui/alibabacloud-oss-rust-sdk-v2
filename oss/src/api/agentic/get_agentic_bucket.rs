use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::bucket::ServerSideEncryptionRule;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetAgenticBucketRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    pub common: RequestCommon,
}

/// The information about an agentic bucket.
#[derive(Debug, Default, Deserialize)]
pub struct AgenticBucketInfo {
    /// The physical name of the agentic bucket.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The owner of the agentic bucket.
    #[serde(rename = "Owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    /// The region in which the agentic bucket is located.
    #[serde(rename = "Region", skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// The storage class of the agentic bucket.
    #[serde(rename = "StorageClass", skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,

    /// The data redundancy type of the agentic bucket.
    #[serde(rename = "DataRedundancyType", skip_serializing_if = "Option::is_none")]
    pub data_redundancy_type: Option<String>,

    /// The status of the agentic bucket.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// The resource type of the agentic bucket.
    #[serde(rename = "BucketResourceType", skip_serializing_if = "Option::is_none")]
    pub bucket_resource_type: Option<String>,

    /// The time when the agentic bucket was created.
    #[serde(rename = "CreateTime", skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,

    /// The access control list (ACL) of the agentic bucket.
    #[serde(rename = "ACL", skip_serializing_if = "Option::is_none")]
    pub acl: Option<String>,

    /// The Block Public Access configuration of the agentic bucket.
    #[serde(rename = "PublicAccessBlock", skip_serializing_if = "Option::is_none")]
    pub public_access_block: Option<String>,

    /// The server-side encryption rule of the agentic bucket.
    #[serde(
        rename = "ServerSideEncryptionRule",
        skip_serializing_if = "Option::is_none"
    )]
    pub server_side_encryption_rule: Option<ServerSideEncryptionRule>,

    /// The versioning state of the agentic bucket.
    #[serde(rename = "Versioning", skip_serializing_if = "Option::is_none")]
    pub versioning: Option<String>,

    /// The policy of the agentic bucket, in JSON format.
    #[serde(rename = "BucketPolicy", skip_serializing_if = "Option::is_none")]
    pub bucket_policy: Option<String>,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetAgenticBucketResult {
    /// The information about the agentic bucket.
    #[serde(skip)]
    pub agentic_bucket_info: Option<AgenticBucketInfo>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the information about an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetAgenticBucketRequest` containing the bucket
    ///   prefix.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::GetAgenticBucketRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = GetAgenticBucketRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_agentic_bucket(&request).await {
    ///     Ok(result) => println!("{:?}", result.agentic_bucket_info),
    ///     Err(error) => eprintln!("failed to get agentic bucket: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn get_agentic_bucket(
        &self,
        request: &GetAgenticBucketRequest,
    ) -> Result<GetAgenticBucketResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetAgenticBucket".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
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
        let mut result = GetAgenticBucketResult::default();
        if !body_data.is_empty() {
            let info: AgenticBucketInfo =
                quick_xml::de::from_str(&String::from_utf8_lossy(&body_data))?;
            result.agentic_bucket_info = Some(info);
        }
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, ensure_agentic_bucket};

    const BODY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<AgenticBucketInfo>
  <Name>my-agentic-1234567890123456-cn-hangzhou-ab-apsr</Name>
  <Owner>1234567890123456</Owner>
  <Region>cn-hangzhou</Region>
  <StorageClass>Standard</StorageClass>
  <DataRedundancyType>LRS</DataRedundancyType>
  <Status>enabled</Status>
  <BucketResourceType>AgenticBucket</BucketResourceType>
  <CreateTime>2024-01-01T00:00:00.000Z</CreateTime>
  <ACL>private</ACL>
  <PublicAccessBlock>true</PublicAccessBlock>
  <Versioning>Enabled</Versioning>
  <BucketPolicy>{"Version":"1"}</BucketPolicy>
  <ServerSideEncryptionRule>
    <ApplyServerSideEncryptionByDefault>
      <SSEAlgorithm>AES256</SSEAlgorithm>
    </ApplyServerSideEncryptionByDefault>
  </ServerSideEncryptionRule>
</AgenticBucketInfo>"#;

    #[test]
    fn test_get_agentic_bucket_info_deserialize() {
        let info: AgenticBucketInfo = quick_xml::de::from_str(BODY).unwrap();

        assert_eq!(
            info.name.as_deref(),
            Some("my-agentic-1234567890123456-cn-hangzhou-ab-apsr")
        );
        assert_eq!(info.owner.as_deref(), Some("1234567890123456"));
        assert_eq!(info.region.as_deref(), Some("cn-hangzhou"));
        assert_eq!(info.storage_class.as_deref(), Some("Standard"));
        assert_eq!(info.data_redundancy_type.as_deref(), Some("LRS"));
        assert_eq!(info.status.as_deref(), Some("enabled"));
        assert_eq!(info.bucket_resource_type.as_deref(), Some("AgenticBucket"));
        assert_eq!(
            info.create_time.as_deref(),
            Some("2024-01-01T00:00:00.000Z")
        );
        assert_eq!(info.acl.as_deref(), Some("private"));
        assert_eq!(info.public_access_block.as_deref(), Some("true"));
        assert_eq!(info.versioning.as_deref(), Some("Enabled"));
        assert_eq!(info.bucket_policy.as_deref(), Some(r#"{"Version":"1"}"#));
        assert_eq!(
            info.server_side_encryption_rule
                .unwrap()
                .apply_server_side_encryption_by_default
                .unwrap()
                .sse_algorithm
                .as_deref(),
            Some("AES256")
        );
    }

    #[test]
    fn test_get_agentic_bucket_info_deserialize_partial() {
        let info: AgenticBucketInfo = quick_xml::de::from_str(
            "<AgenticBucketInfo><Status>disabled</Status></AgenticBucketInfo>",
        )
        .unwrap();

        assert_eq!(info.status.as_deref(), Some("disabled"));
        assert!(info.name.is_none());
        assert!(info.server_side_encryption_rule.is_none());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_agentic_bucket() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .get_agentic_bucket(&GetAgenticBucketRequest {
                bucket: prefix.clone(),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("get_agentic_bucket rejected: {}", error);
        }
    }
}
