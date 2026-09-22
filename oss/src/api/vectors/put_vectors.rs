use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Serialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// Creates vectors in an index.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct PutVectorsRequest {
    /// The name of the vector bucket.
    #[serde(skip)]
    pub bucket: String,

    /// The name of the index.
    #[serde(rename = "indexName")]
    pub index_name: Option<String>,

    /// The vectors to create, each a JSON object.
    #[serde(default)]
    pub vectors: Vec<serde_json::Value>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct PutVectorsResult {
    pub common: ResultCommon,
}

impl Client {
    /// Creates vectors in an index.
    ///
    /// Requires a client built with [`Client::new_vectors`].
    pub async fn put_vectors(
        &self,
        request: &PutVectorsRequest,
    ) -> Result<PutVectorsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "PutVectors".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("putVectors", "")]
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

        let mut result = PutVectorsResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_vectors_request_body() {
        let request = PutVectorsRequest {
            bucket: "my-vector".to_string(),
            index_name: Some("idx".to_string()),
            vectors: vec![serde_json::json!({"key": "v-1", "data": {"float32": [0.1, 0.2]}})],
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"indexName":"idx","vectors":[{"data":{"float32":[0.1,0.2]},"key":"v-1"}]}"#
        );
    }
}
