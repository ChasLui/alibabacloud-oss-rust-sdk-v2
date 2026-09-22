use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// Queries vectors by key.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct GetVectorsRequest {
    /// The name of the vector bucket.
    #[serde(skip)]
    pub bucket: String,

    /// The name of the index.
    #[serde(rename = "indexName")]
    pub index_name: Option<String>,

    /// The keys of the vectors to query.
    #[serde(default)]
    pub keys: Vec<String>,

    /// Whether to return the vector data.
    #[serde(rename = "returnData", skip_serializing_if = "Option::is_none")]
    pub return_data: Option<bool>,

    /// Whether to return the vector metadata.
    #[serde(rename = "returnMetadata", skip_serializing_if = "Option::is_none")]
    pub return_metadata: Option<bool>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct GetVectorsResult {
    /// The vectors that were found.
    #[serde(rename = "vectors", default)]
    pub vectors: Vec<serde_json::Value>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries vectors by key.
    ///
    /// Requires a client built with [`Client::new_vectors`].
    pub async fn get_vectors(
        &self,
        request: &GetVectorsRequest,
    ) -> Result<GetVectorsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetVectors".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("getVectors", "")]
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
        let mut result: GetVectorsResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_vectors_request_body() {
        let request = GetVectorsRequest {
            bucket: "my-vector".to_string(),
            index_name: Some("idx".to_string()),
            keys: vec!["v-1".to_string()],
            return_data: Some(true),
            return_metadata: Some(false),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"indexName":"idx","keys":["v-1"],"returnData":true,"returnMetadata":false}"#
        );
    }

    #[test]
    fn test_get_vectors_result_deserialize() {
        let body = r#"{"vectors":[{"key":"v-1","data":{"float32":[0.1]}}]}"#;

        let result: GetVectorsResult = serde_json::from_str(body).unwrap();

        assert_eq!(result.vectors.len(), 1);
        assert_eq!(result.vectors[0]["key"], "v-1");
    }
}
