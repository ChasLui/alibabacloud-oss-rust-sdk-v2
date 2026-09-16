use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::create_access_point_for_object_process::ObjectProcessConfiguration;
use crate::api::service::PublicAccessBlockConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{modify_request, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetAccessPointConfigForObjectProcessRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the Object FC Access Point.
    #[field(type = "header", rename = "x-oss-access-point-for-object-process-name")]
    pub access_point_for_object_process_name: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "GetAccessPointConfigForObjectProcessResult")]
pub struct GetAccessPointConfigForObjectProcessResult {
    /// The container that stores the processing information about the Object FC Access Point.
    #[serde(rename = "ObjectProcessConfiguration", skip_serializing_if = "Option::is_none")]
    pub object_process_configuration: Option<ObjectProcessConfiguration>,

    /// Whether allow anonymous user to access this FC Access Point.
    #[serde(rename = "AllowAnonymousAccessForObjectProcess", skip_serializing_if = "Option::is_none")]
    pub allow_anonymous_access_for_object_process: Option<String>,

    /// The container in which the Block Public Access configurations are stored.
    #[serde(rename = "PublicAccessBlockConfiguration", skip_serializing_if = "Option::is_none")]
    pub public_access_block_configuration: Option<PublicAccessBlockConfiguration>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the configurations of an Object FC Access Point.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetAccessPointConfigForObjectProcessRequest` containing
    ///   the bucket name and the Object FC Access Point name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetAccessPointConfigForObjectProcessRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetAccessPointConfigForObjectProcessRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     access_point_for_object_process_name: Some("fc-ap-01".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_access_point_config_for_object_process(&request).await {
    ///     Ok(result) => {
    ///         println!("Access point config: {:?}", result.object_process_configuration.is_some());
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get access point config for object process: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_access_point_config_for_object_process(
        &self,
        request: &GetAccessPointConfigForObjectProcessRequest,
    ) -> Result<GetAccessPointConfigForObjectProcessResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "GetAccessPointConfigForObjectProcess".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("accessPointConfigForObjectProcess", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["accessPointConfigForObjectProcess".to_string()]),
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
        let mut result: GetAccessPointConfigForObjectProcessResult =
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
    fn test_get_config_result_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<GetAccessPointConfigForObjectProcessResult>
  <ObjectProcessConfiguration>
    <AllowedFeatures/>
    <TransformationConfigurations>
      <TransformationConfiguration>
        <Actions>
          <Action>getobject</Action>
        </Actions>
        <ContentTransformation>
          <FunctionCompute>
            <FunctionAssumeRoleArn>acs:ram::1119:role/aliyunfcdefaultrole</FunctionAssumeRoleArn>
            <FunctionArn>acs:fc:cn-qingdao:1119:services/svc.LATEST/functions/fc-01</FunctionArn>
          </FunctionCompute>
        </ContentTransformation>
      </TransformationConfiguration>
    </TransformationConfigurations>
  </ObjectProcessConfiguration>
  <PublicAccessBlockConfiguration>
    <BlockPublicAccess>true</BlockPublicAccess>
  </PublicAccessBlockConfiguration>
</GetAccessPointConfigForObjectProcessResult>"#;
        let result: GetAccessPointConfigForObjectProcessResult =
            quick_xml::de::from_str(xml).unwrap();
        let opc = result.object_process_configuration.unwrap();
        assert!(opc.allowed_features.is_some());
        let tcs = opc.transformation_configurations.unwrap();
        assert_eq!(tcs.transformation_configurations.len(), 1);
        assert_eq!(
            tcs.transformation_configurations[0]
                .actions
                .as_ref()
                .unwrap()
                .actions,
            vec!["getobject".to_string()]
        );
        assert_eq!(
            result
                .public_access_block_configuration
                .unwrap()
                .block_public_access,
            Some(true)
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_access_point_config_for_object_process() {
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

        // Querying a non-existent Object FC Access Point is expected to be
        // rejected by the server; the call exercises the request path.
        let result = client
            .get_access_point_config_for_object_process(
                &GetAccessPointConfigForObjectProcessRequest {
                    bucket: config.bucket.clone(),
                    access_point_for_object_process_name: Some("fc-ap-nonexistent".to_string()),
                    ..Default::default()
                },
            )
            .await;
        if let Err(error) = &result {
            eprintln!(
                "get_access_point_config_for_object_process rejected (access point may not exist): {}",
                error
            );
        }
    }
}
