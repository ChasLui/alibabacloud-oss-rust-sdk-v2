use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_md5};
use crate::{OperationInput, OperationOutput};

#[derive(Debug, Default, OssRequestModel)]
pub struct GetSymlinkRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// Version of the object.
    #[field(type = "query", rename = "versionId")]
    pub version_id: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct GetSymlinkResult {
    /// Version of the object.
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    /// Indicates the target object that the symbol link directs to.
    #[field(type = "header", rename = "x-oss-symlink-target")]
    pub target: Option<String>,

    /// Entity tag for the uploaded object.
    #[field(type = "header", rename = "ETag")]
    pub etag: Option<String>,

    /// The metadata of the object that you want to symlink.
    #[field(type = "header", rename = "x-oss-meta-", usermeta)]
    pub metadata: std::collections::HashMap<String, String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Obtains a symbol link. To perform GetSymlink operations, you must have
    /// the read permission on the symbol link.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetSymlinkRequest` containing the bucket name and the
    ///   symbolic link name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::GetSymlinkRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetSymlinkRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-symlink".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_symlink(&request).await {
    ///     Ok(result) => {
    ///         println!("Symlink target: {:?}", result.target);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to get symlink: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_symlink(
        &self,
        request: &GetSymlinkRequest,
    ) -> Result<GetSymlinkResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetSymlink".to_string(),
            method: http::Method::GET,
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

        let mut result = GetSymlinkResult::default();
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
    async fn test_get_symlink() {
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

        // Prepare the destination object and the symbolic link.
        client
            .put_object(crate::api::object::PutObjectRequest {
                bucket: config.bucket.clone(),
                key: target_name.clone(),
                body: Some(BodyContent::from_text("symlink-test".to_string(), None)),
                ..Default::default()
            })
            .await
            .unwrap();

        client
            .put_symlink(&crate::api::object::PutSymlinkRequest {
                bucket: config.bucket.clone(),
                key: symlink_name.clone(),
                target: Some(target_name.clone()),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = client
            .get_symlink(&GetSymlinkRequest {
                bucket: config.bucket.clone(),
                key: symlink_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "get_symlink failed: {:?}", result.err());
        assert_eq!(
            result.unwrap().target.as_deref(),
            Some(target_name.as_str())
        );

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
