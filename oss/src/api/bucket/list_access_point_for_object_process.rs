use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::create_access_point_for_object_process::AccessPointsForObjectProcess;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{modify_request, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct ListAccessPointsForObjectProcessRequest {
    /// The maximum number of Object FC Access Points to return. Valid values: 1
    /// to 1000. If the list cannot be complete at a time due to the
    /// configurations of the max-keys element, the NextContinuationToken element
    /// is included in the response as the token for the next list.
    #[field(type = "query", rename = "max-keys")]
    pub max_keys: Option<i64>,

    /// The token from which the list operation must start. You can obtain this
    /// token from the NextContinuationToken element in the returned result.
    #[field(type = "query", rename = "continuation-token")]
    pub continuation_token: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "ListAccessPointsForObjectProcessResult")]
pub struct ListAccessPointsForObjectProcessResult {
    /// The container that stores information about all Object FC Access Points.
    #[serde(rename = "AccessPointsForObjectProcess", skip_serializing_if = "Option::is_none")]
    pub access_points_for_object_process: Option<AccessPointsForObjectProcess>,

    /// Indicates whether the returned results are truncated. true: not all
    /// results are returned. false: all results are returned.
    #[serde(rename = "IsTruncated", skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,

    /// The token for the next list operation.
    #[serde(rename = "NextContinuationToken", skip_serializing_if = "Option::is_none")]
    pub next_continuation_token: Option<String>,

    /// The UID of the Alibaba Cloud account to which the Object FC Access
    /// Points belong.
    #[serde(rename = "AccountId", skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists information about Object FC Access Points in an Alibaba Cloud account.
    ///
    /// # Arguments
    ///
    /// * `request` - The `ListAccessPointsForObjectProcessRequest` containing
    ///   the optional max-keys and continuation-token.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::ListAccessPointsForObjectProcessRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListAccessPointsForObjectProcessRequest {
    ///     max_keys: Some(100),
    ///     ..Default::default()
    /// };
    ///
    /// match client.list_access_point_for_object_process(&request).await {
    ///     Ok(result) => {
    ///         println!("Is truncated: {:?}", result.is_truncated);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to list access points for object process: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn list_access_point_for_object_process(
        &self,
        request: &ListAccessPointsForObjectProcessRequest,
    ) -> Result<ListAccessPointsForObjectProcessResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "ListAccessPointsForObjectProcess".to_string(),
            method: http::Method::GET,
            parameters: [("accessPointForObjectProcess", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["accessPointForObjectProcess".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: ListAccessPointsForObjectProcessResult =
            quick_xml::de::from_str(&data_str)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_list_result_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListAccessPointsForObjectProcessResult>
   <IsTruncated>true</IsTruncated>
   <NextContinuationToken>abc</NextContinuationToken>
   <AccountId>1119</AccountId>
   <AccessPointsForObjectProcess>
      <AccessPointForObjectProcess>
          <AccessPointNameForObjectProcess>fc-ap-01</AccessPointNameForObjectProcess>
          <AccessPointForObjectProcessAlias>fc-ap-01-alias</AccessPointForObjectProcessAlias>
          <AccessPointName>fc-01</AccessPointName>
          <Status>enable</Status>
      </AccessPointForObjectProcess>
   </AccessPointsForObjectProcess>
</ListAccessPointsForObjectProcessResult>"#;
        let result: ListAccessPointsForObjectProcessResult =
            quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.is_truncated, Some(true));
        assert_eq!(result.next_continuation_token.as_deref(), Some("abc"));
        assert_eq!(result.account_id.as_deref(), Some("1119"));
        let list = result.access_points_for_object_process.unwrap();
        assert_eq!(list.access_point_for_object_processes.len(), 1);
        assert_eq!(
            list.access_point_for_object_processes[0]
                .access_point_name_for_object_process
                .as_deref(),
            Some("fc-ap-01")
        );
        assert_eq!(
            list.access_point_for_object_processes[0].status.as_deref(),
            Some("enable")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_access_point_for_object_process() {
        let config = match load_test_config() {
            Some(cfg) => cfg,
            None => {
                eprintln!("Test configuration not found. Skipping test.");
                return;
            }
        };

        let client = Client::new(
            &Config::default()
                .with_region(&config.region)
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    &config.access_key_id,
                    &config.access_key_secret,
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V4),
        );

        let result = client
            .list_access_point_for_object_process(&ListAccessPointsForObjectProcessRequest {
                max_keys: Some(100),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("list_access_point_for_object_process rejected: {}", error);
        }
    }
}
