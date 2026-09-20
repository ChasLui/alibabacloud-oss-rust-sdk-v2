use std::rc::Rc;
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use reqwest::Response;

use super::{apply_operation_metadata, apply_operation_opt, Client, ClientOptions, OssResponse};
use crate::credential::AnonymousCredentialsProvider;
use crate::retry::DEFAULT_MAX_ATTEMPTS;
use crate::signer::{SigningContext, SIGN_TIME, SUB_RESOURCE};
use crate::utils::{build_url, header_map_to_hash_map, is_valid_endpoint, sleep_with_context};
use crate::{AuthMethodType, OperationInput, OperationMetadata, OperationOutput, ServiceError, ClientError, HEADER_OSS_DATE, HTTP_HEADER_USER_AGENT, BodyStream, BodyContent};

impl Client {
    /// Invokes an operation on the Alibaba Cloud OSS service.
    ///
    /// This method takes an [OperationInput] object and a vector of closure
    /// functions that modify the [ClientOptions] for the operation. It
    /// sends an HTTP request to the OSS service with the specified input
    /// and options, and returns the operation output or an error.
    ///
    /// # Arguments
    ///
    /// * `input` - The [OperationInput] object containing the input parameters
    ///   for the operation.
    /// * `option_modifiers` - A vector of closure functions that modify the
    ///   [ClientOptions] for the operation.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the [OperationOutput] if the operation is
    /// successful, or a `Box<dyn std::error::Error + Send + Sync>` if an
    /// error occurs.
    #[allow(clippy::type_complexity)]
    pub(crate) async fn invoke_operation_inner(
        &self,
        input: OperationInput,
        option_modifiers: Vec<fn(&mut ClientOptions)>,
    ) -> Result<OperationOutput, Box<dyn std::error::Error + Send + Sync>> {
        let logger = self.inner_options.logger.as_ref().expect("Logger not set");

        // 提前获取需要在后面使用的值，避免在移动input后访问
        let input_op_name = input.op_name.clone();
        let input_bucket = input.bucket.clone();
        let input_key = input.key.clone();

        logger.info(
            format!(
                "InvokeOperation Start\ninput: {:#?}\nOpName: {}\nBucket: {:#?}\nKey: {:#?}",
                input, input_op_name, input_bucket, input_key
            )
            .as_str(),
        );

        let mut options = self.options.clone();
        let mut modified_options = ClientOptions::default();

        for option_modifier in option_modifiers {
            option_modifier(&mut modified_options);
        }

        apply_operation_opt(&mut options, &modified_options);
        apply_operation_metadata(&input, &mut options);  // 使用 &input 而不是 input
        // apply_operation_context()

        let request_result = self.send_request(input, Some(&options)).await;  // 传递所有权给send_request

        logger.info(
            format!(
                "InvokeOperation End\ninput_op_name: {}\ninput_bucket: {:#?}\ninput_key: {:#?}\nResult<Output, Err>: {:#?}",
                input_op_name,  // 使用预先保存的值
                input_bucket,   // 使用预先保存的值
                input_key,      // 使用预先保存的值
                request_result.as_ref()
            )
            .as_str(),
        );

        request_result
    }

    /// Asynchronously sends a request to a specified endpoint.
    ///
    /// # Arguments
    ///
    /// * `&self` - A reference to the current instance of the class.
    /// * `input` - A reference to an instance of [OperationInput] which
    ///   contains the details of the operation to be performed.
    /// * `options` - An optional reference to [ClientOptions] which contains
    ///   additional options for the client.
    async fn send_request(
        &self,
        input: OperationInput,  // 接收所有权
        options: Option<&ClientOptions>,
    ) -> Result<OperationOutput, Box<dyn std::error::Error + Send + Sync>> {
        let logger = self.inner_options.logger.as_ref().expect("Logger not set");

        // 提前获取需要在后面使用的值，避免在移动input后访问
        let input_op_name = input.op_name.clone();
        let input_bucket = input.bucket.clone();
        let input_key = input.key.clone();
        let input_clone = input.clone(); // 创建完整副本用于最终的OperationOutput

        logger.info(format!("sendRequest Start:\ninput: {:#?}", &input).as_str());  // 使用引用

        let signing_context = self.build_signing_context(input, options).await?;


        // Send request
        let response = self
            .send_http_request(signing_context, options)
            .await?;

        logger.info(
            format!(
                "sendRequest End:\ninput_op_name: {}\ninput_bucket: {:#?}\ninput_key: {:#?}\nresponse: {:#?}",
                input_op_name,  // 使用之前保存的值
                input_bucket,   // 使用之前保存的值
                input_key,      // 使用之前保存的值
                &response
            )
            .as_str(),
        );

        let status = response.status();
        
        // Clone headers before consuming the response for error handling
        let headers = header_map_to_hash_map(response.headers());
        // let request_clone = request.try_clone().expect("Unable to clone request");
        
        if status.is_success() {
            let body = Some(Box::pin(response.bytes_stream()) as BodyStream);
            Ok(OperationOutput {
                input: Some(Rc::new(input_clone)),  // 使用之前克隆的完整input
                status,
                headers,
                body,
                // http_request: Some(Rc::new(request_clone)),
                op_metadata: OperationMetadata::default(),
                // body_data: None, // 添加这一行
            })
        } else {
            //will not go to here
            Err("It will not go to here! All err status should return corresponding ServiceErr。Status must be success at this point。".into())
        }
    }

    /// Builds the signing context for an operation input: validates client
    /// options and input parameters, builds the request URL, applies headers
    /// and body, and constructs the [SigningContext].
    ///
    /// This neither signs the request nor sends it, so it can be reused by
    /// both the normal send path and the presign path.
    ///
    /// # Arguments
    ///
    /// * `input` - The [OperationInput] object containing the input parameters
    ///   for the operation. Ownership is consumed (the body is moved into the
    ///   built request).
    /// * `options` - An optional reference to [ClientOptions] which contains
    ///   additional options for the client.
    pub(super) async fn build_signing_context(
        &self,
        input: OperationInput,
        options: Option<&ClientOptions>,
    ) -> Result<SigningContext, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        // 提前获取需要在后面使用的值，避免在移动input后访问
        let input_bucket = input.bucket.clone();
        let input_key = input.key.clone();

        // Validate client options and input parameters to catch client errors early

        // Check for invalid retry_max_attempts
        if let Some(max_attempts) = options.and_then(|o| o.retry_max_attempts) {
            if max_attempts <= 0 {
                let client_error = ClientError {
                    code: "InvalidParameter".to_string(),
                    message: "retry_max_attempts must be greater than zero".to_string(),
                    err: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "retry_max_attempts must be greater than zero")),
                };
                return Err(Box::new(client_error));
            }
        }

        // Validate input parameters early to catch client errors
        if let Some(bucket) = &input.bucket {
            if bucket.is_empty() {
                let client_error = ClientError {
                    code: "InvalidParameter".to_string(),
                    message: "bucket parameter cannot be empty".to_string(),
                    err: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "bucket parameter cannot be empty")),
                };
                return Err(Box::new(client_error));
            }
        }

        if let Some(key) = &input.key {
            if key.is_empty() {
                let client_error = ClientError {
                    code: "InvalidParameter".to_string(),
                    message: "key parameter cannot be empty".to_string(),
                    err: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "key parameter cannot be empty")),
                };
                return Err(Box::new(client_error));
            }
        }

        // Endpoint validation
        let endpoint_option = options.and_then(|options| options.endpoint.as_ref());
        if endpoint_option.is_none() {
            let client_error = ClientError {
                code: "InvalidConfiguration".to_string(),
                message: "endpoint is not set".to_string(),
                err: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "endpoint is not set")),
            };
            return Err(Box::new(client_error));
        }

        let endpoint = endpoint_option.unwrap();
        if !is_valid_endpoint(endpoint.as_str()) {
            let client_error = ClientError {
                code: "InvalidConfiguration".to_string(),
                message: format!("endpoint {} is invalid", endpoint),
                err: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("endpoint {} is invalid", endpoint))),
            };
            return Err(Box::new(client_error));
        }

        // Region validation - only required for V4 signature version
        let region = &options.as_ref().expect("Options not set").region;

        // Determine if we're using V4 signature which requires region by checking the signer type
        let current_signer = options
            .and_then(|opt| opt.signer.as_ref())
            .or(self.options.signer.as_ref());

        // Check if the current signer is a V4 signer
        let is_v4_signer = current_signer.map_or(false, |signer| {
            // `type_id()` on `&dyn Signer` returns the trait object's own TypeId
            // (std blanket impl), so it can never match SignerV4; use as_any() instead.
            signer.as_ref().as_any().is::<crate::signer::v4::SignerV4>()
        });

        // Only validate region if we're using V4 signature
        if is_v4_signer && region.is_empty() {
            let client_error = ClientError {
                code: "InvalidConfiguration".to_string(),
                message: "region is not set (required for V4 signature)".to_string(),
                err: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "region is not set (required for V4 signature)")),
            };
            return Err(Box::new(client_error));
        }

        // 为了避免部分移动，在创建 URL 前先提取需要的值
        let method = input.method.clone();  // 复制或克隆 method
        let (host, path) = build_url(&input, options.as_ref().expect("Options not set"));  // 使用引用
        let mut url = format!("{}://{}{}", endpoint.scheme(), host, path);

        // Queries
        if !input.parameters.is_empty() {
            url = format!(
                "{}?{}",
                url,
                input
                    .parameters
                    .iter()
                    .map(|(k, v)| if !v.is_empty() {
                        format!("{}={}", k, v)
                    } else {
                        k.to_string()
                    })
                    .collect::<Vec<String>>()
                    .join("&")
            );
        }

        // New request
        let mut request_builder = client.request(method, &url);  // 使用之前提取的 method

        // Headers
        for (k, v) in &input.headers {  // 使用引用
            request_builder = request_builder.header(k, v);
        }
        request_builder =
            request_builder.header(HTTP_HEADER_USER_AGENT, &self.inner_options.user_agent);

        // Body
        let body_content = input.body;  // 移动 body_content

        if let Some(content) = body_content {  // 移动 content
            let body = content.into_reqwest_body().await?;
            request_builder = request_builder.body(body);
        }

        // Signing context - 在处理完 body 后构建签名上下文
        let sub_resource: Vec<String> = input
            .op_metadata
            .get(SUB_RESOURCE)
            .and_then(|value| value.downcast_ref())
            .cloned()
            .unwrap_or_default();

        let clock_offset = self.inner_options.clock_offset;
        let request = request_builder.build().expect("Unable to build request");

        let sign_time = if let Some(date_str) = request
            .headers()
            .get(HEADER_OSS_DATE)
            .map(|v| v.to_str().expect("Invalid header value"))
        {
            let datetime: DateTime<Utc> = date_str.parse().expect("Invalid date string");
            Some(datetime.into())
        } else if let Some(sign_time) = input.op_metadata.get(SIGN_TIME) {
            Some(
                *sign_time
                    .downcast_ref::<SystemTime>()
                    .expect("Invalid sign time"),
            )
        } else {
            None // 显式处理未匹配情况
        };


        let mut signing_context = SigningContext {
            product: Some(options.expect("Options not set").product.clone()),
            region: Some(options.expect("Options not set").region.clone()),
            bucket: input_bucket.clone(),  // 使用克隆的值
            key: input_key.clone(),  // 使用克隆的值
            request: Some(request),
            sub_resource: sub_resource.clone(),
            auth_method_query: options
                .and_then(|opts| opts.auth_method.as_ref())
                .map_or(false, |auth_method| auth_method.eq(&AuthMethodType::Query)),
            clock_offset,
            additional_headers: options.expect("Options not set").additional_headers.clone(),

            ..Default::default()
        };

        signing_context.time = sign_time;

        Ok(signing_context)
    }

    /// Asynchronously sends an HTTP request to the specified endpoint.
    ///
    /// # Arguments
    ///
    /// * `signing_ctx` - A mutable reference to a [SigningContext] object that
    ///   contains the signing context for the request.
    /// * `options` - An optional reference to [ClientOptions] object that
    ///   contains additional options for the client.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the [Response] if the request is
    /// successful, or a `Box<dyn std::error::Error + Send + Sync>` if an
    /// error occurs.
    async fn send_http_request(
        &self,
        signing_ctx: SigningContext,
        options: Option<&ClientOptions>,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        self.send_http_request_once(signing_ctx, options).await
    }

    /// Determines the maximum number of retry attempts for a request.
    ///
    /// # Arguments
    ///
    /// * `&self` - A reference to the current instance of the class.
    /// * `options` - An optional reference to [ClientOptions] which contains
    ///   additional options for the client.
    ///
    /// # Returns
    ///
    /// * `u32` - The maximum number of retry attempts. This is determined by
    ///   the `retry_max_attempts` field in the provided options, the
    ///   `max_attempts` method of the `retryer` in the provided options, or the
    ///   [DEFAULT_MAX_ATTEMPTS] constant, in that order.
    fn retry_max_attempts(&self, options: Option<&ClientOptions>) -> u32 {
        // Use the provided options if available, otherwise default to the client's
        // options
        let options = options.unwrap_or(&self.options);

        if let Some(retry_max_attempts) = options.retry_max_attempts {
            if retry_max_attempts <= 0 {
                return DEFAULT_MAX_ATTEMPTS; // Use default if set to 0 or negative
            }
            retry_max_attempts
        } else if let Some(ref retryer) = options.retryer {
            retryer.max_attempts()
        } else {
            DEFAULT_MAX_ATTEMPTS
        }
    }

    /// Sends an HTTP request once to a specified endpoint, signing the request
    /// if necessary.
    ///
    /// # Arguments
    ///
    /// * `&self` - A reference to the current instance of the class.
    /// * [signing_ctx](file:///Users/zhouao/codespace/aliyun-oss-sdk-rust-v2/oss/src/signer.rs#L25-L58) - A mutable reference to [SigningContext] which contains
    ///   the details of the signing context.
    /// * `options` - An optional reference to [ClientOptions] which contains
    ///   additional options for the client.
    async fn send_http_request_once(
        &self,
        mut signing_ctx: SigningContext,
        options: Option<&ClientOptions>,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let logger = self.inner_options.logger.as_ref().expect("Logger not set");

        let opts = options.unwrap_or(&self.options);

        logger.info(
            format!(
                "send_http_request_once begins:\nrequest: {:#?}",
                signing_ctx.request.as_ref().expect("Request not set")
            )
            .as_str(),
        );

        self.sign_request(&mut signing_ctx, options).await?;

        // Log HTTP request
        // logger.debug(
        //     format!(
        //         "send_http_request_once::request:\n{:?}",
        //         &signing_ctx.request.as_ref().expect("Request not set")
        //     )
        //     .as_str(),
        // );

        // Log HTTP URL
        logger.debug(
            format!(
                "send_http_request_once::url:\n{:?}",
                &signing_ctx
                    .request
                    .as_ref()
                    .expect("Request not set")
                    .url()
                    .as_str()
            )
            .as_str(),
        );

        // Send HTTP request. Capture the request headers before the request is
        // moved into the client: OSS reports a failed callback as HTTP 203
        // (Non-Authoritative Information), which is not an error status under
        // `is_success()`, so the callback case has to be detected explicitly.
        // Mirrors Go `callbackErrorResponseHandler`.
        let request = signing_ctx
            .request
            .expect("Unable to clone request");
        let request_headers = request.headers().clone();

        let response = opts
            .http_client
            .as_ref()
            .expect("Client not set")
            .execute(request)
            .await?;

        if !is_callback_error(response.status(), &request_headers) {
            let ossRes = OssResponse::SucResponse(&response);

            for handler in &opts.response_handlers {
                handler(&ossRes)?;
            }

            Ok(response)
        } else {
            logger.debug(format!("send_http_request_once::response:\n{:?}", &response).as_str());
            // let ossRes = OssResponse::from_response_and_context(response).await?;
            let status = response.status();
            let headers = response.headers().clone();
            let url = response.url().to_string();

            let body = response.text().await?;
            let ossRes2 = OssResponse::ErrResponse {
                status,
                body,
                headers,
                url,
            };

            // Response handlers
            for handler in &opts.response_handlers {
                handler(&ossRes2)?;
            }

            unreachable!("response handlers must convert error responses into a ServiceError")
        }
    }

    /// Signs the request in the given signing context using the configured
    /// credentials provider and signer.
    ///
    /// Anonymous credentials skip signing entirely. Extracted from
    /// `send_http_request_once` so the presign path can sign without sending.
    ///
    /// # Arguments
    ///
    /// * `signing_ctx` - A mutable reference to the [SigningContext] to sign.
    /// * `options` - An optional reference to [ClientOptions] which contains
    ///   the credentials provider and signer.
    pub(super) async fn sign_request(
        &self,
        signing_ctx: &mut SigningContext,
        options: Option<&ClientOptions>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let logger = self.inner_options.logger.as_ref().expect("Logger not set");

        let opts = options.unwrap_or(&self.options);

        // Check credential provider
        if let Some(credentials_provider) = &opts.credentials_provider {
            if !credentials_provider.as_any().is::<AnonymousCredentialsProvider>() {
                let cred = credentials_provider.get_credentials().await?;
                signing_ctx.credentials = Some(cred);

                opts.signer
                    .as_ref()
                    .expect("Signer not set")
                    .sign(signing_ctx)?;
                logger.debug(
                    format!(
                        "sign_request::sign:\nsigning_ctx: {:#?}",
                        signing_ctx
                    )
                    .as_str(),
                );
            }
        }

        Ok(())
    }

    #[allow(unused_variables)]
    fn post_send_http_request_once(
        &self,
        signing_ctx: &mut SigningContext,
        err: &(dyn std::error::Error + 'static),
    ) {
        // TODO require casting to concrete error type
    }
}

/// OSS reports a failed callback as HTTP 203 (Non-Authoritative
/// Information). Because 203 is a 2xx status, it would otherwise be treated
/// as success and the error body swallowed. Mirrors Go
/// `callbackErrorResponseHandler`, which only converts the response when the
/// request carried an `x-oss-callback` header.
fn is_callback_error(status: http::StatusCode, request_headers: &http::HeaderMap) -> bool {
    status == http::StatusCode::NON_AUTHORITATIVE_INFORMATION
        && request_headers.contains_key(crate::HEADER_OSS_CALLBACK)
}

// Helper function to convert HashMap to HeaderMap
fn convert_hashmap_to_headermap(
    hashmap: std::collections::HashMap<String, String>,
) -> Result<http::HeaderMap, Box<dyn std::error::Error + Send + Sync>> {
    use http::HeaderMap;
    let mut header_map = HeaderMap::new();

    for (key, value) in hashmap {
        if let (Ok(header_name), Ok(header_value)) = (
            http::HeaderName::from_bytes(key.as_bytes()),
            http::HeaderValue::from_str(&value)
        ) {
            header_map.insert(header_name, header_value);
        }
    }

    Ok(header_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::{SignatureVersionType, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};
    use crate::test_utils::{load_test_config, TestConfig};

    /// 203 counts as success for `is_success()`, so only a callback request
    /// may treat it as an error; a callback-less 203 stays a success response.
    #[test]
    fn test_callback_error_detection() {
        let with_callback = {
            let mut h = http::HeaderMap::new();
            h.insert(
                http::HeaderName::from_static("x-oss-callback"),
                http::HeaderValue::from_static("eyJjYWxsYmFja0JvZHkiOiAidGVzdCJ9"),
            );
            h
        };
        let without_callback = http::HeaderMap::new();

        assert!(is_callback_error(
            http::StatusCode::NON_AUTHORITATIVE_INFORMATION,
            &with_callback
        ));
        assert!(!is_callback_error(
            http::StatusCode::NON_AUTHORITATIVE_INFORMATION,
            &without_callback
        ));
        assert!(!is_callback_error(http::StatusCode::OK, &with_callback));
        assert!(!is_callback_error(
            http::StatusCode::NOT_FOUND,
            &with_callback
        ));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_invoke_get_bucket_operation() {
        let config = match load_test_config() {
            Some(cfg) => cfg,
            None => {
                eprintln!("Test configuration not found. Skipping test.");
                return;
            }
        };

        let input = OperationInput {
            op_name: "GetBucketInfo".to_string(),
            method: http::Method::GET,
            bucket: Some(config.bucket.to_string()),
            body: None,
            parameters: [("bucketInfo".to_string(), "".to_string())]
                .iter()
                .cloned()
                .collect(),
            headers: [(
                HTTP_HEADER_CONTENT_TYPE.to_string(),
                DEFAULT_CONTENT_TYPE.to_string(),
            )]
            .iter()
            .cloned()
            .collect(),
            ..Default::default()
        };

        if let Ok(output) = Client::new(
            &Config::default()
                .with_region(&config.region)
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    &config.access_key_id,
                    &config.access_key_secret,
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V4)
                .with_log_level(LogLevel::Debug),
        )
        .invoke_operation_inner(input, vec![])  // 移除 & 符号
        .await
        {
            assert!(output.status.is_success());
        } else {
            panic!("Invoke operation failed");
        }
    }

    #[tokio::test]
    async fn test_build_signing_context_v4_requires_region() {
        // Regression: is_v4_signer used to compare TypeIds obtained from
        // `&dyn Signer`, which is the trait object's own TypeId — the check
        // never fired. Now a V4 signer with an empty region must be rejected
        // before any request is built.
        let client = Client::new(
            &Config::default()
                .with_endpoint("https://oss-cn-hangzhou.aliyuncs.com")
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    "test-ak",
                    "test-sk",
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V4),
        );
        assert!(client.options.region.is_empty());

        let input = OperationInput {
            op_name: "GetBucketInfo".to_string(),
            method: http::Method::GET,
            bucket: Some("my-bucket".to_string()),
            ..Default::default()
        };

        let err = client
            .build_signing_context(input, Some(&client.options))
            .await
            .expect_err("V4 signer without region must fail");
        assert!(
            err.to_string().contains("region is not set"),
            "unexpected error: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_sign_request_skips_anonymous_provider() {
        // Regression: anonymous detection used `Rc<dyn CredentialsProvider>::type_id()`,
        // which is the trait object's TypeId — anonymous requests were still signed
        // and failed with "Credentials is null or empty" under V4.
        let client = Client::new(
            &Config::default()
                .with_region("cn-hangzhou")
                .with_credentials_provider(Rc::new(AnonymousCredentialsProvider::new()))
                .with_signature_version(SignatureVersionType::V4),
        );

        let mut ctx = SigningContext::default();
        client
            .sign_request(&mut ctx, None)
            .await
            .expect("anonymous provider must skip signing");
        assert!(ctx.credentials.is_none());
        assert!(ctx.signed_headers.is_empty());
    }
}