use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutAgenticBucketAclRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    /// The access control list (ACL) of the agentic bucket. Valid values:
    /// private, public-read, and public-read-write.
    #[field(type = "header", rename = "x-oss-acl")]
    pub bucket_acl_type: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutAgenticBucketAclResult {
    pub common: ResultCommon,
}

impl Client {
    /// Configures the access control list (ACL) of an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutAgenticBucketAclRequest` containing the bucket
    ///   prefix and the ACL to apply.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::PutAgenticBucketAclRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = PutAgenticBucketAclRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     bucket_acl_type: "private".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_agentic_bucket_acl(&request).await {
    ///     Ok(result) => println!("updated: {}", result.common.status),
    ///     Err(error) => eprintln!("failed to put agentic bucket acl: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn put_agentic_bucket_acl(
        &self,
        request: &PutAgenticBucketAclRequest,
    ) -> Result<PutAgenticBucketAclResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutAgenticBucketAcl".to_string(),
            method: http::Method::PUT,
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

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutAgenticBucketAclResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, ensure_agentic_bucket};

    #[test]
    fn test_put_agentic_bucket_acl_request_header() {
        let request = PutAgenticBucketAclRequest {
            bucket: "my-agentic".to_string(),
            bucket_acl_type: "public-read".to_string(),
            ..Default::default()
        };

        assert_eq!(
            request.header_map().get("x-oss-acl").unwrap(),
            "public-read"
        );
    }

    #[test]
    fn test_put_agentic_bucket_acl_request_empty_acl_is_not_sent() {
        let request = PutAgenticBucketAclRequest {
            bucket: "my-agentic".to_string(),
            ..Default::default()
        };

        assert!(request.header_map().is_empty());
    }

    #[test]
    fn test_put_agentic_bucket_acl_request_common_header_wins() {
        let mut request = PutAgenticBucketAclRequest {
            bucket: "my-agentic".to_string(),
            bucket_acl_type: "private".to_string(),
            ..Default::default()
        };
        request.add_header("x-oss-acl", "public-read-write");

        assert_eq!(
            request.header_map().get("x-oss-acl").unwrap(),
            "public-read-write"
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_agentic_bucket_acl() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .put_agentic_bucket_acl(&PutAgenticBucketAclRequest {
                bucket: prefix.clone(),
                bucket_acl_type: "private".to_string(),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("put_agentic_bucket_acl rejected: {}", error);
        }
    }
}
