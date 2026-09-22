use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use super::create_access_point_for_object_process::ObjectProcessConfiguration;
use crate::api::service::PublicAccessBlockConfiguration;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The request body of PutAccessPointConfigForObjectProcess.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PutAccessPointConfigForObjectProcessConfiguration {
    /// Whether allow anonymous user to access this FC Access Point.
    #[serde(
        rename = "AllowAnonymousAccessForObjectProcess",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_anonymous_access_for_object_process: Option<String>,

    /// The container in which the Block Public Access configurations are
    /// stored.
    #[serde(
        rename = "PublicAccessBlockConfiguration",
        skip_serializing_if = "Option::is_none"
    )]
    pub public_access_block_configuration: Option<PublicAccessBlockConfiguration>,

    /// The container that stores the processing information about the Object FC
    /// Access Point.
    #[serde(
        rename = "ObjectProcessConfiguration",
        skip_serializing_if = "Option::is_none"
    )]
    pub object_process_configuration: Option<ObjectProcessConfiguration>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct PutAccessPointConfigForObjectProcessRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the Object FC Access Point. The name of an Object FC Access
    /// Point cannot exceed 63 characters in length, can contain only lowercase
    /// letters, digits, and hyphens (-), cannot start or end with a hyphen (-),
    /// and must be unique in the current region.
    #[field(type = "header", rename = "x-oss-access-point-for-object-process-name")]
    pub access_point_for_object_process_name: Option<String>,

    /// The request body.
    pub put_access_point_config_for_object_process_configuration:
        PutAccessPointConfigForObjectProcessConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutAccessPointConfigForObjectProcessResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Changes the configurations of an Object FC Access Point.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutAccessPointConfigForObjectProcessRequest`
    ///   containing the bucket name, the Object FC Access Point name and the
    ///   configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    /// #     PutAccessPointConfigForObjectProcessConfiguration,
    /// #     PutAccessPointConfigForObjectProcessRequest,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutAccessPointConfigForObjectProcessRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     access_point_for_object_process_name: Some("fc-ap-01".to_string()),
    ///     put_access_point_config_for_object_process_configuration:
    ///         PutAccessPointConfigForObjectProcessConfiguration {
    ///             allow_anonymous_access_for_object_process: Some("false".to_string()),
    ///             ..Default::default()
    ///         },
    ///     ..Default::default()
    /// };
    ///
    /// match client
    ///     .put_access_point_config_for_object_process(&request)
    ///     .await
    /// {
    ///     Ok(result) => {
    ///         println!("Access point config updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!(
    ///             "Failed to put access point config for object process: {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_access_point_config_for_object_process(
        &self,
        request: &PutAccessPointConfigForObjectProcessRequest,
    ) -> Result<PutAccessPointConfigForObjectProcessResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "PutAccessPointConfigForObjectProcess".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("accessPointConfigForObjectProcess", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["accessPointConfigForObjectProcess".to_string()]),
        );

        let xml_body = quick_xml::se::to_string_with_root(
            "PutAccessPointConfigForObjectProcessConfiguration",
            &request.put_access_point_config_for_object_process_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutAccessPointConfigForObjectProcessResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{
        AccessPointActions, ContentTransformation, ObjectProcessFunctionCompute,
        TransformationConfiguration, TransformationConfigurations,
    };
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_put_config_serde_round_trip() {
        let config = PutAccessPointConfigForObjectProcessConfiguration {
            allow_anonymous_access_for_object_process: Some("false".to_string()),
            public_access_block_configuration: Some(PublicAccessBlockConfiguration {
                block_public_access: Some(true),
            }),
            object_process_configuration: Some(ObjectProcessConfiguration {
                transformation_configurations: Some(TransformationConfigurations {
                    transformation_configurations: vec![TransformationConfiguration {
                        actions: Some(AccessPointActions {
                            actions: vec!["GetObject".to_string()],
                        }),
                        content_transformation: Some(ContentTransformation {
                            function_compute: Some(ObjectProcessFunctionCompute {
                                function_arn: Some(
                                    "acs:fc:cn-qingdao:1234567890:services/svc.LATEST/functions/\
                                     fc-01"
                                        .to_string(),
                                ),
                                function_assume_role_arn: Some(
                                    "acs:ram::1234567890:role/aliyunfcdefaultrole".to_string(),
                                ),
                            }),
                            ..Default::default()
                        }),
                    }],
                }),
                ..Default::default()
            }),
        };

        let xml = quick_xml::se::to_string_with_root(
            "PutAccessPointConfigForObjectProcessConfiguration",
            &config,
        )
        .unwrap();
        assert!(xml.contains("<PutAccessPointConfigForObjectProcessConfiguration>"));
        assert!(xml.contains("<BlockPublicAccess>true</BlockPublicAccess>"));
        assert!(xml.contains("<FunctionArn>"));

        let parsed: PutAccessPointConfigForObjectProcessConfiguration =
            quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(
            parsed.allow_anonymous_access_for_object_process.as_deref(),
            Some("false")
        );
        assert_eq!(
            parsed
                .public_access_block_configuration
                .unwrap()
                .block_public_access,
            Some(true)
        );
        let opc = parsed.object_process_configuration.unwrap();
        assert_eq!(
            opc.transformation_configurations
                .unwrap()
                .transformation_configurations
                .len(),
            1
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_access_point_config_for_object_process() {
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

        // Configuring a non-existent Object FC Access Point is expected to be
        // rejected by the server; the call exercises the request path.
        let result = client
            .put_access_point_config_for_object_process(
                &PutAccessPointConfigForObjectProcessRequest {
                    bucket: config.bucket.clone(),
                    access_point_for_object_process_name: Some("fc-ap-nonexistent".to_string()),
                    put_access_point_config_for_object_process_configuration:
                        PutAccessPointConfigForObjectProcessConfiguration {
                            allow_anonymous_access_for_object_process: Some("false".to_string()),
                            ..Default::default()
                        },
                    ..Default::default()
                },
            )
            .await;
        if let Err(error) = &result {
            eprintln!(
                "put_access_point_config_for_object_process rejected (access point may not \
                 exist): {}",
                error
            );
        }
    }
}
