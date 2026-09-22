use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::bucket::Owner;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{acl_grant_de, modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetAgenticBucketAclRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "AccessControlPolicy")]
pub struct GetAgenticBucketAclResult {
    /// The access control list (ACL) of the agentic bucket.
    #[serde(rename = "AccessControlList", with = "acl_grant_de")]
    pub acl: Option<String>,

    /// The owner of the agentic bucket.
    #[serde(rename = "Owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<Owner>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the access control list (ACL) of an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetAgenticBucketAclRequest` containing the bucket
    ///   prefix.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::GetAgenticBucketAclRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = GetAgenticBucketAclRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_agentic_bucket_acl(&request).await {
    ///     Ok(result) => println!("acl: {:?}", result.acl),
    ///     Err(error) => eprintln!("failed to get agentic bucket acl: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn get_agentic_bucket_acl(
        &self,
        request: &GetAgenticBucketAclRequest,
    ) -> Result<GetAgenticBucketAclResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetAgenticBucketAcl".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("agenticBucket", ""), ("acl", "")]
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
        let mut result: GetAgenticBucketAclResult =
            quick_xml::de::from_str(&String::from_utf8_lossy(&body_data))?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, ensure_agentic_bucket};

    #[test]
    fn test_get_agentic_bucket_acl_result_deserialize() {
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<AccessControlPolicy>
  <Owner>
    <ID>1234567890123456</ID>
    <DisplayName>owner-name</DisplayName>
  </Owner>
  <AccessControlList>
    <Grant>private</Grant>
  </AccessControlList>
</AccessControlPolicy>"#;

        let result: GetAgenticBucketAclResult = quick_xml::de::from_str(body).unwrap();

        assert_eq!(result.acl.as_deref(), Some("private"));
        let owner = result.owner.unwrap();
        assert_eq!(owner.id.as_deref(), Some("1234567890123456"));
        assert_eq!(owner.display_name.as_deref(), Some("owner-name"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_agentic_bucket_acl() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .get_agentic_bucket_acl(&GetAgenticBucketAclRequest {
                bucket: prefix.clone(),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("get_agentic_bucket_acl rejected: {}", error);
        }
    }
}
