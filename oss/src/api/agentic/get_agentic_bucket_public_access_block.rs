use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::service::PublicAccessBlockConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetAgenticBucketPublicAccessBlockRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetAgenticBucketPublicAccessBlockResult {
    /// The Block Public Access configuration of the agentic bucket.
    #[serde(skip)]
    pub public_access_block_configuration: Option<PublicAccessBlockConfiguration>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the Block Public Access configuration of an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetAgenticBucketPublicAccessBlockRequest` containing
    ///   the bucket prefix.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::GetAgenticBucketPublicAccessBlockRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = GetAgenticBucketPublicAccessBlockRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_agentic_bucket_public_access_block(&request).await {
    ///     Ok(result) => println!("{:?}", result.public_access_block_configuration),
    ///     Err(error) => eprintln!("failed to get agentic bucket public access block: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn get_agentic_bucket_public_access_block(
        &self,
        request: &GetAgenticBucketPublicAccessBlockRequest,
    ) -> Result<GetAgenticBucketPublicAccessBlockResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "GetAgenticBucketPublicAccessBlock".to_string(),
            method: http::Method::GET,
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

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let mut result = GetAgenticBucketPublicAccessBlockResult::default();
        if !body_data.is_empty() {
            let configuration: PublicAccessBlockConfiguration =
                quick_xml::de::from_str(&String::from_utf8_lossy(&body_data))?;
            result.public_access_block_configuration = Some(configuration);
        }
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, ensure_agentic_bucket};

    #[test]
    fn test_get_agentic_bucket_public_access_block_result_deserialize() {
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<PublicAccessBlockConfiguration>
  <BlockPublicAccess>true</BlockPublicAccess>
</PublicAccessBlockConfiguration>"#;

        let configuration: PublicAccessBlockConfiguration = quick_xml::de::from_str(body).unwrap();

        assert_eq!(configuration.block_public_access, Some(true));
    }

    #[test]
    fn test_get_agentic_bucket_public_access_block_result_deserialize_false() {
        let configuration: PublicAccessBlockConfiguration = quick_xml::de::from_str(
            "<PublicAccessBlockConfiguration><BlockPublicAccess>false</BlockPublicAccess></\
             PublicAccessBlockConfiguration>",
        )
        .unwrap();

        assert_eq!(configuration.block_public_access, Some(false));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_agentic_bucket_public_access_block() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .get_agentic_bucket_public_access_block(&GetAgenticBucketPublicAccessBlockRequest {
                bucket: prefix.clone(),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("get_agentic_bucket_public_access_block rejected: {}", error);
        }
    }
}
