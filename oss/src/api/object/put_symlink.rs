use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct PutSymlinkRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// The destination object to which the symbolic link points.
    #[field(type = "header", rename = "x-oss-symlink-target")]
    pub target: Option<String>,

    /// Specifies whether the PutSymlink operation overwrites the object that
    /// has the same name. Valid values: true and false.
    #[field(type = "header", rename = "x-oss-forbid-overwrite")]
    pub forbid_overwrite: Option<String>,

    /// The ACL of the object. Default value: default.
    #[field(type = "header", rename = "x-oss-object-acl")]
    pub acl: Option<String>,

    /// The storage class of the object.
    #[field(type = "header", rename = "x-oss-storage-class")]
    pub storage_class: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutSymlinkResult {
    /// Version of the object.
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Creates a symbolic link that points to a destination object. You can
    /// use the symbolic link to access the destination object.
    ///
    /// # Arguments
    ///
    /// * `request` - The `PutSymlinkRequest` containing the bucket name, the
    ///   symbolic link name and the destination object.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::PutSymlinkRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = PutSymlinkRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-symlink".to_string(),
    ///     target: Some("my-object".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.put_symlink(&request).await {
    ///     Ok(result) => {
    ///         println!("Symlink created: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to put symlink: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn put_symlink(
        &self,
        request: &PutSymlinkRequest,
    ) -> Result<PutSymlinkResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutSymlink".to_string(),
            method: http::Method::PUT,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("symlink", "")]
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

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutSymlinkResult::default();
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
    use crate::{BodyContent, SignatureVersionType};

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_symlink() {
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

        let target_name = crate::test_utils::generate_unique_object_name("symlink-target");
        let symlink_name = crate::test_utils::generate_unique_object_name("symlink");

        // Prepare the destination object.
        client
            .put_object(crate::api::object::PutObjectRequest {
                bucket: config.bucket.clone(),
                key: target_name.clone(),
                body: Some(BodyContent::from_text("symlink-test".to_string(), None)),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .put_symlink(&PutSymlinkRequest {
                bucket: config.bucket.clone(),
                key: symlink_name.clone(),
                target: Some(target_name.clone()),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "put_symlink failed: {:?}", result.err());

        // Clean up
        let _ = client
            .delete_object(crate::api::object::DeleteObjectRequest {
                bucket: config.bucket.clone(),
                key: symlink_name,
                ..Default::default()
            })
            .await;
        let _ = client
            .delete_object(crate::api::object::DeleteObjectRequest {
                bucket: config.bucket.clone(),
                key: target_name,
                ..Default::default()
            })
            .await;
    }
}
