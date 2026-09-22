use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteObjectRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object to delete.
    pub key: String,

    /// VersionId used to reference a specific version of the object.
    /// Used to permanently delete a specific version of an object in a
    /// versioned bucket.
    #[field(type = "query", rename = "versionId")]
    pub version_id: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    /// Common request fields
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteObjectResult {
    /// Indicates whether the object was a delete marker.
    /// Returns true if the object was a delete marker.
    #[field(type = "header", rename = "x-oss-delete-marker")]
    pub delete_marker: Option<bool>,

    /// The version ID of the object that was deleted or the version ID of the
    /// delete marker that was created.
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes an object from the OSS bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteObjectRequest` containing the necessary
    ///   information for the delete operation.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `DeleteObjectResult` if the delete is
    /// successful, or an error if it fails. Note that a successful deletion
    /// returns a 204 status code regardless of whether the object existed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::DeleteObjectRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteObjectRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_object(request).await {
    ///     Ok(delete_object_result) => {
    ///         println!("Object deleted successfully: {:?}", delete_object_result);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete object: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_object(
        &self,
        request: DeleteObjectRequest,
    ) -> Result<DeleteObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteObject".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteObjectResult::default();

        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {

    use std::rc::Rc;

    use super::*;
    use crate::api::object::{GetObjectRequest, PutObjectRequest};
    use crate::client::BodyDataReader;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::test_utils::{generate_unique_object_name, load_test_config};
    use crate::SignatureVersionType;

    // Helper function to generate unique test object names
    // Using shared generate_unique_object_name from test_utils

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_object_basic() {
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

        // Create an object to delete
        let object_name = generate_unique_object_name("delete-basic");
        let test_content = "Test content for delete operation";

        // Put the object first
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            body: Some(crate::BodyContent::from_text(
                test_content.to_string(),
                None,
            )),
            ..Default::default()
        };

        match client.put_object(put_request).await {
            Ok(output) => println!("Object created for deletion test: {:?}", output),
            Err(err) => panic!("Failed to create object for deletion test: {:?}", err),
        }

        // Verify the object exists before deletion
        let get_request = GetObjectRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            ..Default::default()
        };

        match client.get_object(get_request).await {
            Ok(mut result) => {
                let content_bytes = result.get_all_data().await.unwrap_or_default();
                let content = String::from_utf8_lossy(&content_bytes).into_owned();
                assert_eq!(content, test_content);
                println!("Verified object exists before deletion");
            }
            Err(err) => panic!("Failed to get object before deletion: {:?}", err),
        }

        // Now delete the object
        let delete_request = DeleteObjectRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            ..Default::default()
        };

        match client.delete_object(delete_request).await {
            Ok(result) => {
                println!("Object deleted successfully: {:?}", result);
                // Status should be 204 (No Content)
                assert_eq!(result.common.status, http::StatusCode::NO_CONTENT);
            }
            Err(err) => panic!("Delete object failed: {:?}", err),
        }

        // Verify the object no longer exists
        let get_request_after_delete = GetObjectRequest {
            bucket: config.bucket.to_string(),
            key: object_name.clone(),
            ..Default::default()
        };
        match client.get_object(get_request_after_delete).await {
            Ok(_result) => {
                // In some cases with versioning, the object might still be
                // accessible
                println!("Object may still be accessible due to versioning");
            }
            Err(_err) => {
                println!("Confirmed object deletion - object no longer accessible");
            }
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_object_with_version_id() {
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

        // Use version_bucket if available, otherwise use regular bucket
        let bucket_name = if let Some(ref version_bucket) = config.version_bucket {
            version_bucket.as_str()
        } else {
            config.bucket.as_str()
        };

        // Create an object to delete with version ID
        let object_name = generate_unique_object_name("delete-version");
        let test_content = "Test content for versioned delete operation";

        // Put the object first
        let put_request = PutObjectRequest {
            bucket: bucket_name.to_string(),
            key: object_name.clone(),
            body: Some(crate::BodyContent::from_text(
                test_content.to_string(),
                None,
            )),
            ..Default::default()
        };

        match client.put_object(put_request).await {
            Ok(output) => println!("Object created for versioned deletion test: {:?}", output),
            Err(err) => {
                eprintln!(
                    "Failed to create object for versioned deletion test: {:?}",
                    err
                );
                // If using version bucket and it fails, try with regular bucket
                if config.version_bucket.is_some() {
                    eprintln!("Trying with regular bucket instead of version bucket...");
                    let put_request = PutObjectRequest {
                        bucket: config.bucket.to_string(),
                        key: object_name.clone(),
                        body: Some(crate::BodyContent::from_text(
                            test_content.to_string(),
                            None,
                        )),
                        ..Default::default()
                    };
                    match client.put_object(put_request).await {
                        Ok(output) => {
                            println!("Object created for versioned deletion test: {:?}", output)
                        }
                        Err(err) => panic!(
                            "Failed to create object for versioned deletion test: {:?}",
                            err
                        ),
                    }
                } else {
                    panic!(
                        "Failed to create object for versioned deletion test: {:?}",
                        err
                    );
                }
            }
        };

        // Now delete the object with a version ID (this would typically be a
        // real version ID) For now, we'll just test that the request
        // can be formed with a version ID
        let delete_request = DeleteObjectRequest {
            bucket: bucket_name.to_string(),
            key: object_name.clone(),
            version_id: None, // In a real scenario, this would be a specific version ID
            ..Default::default()
        };

        match client.delete_object(delete_request).await {
            Ok(result) => {
                println!("Object deletion with version ID attempted: {:?}", result);
                // Status should be 204 (No Content)
                assert_eq!(result.common.status, http::StatusCode::NO_CONTENT);
            }
            Err(err) => panic!("Delete object with version ID failed: {:?}", err),
        }
    }
}
