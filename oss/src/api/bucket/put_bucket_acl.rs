use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutBucketAclRequest {
    /// The name of the bucket containing the objects
    pub bucket: String,

    /// The access control list (ACL) of the object.
    /// Possible value: public-read-write, public-read, private
    #[field(type = "header", rename = "x-oss-acl")]
    pub bucket_acl_type: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutBucketAclResult {
    pub common: ResultCommon,
}

impl PutBucketAclRequest {
    pub fn new(bucket: &str, bucket_acl_type: &str) -> Self {
        Self {
            bucket: bucket.to_string(),
            bucket_acl_type: bucket_acl_type.to_string(),
            ..Default::default()
        }
    }
}

impl Client {
    /// Implements the `put_bucket_acl` method for the `Client` struct.
    ///
    /// This method is used to update the access control list (ACL) for a
    /// bucket. It takes a `PutBucketAclRequest` as input and returns a
    /// `Result` indicating whether the operation was successful or an error
    /// occurred.
    ///
    /// # Arguments
    ///
    /// * `request` - A reference to a `PutBucketAclRequest` object containing
    ///   the necessary information for the operation.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::PutBucketAclRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutBucketAclRequest::new("my-bucket", "public-read");
    ///
    /// match client.put_bucket_acl(&request).await {
    ///     Ok(acl) => {
    ///         // Handle the ACL response
    ///     }
    ///     Err(err) => {
    ///         // Handle the error
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_bucket_acl(
        &self,
        request: &PutBucketAclRequest,
    ) -> Result<PutBucketAclResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutBucketAcl".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            parameters: [("acl", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutBucketAclResult::default();

        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::bucket::{CreateBucketRequest, DeleteBucketRequest, GetBucketAclRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    async fn put_acl(client: &Client, bucket: &str, acl: &str) {
        match client
            .put_bucket_acl(&PutBucketAclRequest::new(bucket, acl))
            .await
        {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }
    }

    async fn assert_acl(client: &Client, bucket: &str, acl: &str) {
        match client
            .get_bucket_acl(&GetBucketAclRequest::new(bucket))
            .await
        {
            Ok(output) => {
                println!("{:?}", output);
                assert_eq!(output.acl.as_ref().unwrap(), acl);
            }
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_bucket_acl() {
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

        // Generate a unique bucket name for this test
        let bucket_name = generate_unique_bucket_name("put-bucket-acl");

        // Create the bucket first
        let create_request = CreateBucketRequest {
            bucket: bucket_name.clone(),
            ..Default::default()
        };

        match client.create_bucket(&create_request).await {
            Ok(_) => println!("Bucket created: {}", bucket_name),
            Err(err) => panic!("Failed to create bucket: {:?}", err),
        };

        // // make modification
        // put_acl(&client, &bucket_name, "public-read").await;
        //
        // // check whether modification is made
        // assert_acl(&client, &bucket_name, "public-read").await;

        // revert to private
        put_acl(&client, &bucket_name, "private").await;

        // check whether modification is reverted
        assert_acl(&client, &bucket_name, "private").await;

        // Clean up: delete the bucket
        let delete_request = DeleteBucketRequest {
            bucket: bucket_name.clone(),
            ..Default::default()
        };
        match client.delete_bucket(&delete_request).await {
            Ok(_) => println!("Bucket deleted: {}", bucket_name),
            Err(err) => eprintln!("Failed to delete bucket: {:?}", err),
        };
    }
}
