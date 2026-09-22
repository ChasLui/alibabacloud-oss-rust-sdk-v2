use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5, xml_escape_str_ser};
use crate::{BodyContent, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, Serialize, OssRequestModel)]
pub struct DeleteMultipleObjectsRequest {
    /// The name of the bucket.
    #[serde(skip)]
    pub bucket: String,

    /// The encoding type of the object names in the response. Valid value: url
    #[serde(skip)]
    #[field(type = "query", rename = "encoding-type")]
    pub encoding_type: Option<String>,

    /// The size of the data in the HTTP message body. Unit: bytes.
    #[serde(skip)]
    #[field(type = "header", rename = "Content-Length")]
    pub content_length: Option<u64>,

    /// The container that stores information about you want to delete objects.
    #[serde(rename = "Object", default)]
    pub objects: Vec<DeleteObject>,

    /// Specifies whether to enable the Quiet return mode.
    /// The DeleteMultipleObjects operation provides the following return modes:
    /// Valid value: true, false
    #[serde(rename = "Quiet")]
    pub quiet: bool,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    #[serde(skip)]
    pub common: RequestCommon,
}

#[derive(Debug, Deserialize, OssResultModel)]
pub struct DeleteMultipleObjectsResult {
    /// The container that stores information about the deleted objects.
    #[serde(rename = "Deleted", default)]
    pub deleted_objects: Vec<DeleteInfo>,

    /// The encoding type of the name of the deleted object in the response.
    /// If encoding-type is specified in the request, the object name is encoded
    /// in the returned result.
    #[serde(rename = "EncodingType", skip_serializing_if = "Option::is_none")]
    pub encoding_type: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

#[derive(Debug, Default, Serialize)]
pub struct DeleteObject {
    /// The name of the object.
    #[serde(rename = "Key", with = "xml_escape_str_ser")]
    pub key: String,

    /// The version ID of the source object.
    #[serde(rename = "VersionId", skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct DeleteInfo {
    /// The name of the object.
    #[serde(rename = "Key")]
    pub key: String,

    /// The version ID of the source object.
    #[serde(rename = "VersionId", skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,

    /// Indicates whether the deleted version is a delete marker.
    #[serde(rename = "DeleteMarker", skip_serializing_if = "Option::is_none")]
    pub delete_marker: Option<bool>,

    /// The version ID of the delete marker.
    #[serde(
        rename = "DeleteMarkerVersionId",
        skip_serializing_if = "Option::is_none"
    )]
    pub delete_marker_version_id: Option<String>,
}

impl Client {
    /// Deletes multiple objects from the OSS bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteMultipleObjectsRequest` containing the bucket
    ///   name and object keys to delete.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `DeleteMultipleObjectsResult` if the
    /// operation is successful, or a boxed `dyn std::error::Error` if an error
    /// occurs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::{
    /// #     DeleteMultipleObjectsRequest, DeleteObject, GetObjectAclRequest,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteMultipleObjectsRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     objects: vec![
    ///         DeleteObject {
    ///             key: "obj1".to_string(),
    ///             ..Default::default()
    ///         },
    ///         DeleteObject {
    ///             key: "obj2".to_string(),
    ///             ..Default::default()
    ///         },
    ///     ],
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_multiple_objects(request).await {
    ///     Ok(result) => {
    ///         // handle result
    ///     }
    ///     Err(err) => {
    ///         eprintln!("Error: {}", err);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_multiple_objects(
        &self,
        request: DeleteMultipleObjectsRequest,
    ) -> Result<DeleteMultipleObjectsResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteMultipleObjects".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("delete", ""), ("encoding-type", "url")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.body = Some(BodyContent::from_text(
            quick_xml::se::to_string_with_root("Delete", &request)?,
            None, // MD5 will be calculated automatically for Text
        ));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: DeleteMultipleObjectsResult = quick_xml::de::from_str(&data_str)?;

        result.update_result(&output);

        // Decode object keys when the response is URL-encoded. Mirrors Go
        // `unmarshalEncodeType` for DeleteMultipleObjectsResult.
        let is_url_encoding = result
            .encoding_type
            .as_deref()
            .map(|v| v.eq_ignore_ascii_case("url"))
            .unwrap_or(false);
        if is_url_encoding {
            for deleted_obj in &mut result.deleted_objects {
                deleted_obj.key = urlencoding::decode(&deleted_obj.key)
                    .unwrap_or_else(|_| std::borrow::Cow::Borrowed(&deleted_obj.key))
                    .to_string();
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore = "avoid testing delete-object exclusively, tested in `test_put_get_delete_object`"]
    async fn test_delete_multiple_objects() {
        unimplemented!("no need for testing here")
    }
}
