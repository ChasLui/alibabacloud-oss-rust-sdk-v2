use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// Information about a vector bucket.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VectorBucketInfo {
    /// The name of the bucket.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The region in which the bucket is located.
    #[serde(rename = "Location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// The time when the bucket was created.
    #[serde(rename = "CreationDate", skip_serializing_if = "Option::is_none")]
    pub creation_date: Option<String>,

    /// The public endpoint used to access the bucket over the Internet.
    #[serde(rename = "ExtranetEndpoint", skip_serializing_if = "Option::is_none")]
    pub extranet_endpoint: Option<String>,

    /// The internal endpoint used to access the bucket from ECS instances in
    /// the same region.
    #[serde(rename = "IntranetEndpoint", skip_serializing_if = "Option::is_none")]
    pub intranet_endpoint: Option<String>,

    /// The ID of the resource group to which the bucket belongs.
    #[serde(rename = "ResourceGroupId", skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<String>,
}

/// Queries information about a vector bucket.
#[derive(Debug, Default, OssRequestModel)]
pub struct GetVectorBucketRequest {
    /// The name of the bucket to query.
    pub bucket: String,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel, Serialize, Deserialize)]
pub struct GetVectorBucketResult {
    /// The container that stores the bucket information.
    #[serde(rename = "BucketInfo", skip_serializing_if = "Option::is_none")]
    pub bucket_info: Option<VectorBucketInfo>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries information about a vector bucket.
    ///
    /// Requires a client built with [`Client::new_vectors`].
    pub async fn get_vector_bucket(
        &self,
        request: &GetVectorBucketRequest,
    ) -> Result<GetVectorBucketResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetVectorBucket".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            parameters: [("bucketInfo", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/json")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let mut result: GetVectorBucketResult = serde_json::from_slice(&body_data)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_vector_bucket_result_deserialize() {
        let body = r#"{"BucketInfo":{"Name":"my-vector","Location":"cn-hangzhou",
            "ExtranetEndpoint":"my-vector-123.cn-hangzhou.oss-vectors.aliyuncs.com",
            "ResourceGroupId":"rg-123"}}"#;

        let result: GetVectorBucketResult = serde_json::from_str(body).unwrap();

        let info = result.bucket_info.expect("bucket info");
        assert_eq!(info.name.as_deref(), Some("my-vector"));
        assert_eq!(info.location.as_deref(), Some("cn-hangzhou"));
        assert_eq!(info.resource_group_id.as_deref(), Some("rg-123"));
    }

    #[test]
    fn test_get_vector_bucket_result_without_body() {
        let result: GetVectorBucketResult = serde_json::from_str("{}").unwrap();
        assert!(result.bucket_info.is_none());
    }
}
