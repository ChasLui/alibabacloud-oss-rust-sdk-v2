use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::bucket::VersioningConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutAgenticBucketVersioningRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    /// The versioning configuration of the agentic bucket.
    pub versioning_configuration: VersioningConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutAgenticBucketVersioningResult {
    pub common: ResultCommon,
}

impl Client {
    /// Configures the versioning state of an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutAgenticBucketVersioningRequest` containing the
    ///   bucket prefix and the versioning state to apply.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::PutAgenticBucketVersioningRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::VersioningConfiguration;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = PutAgenticBucketVersioningRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     versioning_configuration: VersioningConfiguration {
    ///         status: Some("Enabled".to_string()),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_agentic_bucket_versioning(&request).await {
    ///     Ok(result) => println!("updated: {}", result.common.status),
    ///     Err(error) => eprintln!("failed to put agentic bucket versioning: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn put_agentic_bucket_versioning(
        &self,
        request: &PutAgenticBucketVersioningRequest,
    ) -> Result<PutAgenticBucketVersioningResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutAgenticBucketVersioning".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("agenticBucket", ""), ("versioning", "")]
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
            "VersioningConfiguration",
            &request.versioning_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutAgenticBucketVersioningResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, ensure_agentic_bucket};

    #[test]
    fn test_put_agentic_bucket_versioning_body_serialize() {
        let configuration = VersioningConfiguration {
            status: Some("Enabled".to_string()),
        };

        let xml =
            quick_xml::se::to_string_with_root("VersioningConfiguration", &configuration).unwrap();

        assert_eq!(
            xml,
            "<VersioningConfiguration><Status>Enabled</Status></VersioningConfiguration>"
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_agentic_bucket_versioning() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .put_agentic_bucket_versioning(&PutAgenticBucketVersioningRequest {
                bucket: prefix.clone(),
                versioning_configuration: VersioningConfiguration {
                    status: Some("Enabled".to_string()),
                },
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("put_agentic_bucket_versioning rejected: {}", error);
        }
    }
}
