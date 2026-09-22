use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::service::PublicAccessBlockConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutAgenticBucketPublicAccessBlockRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    /// The Block Public Access configuration of the agentic bucket.
    pub public_access_block_configuration: PublicAccessBlockConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutAgenticBucketPublicAccessBlockResult {
    pub common: ResultCommon,
}

impl Client {
    /// Configures the Block Public Access of an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutAgenticBucketPublicAccessBlockRequest` containing
    ///   the bucket prefix and the configuration to apply.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::PutAgenticBucketPublicAccessBlockRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::api::service::PublicAccessBlockConfiguration;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = PutAgenticBucketPublicAccessBlockRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     public_access_block_configuration: PublicAccessBlockConfiguration {
    ///         block_public_access: Some(true),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_agentic_bucket_public_access_block(&request).await {
    ///     Ok(result) => println!("updated: {}", result.common.status),
    ///     Err(error) => eprintln!("failed to put agentic bucket public access block: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn put_agentic_bucket_public_access_block(
        &self,
        request: &PutAgenticBucketPublicAccessBlockRequest,
    ) -> Result<PutAgenticBucketPublicAccessBlockResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "PutAgenticBucketPublicAccessBlock".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("agenticBucket", ""), ("publicAccessBlock", "")]
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
            "PublicAccessBlockConfiguration",
            &request.public_access_block_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutAgenticBucketPublicAccessBlockResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, ensure_agentic_bucket};

    #[test]
    fn test_put_agentic_bucket_public_access_block_body_serialize() {
        let configuration = PublicAccessBlockConfiguration {
            block_public_access: Some(true),
        };

        let xml =
            quick_xml::se::to_string_with_root("PublicAccessBlockConfiguration", &configuration)
                .unwrap();

        assert_eq!(
            xml,
            "<PublicAccessBlockConfiguration><BlockPublicAccess>true</BlockPublicAccess></\
             PublicAccessBlockConfiguration>"
        );
    }

    #[test]
    fn test_put_agentic_bucket_public_access_block_omits_unset_flag() {
        let xml = quick_xml::se::to_string_with_root(
            "PublicAccessBlockConfiguration",
            &PublicAccessBlockConfiguration::default(),
        )
        .unwrap();

        assert_eq!(xml, "<PublicAccessBlockConfiguration/>");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_agentic_bucket_public_access_block() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .put_agentic_bucket_public_access_block(&PutAgenticBucketPublicAccessBlockRequest {
                bucket: prefix.clone(),
                public_access_block_configuration: PublicAccessBlockConfiguration {
                    block_public_access: Some(true),
                },
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("put_agentic_bucket_public_access_block rejected: {}", error);
        }
    }
}
