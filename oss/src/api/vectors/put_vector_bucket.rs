use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// Creates a vector bucket.
#[derive(Debug, Default, OssRequestModel)]
pub struct PutVectorBucketRequest {
    /// The name of the bucket to create.
    pub bucket: String,

    /// The ID of the resource group.
    #[field(type = "header", rename = "x-oss-resource-group-id")]
    pub resource_group_id: Option<String>,

    /// The tagging of the bucket.
    #[field(type = "header", rename = "x-oss-bucket-tagging")]
    pub tagging: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutVectorBucketResult {
    pub common: ResultCommon,
}

impl Client {
    /// Creates a vector bucket.
    ///
    /// Requires a client built with [`Client::new_vectors`].
    pub async fn put_vector_bucket(
        &self,
        request: &PutVectorBucketRequest,
    ) -> Result<PutVectorBucketResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutVectorBucket".to_string(),
            method: http::Method::PUT,
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

        let mut result = PutVectorBucketResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_vector_bucket_request_headers() {
        let request = PutVectorBucketRequest {
            bucket: "my-vector".to_string(),
            resource_group_id: Some("rg-123".to_string()),
            tagging: Some("k=v".to_string()),
            ..Default::default()
        };

        let headers = request.header_map();
        assert_eq!(headers.get("x-oss-resource-group-id").unwrap(), "rg-123");
        assert_eq!(headers.get("x-oss-bucket-tagging").unwrap(), "k=v");
    }

    #[test]
    fn test_put_vector_bucket_optional_headers_are_not_sent() {
        let request = PutVectorBucketRequest {
            bucket: "my-vector".to_string(),
            ..Default::default()
        };

        assert!(request.header_map().is_empty());
    }
}
