use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the supported OSS API operations.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AccessPointActions {
    /// The supported OSS API operations. Only the GetObject operation is
    /// supported.
    #[serde(rename = "Action", default)]
    pub actions: Vec<String>,
}

/// The container that stores the custom forward headers.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ObjectProcessCustomForwardHeaders {
    /// The custom headers to forward to Function Compute.
    #[serde(rename = "CustomForwardHeader", default)]
    pub custom_forward_headers: Vec<String>,
}

/// The container that stores the information about Additional Features.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ObjectProcessAdditionalFeatures {
    /// The container that stores the custom forward headers.
    #[serde(
        rename = "CustomForwardHeaders",
        skip_serializing_if = "Option::is_none"
    )]
    pub custom_forward_headers: Option<ObjectProcessCustomForwardHeaders>,
}

/// The container that stores the information about Function Compute.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ObjectProcessFunctionCompute {
    /// The ARN of the function.
    #[serde(rename = "FunctionArn", skip_serializing_if = "Option::is_none")]
    pub function_arn: Option<String>,

    /// The Alibaba Cloud Resource Name (ARN) of the role that Function Compute
    /// uses to access your resources in other cloud services. The default role
    /// is AliyunFCDefaultRole.
    #[serde(
        rename = "FunctionAssumeRoleArn",
        skip_serializing_if = "Option::is_none"
    )]
    pub function_assume_role_arn: Option<String>,
}

/// The container that stores the content of the transformation configurations.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ContentTransformation {
    /// The container that stores the information about Function Compute.
    #[serde(rename = "FunctionCompute", skip_serializing_if = "Option::is_none")]
    pub function_compute: Option<ObjectProcessFunctionCompute>,

    /// The container that stores the information about Additional Features.
    #[serde(rename = "AdditionalFeatures", skip_serializing_if = "Option::is_none")]
    pub additional_features: Option<ObjectProcessAdditionalFeatures>,
}

/// The container that stores the transformation configuration.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TransformationConfiguration {
    /// The container that stores the operations.
    #[serde(rename = "Actions", skip_serializing_if = "Option::is_none")]
    pub actions: Option<AccessPointActions>,

    /// The container that stores the content of the transformation
    /// configurations.
    #[serde(
        rename = "ContentTransformation",
        skip_serializing_if = "Option::is_none"
    )]
    pub content_transformation: Option<ContentTransformation>,
}

/// The container that stores the transformation configurations.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TransformationConfigurations {
    /// The container that stores the transformation configurations.
    #[serde(rename = "TransformationConfiguration", default)]
    pub transformation_configurations: Vec<TransformationConfiguration>,
}

/// The container that stores allowed features.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ObjectProcessAllowedFeatures {
    /// Specifies that Function Compute supports Range GetObject requests.
    #[serde(rename = "AllowedFeature", default)]
    pub allowed_features: Vec<String>,
}

/// The container that stores the processing information about the Object FC
/// Access Point.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ObjectProcessConfiguration {
    /// The container that stores the transformation configurations.
    #[serde(
        rename = "TransformationConfigurations",
        skip_serializing_if = "Option::is_none"
    )]
    pub transformation_configurations: Option<TransformationConfigurations>,

    /// The container that stores allowed features.
    #[serde(rename = "AllowedFeatures", skip_serializing_if = "Option::is_none")]
    pub allowed_features: Option<ObjectProcessAllowedFeatures>,
}

/// The container that stores the endpoints of the Object FC Access Point.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AccessPointEndpoints {
    /// The internal endpoint of the Object FC Access Point.
    #[serde(rename = "InternalEndpoint", skip_serializing_if = "Option::is_none")]
    pub internal_endpoint: Option<String>,

    /// The public endpoint of the Object FC Access Point.
    #[serde(rename = "PublicEndpoint", skip_serializing_if = "Option::is_none")]
    pub public_endpoint: Option<String>,
}

/// The container that stores information about a single Object FC Access Point.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AccessPointForObjectProcess {
    /// The status of the Object FC Access Point. Valid values: enable, disable,
    /// creating and deleting.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Whether allow anonymous user access this FC Access Point.
    #[serde(
        rename = "AllowAnonymousAccessForObjectProcess",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_anonymous_access_for_object_process: Option<String>,

    /// The name of the Object FC Access Point.
    #[serde(
        rename = "AccessPointNameForObjectProcess",
        skip_serializing_if = "Option::is_none"
    )]
    pub access_point_name_for_object_process: Option<String>,

    /// The alias of the Object FC Access Point.
    #[serde(
        rename = "AccessPointForObjectProcessAlias",
        skip_serializing_if = "Option::is_none"
    )]
    pub access_point_for_object_process_alias: Option<String>,

    /// The name of the access point.
    #[serde(rename = "AccessPointName", skip_serializing_if = "Option::is_none")]
    pub access_point_name: Option<String>,
}

/// The container that stores information about all Object FC Access Points.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AccessPointsForObjectProcess {
    /// The container that stores information about a single Object FC Access
    /// Point.
    #[serde(rename = "AccessPointForObjectProcess", default)]
    pub access_point_for_object_processes: Vec<AccessPointForObjectProcess>,
}

/// The request body of CreateAccessPointForObjectProcess.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CreateAccessPointForObjectProcessConfiguration {
    /// Whether allow anonymous user to access this FC Access Point.
    #[serde(
        rename = "AllowAnonymousAccessForObjectProcess",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_anonymous_access_for_object_process: Option<String>,

    /// The name of the access point.
    #[serde(rename = "AccessPointName", skip_serializing_if = "Option::is_none")]
    pub access_point_name: Option<String>,

    /// The container that stores the processing information about the Object FC
    /// Access Point.
    #[serde(
        rename = "ObjectProcessConfiguration",
        skip_serializing_if = "Option::is_none"
    )]
    pub object_process_configuration: Option<ObjectProcessConfiguration>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct CreateAccessPointForObjectProcessRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the Object FC Access Point.
    #[field(type = "header", rename = "x-oss-access-point-for-object-process-name")]
    pub access_point_for_object_process_name: Option<String>,

    /// The request body.
    pub create_access_point_for_object_process_configuration:
        CreateAccessPointForObjectProcessConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
#[serde(rename = "CreateAccessPointForObjectProcessResult")]
pub struct CreateAccessPointForObjectProcessResult {
    /// The ARN of the Object FC Access Point.
    #[serde(
        rename = "AccessPointForObjectProcessArn",
        skip_serializing_if = "Option::is_none"
    )]
    pub access_point_for_object_process_arn: Option<String>,

    /// The alias of the Object FC Access Point.
    #[serde(
        rename = "AccessPointForObjectProcessAlias",
        skip_serializing_if = "Option::is_none"
    )]
    pub access_point_for_object_process_alias: Option<String>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Creates an Object FC Access Point.
    ///
    /// # Arguments
    ///
    /// * `request` - The `CreateAccessPointForObjectProcessRequest` containing
    ///   the bucket name, the Object FC Access Point name and the
    ///   configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    /// #     AccessPointActions, ContentTransformation,
    /// #     CreateAccessPointForObjectProcessConfiguration, CreateAccessPointForObjectProcessRequest,
    /// #     ObjectProcessConfiguration, ObjectProcessFunctionCompute, TransformationConfiguration,
    /// #     TransformationConfigurations,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = CreateAccessPointForObjectProcessRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     access_point_for_object_process_name: Some("fc-ap-01".to_string()),
    ///     create_access_point_for_object_process_configuration:
    ///         CreateAccessPointForObjectProcessConfiguration {
    ///             access_point_name: Some("ap-01".to_string()),
    ///             object_process_configuration: Some(ObjectProcessConfiguration {
    ///                 transformation_configurations: Some(TransformationConfigurations {
    ///                     transformation_configurations: vec![TransformationConfiguration {
    ///                         actions: Some(AccessPointActions {
    ///                             actions: vec!["GetObject".to_string()],
    ///                         }),
    ///                         content_transformation: Some(ContentTransformation {
    ///                             function_compute: Some(ObjectProcessFunctionCompute {
    ///                                 function_arn: Some("acs:fc:cn-hangzhou:111:functions/fc-01".to_string()),
    ///                                 ..Default::default()
    ///                             }),
    ///                             ..Default::default()
    ///                         }),
    ///                     }],
    ///                 }),
    ///                 ..Default::default()
    ///             }),
    ///             ..Default::default()
    ///         },
    ///     ..Default::default()
    /// };
    ///
    /// match client.create_access_point_for_object_process(&request).await {
    ///     Ok(result) => {
    ///         println!("Access point arn: {:?}", result.access_point_for_object_process_arn);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to create access point for object process: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn create_access_point_for_object_process(
        &self,
        request: &CreateAccessPointForObjectProcessRequest,
    ) -> Result<CreateAccessPointForObjectProcessResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "CreateAccessPointForObjectProcess".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("accessPointForObjectProcess", "")]
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
            std::rc::Rc::new(vec!["accessPointForObjectProcess".to_string()]),
        );

        let xml_body = quick_xml::se::to_string_with_root(
            "CreateAccessPointForObjectProcessConfiguration",
            &request.create_access_point_for_object_process_configuration,
        )?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: CreateAccessPointForObjectProcessResult =
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

    fn sample_configuration() -> CreateAccessPointForObjectProcessConfiguration {
        CreateAccessPointForObjectProcessConfiguration {
            allow_anonymous_access_for_object_process: Some("false".to_string()),
            access_point_name: Some("ap-01".to_string()),
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
                            additional_features: Some(ObjectProcessAdditionalFeatures {
                                custom_forward_headers: Some(ObjectProcessCustomForwardHeaders {
                                    custom_forward_headers: vec!["x-oss-meta-a".to_string()],
                                }),
                            }),
                        }),
                    }],
                }),
                allowed_features: Some(ObjectProcessAllowedFeatures {
                    allowed_features: vec!["GetObject-Range".to_string()],
                }),
            }),
        }
    }

    #[test]
    fn test_create_configuration_serde_round_trip() {
        let config = sample_configuration();
        let xml = quick_xml::se::to_string_with_root(
            "CreateAccessPointForObjectProcessConfiguration",
            &config,
        )
        .unwrap();
        assert!(xml.contains("<CreateAccessPointForObjectProcessConfiguration>"));
        assert!(xml.contains("<AccessPointName>ap-01</AccessPointName>"));
        assert!(xml.contains("<Action>GetObject</Action>"));
        assert!(xml.contains("<AllowedFeature>GetObject-Range</AllowedFeature>"));
        assert!(xml.contains("<CustomForwardHeader>x-oss-meta-a</CustomForwardHeader>"));

        let parsed: CreateAccessPointForObjectProcessConfiguration =
            quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.access_point_name.as_deref(), Some("ap-01"));
        let opc = parsed.object_process_configuration.unwrap();
        let tcs = opc.transformation_configurations.unwrap();
        assert_eq!(tcs.transformation_configurations.len(), 1);
        assert_eq!(
            tcs.transformation_configurations[0]
                .actions
                .as_ref()
                .unwrap()
                .actions,
            vec!["GetObject".to_string()]
        );
        assert_eq!(
            opc.allowed_features.unwrap().allowed_features,
            vec!["GetObject-Range".to_string()]
        );
    }

    #[test]
    fn test_create_result_deserialize() {
        let xml =
            "<CreateAccessPointForObjectProcessResult><AccessPointForObjectProcessArn>acs:oss:\
             cn-qingdao:123:accesspointforobjectprocess/fc-ap-01</\
             AccessPointForObjectProcessArn><AccessPointForObjectProcessAlias>fc-ap-01-alias</\
             AccessPointForObjectProcessAlias></CreateAccessPointForObjectProcessResult>";
        let result: CreateAccessPointForObjectProcessResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(
            result.access_point_for_object_process_arn.as_deref(),
            Some("acs:oss:cn-qingdao:123:accesspointforobjectprocess/fc-ap-01")
        );
        assert_eq!(
            result.access_point_for_object_process_alias.as_deref(),
            Some("fc-ap-01-alias")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_access_point_for_object_process() {
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

        // Creating an Object FC Access Point requires a real Function Compute
        // function, so a regular account may reject this call.
        let result = client
            .create_access_point_for_object_process(&CreateAccessPointForObjectProcessRequest {
                bucket: config.bucket.clone(),
                access_point_for_object_process_name: Some("fc-ap-test".to_string()),
                create_access_point_for_object_process_configuration: sample_configuration(),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!(
                "create_access_point_for_object_process rejected (FC function may not exist): {}",
                error
            );
        }

        // Clean up
        let _ = client
            .delete_access_point_for_object_process(
                &crate::api::bucket::DeleteAccessPointForObjectProcessRequest {
                    bucket: config.bucket.clone(),
                    access_point_for_object_process_name: Some("fc-ap-test".to_string()),
                    ..Default::default()
                },
            )
            .await;
    }
}
