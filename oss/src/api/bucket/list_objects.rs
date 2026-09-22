use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::CommonPrefix;
use crate::api::object::ObjectProperties;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_md5};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};
#[derive(Debug, Default, Clone, OssRequestModel)]
pub struct ListObjectsRequest {
    /// The name of the bucket containing the objects.
    pub bucket: String,

    /// The character that is used to group objects by name. If you specify
    /// the delimiter parameter in the request, the response contains the
    /// CommonPrefixes parameter. The objects whose names contain the same
    /// string from the prefix to the next occurrence of the delimiter are
    /// grouped as a single result element in CommonPrefixes.
    #[field(type = "query")]
    pub delimiter: Option<String>,

    /// The encoding type of the content in the response. Valid value: url.
    #[field(type = "query", rename = "encoding-type")]
    pub encoding_type: Option<String>,

    /// The name of the object after which the ListObjects (GetBucket)
    /// operation starts. If this parameter is specified, objects whose names
    /// are alphabetically greater than the marker value are returned.
    #[field(type = "query")]
    pub marker: Option<String>,

    /// The maximum number of objects that you want to return. If the list
    /// operation cannot be complete at a time because the max-keys
    /// parameter is specified, the NextMarker element is included in the
    /// response as the marker for the next list operation.
    #[field(type = "query", rename = "max-keys")]
    pub max_keys: Option<i32>,

    /// The prefix that the names of the returned objects must contain.
    #[field(type = "query")]
    pub prefix: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs.
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Deserialize, OssResultModel)]
pub struct ListObjectsResult {
    /// The name of the bucket.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The prefix contained in the returned object names.
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// The name of the object after which the list operation begins.
    #[serde(rename = "Marker", skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,

    /// The maximum number of returned objects in the response.
    #[serde(rename = "MaxKeys")]
    pub max_keys: i32,

    /// The character that is used to group objects by name.
    #[serde(rename = "Delimiter", skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<String>,

    /// Indicates whether the returned results are truncated.
    /// true indicates that not all results are returned this time.
    /// false indicates that all results are returned this time.
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,

    /// The position from which the next list operation starts.
    #[serde(rename = "NextMarker", skip_serializing_if = "Option::is_none")]
    pub next_marker: Option<String>,

    /// The encoding type of the content in the response.
    #[serde(rename = "EncodingType", skip_serializing_if = "Option::is_none")]
    pub encoding_type: Option<String>,

    /// The container that stores the metadata of the returned objects.
    #[serde(rename = "Contents", default)]
    pub contents: Vec<ObjectProperties>,

    /// If the Delimiter parameter is specified in the request, the response
    /// contains the CommonPrefixes element.
    #[serde(rename = "CommonPrefixes", default)]
    pub common_prefixes: Vec<CommonPrefix>,

    #[serde(skip)]
    pub common: ResultCommon,
}

/// Decodes the URL-encoded fields of the result when EncodingType is url.
/// Mirrors Go `unmarshalEncodeType` for ListObjectsResult.
fn decode_result(result: &mut ListObjectsResult) {
    let is_url_encoding = result
        .encoding_type
        .as_deref()
        .map(|v| v.eq_ignore_ascii_case("url"))
        .unwrap_or(false);
    if !is_url_encoding {
        return;
    }

    for value in [
        &mut result.prefix,
        &mut result.marker,
        &mut result.delimiter,
        &mut result.next_marker,
    ]
    .into_iter()
    .flatten()
    {
        *value = urlencoding::decode(value)
            .unwrap_or(std::borrow::Cow::Borrowed(value.as_str()))
            .into_owned();
    }

    for obj in &mut result.contents {
        if let Some(key) = &mut obj.key {
            *key = urlencoding::decode(key)
                .unwrap_or(std::borrow::Cow::Borrowed(key.as_str()))
                .into_owned();
        }
    }

    for prefix in &mut result.common_prefixes {
        prefix.prefix = urlencoding::decode(&prefix.prefix)
            .unwrap_or(std::borrow::Cow::Borrowed(prefix.prefix.as_str()))
            .into_owned();
    }
}

impl Client {
    /// Lists the information about all objects in a bucket.
    ///
    /// This method sends a GET request to the server with the specified
    /// parameters and headers. It returns a `Result` containing the
    /// deserialized `ListObjectsResult` or an error.
    ///
    /// # Arguments
    ///
    /// * `request` - A reference to a `ListObjectsRequest` struct that contains
    ///   the request parameters.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::ListObjectsRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListObjectsRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.list_objects(request).await {
    ///     Ok(result) => {
    ///         // Handle result
    ///     }
    ///     Err(err) => {
    ///         // Handle error
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn list_objects(
        &self,
        request: ListObjectsRequest,
    ) -> Result<ListObjectsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListObjects".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("encoding-type", "url")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: ListObjectsResult = quick_xml::de::from_str(&data_str)?;

        result.update_result(&output);

        decode_result(&mut result);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::object::{DeleteObjectRequest, PutObjectRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::test_utils::{generate_unique_object_name, load_test_config};
    use crate::SignatureVersionType;

    #[test]
    fn test_list_objects_result_xml_and_url_decoding() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult>
  <Name>my-bucket</Name>
  <Prefix>dir%2Bname%2F</Prefix>
  <Marker>start%2Bkey</Marker>
  <MaxKeys>100</MaxKeys>
  <Delimiter>%2F</Delimiter>
  <IsTruncated>true</IsTruncated>
  <NextMarker>next%2Bmarker%2Fkey</NextMarker>
  <EncodingType>url</EncodingType>
  <Contents>
    <Key>dir%2Bname%2Fobj%201.txt</Key>
    <Type>Normal</Type>
    <Size>10</Size>
    <ETag>"abc"</ETag>
    <LastModified>2024-01-01T00:00:00.000Z</LastModified>
    <StorageClass>Standard</StorageClass>
  </Contents>
  <CommonPrefixes>
    <Prefix>dir%2Bname%2Fsub%2F</Prefix>
  </CommonPrefixes>
</ListBucketResult>"#;

        let mut result: ListObjectsResult = quick_xml::de::from_str(xml).unwrap();
        decode_result(&mut result);

        assert_eq!(result.name.as_deref(), Some("my-bucket"));
        assert_eq!(result.prefix.as_deref(), Some("dir+name/"));
        assert_eq!(result.marker.as_deref(), Some("start+key"));
        assert_eq!(result.delimiter.as_deref(), Some("/"));
        assert_eq!(result.next_marker.as_deref(), Some("next+marker/key"));
        assert!(result.is_truncated);
        assert_eq!(result.max_keys, 100);
        assert_eq!(result.contents.len(), 1);
        assert_eq!(
            result.contents[0].key.as_deref(),
            Some("dir+name/obj 1.txt")
        );
        assert_eq!(result.common_prefixes.len(), 1);
        assert_eq!(result.common_prefixes[0].prefix, "dir+name/sub/");
    }

    #[test]
    fn test_list_objects_result_no_decoding_without_url_encoding() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult>
  <Name>my-bucket</Name>
  <Prefix>dir%2Bname</Prefix>
  <MaxKeys>100</MaxKeys>
  <IsTruncated>false</IsTruncated>
  <NextMarker>next%2Bmarker</NextMarker>
</ListBucketResult>"#;

        let mut result: ListObjectsResult = quick_xml::de::from_str(xml).unwrap();
        decode_result(&mut result);

        // EncodingType is absent, so values must stay untouched.
        assert_eq!(result.prefix.as_deref(), Some("dir%2Bname"));
        assert_eq!(result.next_marker.as_deref(), Some("next%2Bmarker"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_objects_live() {
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
                .with_signature_version(SignatureVersionType::V4)
                .with_log_level(LogLevel::Debug),
        );

        let object_key = generate_unique_object_name("list-objects-obj");

        // Upload a test object
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: object_key.clone(),
            body: Some(crate::BodyContent::from_bytes(
                b"content for list objects".to_vec(),
                None,
            )),
            ..Default::default()
        };
        if let Err(err) = client.put_object(put_request).await {
            panic!("Failed to upload test object: {:?}", err);
        }

        // List objects filtered by the unique object name
        let list_request = ListObjectsRequest {
            bucket: config.bucket.to_string(),
            prefix: Some(object_key.clone()),
            ..Default::default()
        };

        let list_result = client.list_objects(list_request).await;

        // Clean up the test object regardless of the list result
        let delete_request = DeleteObjectRequest {
            bucket: config.bucket.to_string(),
            key: object_key.clone(),
            ..Default::default()
        };
        let _ = client.delete_object(delete_request).await;

        let result = match list_result {
            Ok(result) => result,
            Err(err) => panic!("List objects failed: {:?}", err),
        };

        assert!(
            result
                .contents
                .iter()
                .any(|obj| obj.key.as_deref() == Some(object_key.as_str())),
            "Expected to find the uploaded object in the list result"
        );
        assert_eq!(result.prefix.as_deref(), Some(object_key.as_str()));
    }
}
