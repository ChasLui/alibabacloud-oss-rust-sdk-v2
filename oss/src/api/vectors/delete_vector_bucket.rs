use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// Deletes a vector bucket.
#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteVectorBucketRequest {
    /// The name of the bucket to delete.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteVectorBucketResult {
    pub common: ResultCommon,
}

impl Client {
    /// Deletes a vector bucket.
    ///
    /// Requires a client built with [`Client::new_vectors`].
    pub async fn delete_vector_bucket(
        &self,
        request: &DeleteVectorBucketRequest,
    ) -> Result<DeleteVectorBucketResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteVectorBucket".to_string(),
            method: http::Method::DELETE,
            bucket: Some(request.bucket.clone()),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/json")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteVectorBucketResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_vector_bucket_request_defaults() {
        let request = DeleteVectorBucketRequest {
            bucket: "my-vector".to_string(),
            ..Default::default()
        };

        assert_eq!(request.bucket, "my-vector");
        assert!(request.header_map().is_empty());
    }
}
