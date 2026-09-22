use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// Lists the vectors in an index.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct ListVectorsRequest {
    /// The name of the vector bucket.
    #[serde(skip)]
    pub bucket: String,

    /// The name of the index.
    #[serde(rename = "indexName")]
    pub index_name: Option<String>,

    /// The maximum number of vectors returned.
    #[serde(rename = "maxResults", skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,

    /// The token to pass to the next request.
    #[serde(rename = "nextToken", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,

    /// Whether to return the vector data.
    #[serde(rename = "returnData", skip_serializing_if = "Option::is_none")]
    pub return_data: Option<bool>,

    /// Whether to return the vector metadata.
    #[serde(rename = "returnMetadata", skip_serializing_if = "Option::is_none")]
    pub return_metadata: Option<bool>,

    /// The number of segments the listing is split into.
    #[serde(rename = "segmentCount", skip_serializing_if = "Option::is_none")]
    pub segment_count: Option<i32>,

    /// The segment this request lists.
    #[serde(rename = "segmentIndex", skip_serializing_if = "Option::is_none")]
    pub segment_index: Option<i32>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct ListVectorsResult {
    /// The token to pass to the next request.
    #[serde(rename = "NextToken", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,

    /// The container that stores information about vectors.
    #[serde(rename = "Vectors", default)]
    pub vectors: Vec<serde_json::Value>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists the vectors in an index.
    ///
    /// Requires a client built with [`Client::new_vectors`].
    pub async fn list_vectors(
        &self,
        request: &ListVectorsRequest,
    ) -> Result<ListVectorsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListVectors".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("listVectors", "")]
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
        let mut result: ListVectorsResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_vectors_request_body() {
        let request = ListVectorsRequest {
            bucket: "my-vector".to_string(),
            index_name: Some("idx".to_string()),
            max_results: Some(10),
            next_token: Some("t-1".to_string()),
            segment_count: Some(2),
            segment_index: Some(0),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"indexName":"idx","maxResults":10,"nextToken":"t-1","segmentCount":2,"segmentIndex":0}"#
        );
    }

    #[test]
    fn test_list_vectors_result_deserialize() {
        let body = r#"{"NextToken":"t-2","Vectors":[{"key":"v-1"}]}"#;

        let result: ListVectorsResult = serde_json::from_str(body).unwrap();

        assert_eq!(result.next_token.as_deref(), Some("t-2"));
        assert_eq!(result.vectors.len(), 1);
    }
}
