use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// Runs a similarity query against an index.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct QueryVectorsRequest {
    /// The name of the vector bucket.
    #[serde(skip)]
    pub bucket: String,

    /// The name of the index.
    #[serde(rename = "indexName")]
    pub index_name: Option<String>,

    /// The vector to search with.
    #[serde(rename = "queryVector", skip_serializing_if = "Option::is_none")]
    pub query_vector: Option<serde_json::Value>,

    /// The number of nearest neighbours to return.
    #[serde(rename = "topK", skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,

    /// The metadata filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<serde_json::Value>,

    /// Whether to return the distance of each result.
    #[serde(rename = "returnDistance", skip_serializing_if = "Option::is_none")]
    pub return_distance: Option<bool>,

    /// Whether to return the metadata of each result.
    #[serde(rename = "returnMetadata", skip_serializing_if = "Option::is_none")]
    pub return_metadata: Option<bool>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct QueryVectorsResult {
    /// The vectors that matched, ordered by distance.
    #[serde(rename = "vectors", default)]
    pub vectors: Vec<serde_json::Value>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Runs a similarity query against an index.
    ///
    /// Requires a client built with [`Client::new_vectors`].
    pub async fn query_vectors(
        &self,
        request: &QueryVectorsRequest,
    ) -> Result<QueryVectorsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "QueryVectors".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("queryVectors", "")]
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
        let mut result: QueryVectorsResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_vectors_request_body() {
        let request = QueryVectorsRequest {
            bucket: "my-vector".to_string(),
            index_name: Some("idx".to_string()),
            query_vector: Some(serde_json::json!({"float32": [0.1, 0.2]})),
            top_k: Some(5),
            return_distance: Some(true),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"indexName":"idx","queryVector":{"float32":[0.1,0.2]},"topK":5,"returnDistance":true}"#
        );
    }

    #[test]
    fn test_query_vectors_result_deserialize() {
        let body = r#"{"vectors":[{"key":"v-1","distance":0.1}]}"#;

        let result: QueryVectorsResult = serde_json::from_str(body).unwrap();

        assert_eq!(result.vectors.len(), 1);
        assert_eq!(result.vectors[0]["key"], "v-1");
    }
}
