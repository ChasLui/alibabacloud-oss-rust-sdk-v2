use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteFileMetaRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the dataset.
    #[field(type = "query", rename = "datasetName")]
    pub dataset_name: String,

    /// The URI of the object whose metadata is deleted.
    #[field(type = "query", rename = "uri")]
    pub uri: String,

    pub common: RequestCommon,
}

impl DeleteFileMetaRequest {
    /// Creates a request that deletes the metadata of a single object.
    pub fn new(bucket: &str, dataset_name: &str, uri: &str) -> Self {
        DeleteFileMetaRequest {
            bucket: bucket.to_string(),
            dataset_name: dataset_name.to_string(),
            uri: uri.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteFileMetaResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes the metadata of a single object from a dataset.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteFileMetaRequest` containing the bucket name,
    ///   the dataset name and the object URI.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::DeleteFileMetaRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteFileMetaRequest::new("my-bucket", "my-dataset", "oss://my-bucket/object");
    ///
    /// match client.delete_file_meta(&request).await {
    ///     Ok(_) => println!("file metadata deleted"),
    ///     Err(error) => eprintln!("Failed to delete file metadata: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn delete_file_meta(
        &self,
        request: &DeleteFileMetaRequest,
    ) -> Result<DeleteFileMetaResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteFileMeta".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", ""), ("action", "deleteFileMeta")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
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

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteFileMetaResult::default();
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

    #[test]
    fn test_delete_file_meta_request_query_map() {
        let request = DeleteFileMetaRequest::new("bucket", "test-dataset", "oss://bucket/object");
        let query = request.query_map();
        assert_eq!(
            query.get("datasetName").map(String::as_str),
            Some("test-dataset")
        );
        assert_eq!(
            query.get("uri").map(String::as_str),
            Some("oss://bucket/object")
        );
        assert_eq!(query.len(), 2);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_file_meta() {
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

        let result = client
            .delete_file_meta(&DeleteFileMetaRequest::new(
                &config.bucket,
                "sdk-test-dataset",
                "oss://sdk-test-dataset/object",
            ))
            .await;
        if let Err(error) = &result {
            eprintln!("delete_file_meta rejected: {}", error);
        }
    }
}
