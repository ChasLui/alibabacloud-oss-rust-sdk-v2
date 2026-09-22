use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct AbortMultipartUploadRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// The upload ID of the multipart upload to abort.
    #[field(type = "query", rename = "uploadId")]
    pub upload_id: String,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct AbortMultipartUploadResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Aborts a multipart upload and deletes the corresponding part data.
    ///
    /// This method sends a DELETE request with the upload ID parameter to
    /// cancel a multipart upload event and delete the associated part data.
    /// This operation is useful when you want to cancel an in-progress
    /// multipart upload and clean up the uploaded parts to avoid storage
    /// charges.
    ///
    /// # Arguments
    ///
    /// * `request` - The `AbortMultipartUploadRequest` containing the necessary
    ///   information for aborting the multipart upload, including bucket, key,
    ///   and the upload ID to abort.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `AbortMultipartUploadResult` if the
    /// operation is successful, or an error if it fails. Note that if the
    /// upload has already been completed, this operation will return a
    /// NoSuchUpload error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::AbortMultipartUploadRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = AbortMultipartUploadRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     upload_id: "upload-id-from-initiate-multipart-upload".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.abort_multipart_upload(&request).await {
    ///     Ok(result) => {
    ///         println!("Multipart upload aborted successfully");
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to abort multipart upload: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn abort_multipart_upload(
        &self,
        request: &AbortMultipartUploadRequest,
    ) -> Result<AbortMultipartUploadResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "AbortMultipartUpload".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("uploadId", request.upload_id.clone())]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = AbortMultipartUploadResult::default();

        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::object::InitiateMultipartUploadRequest;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::test_utils::{generate_unique_object_name, load_test_config};
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_abort_multipart_upload_basic() {
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
                .with_signature_version(SignatureVersionType::V4)
                .with_log_level(LogLevel::Debug),
        );

        // Generate a unique object name for this test
        let object_name = generate_unique_object_name("abort-multipart");

        // First, initiate a multipart upload to get an upload ID
        let initiate_request = InitiateMultipartUploadRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            ..Default::default()
        };

        let initiate_result = match client.initiate_multipart_upload(&initiate_request).await {
            Ok(result) => {
                println!("Multipart upload initiated: {:?}", result.upload_id);
                result
            }
            Err(err) => panic!("Initiate multipart upload failed: {:?}", err),
        };

        let upload_id = initiate_result.upload_id.unwrap();

        // Now abort the multipart upload
        let abort_request = AbortMultipartUploadRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            upload_id: upload_id.clone(),
            ..Default::default()
        };

        match client.abort_multipart_upload(&abort_request).await {
            Ok(result) => {
                println!("Multipart upload aborted successfully: {:?}", result);
                println!("Upload ID {} has been aborted", upload_id);
            }
            Err(err) => panic!("Abort multipart upload failed: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_abort_multipart_upload_nonexistent() {
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
                .with_signature_version(SignatureVersionType::V4)
                .with_log_level(LogLevel::Debug),
        );

        // Try to abort a multipart upload with a fake upload ID
        let abort_request = AbortMultipartUploadRequest {
            bucket: config.bucket.to_string(),
            key: generate_unique_object_name("nonexistent"),
            upload_id: "fake-upload-id-for-test".to_string(),
            ..Default::default()
        };

        match client.abort_multipart_upload(&abort_request).await {
            Ok(result) => {
                println!(
                    "Unexpectedly aborted nonexistent multipart upload: {:?}",
                    result
                );
            }
            Err(err) => {
                // This is expected - the upload ID doesn't exist
                println!(
                    "Failed to abort nonexistent multipart upload as expected: {:?}",
                    err
                );
            }
        }
    }
}
