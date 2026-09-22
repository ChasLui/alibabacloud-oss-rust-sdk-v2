use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Serialize;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// Deletes vectors by key.
#[derive(Debug, Default, OssRequestModel, Serialize)]
pub struct DeleteVectorsRequest {
    /// The name of the vector bucket.
    #[serde(skip)]
    pub bucket: String,

    /// The name of the index.
    #[serde(rename = "indexName")]
    pub index_name: Option<String>,

    /// The keys of the vectors to delete.
    #[serde(default)]
    pub keys: Vec<String>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteVectorsResult {
    pub common: ResultCommon,
}

impl Client {
    /// Deletes vectors by key.
    ///
    /// Requires a client built with [`Client::new_vectors`].
    pub async fn delete_vectors(
        &self,
        request: &DeleteVectorsRequest,
    ) -> Result<DeleteVectorsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteVectors".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("deleteVectors", "")]
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

        let mut result = DeleteVectorsResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_vectors_request_body() {
        let request = DeleteVectorsRequest {
            bucket: "my-vector".to_string(),
            index_name: Some("idx".to_string()),
            keys: vec!["v-1".to_string(), "v-2".to_string()],
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"indexName":"idx","keys":["v-1","v-2"]}"#
        );
    }
}
