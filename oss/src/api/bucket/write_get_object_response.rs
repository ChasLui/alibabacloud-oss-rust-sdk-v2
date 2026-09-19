use crate::HTTP_HEADER_CONTENT_TYPE;
use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct WriteGetObjectResponseRequest {
    /// The router forwarding address obtained from the event parameter of
    /// Function Compute.
    #[field(type = "header", rename = "x-oss-request-route")]
    pub request_route: Option<String>,

    /// The unique forwarding token obtained from the event parameter of
    /// Function Compute.
    #[field(type = "header", rename = "x-oss-request-token")]
    pub request_token: Option<String>,

    /// The HTTP status code returned by the backend server.
    #[field(type = "header", rename = "x-oss-fwd-status")]
    pub fwd_status: Option<String>,

    /// The HTTP response header returned by the backend server. It is used to
    /// specify the scope of the resources that you want to query.
    #[field(type = "header", rename = "x-oss-fwd-header-Accept-Ranges")]
    pub fwd_header_accept_ranges: Option<String>,

    /// The HTTP response header returned by the backend server. It is used to
    /// specify the resource cache method that the client uses. Valid values:
    /// no-cache, no-store, public, private, max-age
    #[field(type = "header", rename = "x-oss-fwd-header-Cache-Control")]
    pub fwd_header_cache_control: Option<String>,

    /// The HTTP response header returned by the backend server.
    #[field(type = "header", rename = "x-oss-fwd-header-Content-Disposition")]
    pub fwd_header_content_disposition: Option<String>,

    /// The HTTP response header returned by the backend server.
    #[field(type = "header", rename = "x-oss-fwd-header-Content-Encoding")]
    pub fwd_header_content_encoding: Option<String>,

    /// The HTTP response header returned by the backend server.
    #[field(type = "header", rename = "x-oss-fwd-header-Content-Language")]
    pub fwd_header_content_language: Option<String>,

    /// The HTTP response header returned by the backend server.
    #[field(type = "header", rename = "x-oss-fwd-header-Content-Range")]
    pub fwd_header_content_range: Option<String>,

    /// The HTTP response header returned by the backend server. It is used to
    /// specify the type of the received or sent data.
    #[field(type = "header", rename = "x-oss-fwd-header-Content-Type")]
    pub fwd_header_content_type: Option<String>,

    /// The HTTP response header returned by the backend server. It uniquely
    /// identifies the object.
    #[field(type = "header", rename = "x-oss-fwd-header-ETag")]
    pub fwd_header_etag: Option<String>,

    /// The HTTP response header returned by the backend server. It specifies the
    /// absolute expiration time of the cache.
    #[field(type = "header", rename = "x-oss-fwd-header-Expires")]
    pub fwd_header_expires: Option<String>,

    /// The HTTP response header returned by the backend server. It specifies the
    /// time when the requested resource was last modified.
    #[field(type = "header", rename = "x-oss-fwd-header-Last-Modified")]
    pub fwd_header_last_modified: Option<String>,

    /// The data returned to the client.
    pub body: Option<BodyContent>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct WriteGetObjectResponseResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Customizes return data and response headers for a GetObject request that
    /// is processed by Function Compute. This is a host-level request routed by
    /// the x-oss-request-route and x-oss-request-token headers.
    ///
    /// # Arguments
    ///
    /// * `request` - The `WriteGetObjectResponseRequest` containing the route,
    ///   token, status, response headers and the body data to return.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::WriteGetObjectResponseRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// # use alibabacloud_oss_sdk_rust_v2::BodyContent;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = WriteGetObjectResponseRequest {
    ///     request_route: Some("route-from-fc-event".to_string()),
    ///     request_token: Some("token-from-fc-event".to_string()),
    ///     fwd_status: Some("200".to_string()),
    ///     body: Some(BodyContent::from_text("customized data".to_string(), None)),
    ///     ..Default::default()
    /// };
    ///
    /// match client.write_get_object_response(request).await {
    ///     Ok(result) => {
    ///         println!("Write get object response done: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to write get object response: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn write_get_object_response(
        &self,
        mut request: WriteGetObjectResponseRequest,
    ) -> Result<WriteGetObjectResponseResult, Box<dyn std::error::Error + Send + Sync>> {
        let headers = request.header_map();
        let queries = request.query_map();

        let mut input = OperationInput {
            op_name: "WriteGetObjectResponse".to_string(),
            method: http::Method::POST,
            parameters: [("x-oss-write-get-object-response", "")]
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
            std::rc::Rc::new(vec!["x-oss-write-get-object-response".to_string()]),
        );

        modify_request(
            &mut input,
            headers,
            queries,
            vec![update_content_md5, update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = WriteGetObjectResponseResult::default();
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
    async fn test_write_get_object_response() {
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

        // A valid route and token can only be obtained from a Function Compute
        // event, so the server is expected to reject this call; the call
        // exercises the request path.
        let result = client
            .write_get_object_response(WriteGetObjectResponseRequest {
                request_route: Some("fc-test-route".to_string()),
                request_token: Some("fc-test-token".to_string()),
                fwd_status: Some("200".to_string()),
                body: Some(BodyContent::from_text("test-data".to_string(), None)),
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!(
                "write_get_object_response rejected (route/token require an FC event): {}",
                error
            );
        }
    }
}
