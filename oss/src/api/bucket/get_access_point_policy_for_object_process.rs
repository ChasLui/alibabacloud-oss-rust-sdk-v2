use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::utils::{modify_request, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetAccessPointPolicyForObjectProcessRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the Object FC Access Point.
    #[field(type = "header", rename = "x-oss-access-point-for-object-process-name")]
    pub access_point_for_object_process_name: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetAccessPointPolicyForObjectProcessResult {
    /// The configurations of the access point policy for object process, in json format.
    pub body: String,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Queries the policies of an Object FC Access Point.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetAccessPointPolicyForObjectProcessRequest` containing
    ///   the bucket name and the Object FC Access Point name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::GetAccessPointPolicyForObjectProcessRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetAccessPointPolicyForObjectProcessRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     access_point_for_object_process_name: Some("fc-ap-01".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_access_point_policy_for_object_process(&request).await {
    ///     Ok(result) => {
    ///         println!("Access point policy: {}", result.body);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get access point policy for object process: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_access_point_policy_for_object_process(
        &self,
        request: &GetAccessPointPolicyForObjectProcessRequest,
    ) -> Result<GetAccessPointPolicyForObjectProcessResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut input = OperationInput {
            op_name: "GetAccessPointPolicyForObjectProcess".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("accessPointPolicyForObjectProcess", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["accessPointPolicyForObjectProcess".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        // The response body is a raw json policy document, not XML.
        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);

        let mut result = GetAccessPointPolicyForObjectProcessResult {
            body: data_str.into_owned(),
            ..Default::default()
        };
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

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_access_point_policy_for_object_process() {
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

        // Querying a policy on a non-existent Object FC Access Point is
        // expected to be rejected by the server; the call exercises the request path.
        let result = client
            .get_access_point_policy_for_object_process(
                &GetAccessPointPolicyForObjectProcessRequest {
                    bucket: config.bucket.clone(),
                    access_point_for_object_process_name: Some("fc-ap-nonexistent".to_string()),
                    ..Default::default()
                },
            )
            .await;
        if let Err(error) = &result {
            eprintln!(
                "get_access_point_policy_for_object_process rejected (access point may not exist): {}",
                error
            );
        }
    }
}
