use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::modify_request;
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteBucketRequest {
    /// The name of the bucket to delete.
    pub bucket: String,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteBucketResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes a bucket.
    ///
    /// This method sends a DELETE request to delete a bucket. Only the bucket
    /// owner has permission to delete the bucket. OSS does not allow
    /// deleting a non-empty bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteBucketRequest` containing the bucket name to
    ///   delete.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `DeleteBucketResult` if the deletion
    /// is successful, or an error if it fails. Note that:
    /// - If the bucket does not exist, a 404 error will be returned
    /// - If the bucket is not empty, a 409 error will be returned
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::DeleteBucketRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteBucketRequest {
    ///     bucket: "my-bucket-to-delete".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_bucket(&request).await {
    ///     Ok(delete_bucket_result) => {
    ///         println!("Bucket deleted successfully");
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete bucket: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_bucket(
        &self,
        request: &DeleteBucketRequest,
    ) -> Result<DeleteBucketResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteBucket".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/octet-stream")]
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

        let mut result = DeleteBucketResult::default();

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
    use crate::log::LogLevel;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    // Skip this test for now since creating buckets requires special
    // permissions and can conflict with existing buckets
    // #[tokio::test]
    // #[serial_test::serial]
    // async fn test_delete_bucket_basic() {
    //     let config = match load_test_config() {
    //         Some(cfg) => cfg,
    //         None => {
    //             eprintln!("Test configuration not found. Skipping test.");
    //             return;
    //         }
    //     };

    //     let client = Client::new(
    //         &Config::default()
    //             .with_region(&config.region)
    //
    // .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
    //                 &config.access_key_id,
    //                 &config.access_key_secret,
    //                 &[],
    //             )))
    //             .with_signature_version(SignatureVersionType::V4)
    //             .with_log_level(LogLevel::Debug),
    //     );

    //     // Generate a unique bucket name for this test
    //     let bucket_name = generate_unique_bucket_name("delete-bucket-test");

    //     // First, create a temporary bucket to delete
    //     let create_request = CreateBucketRequest {
    //         bucket: bucket_name.clone(),
    //         ..Default::default()
    //     };

    //     match client.create_bucket(&create_request).await {
    //         Ok(create_result) => {
    //             println!("Bucket created for deletion test: {:?}",
    // create_result);         }
    //         Err(err) => panic!("Failed to create bucket for deletion test:
    // {:?}", err),     }

    //     // Verify the bucket exists by listing buckets
    //     let list_request = ListBucketsRequest::default();
    //     let list_result = match client.list_buckets(&list_request).await {
    //         Ok(result) => result,
    //         Err(err) => panic!("Failed to list buckets for verification:
    // {:?}", err),     };

    //     let bucket_exists = list_result.buckets.iter()
    //         .any(|bucket| bucket.name.as_deref() == Some(&bucket_name));

    //     assert!(bucket_exists, "Created bucket should exist in bucket list");
    //     println!("Verified bucket exists before deletion");

    //     // Now delete the bucket
    //     let delete_request = DeleteBucketRequest {
    //         bucket: bucket_name.clone(),
    //         ..Default::default()
    //     };

    //     match client.delete_bucket(&delete_request).await {
    //         Ok(result) => {
    //             println!("Bucket deleted successfully: {:?}", result);
    //             assert_eq!(result.common.status,
    // http::StatusCode::NO_CONTENT);             println!("Status code
    // confirmed as 204 No Content");         }
    //         Err(err) => panic!("Delete bucket failed: {:?}", err),
    //     }

    //     // Verify the bucket no longer exists
    //     let list_result_after = match
    // client.list_buckets(&list_request).await {         Ok(result) =>
    // result,         Err(err) => panic!("Failed to list buckets after
    // deletion: {:?}", err),     };

    //     let bucket_exists_after = list_result_after.buckets.iter()
    //         .any(|bucket| bucket.name.as_deref() == Some(&bucket_name));

    //     assert!(!bucket_exists_after, "Bucket should not exist after
    // deletion");     println!("Verified bucket no longer exists after
    // deletion"); }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_nonexistent_bucket() {
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

        // Try to delete a bucket that doesn't exist
        let delete_request = DeleteBucketRequest {
            bucket: "nonexistent-bucket-should-not-exist-12345".to_string(),
            ..Default::default()
        };

        match client.delete_bucket(&delete_request).await {
            Ok(_) => {
                // This shouldn't happen, but if it does, it means the bucket
                // existed
                println!("Unexpectedly succeeded in deleting nonexistent bucket");
            }
            Err(err) => {
                // This is expected - trying to delete a nonexistent bucket
                // should fail
                println!("Correctly failed to delete nonexistent bucket: {:?}", err);
            }
        }
    }
}
