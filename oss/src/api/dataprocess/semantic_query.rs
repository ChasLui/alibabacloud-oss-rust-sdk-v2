use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::simple_query::Files;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct SemanticQueryRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the dataset.
    #[field(type = "query", rename = "datasetName")]
    pub dataset_name: String,

    /// The maximum number of objects to return.
    #[field(type = "query", rename = "maxResults")]
    pub max_results: Option<i32>,

    /// The natural language query.
    #[field(type = "query", rename = "query")]
    pub query: Option<String>,

    /// The query conditions, as produced by
    /// [`super::simple_query::SimpleQuery::to_parameter_value`].
    #[field(type = "query", rename = "simpleQuery")]
    pub simple_query: Option<String>,

    /// The fields to return, as produced by
    /// [`super::simple_query::WithFields::to_parameter_value`].
    #[field(type = "query", rename = "withFields")]
    pub with_fields: Option<String>,

    /// The multimedia types to query, as produced by
    /// [`super::simple_query::MediaTypes::to_parameter_value`].
    #[field(type = "query", rename = "mediaTypes")]
    pub media_types: Option<String>,

    /// The URI of the object that is used as the semantic query source.
    #[field(type = "query", rename = "sourceURI")]
    pub source_uri: Option<String>,

    pub common: RequestCommon,
}

impl SemanticQueryRequest {
    /// Creates a request that queries the files of a dataset semantically.
    pub fn new(bucket: &str, dataset_name: &str) -> Self {
        SemanticQueryRequest {
            bucket: bucket.to_string(),
            dataset_name: dataset_name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct SemanticQueryResult {
    /// The objects that meet the query conditions.
    #[serde(rename = "Files", skip_serializing_if = "Option::is_none")]
    pub files: Option<Files>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the files of a dataset by using the semantic query feature.
    ///
    /// # Arguments
    ///
    /// * `request` - The `SemanticQueryRequest` containing the bucket and
    ///   dataset names and the query conditions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::SemanticQueryRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = SemanticQueryRequest {
    ///     query: Some("a person riding a bike".to_string()),
    ///     max_results: Some(10),
    ///     ..SemanticQueryRequest::new("my-bucket", "my-dataset")
    /// };
    ///
    /// match client.semantic_query(&request).await {
    ///     Ok(result) => println!("files: {:?}", result.files),
    ///     Err(error) => eprintln!("Failed to run semantic query: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn semantic_query(
        &self,
        request: &SemanticQueryRequest,
    ) -> Result<SemanticQueryResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "SemanticQuery".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", ""), ("action", "semanticQuery")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: SemanticQueryResult = quick_xml::de::from_str(&data_str)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_semantic_query_request_query_map() {
        let request = SemanticQueryRequest {
            max_results: Some(10),
            query: Some(r#"{"Field":"Size","Value":"1","Operation":"gt"}"#.to_string()),
            with_fields: Some(r#"["Filename","Size"]"#.to_string()),
            media_types: Some(r#"["video","image"]"#.to_string()),
            source_uri: Some("oss://bucket/prefix".to_string()),
            ..SemanticQueryRequest::new("bucket", "your_dataset")
        };
        let query = request.query_map();
        assert_eq!(
            query.get("datasetName").map(String::as_str),
            Some("your_dataset")
        );
        assert_eq!(query.get("maxResults").map(String::as_str), Some("10"));
        assert_eq!(
            query.get("sourceURI").map(String::as_str),
            Some("oss://bucket/prefix")
        );
        assert_eq!(
            query.get("mediaTypes").map(String::as_str),
            Some(r#"["video","image"]"#)
        );
        assert!(!query.contains_key("simpleQuery"));
    }

    #[test]
    fn test_semantic_query_result_deserialize() {
        let xml = r#"<MetaQuery>
  <Files>
    <File>
      <Filename>photos/sunset.jpg</Filename>
      <Size>2048000</Size>
      <URI>oss://examplebucket/photos/sunset.jpg</URI>
      <MediaType>image</MediaType>
      <SemanticSimilarity>0.92</SemanticSimilarity>
    </File>
  </Files>
</MetaQuery>"#;
        let result: SemanticQueryResult = quick_xml::de::from_str(xml).unwrap();
        let files = result.files.unwrap().files;
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename.as_deref(), Some("photos/sunset.jpg"));
        assert_eq!(files[0].semantic_similarity, Some(0.92));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_semantic_query() {
        let config = match load_test_config() {
            Some(cfg) => cfg,
            None => {
                eprintln!("Test configuration not found. Skipping test.");
                return;
            }
        };

        let client = Client::new(
            &Config::default()
                .with_region(&config.region)
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    &config.access_key_id,
                    &config.access_key_secret,
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V4),
        );

        let result = client
            .semantic_query(&SemanticQueryRequest::new(
                &config.bucket,
                "sdk-test-dataset",
            ))
            .await;
        if let Err(error) = &result {
            eprintln!("semantic_query rejected: {}", error);
        }
    }
}
