use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use crate::api::bucket::Owner;
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{acl_grant_de, modify_request};
use crate::{
    OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE,
};
use crate::client::BodyDataReader;


#[derive(Debug, Default, OssRequestModel)]
pub struct GetObjectAclRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// The version ID of the source object.
    #[field(type = "query", rename = "versionId")]
    pub version_id: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Debug, Deserialize, OssResultModel)]
pub struct GetObjectAclResult {
    /// The container that stores the access control list (ACL) information
    /// about the bucket.
    #[serde(rename = "AccessControlList", with = "acl_grant_de")]
    pub acl: Option<String>,

    /// The container that stores information about the bucket owner.
    #[serde(rename = "Owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<Owner>,

    /// Version of the object.
    #[serde(skip)]
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Retrieves the access control list (ACL) for an object in the OSS bucket.
    ///
    /// This method sends a GET request to the OSS server to retrieve the ACL
    /// for the specified object. It returns a `GetObjectAclResult` struct
    /// containing the ACL information.
    ///
    /// # Arguments
    ///
    /// * `request` - A reference to a `GetObjectAclRequest` struct that
    ///   specifies the bucket and key of the object.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `GetObjectAclResult` on success, or a boxed
    /// `dyn std::error::Error` on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::GetObjectAclRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetObjectAclRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_object_acl(&request).await {
    ///     Ok(result) => {
    ///         println!("ACL: {:?}", result.acl);
    ///         println!("Version ID: {:?}", result.version_id);
    ///     }
    ///     Err(err) => {
    ///         eprintln!("Error: {}", err);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_object_acl(
        &self,
        request: &GetObjectAclRequest,
    ) -> Result<GetObjectAclResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetObjectAcl".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("acl", "")]
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
            vec![],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: GetObjectAclResult =
            quick_xml::de::from_str(&data_str)?;

        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::object::{PutObjectRequest, DeleteObjectRequest};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::SignatureVersionType;
    use crate::test_utils::{load_test_config, TestConfig, generate_unique_object_name};

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_object_acl() {
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

        // Step 1: Create a test object
        let test_object_name = generate_unique_object_name("get-object-acl");
        let test_content = "Test content for get object ACL operation";
        
        println!("Step 1: Creating test object '{}'", test_object_name);
        
        let put_request = PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            body: Some(crate::BodyContent::from_text(test_content.to_string(), None)),
            ..Default::default()
        };

        match client.put_object(put_request).await {
            Ok(output) => {
                println!("Object created successfully: {:?}", output);
                assert!(output.etag.is_some(), "Object should have an ETag");
            }
            Err(err) => {
                panic!("Failed to create test object: {:?}", err);
            }
        }

        // Step 2: Get object ACL
        println!("Step 2: Getting ACL for object '{}'", test_object_name);
        
        let get_acl_request = GetObjectAclRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            ..Default::default()
        };

        match client.get_object_acl(&get_acl_request).await {
            Ok(result) => {
                println!("Get object ACL result: {:?}", result);
                
                // Verify the result contains expected fields
                assert!(result.owner.is_some(), "ACL result should contain owner information");
                assert!(result.acl.is_some(), "ACL result should contain ACL information");
                
                // Print ACL details
                if let Some(owner) = &result.owner {
                    println!("Object owner: ID={}, DisplayName={}", 
                             owner.id.as_deref().unwrap_or("N/A"), 
                             owner.display_name.as_deref().unwrap_or("N/A"));
                }
                
                if let Some(acl) = &result.acl {
                    println!("Object ACL: {}", acl);
                }
            }
            Err(err) => {
                panic!("Failed to get object ACL: {:?}", err);
            }
        }

        // Step 3: Clean up - Delete the test object
        println!("Step 3: Cleaning up test object '{}'", test_object_name);
        
        let delete_request = DeleteObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            ..Default::default()
        };

        match client.delete_object(delete_request).await {
            Ok(result) => {
                println!("Object deleted successfully: {:?}", result);
                // Status should be 204 (No Content) for successful deletion
                assert_eq!(result.common.status, http::StatusCode::NO_CONTENT, 
                          "Deletion should return 204 No Content status");
            }
            Err(err) => {
                eprintln!("Warning: Failed to delete test object: {:?}", err);
                // Don't panic here as the main test objective (get ACL) was achieved
            }
        }

        println!("Get object ACL test completed successfully!");
    }
}
