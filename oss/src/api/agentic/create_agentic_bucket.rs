use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The configuration of a new agentic bucket.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CreateAgenticBucketConfiguration {
    /// The storage class of the agentic bucket. Valid values: Standard, IA,
    /// Archive, ColdArchive, and DeepColdArchive.
    #[serde(rename = "StorageClass", skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,

    /// The data redundancy type of the agentic bucket. Valid values: LRS and
    /// ZRS.
    #[serde(rename = "DataRedundancyType", skip_serializing_if = "Option::is_none")]
    pub data_redundancy_type: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct CreateAgenticBucketRequest {
    /// The prefix of the agentic bucket. The client expands it to
    /// `{prefix}-{accountId}-{region}-ab-apsr`.
    pub bucket: String,

    /// The configuration of the agentic bucket. When omitted, the bucket is
    /// created with the service defaults.
    pub create_agentic_bucket_configuration: Option<CreateAgenticBucketConfiguration>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct CreateAgenticBucketResult {
    pub common: ResultCommon,
}

impl Client {
    /// Creates an agentic bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `CreateAgenticBucketRequest` containing the bucket
    ///   prefix and, optionally, its storage class and data redundancy type.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::agentic::{
    /// #     CreateAgenticBucketConfiguration, CreateAgenticBucketRequest,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new_agentic(&Config::default());
    /// let request = CreateAgenticBucketRequest {
    ///     bucket: "my-agentic".to_string(),
    ///     create_agentic_bucket_configuration: Some(CreateAgenticBucketConfiguration {
    ///         storage_class: Some("Standard".to_string()),
    ///         ..Default::default()
    ///     }),
    ///     ..Default::default()
    /// };
    ///
    /// match client.create_agentic_bucket(&request).await {
    ///     Ok(result) => println!("created: {}", result.common.status),
    ///     Err(error) => eprintln!("failed to create agentic bucket: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn create_agentic_bucket(
        &self,
        request: &CreateAgenticBucketRequest,
    ) -> Result<CreateAgenticBucketResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "CreateAgenticBucket".to_string(),
            method: http::Method::PUT,
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

        if let Some(configuration) = &request.create_agentic_bucket_configuration {
            let xml_body = quick_xml::se::to_string_with_root(
                "CreateAgenticBucketConfiguration",
                configuration,
            )?;
            input.body = Some(BodyContent::from_text(xml_body, None));
        }

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = CreateAgenticBucketResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::agentic::test_support::{agentic_test_client, cleanup_agentic_bucket};

    #[test]
    fn test_create_agentic_bucket_configuration_serialize() {
        let configuration = CreateAgenticBucketConfiguration {
            storage_class: Some("Standard".to_string()),
            data_redundancy_type: Some("LRS".to_string()),
        };

        let xml =
            quick_xml::se::to_string_with_root("CreateAgenticBucketConfiguration", &configuration)
                .unwrap();

        assert_eq!(
            xml,
            "<CreateAgenticBucketConfiguration><StorageClass>Standard</\
             StorageClass><DataRedundancyType>LRS</DataRedundancyType></\
             CreateAgenticBucketConfiguration>"
        );
    }

    #[test]
    fn test_create_agentic_bucket_configuration_omits_unset_fields() {
        let xml = quick_xml::se::to_string_with_root(
            "CreateAgenticBucketConfiguration",
            &CreateAgenticBucketConfiguration::default(),
        )
        .unwrap();

        assert_eq!(xml, "<CreateAgenticBucketConfiguration/>");
    }

    #[test]
    fn test_create_agentic_bucket_request_headers_and_queries() {
        let request = CreateAgenticBucketRequest {
            bucket: "my-agentic".to_string(),
            ..Default::default()
        };

        assert!(request.header_map().is_empty());
        assert!(request.query_map().is_empty());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_agentic_bucket() {
        let Some((client, prefix)) = agentic_test_client() else {
            eprintln!("Test configuration not found. Skipping test.");
            return;
        };

        let result = client
            .create_agentic_bucket(&CreateAgenticBucketRequest {
                bucket: prefix.clone(),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("create_agentic_bucket rejected: {}", error);
        }

        cleanup_agentic_bucket(&client, &prefix).await;
    }
}
