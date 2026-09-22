use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutAccessPointPolicyForObjectProcessRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the Object FC Access Point.
    #[field(type = "header", rename = "x-oss-access-point-for-object-process-name")]
    pub access_point_for_object_process_name: Option<String>,

    /// The json format permission policies for an Object FC Access Point.
    pub body: Option<BodyContent>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutAccessPointPolicyForObjectProcessResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Configures policies for an Object FC Access Point.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutAccessPointPolicyForObjectProcessRequest`
    ///   containing the bucket name, the Object FC Access Point name and the
    ///   json format permission policies.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::PutAccessPointPolicyForObjectProcessRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// # use alibabacloud_oss_sdk_rust_v2::BodyContent;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutAccessPointPolicyForObjectProcessRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     access_point_for_object_process_name: Some("fc-ap-01".to_string()),
    ///     body: Some(BodyContent::from_text(r#"{"Version":"1","Statement":[]}"#.to_string(), None)),
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_access_point_policy_for_object_process(request).await {
    ///     Ok(result) => {
    ///         println!("Access point policy updated: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put access point policy for object process: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_access_point_policy_for_object_process(
        &self,
        mut request: PutAccessPointPolicyForObjectProcessRequest,
    ) -> Result<PutAccessPointPolicyForObjectProcessResult, Box<dyn std::error::Error + Send + Sync>>
    {
        let headers = request.header_map();
        let queries = request.query_map();
        let bucket = request.bucket.clone();

        let mut input = OperationInput {
            op_name: "PutAccessPointPolicyForObjectProcess".to_string(),
            method: http::Method::PUT,
            bucket: Some(bucket),
            parameters: [("accessPointPolicyForObjectProcess", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            body: request.body.take(),
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
            headers,
            queries,
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = PutAccessPointPolicyForObjectProcessResult::default();
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
    async fn test_put_access_point_policy_for_object_process() {
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

        // Configuring a policy on a non-existent Object FC Access Point is
        // expected to be rejected by the server; the call exercises the request
        // path.
        let result = client
            .put_access_point_policy_for_object_process(
                PutAccessPointPolicyForObjectProcessRequest {
                    bucket: config.bucket.clone(),
                    access_point_for_object_process_name: Some("fc-ap-nonexistent".to_string()),
                    body: Some(BodyContent::from_text(
                        r#"{"Version":"1","Statement":[]}"#.to_string(),
                        None,
                    )),
                    ..Default::default()
                },
            )
            .await;
        if let Err(error) = &result {
            eprintln!(
                "put_access_point_policy_for_object_process rejected (access point may not \
                 exist): {}",
                error
            );
        }
    }
}
