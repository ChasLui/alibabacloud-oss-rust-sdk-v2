use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteAccessPointPolicyForObjectProcessRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the Object FC Access Point.
    #[field(type = "header", rename = "x-oss-access-point-for-object-process-name")]
    pub access_point_for_object_process_name: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteAccessPointPolicyForObjectProcessResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes the policies of an Object FC Access Point.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteAccessPointPolicyForObjectProcessRequest`
    ///   containing the bucket name and the Object FC Access Point name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::DeleteAccessPointPolicyForObjectProcessRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteAccessPointPolicyForObjectProcessRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     access_point_for_object_process_name: Some("fc-ap-01".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_access_point_policy_for_object_process(&request).await {
    ///     Ok(result) => {
    ///         println!("Access point policy deleted: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete access point policy for object process: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_access_point_policy_for_object_process(
        &self,
        request: &DeleteAccessPointPolicyForObjectProcessRequest,
    ) -> Result<
        DeleteAccessPointPolicyForObjectProcessResult,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let mut input = OperationInput {
            op_name: "DeleteAccessPointPolicyForObjectProcess".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            parameters: [("accessPointPolicyForObjectProcess", "")]
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
            std::rc::Rc::new(vec!["accessPointPolicyForObjectProcess".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteAccessPointPolicyForObjectProcessResult::default();
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
    async fn test_delete_access_point_policy_for_object_process() {
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

        // Deleting a policy on a non-existent Object FC Access Point is
        // expected to be rejected by the server; the call exercises the request
        // path.
        let result = client
            .delete_access_point_policy_for_object_process(
                &DeleteAccessPointPolicyForObjectProcessRequest {
                    bucket: config.bucket.clone(),
                    access_point_for_object_process_name: Some("fc-ap-nonexistent".to_string()),
                    ..Default::default()
                },
            )
            .await;
        if let Err(error) = &result {
            eprintln!(
                "delete_access_point_policy_for_object_process rejected (access point may not \
                 exist): {}",
                error
            );
        }
    }
}
