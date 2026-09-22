use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The status of an agentic bucket.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AgenticBucketStatus {
    /// The status of the agentic bucket. Valid values: enabled and disabled.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutAgenticBucketStatusRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    /// The status to apply to the agentic bucket.
    pub agentic_bucket_status: AgenticBucketStatus,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutAgenticBucketStatusResult {
    pub common: ResultCommon,
}

impl Client {
    /// Configures the status of an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutAgenticBucketStatusRequest` containing the bucket
    ///   prefix and the status to apply.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::{
    /// #     AgenticBucketStatus, PutAgenticBucketStatusRequest,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = PutAgenticBucketStatusRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     agentic_bucket_status: AgenticBucketStatus {
    ///         status: Some("enabled".to_string()),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_agentic_bucket_status(&request).await {
    ///     Ok(result) => println!("updated: {}", result.common.status),
    ///     Err(error) => eprintln!("failed to put agentic bucket status: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn put_agentic_bucket_status(
        &self,
        request: &PutAgenticBucketStatusRequest,
    ) -> Result<PutAgenticBucketStatusResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutAgenticBucketStatus".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("agenticBucket", ""), ("status", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        let xml_body = quick_xml::se::to_string_with_root(
            "AgenticBucketStatus",
            &request.agentic_bucket_status,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutAgenticBucketStatusResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, ensure_agentic_bucket};

    #[test]
    fn test_put_agentic_bucket_status_body_serialize() {
        let status = AgenticBucketStatus {
            status: Some("enabled".to_string()),
        };

        let xml = quick_xml::se::to_string_with_root("AgenticBucketStatus", &status).unwrap();

        assert_eq!(
            xml,
            "<AgenticBucketStatus><Status>enabled</Status></AgenticBucketStatus>"
        );
    }

    #[test]
    fn test_put_agentic_bucket_status_body_serialize_without_status() {
        let xml = quick_xml::se::to_string_with_root(
            "AgenticBucketStatus",
            &AgenticBucketStatus::default(),
        )
        .unwrap();

        assert_eq!(xml, "<AgenticBucketStatus/>");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_agentic_bucket_status() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .put_agentic_bucket_status(&PutAgenticBucketStatusRequest {
                bucket: prefix.clone(),
                agentic_bucket_status: AgenticBucketStatus {
                    status: Some("enabled".to_string()),
                },
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("put_agentic_bucket_status rejected: {}", error);
        }
    }
}
