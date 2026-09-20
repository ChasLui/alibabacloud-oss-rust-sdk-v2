use std::rc::Rc;

use super::{try_convert_service_error, ClientOptions, OssResponse};
use crate::{OperationInput, OP_META_KEY_RESPONSE_HANDLER};
use log::debug;

/// Applies the operation options to the base options.
///
/// This function takes the base options and the optional options as input and
/// applies the specified options to the base options. It updates the base
/// options with the values from the optional options, if they are present.
///
/// # Arguments
///
/// * `base_options` - A mutable reference to the base options.
/// * `opt_options` - A reference to the optional options.
pub(super) fn apply_operation_opt(
    base_options: &mut ClientOptions,
    modified_options: &ClientOptions,
) {
    if let Some(endpoint) = &modified_options.endpoint {
        base_options.endpoint = Some(endpoint.clone());
    }

    if let Some(retry_max_attempts) = modified_options.retry_max_attempts {
        if retry_max_attempts > 0 {
            base_options.retry_max_attempts = Some(retry_max_attempts);
        }
    }

    if let Some(retryer) = &modified_options.retryer {
        base_options.retryer = Some(retryer.clone());
    }

    if base_options.retryer.is_none() {
        // Go defaults to `retry.NewStandard()` (3 attempts, with the standard
        // set of retryable-error predicates) rather than disabling retries.
        base_options.retryer = Some(Rc::new(crate::retry::Standard::new()));
    }

    if let Some(op_read_write_timeout) = modified_options.op_read_write_timeout {
        base_options.op_read_write_timeout = Some(op_read_write_timeout);
    }

    if let Some(http_client) = &modified_options.http_client {
        base_options.http_client = Some(http_client.clone());
    }

    if let Some(auth_method) = &modified_options.auth_method {
        base_options.auth_method = Some(auth_method.clone());
    }

    #[allow(clippy::type_complexity)]
    let mut handlers: Vec<
        Rc<dyn Fn(&OssResponse) -> Result<(), Box<dyn std::error::Error + Send + Sync>>>,
    > = Vec::new();
    handlers.push(Rc::new(
        |response: &OssResponse| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            match &response {
                OssResponse::SucResponse(_) => {
                    Ok(())/* resp: &Response */
                }
                OssResponse::ErrResponse {..} => {
                    // debug!("Response failed with status: {}", response.status());
                    try_convert_service_error(response)

                }
            }


            // Only log the error, don't interrupt the flow
            // The actual error handling will be done in send_request based on status code
            // if !response.status().is_success() {
            //     debug!("Response failed with status: {}", response.status());
            //     // crate::client::try_convert_service_error(response);
            // }
            //
            // match response{
            //     SucResponse => {
            //        OK(())
            //     }
            //     ErrResponse => {
            //         try_convert_service_error(response);
            //     }
            // }

            // Ok(())
        },
    ));

    handlers.extend(base_options.response_handlers.iter().cloned());
    handlers.extend(modified_options.response_handlers.iter().cloned());
    base_options.response_handlers = handlers;
}

#[allow(unused)]
pub(crate) fn apply_operation_context() {
    unimplemented!(
        "Set context of OpReadWriteTimeout which is used by the dialer. Since there is no context \
         in Rust, use Config::read_write_timeout instead."
    );
}

/// Applies the operation metadata to the base options.
///
/// This function takes the operation input and the base options as input and
/// applies the response handlers from the operation input to the base options.
/// It adds the response handlers to the existing list of response handlers in
/// the base options.
///
/// # Arguments
///
/// * `input` - A reference to the operation input.
/// * `base_options` - A mutable reference to the base options.
pub(super) fn apply_operation_metadata(input: &OperationInput, base_options: &mut ClientOptions) {
    if let Some(handlers_rc) = input.op_metadata.get(OP_META_KEY_RESPONSE_HANDLER) {
        if let Some(handlers_vec) = handlers_rc.downcast_ref::<Vec<
            Rc<dyn Fn(&OssResponse) -> Result<(), Box<dyn std::error::Error + Send + Sync>>>,
        >>() {
            for handler in handlers_vec {
                base_options.response_handlers.push(handler.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::api::service::ListBucketsRequest;
    use crate::client::Client;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::test_utils::load_test_config;

    #[tokio::test]
    async fn test_service_error_403() {
        match Client::new(
            &Config::default()
                .with_region("cn-hongkong")
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    "wrong ak",
                    "wrong sk",
                    &[],
                )))
                .with_log_level(LogLevel::Debug),
        )
        .list_buckets(&ListBucketsRequest::default())
        .await
        {
            Ok(output) => panic!("Invoke operation should not succeed: {:?}", output),
            Err(err) => println!("Invoke operation failed: {:?}", err),
        }
    }
}