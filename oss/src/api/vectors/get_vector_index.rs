use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// A vector index.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VectorIndex {
    /// The time when the index was created.
    #[serde(rename = "createTime", skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,

    /// The data type of the vectors.
    #[serde(rename = "dataType", skip_serializing_if = "Option::is_none")]
    pub data_type: Option<String>,

    /// The number of dimensions of the vectors.
    #[serde(rename = "dimension", skip_serializing_if = "Option::is_none")]
    pub dimension: Option<i32>,

    /// The distance metric.
    #[serde(rename = "distanceMetric", skip_serializing_if = "Option::is_none")]
    pub distance_metric: Option<String>,

    /// The name of the index.
    #[serde(rename = "indexName", skip_serializing_if = "Option::is_none")]
    pub index_name: Option<String>,

    /// The metadata of the index.
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    /// The status of the index.
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// The ARN of the bucket that holds the index.
    #[serde(rename = "bucketArn", skip_serializing_if = "Option::is_none")]
    pub bucket_arn: Option<String>,

    /// The name of the vector bucket. Deprecated upstream; kept for parity.
    #[serde(rename = "vectorBucketName", skip_serializing_if = "Option::is_none")]
    pub vector_bucket_name: Option<String>,
}

/// Queries a vector index.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct GetVectorIndexRequest {
    /// The name of the vector bucket.
    #[serde(skip)]
    pub bucket: String,

    /// The name of the index.
    #[serde(rename = "indexName")]
    pub index_name: Option<String>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct GetVectorIndexResult {
    /// The container that stores the index.
    #[serde(rename = "index", skip_serializing_if = "Option::is_none")]
    pub index: Option<VectorIndex>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries a vector index.
    ///
    /// Requires a client built with [`Client::new_vectors`].
    pub async fn get_vector_index(
        &self,
        request: &GetVectorIndexRequest,
    ) -> Result<GetVectorIndexResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetVectorIndex".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("getVectorIndex", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/json")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.body = Some(BodyContent::from_text(
            serde_json::to_string(request)?,
            None,
        ));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let mut result: GetVectorIndexResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_vector_index_request_body() {
        let request = GetVectorIndexRequest {
            bucket: "my-vector".to_string(),
            index_name: Some("idx".to_string()),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"indexName":"idx"}"#
        );
    }

    #[test]
    fn test_get_vector_index_result_deserialize() {
        let body = r#"{"index":{"indexName":"idx","dataType":"float32","dimension":768,
            "distanceMetric":"cosine","status":"enabled"}}"#;

        let result: GetVectorIndexResult = serde_json::from_str(body).unwrap();

        let index = result.index.expect("index");
        assert_eq!(index.index_name.as_deref(), Some("idx"));
        assert_eq!(index.dimension, Some(768));
        assert_eq!(index.status.as_deref(), Some("enabled"));
    }
}
