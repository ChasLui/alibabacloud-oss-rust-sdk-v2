use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::bucket::VersioningConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetAgenticBucketVersioningRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct GetAgenticBucketVersioningResult {
    /// The versioning configuration of the agentic bucket.
    #[serde(skip)]
    pub versioning_configuration: Option<VersioningConfiguration>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the versioning state of an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetAgenticBucketVersioningRequest` containing the
    ///   bucket prefix.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::GetAgenticBucketVersioningRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = GetAgenticBucketVersioningRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_agentic_bucket_versioning(&request).await {
    ///     Ok(result) => println!("{:?}", result.versioning_configuration),
    ///     Err(error) => eprintln!("failed to get agentic bucket versioning: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn get_agentic_bucket_versioning(
        &self,
        request: &GetAgenticBucketVersioningRequest,
    ) -> Result<GetAgenticBucketVersioningResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetAgenticBucketVersioning".to_string(),
            method: http::Method::GET,
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

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let mut result = GetAgenticBucketVersioningResult::default();
        if !body_data.is_empty() {
            let configuration: VersioningConfiguration =
                quick_xml::de::from_str(&String::from_utf8_lossy(&body_data))?;
            result.versioning_configuration = Some(configuration);
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
    fn test_get_agentic_bucket_versioning_result_deserialize() {
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<VersioningConfiguration>
  <Status>Enabled</Status>
</VersioningConfiguration>"#;

        let configuration: VersioningConfiguration = quick_xml::de::from_str(body).unwrap();

        assert_eq!(configuration.status.as_deref(), Some("Enabled"));
    }

    #[test]
    fn test_get_agentic_bucket_versioning_result_deserialize_suspended() {
        let configuration: VersioningConfiguration = quick_xml::de::from_str(
            "<VersioningConfiguration><Status>Suspended</Status></VersioningConfiguration>",
        )
        .unwrap();

        assert_eq!(configuration.status.as_deref(), Some("Suspended"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_agentic_bucket_versioning() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        ensure_agentic_bucket(&client, &prefix).await;

        let result = client
            .get_agentic_bucket_versioning(&GetAgenticBucketVersioningRequest {
                bucket: prefix.clone(),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("get_agentic_bucket_versioning rejected: {}", error);
        }
    }
}
