use std::rc::Rc;
use std::sync::Arc;
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use reqwest::Response;

use super::{apply_operation_metadata, apply_operation_opt, Client, ClientOptions, OssResponse};
use crate::credential::AnonymousCredentialsProvider;
use crate::retry::DEFAULT_MAX_ATTEMPTS;
use crate::signer::{SigningContext, SIGN_TIME, SUB_RESOURCE};
use crate::utils::{build_url, header_map_to_hash_map, is_valid_endpoint, sleep_with_context};
use crate::{
    AuthMethodType, BodyStream, BodyTracker, ClientError, HEADER_OSS_DATE, HTTP_HEADER_USER_AGENT,
    OperationInput, OperationMetadata, OperationOutput, ServiceError,
};

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

        // Send request
        let response = self
            .send_http_request(input, options)
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
            // Trackers observe the bytes actually sent, which is how
            // integrity checks (CRC64) see the request body. Registered by
            // `utils::add_crc64_check` under `OP_META_KEY_REQUEST_BODY_TRACKER`.
            let trackers: Vec<Arc<dyn BodyTracker>> = input
                .op_metadata
                .values(crate::OP_META_KEY_REQUEST_BODY_TRACKER)
                .into_iter()
                .flatten()
                .filter_map(|v| v.clone().downcast::<crate::Crc64Tracker>().ok())
                .map(|v| Arc::new((*v).clone()) as Arc<dyn BodyTracker>)
                .collect();

            let body = content.into_reqwest_body_with_trackers(trackers).await?;
            request_builder = request_builder.body(body);
        }

        // Signing context - 在处理完 body 后构建签名上下文
        let sub_resource: Vec<String> = input
            .op_metadata
            .get(SUB_RESOURCE)
            .and_then(|value| value.downcast_ref())
            .cloned()
            .unwrap_or_default();

        let clock_offset = self.inner_options.clock_offset.get();
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
    /// Retries the request according to the configured retryer until it
    /// succeeds, the error is not retryable, or the attempt budget runs out.
    /// The request is rebuilt from `input` for every attempt, so only bodies
    /// that can be replayed (`File`, `Bytes`, `Text`) are retried — an owned
    /// `Stream` is consumed by its first send. Mirrors Go
    /// `Client.sendHttpRequest`.
    ///
    /// # Arguments
    ///
    /// * `&self` - A reference to the current instance of the class.
    /// * `input` - The [OperationInput] containing the details of the
    ///   operation to be performed.
    /// * `options` - An optional reference to [ClientOptions] which contains
    ///   additional options for the client.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the [Response] if the request is
    /// successful, or a `Box<dyn std::error::Error + Send + Sync>` if an
    /// error occurs.
    async fn send_http_request(
        &self,
        input: OperationInput,
        options: Option<&ClientOptions>,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let logger = self.inner_options.logger.as_ref().expect("Logger not set");

        let opts = options.unwrap_or(&self.options);
        let retryer = opts.retryer.clone();
        // A retryer that reports zero attempts (or an unset budget) still has
        // to make one attempt, otherwise nothing is ever sent.
        let max_attempts = self.retry_max_attempts(options).max(1);

        // A body that cannot be replayed gets exactly one attempt: the first
        // send consumes it. Mirrors Go `teeReadNopCloser.IsSeekable`.
        let is_replayable = input
            .body
            .as_ref()
            .map_or(true, |body| body.try_clone().is_some());

        // The original input is consumed by the final attempt; every earlier
        // attempt gets a rebuilt copy.
        let mut input = input;
        let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;

        for attempt in 1..=max_attempts {
            if attempt > 1 {
                let delay = match retryer
                    .as_ref()
                    .expect("Retryer not set")
                    .retry_delay(attempt, last_error.as_deref().expect("Error not set"))
                {
                    Ok(delay) => delay,
                    Err(err) => {
                        logger
                            .warn(format!("Retry aborted while computing delay: {}", err).as_str());
                        break;
                    }
                };

                logger.info(
                    format!("Attempt retry, tries:{}, retry delay:{:?}", attempt, delay).as_str(),
                );
                sleep_with_context(delay).await;
            }

            // A retry resends the body from the start, so anything tracking
            // the bytes sent must start over too. Mirrors Go
            // `teeReadNopCloser.Reset`, which resets its writers alongside the
            // reader position.
            if attempt > 1 {
                if let Some(trackers) = input
                    .op_metadata
                    .values(crate::OP_META_KEY_REQUEST_BODY_TRACKER)
                {
                    for tracker in trackers {
                        if let Ok(tracker) = tracker.clone().downcast::<crate::Crc64Tracker>() {
                            tracker.reset();
                        }
                    }
                }
            }

            // Replayable bodies are rebuilt for every attempt; the final attempt (or a
            // single-shot body) moves the body out, leaving headers and
            // metadata — which carry the request-body trackers — intact.
            let mut attempt_input = input.clone();
            attempt_input.body = if attempt < max_attempts && is_replayable {
                // `OperationInput::clone` drops the body, so restore it.
                input.body.as_ref().and_then(|body| body.try_clone())
            } else {
                input.body.take()
            };

            // Rebuilt per attempt, so each send carries a fresh timestamp and
            // a replayable body.
            let mut signing_ctx = match self.build_signing_context(attempt_input, options).await {
                Ok(signing_ctx) => signing_ctx,
                Err(err) => return Err(err),
            };

            match self.send_http_request_once(&mut signing_ctx, options).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    self.post_send_http_request_once(&mut signing_ctx, options, err.as_ref());

                    let retryable = retryer
                        .as_ref()
                        .expect("Retryer not set")
                        .is_error_retryable(err.as_ref());

                    if !is_replayable || !retryable {
                        return Err(err);
                    }

                    last_error = Some(err);
                }
            }
        }

        Err(last_error.expect("retry attempts exhausted without an error"))
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
    pub(crate) fn retry_max_attempts(&self, options: Option<&ClientOptions>) -> u32 {
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
        signing_ctx: &mut SigningContext,
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

        self.sign_request(signing_ctx, options).await?;

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

        let request = signing_ctx
            .request
            .take()
            .expect("Unable to clone request");
        let request_headers = request.headers().clone();

        let response = opts
            .http_client
            .as_ref()
            .expect("Client not set")
            .execute(request)
            .await?;

        // OSS reports a failed callback as HTTP 203 (Non-Authoritative
        // Information), which is not an error status under `is_success()`, so
        // the callback case has to be detected explicitly. Mirrors Go
        // `callbackErrorResponseHandler`.
        let is_error = !response.status().is_success()
            || is_callback_error(response.status(), &request_headers);

        if !is_error {
            let ossRes = OssResponse::SucResponse(&response);

            for handler in &opts.response_handlers {
                handler(&ossRes)?;
            }

            Ok(response)
        } else {
            logger.debug(format!("send_http_request_once::response:\n{:?}", &response).as_str());
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

    /// Learns the clock correction from a `RequestTimeTooSkewed` failure so the
    /// next attempt is signed with a corrected timestamp.
    ///
    /// Compares the server timestamp carried by the error against the exact
    /// time the failed request was signed with, and stores the difference in
    /// the client so later requests inherit it. Mirrors Go
    /// `Client.postSendHttpRequestOnce`.
    fn post_send_http_request_once(
        &self,
        signing_ctx: &mut SigningContext,
        options: Option<&ClientOptions>,
        err: &(dyn std::error::Error + 'static),
    ) {
        let logger = self.inner_options.logger.as_ref().expect("Logger not set");

        let Some(service_error) = err.downcast_ref::<ServiceError>() else {
            return;
        };

        let opts = options.unwrap_or(&self.options);
        let correct_clock_skew = opts
            .feature_flags
            .contains(crate::FeatureFlagsType::CORRECT_CLOCK_SKEW);
        let Some(server_time) = service_error.timestamp else {
            return;
        };

        if correct_clock_skew && service_error.code == "RequestTimeTooSkewed" {
            // The signer stamped the request with the offset it had at the
            // time; the correction is measured against that same instant.
            let sign_time = signing_ctx
                .time
                .unwrap_or_else(|| crate::signer::now_with_offset(signing_ctx.clock_offset));
            let offset = DateTime::<Utc>::from(server_time) - DateTime::<Utc>::from(sign_time);

            signing_ctx.clock_offset = offset;
            self.inner_options.clock_offset.set(offset);

            logger.warn(
                format!(
                    "Got RequestTimeTooSkewed error, correct clock, ClockOffset:{:?}, Server \
                     Time:{:?}, Client time:{:?}",
                    offset, server_time, sign_time
                )
                .as_str(),
            );
        }
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
    use bytes::Bytes;
    use std::time::Duration;
    use crate::BodyContent;

    /// A V4-signed request carrying a `Bytes` body must be retried on a
    /// retryable server error, and the replayable body must be resent intact
    /// on the retry. `500` twice then `200` proves both: the request reached
    /// the server three times and the final response is a success.
    #[tokio::test]
    async fn test_send_http_request_retries_5xx_and_replays_body() {
        let mut server = mockito::Server::new_async().await;

        // mockito matches the mock with remaining expected hits first, so the
        // single-fire 500 mocks are consumed before the 200 one is used.
        let fail = server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(500)
            .with_body("<Error><Code>InternalError</Code></Error>")
            .expect(2)
            .create_async()
            .await;

        let ok = server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let client = Client::new(
            &Config::default()
                .with_endpoint(server.url().as_str())
                .with_region("cn-hangzhou")
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    "test-ak", "test-sk", &[],
                )))
                .with_signature_version(SignatureVersionType::V1)
                .with_log_level(LogLevel::Off)
                .with_retryer(Rc::new(crate::retry::Standard::new().with_backoff(
                    // Deterministic, zero delay keeps the test fast.
                    Box::new(crate::retry::FixedDelayBackoff::new(Duration::ZERO)),
                ))),
        );

        let input = OperationInput {
            op_name: "PutObject".to_string(),
            method: http::Method::PUT,
            bucket: Some("test-bucket".to_string()),
            key: Some("retry-object".to_string()),
            body: Some(BodyContent::from_bytes(Bytes::from_static(b"payload"), None)),
            ..Default::default()
        };

        let output = client
            .invoke_operation_inner(input, vec![])
            .await
            .expect("retry should turn the 500s into a success");

        assert!(output.status.is_success());
        fail.assert_async().await;
        ok.assert_async().await;
    }

    /// A server error whose status is not retryable must fail after exactly
    /// one attempt, not consume the retry budget.
    #[tokio::test]
    async fn test_send_http_request_does_not_retry_4xx() {
        let mut server = mockito::Server::new_async().await;

        let not_found = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(404)
            .with_body("<Error><Code>NoSuchKey</Code></Error>")
            .expect(1)
            .create_async()
            .await;

        let client = Client::new(
            &Config::default()
                .with_endpoint(server.url().as_str())
                .with_region("cn-hangzhou")
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    "test-ak", "test-sk", &[],
                )))
                .with_signature_version(SignatureVersionType::V1)
                .with_log_level(LogLevel::Off)
                .with_retryer(Rc::new(crate::retry::Standard::new().with_backoff(
                    Box::new(crate::retry::FixedDelayBackoff::new(Duration::ZERO)),
                ))),
        );

        let input = OperationInput {
            op_name: "GetObject".to_string(),
            method: http::Method::GET,
            bucket: Some("test-bucket".to_string()),
            key: Some("missing-object".to_string()),
            ..Default::default()
        };

        // `invoke_operation_inner` returns the error, never an error response.
        let err = client
            .invoke_operation_inner(input, vec![])
            .await
            .expect_err("404 must be reported as an error");

        assert_eq!(
            err.downcast_ref::<ServiceError>().map(|e| e.code.as_str()),
            Some("NoSuchKey"),
            "unexpected error: {}",
            err
        );
        not_found.assert_async().await;
    }

    /// A `RequestTimeTooSkewed` response makes the client learn the server
    /// clock offset, and the retry then succeeds: the corrected timestamp is
    /// what turns the skew rejection into a success.
    #[tokio::test]
    async fn test_send_http_request_corrects_clock_skew() {
        let mut server = mockito::Server::new_async().await;

        // The server reports its clock one minute ahead of the client; that
        // `Date` header is what the correction is derived from.
        let server_time = SystemTime::now() + Duration::from_secs(60);
        let server_date = DateTime::<Utc>::from(server_time).to_rfc2822();

        let skew = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(403)
            .with_header("Date", server_date.as_str())
            .with_body(
                "<Error><Code>RequestTimeTooSkewed</Code><Message>skewed</Message></Error>",
            )
            .expect(1)
            .create_async()
            .await;

        let ok = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let client = Client::new(
            &Config::default()
                .with_endpoint(server.url().as_str())
                .with_region("cn-hangzhou")
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    "test-ak", "test-sk", &[],
                )))
                .with_signature_version(SignatureVersionType::V1)
                .with_log_level(LogLevel::Off)
                .with_retryer(Rc::new(crate::retry::Standard::new().with_backoff(
                    Box::new(crate::retry::FixedDelayBackoff::new(Duration::ZERO)),
                ))),
        );

        let input = OperationInput {
            op_name: "GetBucketInfo".to_string(),
            method: http::Method::GET,
            bucket: Some("test-bucket".to_string()),
            ..Default::default()
        };

        // Bracket the request in time: the signer stamps the request with the
        // local clock at send time, so the learned offset is
        // `server_time - sign_time`, which drifts by however long the suite
        // takes between setup and the request. Asserting against the actual
        // bracket keeps the check tight without assuming instant setup.
        let before = SystemTime::now();
        let output = client
            .invoke_operation_inner(input, vec![])
            .await
            .expect("clock-skewed request should be corrected and retried");
        let after = SystemTime::now();

        assert!(output.status.is_success());
        skew.assert_async().await;
        ok.assert_async().await;

        let offset = client.inner_options.clock_offset.get();
        // `Date` carries second precision, so allow one second of slack at
        // each end of the bracket.
        let upper = DateTime::<Utc>::from(server_time) - DateTime::<Utc>::from(before);
        let lower = DateTime::<Utc>::from(server_time) - DateTime::<Utc>::from(after);
        let slack = chrono::TimeDelta::seconds(1);

        assert!(
            offset <= upper + slack && offset >= lower - slack,
            "expected an offset between {:?} and {:?}, got {:?}",
            lower,
            upper,
            offset
        );
    }

    /// A CRC64 mismatch reported by the server must surface as an error, and
    /// the message must be the one `ConnectionErrorRetryable` recognises —
    /// that is what lets a corrupted upload be retried instead of silently
    /// accepted.
#[tokio::test]
async fn test_put_object_crc64_mismatch_is_rejected() {
    let mut server = mockito::Server::new_async().await;

    // 0 is never the CRC64 of "payload", so the server value is a mismatch.
    let mismatch = server
        .mock("PUT", mockito::Matcher::Any)
        .with_status(200)
        .with_header("x-oss-hash-crc64ecma", "0")
        .with_body("")
        .create_async()
        .await;

    let client = Client::new(
        &Config::default()
            .with_endpoint(server.url().as_str())
            .with_region("cn-hangzhou")
            .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                "test-ak", "test-sk", &[],
            )))
            .with_signature_version(SignatureVersionType::V1)
            .with_log_level(LogLevel::Off)
            .with_retryer(Rc::new(crate::retry::NopRetryer::new())),
    );

    let mut input = OperationInput {
        op_name: "PutObject".to_string(),
        method: http::Method::PUT,
        bucket: Some("test-bucket".to_string()),
        key: Some("crc-object".to_string()),
        body: Some(BodyContent::from_bytes(Bytes::from_static(b"payload"), None)),
        ..Default::default()
    };

    // The check is attached the same way PutObject does it.
    crate::utils::add_crc64_check(
        &mut input,
        0,
        true,
    );

    let err = client
        .invoke_operation_inner(input, vec![])
        .await
        .expect_err("a CRC mismatch must fail the operation");

    assert!(
        err.to_string().contains("crc is inconsistent"),
        "unexpected error: {}",
        err
    );
    mismatch.assert_async().await;
}

/// A matching CRC64 must leave the operation successful.
#[tokio::test]
async fn test_put_object_crc64_match_succeeds() {
    let mut server = mockito::Server::new_async().await;

    // The CRC64 of "payload", which is what the client computes.
    let expected = {
        let mut crc = crate::utils::Crc64::new(0);
        crc.write(b"payload").unwrap();
        crc.sum64().to_string()
    };

    let ok = server
        .mock("PUT", mockito::Matcher::Any)
        .with_status(200)
        .with_header("x-oss-hash-crc64ecma", expected.as_str())
        .with_body("")
        .create_async()
        .await;

    let client = Client::new(
        &Config::default()
            .with_endpoint(server.url().as_str())
            .with_region("cn-hangzhou")
            .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                "test-ak", "test-sk", &[],
            )))
            .with_signature_version(SignatureVersionType::V1)
            .with_log_level(LogLevel::Off)
            .with_retryer(Rc::new(crate::retry::NopRetryer::new())),
    );

    let mut input = OperationInput {
        op_name: "PutObject".to_string(),
        method: http::Method::PUT,
        bucket: Some("test-bucket".to_string()),
        key: Some("crc-object".to_string()),
        body: Some(BodyContent::from_bytes(Bytes::from_static(b"payload"), None)),
        ..Default::default()
    };

    crate::utils::add_crc64_check(&mut input, 0, true);

    let output = client
        .invoke_operation_inner(input, vec![])
        .await
        .expect("a matching CRC must succeed");

    assert!(output.status.is_success());
    ok.assert_async().await;
}

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