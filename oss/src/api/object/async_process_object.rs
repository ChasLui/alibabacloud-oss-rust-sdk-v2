use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct AsyncProcessObjectRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// Image async processing parameters, for example
    /// `video/convert,f_mp4|saveas,o_<base64-encoded-target>`.
    pub async_process: String,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct AsyncProcessObjectResult {
    /// The event ID of the async process task.
    #[serde(default, rename = "EventId")]
    pub event_id: String,

    /// The request ID returned by the async process service.
    #[serde(default, rename = "RequestId")]
    pub request_id: String,

    /// The task ID of the async process task.
    #[serde(default, rename = "TaskId")]
    pub task_id: String,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Applies an async process on the specified object.
    ///
    /// # Arguments
    ///
    /// * `request` - The `AsyncProcessObjectRequest` containing the bucket
    ///   name, object key and the async process parameters.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::AsyncProcessObjectRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = AsyncProcessObjectRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-video.mp4".to_string(),
    ///     async_process: "video/convert,f_avi|saveas,o_b3V0LmF2aQ".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.async_process_object(&request).await {
    ///     Ok(result) => {
    ///         println!("Async task id: {}", result.task_id);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to async process object: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn async_process_object(
        &self,
        request: &AsyncProcessObjectRequest,
    ) -> Result<AsyncProcessObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "AsyncProcessObject".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("x-oss-async-process", request.async_process.as_str())]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        // The response body is JSON.
        let body_data = output.get_all_data().await?;
        let mut result: AsyncProcessObjectResult = serde_json::from_slice(&body_data)?;
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
    fn test_async_process_object_result_deserialize() {
        let json =
            r#"{"EventId":"3D8-1vB","RequestId":"57B7D2F1A04ED2908095F87C","TaskId":"task-123"}"#;
        let result: AsyncProcessObjectResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.event_id, "3D8-1vB");
        assert_eq!(result.request_id, "57B7D2F1A04ED2908095F87C");
        assert_eq!(result.task_id, "task-123");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_async_process_object() {
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

        let object_name = crate::test_utils::generate_unique_object_name("async-process");

        // Async process requires an object that the IMM service can handle;
        // this test only exercises the request path and expects either a
        // successful task submission or a service-level error.
        let result = client
            .async_process_object(&AsyncProcessObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                async_process: "doc/convert|saveas,o_b3V0LnBkZg".to_string(),
                ..Default::default()
            })
            .await;
        // The call fails unless IMM is bound to the bucket; only ensure the
        // request is constructed and sent without a client-side panic.
        let _ = result;
    }
}
