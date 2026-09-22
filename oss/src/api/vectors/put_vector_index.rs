use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Serialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// Creates a vector index.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct PutVectorIndexRequest {
    /// The name of the vector bucket.
    #[serde(skip)]
    pub bucket: String,

    /// The name of the index.
    #[serde(rename = "indexName")]
    pub index_name: Option<String>,

    /// The data type of the vectors, e.g. `float32`.
    #[serde(rename = "dataType")]
    pub data_type: Option<String>,

    /// The number of dimensions of the vectors.
    pub dimension: Option<i32>,

    /// The distance metric, e.g. `cosine`.
    #[serde(rename = "distanceMetric")]
    pub distance_metric: Option<String>,

    /// The metadata of the index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutVectorIndexResult {
    pub common: ResultCommon,
}

impl Client {
    /// Creates a vector index.
    ///
    /// Requires a client built with [`Client::new_vectors`].
    pub async fn put_vector_index(
        &self,
        request: &PutVectorIndexRequest,
    ) -> Result<PutVectorIndexResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutVectorIndex".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("putVectorIndex", "")]
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

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = PutVectorIndexResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_vector_index_body() {
        let request = PutVectorIndexRequest {
            bucket: "my-vector".to_string(),
            index_name: Some("idx".to_string()),
            data_type: Some("float32".to_string()),
            dimension: Some(768),
            distance_metric: Some("cosine".to_string()),
            ..Default::default()
        };

        let body = serde_json::to_string(&request).unwrap();

        assert_eq!(
            body,
            r#"{"indexName":"idx","dataType":"float32","dimension":768,"distanceMetric":"cosine"}"#
        );
    }

    #[test]
    fn test_put_vector_index_body_excludes_bucket() {
        let request = PutVectorIndexRequest {
            bucket: "my-vector".to_string(),
            index_name: Some("idx".to_string()),
            ..Default::default()
        };

        assert!(!serde_json::to_string(&request)
            .unwrap()
            .contains("my-vector"));
    }
}
