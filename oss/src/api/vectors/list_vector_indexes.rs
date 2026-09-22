use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use super::get_vector_index::VectorIndex;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// Lists the vector indexes that belong to a bucket.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct ListVectorIndexesRequest {
    /// The name of the vector bucket.
    #[serde(skip)]
    pub bucket: String,

    /// The token to pass to the next request.
    #[serde(rename = "nextToken", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,

    /// The maximum number of indexes returned.
    #[serde(rename = "maxResults", skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,

    /// The prefix that the names of the returned indexes must contain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct ListVectorIndexesResult {
    /// The token to pass to the next request.
    #[serde(rename = "NextToken", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,

    /// The container that stores information about indexes.
    #[serde(rename = "Indexes", default)]
    pub indexes: Vec<VectorIndex>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Lists the vector indexes that belong to a bucket.
    ///
    /// Requires a client built with [`Client::new_vectors`].
    pub async fn list_vector_indexes(
        &self,
        request: &ListVectorIndexesRequest,
    ) -> Result<ListVectorIndexesResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListVectorIndexes".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("listVectorIndexes", "")]
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
        let mut result: ListVectorIndexesResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_vector_indexes_request_body() {
        let request = ListVectorIndexesRequest {
            bucket: "my-vector".to_string(),
            next_token: Some("t-1".to_string()),
            max_results: Some(10),
            prefix: Some("idx".to_string()),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"nextToken":"t-1","maxResults":10,"prefix":"idx"}"#
        );
    }

    #[test]
    fn test_list_vector_indexes_result_deserialize() {
        let body = r#"{"NextToken":"t-2","Indexes":[{"indexName":"idx","dimension":768}]}"#;

        let result: ListVectorIndexesResult = serde_json::from_str(body).unwrap();

        assert_eq!(result.next_token.as_deref(), Some("t-2"));
        assert_eq!(result.indexes.len(), 1);
        assert_eq!(result.indexes[0].index_name.as_deref(), Some("idx"));
    }

    #[test]
    fn test_list_vector_indexes_result_without_indexes() {
        let result: ListVectorIndexesResult = serde_json::from_str("{}").unwrap();
        assert!(result.indexes.is_empty());
    }
}
