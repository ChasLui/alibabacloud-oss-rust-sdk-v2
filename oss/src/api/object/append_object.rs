use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length};
use crate::{BodyContent, OperationInput, OperationOutput};

#[derive(Default, OssRequestModel)]
pub struct AppendObjectRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// The position from which the AppendObject operation starts.
    /// Each time an AppendObject operation succeeds, the x-oss-next-append-position
    /// header is included in the response to specify the position from which the
    /// next AppendObject operation starts.
    #[field(type = "query", rename = "position")]
    pub position: Option<i64>,

    /// The caching behavior of the web page when the object is downloaded.
    #[field(type = "header", rename = "Cache-Control")]
    pub cache_control: Option<String>,

    /// The method that is used to access the object.
    #[field(type = "header", rename = "Content-Disposition")]
    pub content_disposition: Option<String>,

    /// The method that is used to encode the object.
    #[field(type = "header", rename = "Content-Encoding")]
    pub content_encoding: Option<String>,

    /// The size of the data in the HTTP message body. Unit: bytes.
    #[field(type = "header", rename = "Content-Length")]
    pub content_length: Option<u64>,

    /// The MD5 hash of the object that you want to upload.
    #[field(type = "header", rename = "Content-MD5")]
    pub content_md5: Option<String>,

    /// The expiration time of the cache in UTC.
    #[field(type = "header", rename = "Expires")]
    pub expires: Option<String>,

    /// A standard MIME type describing the format of the contents.
    #[field(type = "header", rename = "Content-Type")]
    pub content_type: Option<String>,

    /// Specifies whether the AppendObject operation overwrites objects with the
    /// same name. Valid values: true and false.
    #[field(type = "header", rename = "x-oss-forbid-overwrite")]
    pub forbid_overwrite: Option<String>,

    /// The method used to encrypt objects on the specified OSS server.
    /// Valid values: AES256, KMS, SM4.
    #[field(type = "header", rename = "x-oss-server-side-encryption")]
    pub server_side_encryption: Option<String>,

    /// Specify the encryption algorithm for the object. Valid values: SM4.
    /// This option is only valid when x-oss-server-side-encryption is KMS.
    #[field(type = "header", rename = "x-oss-server-side-data-encryption")]
    pub server_side_data_encryption: Option<String>,

    /// The ID of the customer master key (CMK) that is managed by Key
    /// Management Service (KMS). This header is valid only when the
    /// x-oss-server-side-encryption header is set to KMS.
    #[field(type = "header", rename = "x-oss-server-side-encryption-key-id")]
    pub server_side_encryption_key_id: Option<String>,

    /// The access control list (ACL) of the object.
    #[field(type = "header", rename = "x-oss-object-acl")]
    pub acl: Option<String>,

    /// The storage class of the object.
    #[field(type = "header", rename = "x-oss-storage-class")]
    pub storage_class: Option<String>,

    /// The tags that are specified for the object by using a key-value pair.
    /// You can specify multiple tags for an object. Example: TagA=A&TagB=B.
    #[field(type = "header", rename = "x-oss-tagging")]
    pub tagging: Option<String>,

    /// The metadata of the object that you want to upload.
    #[field(type = "header", rename = "x-oss-meta-", usermeta)]
    pub metadata: std::collections::HashMap<String, String>,

    /// Specify the speed limit value. The speed limit value ranges from 245760
    /// to 838860800, with a unit of bit/s.
    #[field(type = "header", rename = "x-oss-traffic-limit")]
    pub traffic_limit: Option<i64>,

    /// Object data.
    pub body: Option<BodyContent>,

    /// Specify the initial value of CRC64. If not set, the crc check is ignored.
    /// Note: client-side CRC64 verification is not implemented yet; this field
    /// is kept for API parity with the Go SDK.
    pub init_hash_crc64: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct AppendObjectResult {
    /// Version of the object.
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    /// The 64-bit CRC value of the object.
    /// This value is calculated based on the ECMA-182 standard.
    #[field(type = "header", rename = "x-oss-hash-crc64ecma")]
    pub hash_crc64: Option<String>,

    /// The position that must be provided in the next request, which is the
    /// current length of the object.
    /// Note: declared as String because the result macro only parses header
    /// values as strings; parse it to i64 when needed.
    #[field(type = "header", rename = "x-oss-next-append-position")]
    pub next_position: Option<String>,

    /// The encryption method on the server side when an object is created.
    /// Valid values: AES256, KMS, SM4
    #[field(type = "header", rename = "x-oss-server-side-encryption")]
    pub server_side_encryption: Option<String>,

    /// The server side data encryption algorithm.
    #[field(type = "header", rename = "x-oss-server-side-data-encryption")]
    pub server_side_data_encryption: Option<String>,

    /// The ID of the customer master key (CMK) that is managed by Key
    /// Management Service (KMS). This header is valid only when the
    /// x-oss-server-side-encryption header is set to KMS.
    #[field(type = "header", rename = "x-oss-server-side-encryption-key-id")]
    pub server_side_encryption_key_id: Option<String>,

    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Uploads an object by appending the object to an existing object.
    /// Objects created by using the AppendObject operation are appendable objects.
    ///
    /// # Arguments
    ///
    /// * `request` - The `AppendObjectRequest` containing the bucket name,
    ///   object key, the append position and the data to append.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::AppendObjectRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// # use alibabacloud_oss_sdk_rust_v2::BodyContent;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = AppendObjectRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     position: Some(0),
    ///     body: Some(BodyContent::from_text("hello".to_string(), None)),
    ///     ..Default::default()
    /// };
    ///
    /// match client.append_object(request).await {
    ///     Ok(result) => {
    ///         println!("Next append position: {:?}", result.next_position);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to append object: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn append_object(
        &self,
        mut request: AppendObjectRequest,
    ) -> Result<AppendObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let headers = request.header_map();
        let queries = request.query_map();

        let mut input = OperationInput {
            op_name: "AppendObject".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("append", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            body: request.body.take(),
            ..Default::default()
        };

        modify_request(&mut input, headers, queries, vec![update_content_length])?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = AppendObjectResult::default();
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
    async fn test_append_object() {
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

        let object_name = crate::test_utils::generate_unique_object_name("append");

        // First append at position 0 creates the appendable object.
        let result = client
            .append_object(AppendObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                position: Some(0),
                body: Some(BodyContent::from_text("hello".to_string(), None)),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "append_object failed: {:?}", result.err());
        let next_position = result.unwrap().next_position;
        assert_eq!(next_position.as_deref(), Some("5"));

        // Append again from the returned position.
        let result = client
            .append_object(AppendObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                position: Some(5),
                body: Some(BodyContent::from_text(" world".to_string(), None)),
                ..Default::default()
            })
            .await;
        assert!(result.is_ok(), "second append_object failed: {:?}", result.err());

        // Clean up
        let _ = client
            .delete_object(crate::api::object::DeleteObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name,
                ..Default::default()
            })
            .await;
    }
}
