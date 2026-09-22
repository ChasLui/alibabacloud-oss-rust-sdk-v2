use std::time::SystemTime;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::{CommonPrefix, Owner};
use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, option_time_rfc3339_serde, update_content_md5};
use crate::{OperationInput, OperationOutput};
#[derive(Debug, Default, Clone, OssRequestModel)]
pub struct ListObjectVersionsRequest {
    /// The name of the bucket containing the objects.
    pub bucket: String,

    /// The character that is used to group objects by name. If you specify
    /// the delimiter parameter in the request, the response contains the
    /// CommonPrefixes parameter. The objects whose names contain the same
    /// string from the prefix to the next occurrence of the delimiter are
    /// grouped as a single result element in CommonPrefixes.
    #[field(type = "query")]
    pub delimiter: Option<String>,

    /// Specifies that objects whose names are alphabetically after the value
    /// of the key-marker parameter are returned. This parameter can be
    /// specified together with version-id-marker. By default, this parameter
    /// is left empty.
    #[field(type = "query", rename = "key-marker")]
    pub key_marker: Option<String>,

    /// Specifies that the versions created before the version specified by
    /// version-id-marker for the object whose name is specified by
    /// key-marker are returned by creation time in descending order.
    #[field(type = "query", rename = "version-id-marker")]
    pub version_id_marker: Option<String>,

    /// The maximum number of objects that you want to return. If the list
    /// operation cannot be complete at a time because the max-keys
    /// parameter is specified, the NextMarker element is included in the
    /// response as the marker for the next list operation.
    #[field(type = "query", rename = "max-keys")]
    pub max_keys: Option<i32>,

    /// The prefix that the names of the returned objects must contain.
    #[field(type = "query")]
    pub prefix: Option<String>,

    /// The encoding type of the content in the response. Valid value: url.
    #[field(type = "query", rename = "encoding-type")]
    pub encoding_type: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs.
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    /// To indicate that whether to stores the versions of objects and delete
    /// markers together in one container.
    /// When false(default), the versions of objects are stored into
    /// ListObjectVersionsResult.object_versions, and the delete markers into
    /// ListObjectVersionsResult.object_delete_markers.
    /// When true, the versions and delete markers are stored into
    /// ListObjectVersionsResult.object_versions_delete_markers.
    pub is_mix: bool,

    pub common: RequestCommon,
}

/// The container that stores delete markers.
#[derive(Debug, Deserialize)]
pub struct ObjectDeleteMarkerProperties {
    /// The name of the object.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The version ID of the object.
    #[serde(rename = "VersionId", skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,

    /// Indicates whether the version is the current version.
    #[serde(rename = "IsLatest")]
    pub is_latest: bool,

    /// The time when the returned objects were last modified.
    #[serde(rename = "LastModified", with = "option_time_rfc3339_serde", default)]
    pub last_modified: Option<SystemTime>,

    /// The container that stores information about the bucket owner.
    #[serde(rename = "Owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<Owner>,
}

/// The container that stores the versions of objects, excluding delete
/// markers.
#[derive(Debug, Deserialize)]
pub struct ObjectVersionProperties {
    /// The name of the object.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The version ID of the object.
    #[serde(rename = "VersionId", skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,

    /// Indicates whether the version is the current version.
    #[serde(rename = "IsLatest")]
    pub is_latest: bool,

    /// The time when the returned objects were last modified.
    #[serde(rename = "LastModified", with = "option_time_rfc3339_serde", default)]
    pub last_modified: Option<SystemTime>,

    /// The type of the returned object.
    #[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
    pub obj_type: Option<String>,

    /// The size of the returned object. Unit: bytes.
    #[serde(rename = "Size", skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    /// The entity tag (ETag) that is generated when an object is created.
    /// ETags are used to identify the content of objects.
    #[serde(rename = "ETag", skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,

    /// The storage class of the object.
    #[serde(rename = "StorageClass", skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,

    /// The container that stores information about the bucket owner.
    #[serde(rename = "Owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<Owner>,

    /// The restoration status of the object.
    #[serde(rename = "RestoreInfo", skip_serializing_if = "Option::is_none")]
    pub restore_info: Option<String>,

    /// The time when the storage class of the object is converted to Cold
    /// Archive or Deep Cold Archive based on lifecycle rules.
    #[serde(rename = "TransitionTime", with = "option_time_rfc3339_serde", default)]
    pub transition_time: Option<SystemTime>,
}

impl ObjectVersionProperties {
    /// Checks whether this entry is a delete marker. Only meaningful for
    /// entries in `object_versions_delete_markers` (IsMix mode).
    pub fn is_delete_marker(&self) -> bool {
        self.version_id.is_some() && self.obj_type.is_none()
    }
}

/// The container that stores the versions of objects and delete markers
/// together in the order they are returned. Only valid when
/// ListObjectVersionsRequest.is_mix is set to true.
pub type ObjectMixProperties = ObjectVersionProperties;

#[derive(Debug, Deserialize, OssResultModel)]
pub struct ListObjectVersionsResult {
    /// The name of the bucket.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Indicates the object from which the ListObjectVersions
    /// (GetBucketVersions) operation starts.
    #[serde(rename = "KeyMarker", skip_serializing_if = "Option::is_none")]
    pub key_marker: Option<String>,

    /// The version from which the ListObjectVersions (GetBucketVersions)
    /// operation starts. This parameter is used together with KeyMarker.
    #[serde(rename = "VersionIdMarker", skip_serializing_if = "Option::is_none")]
    pub version_id_marker: Option<String>,

    /// If not all results are returned for the request, the NextKeyMarker
    /// parameter is included in the response to indicate the key-marker
    /// value of the next ListObjectVersions (GetBucketVersions) request.
    #[serde(rename = "NextKeyMarker", skip_serializing_if = "Option::is_none")]
    pub next_key_marker: Option<String>,

    /// If not all results are returned for the request, the
    /// NextVersionIdMarker parameter is included in the response to indicate
    /// the version-id-marker value of the next ListObjectVersions
    /// (GetBucketVersions) request.
    #[serde(
        rename = "NextVersionIdMarker",
        skip_serializing_if = "Option::is_none"
    )]
    pub next_version_id_marker: Option<String>,

    /// The container that stores delete markers.
    #[serde(rename = "DeleteMarker", default)]
    pub object_delete_markers: Vec<ObjectDeleteMarkerProperties>,

    /// The container that stores the versions of objects, excluding delete
    /// markers.
    #[serde(rename = "Version", default)]
    pub object_versions: Vec<ObjectVersionProperties>,

    /// The container that stores the versions of objects and delete markers
    /// together in the order they are returned. Only valid when
    /// ListObjectVersionsRequest.is_mix is set to true.
    #[serde(rename = "ObjectMix", default)]
    pub object_versions_delete_markers: Vec<ObjectMixProperties>,

    /// The prefix contained in the returned object names.
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

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

    /// The encoding type of the content in the response.
    #[serde(rename = "EncodingType", skip_serializing_if = "Option::is_none")]
    pub encoding_type: Option<String>,

    /// If the Delimiter parameter is specified in the request, the response
    /// contains the CommonPrefixes element.
    #[serde(rename = "CommonPrefixes", default)]
    pub common_prefixes: Vec<CommonPrefix>,

    #[serde(skip)]
    pub common: ResultCommon,
}

/// Decodes the URL-encoded fields of the result when EncodingType is url.
/// Mirrors Go `unmarshalEncodeType` for ListObjectVersionsResult.
fn decode_result(result: &mut ListObjectVersionsResult) {
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
        &mut result.key_marker,
        &mut result.delimiter,
        &mut result.next_key_marker,
    ]
    .into_iter()
    .flatten()
    {
        *value = urlencoding::decode(value)
            .unwrap_or(std::borrow::Cow::Borrowed(value.as_str()))
            .into_owned();
    }

    for version in &mut result.object_versions {
        if let Some(key) = &mut version.key {
            *key = urlencoding::decode(key)
                .unwrap_or(std::borrow::Cow::Borrowed(key.as_str()))
                .into_owned();
        }
    }

    for marker in &mut result.object_delete_markers {
        if let Some(key) = &mut marker.key {
            *key = urlencoding::decode(key)
                .unwrap_or(std::borrow::Cow::Borrowed(key.as_str()))
                .into_owned();
        }
    }

    for mix in &mut result.object_versions_delete_markers {
        if let Some(key) = &mut mix.key {
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
    /// Lists the versions of all objects in a bucket, including delete
    /// markers.
    ///
    /// This method sends a GET request to the server with the specified
    /// parameters and headers. It returns a `Result` containing the
    /// deserialized `ListObjectVersionsResult` or an error.
    ///
    /// # Arguments
    ///
    /// * `request` - A reference to a `ListObjectVersionsRequest` struct that
    ///   contains the request parameters.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::ListObjectVersionsRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = ListObjectVersionsRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.list_object_versions(request).await {
    ///     Ok(result) => {
    ///         // Handle result
    ///     }
    ///     Err(err) => {
    ///         // Handle error
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn list_object_versions(
        &self,
        request: ListObjectVersionsRequest,
    ) -> Result<ListObjectVersionsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "ListObjectVersions".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("versions", ""), ("encoding-type", "url")]
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

        // When IsMix is set, the versions and delete markers are stored
        // together in one container: rewrite the Version/DeleteMarker
        // elements to ObjectMix before deserialization, mirroring Go
        // `unmarshalBodyXmlVersions`.
        let mut result: ListObjectVersionsResult = if request.is_mix {
            let replaced = data_str
                .replace("<Version>", "<ObjectMix>")
                .replace("</Version>", "</ObjectMix>")
                .replace("<DeleteMarker>", "<ObjectMix>")
                .replace("</DeleteMarker>", "</ObjectMix>");
            quick_xml::de::from_str(&replaced)?
        } else {
            quick_xml::de::from_str(&data_str)?
        };

        result.update_result(&output);

        decode_result(&mut result);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_list_object_versions_result_xml_and_url_decoding() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListVersionsResult>
  <Name>my-bucket</Name>
  <Prefix>dir%2Bname%2F</Prefix>
  <KeyMarker>start%2Bkey</KeyMarker>
  <VersionIdMarker>v1</VersionIdMarker>
  <NextKeyMarker>next%2Bkey</NextKeyMarker>
  <NextVersionIdMarker>v2</NextVersionIdMarker>
  <MaxKeys>100</MaxKeys>
  <Delimiter>%2F</Delimiter>
  <IsTruncated>true</IsTruncated>
  <EncodingType>url</EncodingType>
  <Version>
    <Key>obj%201.txt</Key>
    <VersionId>v100</VersionId>
    <IsLatest>true</IsLatest>
    <LastModified>2024-01-01T00:00:00.000Z</LastModified>
    <Type>Normal</Type>
    <Size>10</Size>
    <ETag>"abc"</ETag>
    <StorageClass>Standard</StorageClass>
  </Version>
  <DeleteMarker>
    <Key>obj%202.txt</Key>
    <VersionId>v200</VersionId>
    <IsLatest>false</IsLatest>
    <LastModified>2024-01-02T00:00:00.000Z</LastModified>
  </DeleteMarker>
  <CommonPrefixes>
    <Prefix>dir%2Bname%2Fsub%2F</Prefix>
  </CommonPrefixes>
</ListVersionsResult>"#;

        let mut result: ListObjectVersionsResult = quick_xml::de::from_str(xml).unwrap();
        decode_result(&mut result);

        assert_eq!(result.name.as_deref(), Some("my-bucket"));
        assert_eq!(result.prefix.as_deref(), Some("dir+name/"));
        assert_eq!(result.key_marker.as_deref(), Some("start+key"));
        assert_eq!(result.version_id_marker.as_deref(), Some("v1"));
        assert_eq!(result.next_key_marker.as_deref(), Some("next+key"));
        assert_eq!(result.next_version_id_marker.as_deref(), Some("v2"));
        assert_eq!(result.delimiter.as_deref(), Some("/"));
        assert!(result.is_truncated);
        assert_eq!(result.object_versions.len(), 1);
        assert_eq!(result.object_versions[0].key.as_deref(), Some("obj 1.txt"));
        assert_eq!(
            result.object_versions[0].version_id.as_deref(),
            Some("v100")
        );
        assert!(result.object_versions[0].is_latest);
        assert_eq!(result.object_delete_markers.len(), 1);
        assert_eq!(
            result.object_delete_markers[0].key.as_deref(),
            Some("obj 2.txt")
        );
        assert!(result.object_versions_delete_markers.is_empty());
        assert_eq!(result.common_prefixes.len(), 1);
        assert_eq!(result.common_prefixes[0].prefix, "dir+name/sub/");
    }

    #[test]
    fn test_list_object_versions_is_mix_rewrite() {
        // Mirrors Go `unmarshalBodyXmlVersions`: Version/DeleteMarker
        // elements are rewritten to ObjectMix before deserialization.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListVersionsResult>
  <Name>my-bucket</Name>
  <MaxKeys>100</MaxKeys>
  <IsTruncated>false</IsTruncated>
  <EncodingType>url</EncodingType>
  <Version>
    <Key>obj.txt</Key>
    <VersionId>v100</VersionId>
    <IsLatest>false</IsLatest>
    <Type>Normal</Type>
    <Size>10</Size>
  </Version>
  <DeleteMarker>
    <Key>obj.txt</Key>
    <VersionId>v200</VersionId>
    <IsLatest>true</IsLatest>
  </DeleteMarker>
</ListVersionsResult>"#;

        let replaced = xml
            .replace("<Version>", "<ObjectMix>")
            .replace("</Version>", "</ObjectMix>")
            .replace("<DeleteMarker>", "<ObjectMix>")
            .replace("</DeleteMarker>", "</ObjectMix>");
        let mut result: ListObjectVersionsResult = quick_xml::de::from_str(&replaced).unwrap();
        decode_result(&mut result);

        assert!(result.object_versions.is_empty());
        assert!(result.object_delete_markers.is_empty());
        assert_eq!(result.object_versions_delete_markers.len(), 2);
        assert!(!result.object_versions_delete_markers[0].is_delete_marker());
        assert!(result.object_versions_delete_markers[1].is_delete_marker());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_list_object_versions_live() {
        let config = match load_test_config() {
            Some(cfg) => cfg,
            None => {
                eprintln!("Test configuration not found. Skipping test.");
                return;
            }
        };
        let version_bucket = match &config.version_bucket {
            Some(bucket) => bucket.clone(),
            None => {
                eprintln!("Version bucket not configured. Skipping test.");
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

        let request = ListObjectVersionsRequest {
            bucket: version_bucket,
            max_keys: Some(10),
            ..Default::default()
        };

        match client.list_object_versions(request).await {
            Ok(result) => {
                println!(
                    "List object versions: {} versions, {} delete markers, truncated: {}",
                    result.object_versions.len(),
                    result.object_delete_markers.len(),
                    result.is_truncated
                );
            }
            Err(err) => panic!("List object versions failed: {:?}", err),
        }
    }
}
