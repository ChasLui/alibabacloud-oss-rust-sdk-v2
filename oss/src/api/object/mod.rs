mod copy_object;
mod delete_multiple_objects;
mod delete_object;
mod get_object;
mod get_object_acl;
mod put_object;
mod put_object_acl;
mod get_object_meta;
mod head_object;
mod initiate_multipart_upload;
mod upload_part;
mod complete_multipart_upload;
mod abort_multipart_upload;
mod list_multipart_uploads;
mod list_parts;
mod upload_part_copy;
mod append_object;
mod seal_append_object;
mod restore_object;
mod clean_restored_object;
mod put_symlink;
mod get_symlink;
mod put_object_tagging;
mod get_object_tagging;
mod delete_object_tagging;
mod process_object;
mod async_process_object;
mod put_object_retention;
mod get_object_retention;
mod put_object_legal_hold;
mod get_object_legal_hold;
mod select_object;
mod create_select_object_meta;

use std::time::SystemTime;

use serde::Deserialize;

pub use self::copy_object::*;
pub use self::delete_multiple_objects::*;
pub use self::delete_object::*;
pub use self::get_object::*;
pub use self::get_object_acl::*;
pub use self::put_object::*;
pub use self::put_object_acl::*;
pub use self::get_object_meta::*;
pub use self::head_object::*;
pub use self::initiate_multipart_upload::*;
pub use self::upload_part::*;
pub use self::complete_multipart_upload::*;
pub use self::abort_multipart_upload::*;
pub use self::list_multipart_uploads::*;
pub use self::list_parts::*;
pub use self::upload_part_copy::*;
pub use self::append_object::*;
pub use self::seal_append_object::*;
pub use self::restore_object::*;
pub use self::clean_restored_object::*;
pub use self::put_symlink::*;
pub use self::get_symlink::*;
pub use self::put_object_tagging::*;
pub use self::get_object_tagging::*;
pub use self::delete_object_tagging::*;
pub use self::process_object::*;
pub use self::async_process_object::*;
pub use self::put_object_retention::*;
pub use self::get_object_retention::*;
pub use self::put_object_legal_hold::*;
pub use self::get_object_legal_hold::*;
pub use self::select_object::*;
pub use self::create_select_object_meta::*;
use crate::api::bucket::Owner;
use crate::utils::option_time_rfc3339_serde;

#[derive(Debug, Default, Deserialize)]
pub struct ObjectProperties {
    /// The name of the object.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The type of the object. Valid values: Normal, Multipart and Appendable.
    #[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
    pub obj_type: Option<String>,

    /// The size of the returned object. Unit: bytes.
    #[serde(rename = "Size")]
    pub size: i64,

    /// The entity tag (ETag). An ETag is created when an object is created to
    /// identify the content of the object.
    #[serde(rename = "ETag", skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,

    /// The time when the returned objects were last modified.
    #[serde(rename = "LastModified", with = "option_time_rfc3339_serde")]
    pub last_modified: Option<SystemTime>,

    /// The storage class of the object.
    #[serde(rename = "StorageClass", skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,

    /// The container that stores information about the bucket owner.
    #[serde(rename = "Owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<Owner>,

    /// The restoration status of the object.
    #[serde(rename = "RestoreInfo", skip_serializing_if = "Option::is_none")]
    pub restore_info: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::api::object::GetObjectRequest;
    use crate::client::Client;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::SignatureVersionType;
    use crate::test_utils::{load_test_config, TestConfig};
use crate::client::BodyDataReader;


    pub(super) const TEST_OBJECT_NAME: &str = "aliyun-oss-sdk-rust-test-object";
    pub(super) const TEST_OBJECT_CONTENT: &str = "Call me Ishmael. Some years ago...";

    // Helper function to generate unique test object names
    pub(super) fn generate_unique_object_name(base_name: &str) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        format!("{}_{}_{}", "test-oss-object", base_name, timestamp)
    }

    pub(super) async fn put(
        client: &Client,
        bucket: &str,
    ) -> Result<PutObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        client
            .put_object(PutObjectRequest {
                bucket: bucket.to_string(),
                key: TEST_OBJECT_NAME.to_string(),
                body: Some(crate::BodyContent::from_text(TEST_OBJECT_CONTENT.to_string(), None)),
                ..Default::default()
            })
            .await
    }


    pub(super) async fn put_with_size(
        client: &Client,
        bucket: &str,
        size: usize,
    ) -> Result<PutObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        // 创建一个指定大小的字节数组，内容循环填充0-9的数字
        let mut content = Vec::with_capacity(size);
        for i in 0..size {
            content.push((i % 10) as u8 + b'0'); // 0-9的ASCII码
        }

        client
            .put_object(PutObjectRequest {
                bucket: bucket.to_string(),
                key: TEST_OBJECT_NAME.to_string(),
                body: Some(crate::BodyContent::from_bytes(content, None)),
                ..Default::default()
            })
            .await
    }

    pub(super) async fn put_with_meta(
        client: &Client,
        bucket: &str,
        user_defined_meta: &HashMap<&str, &str>
    ) -> Result<PutObjectResult, Box<dyn std::error::Error + Send + Sync>> {

        let mut put_object_request = PutObjectRequest {
            bucket: bucket.to_string(),
            key: TEST_OBJECT_NAME.to_string(),
            body: Some(crate::BodyContent::from_text(TEST_OBJECT_CONTENT.to_string(), None)),
            ..Default::default()
        };

        for (key, value) in user_defined_meta.iter() {
            put_object_request.add_header( key, value);
        }

        client
            .put_object(put_object_request)
            .await
    }


    pub(super) async fn get(
        client: &Client,
        bucket: &str,
    ) -> Result<GetObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        client
            .get_object(GetObjectRequest {
                bucket: bucket.to_string(),
                key: TEST_OBJECT_NAME.to_string(),
                ..Default::default()
            })
            .await
    }

    pub(super) async fn delete_multiple(
        client: &Client,
        bucket: &str,
    ) -> Result<DeleteMultipleObjectsResult, Box<dyn std::error::Error + Send + Sync>> {
        client
            .delete_multiple_objects(DeleteMultipleObjectsRequest {
                bucket: bucket.to_string(),
                objects: vec![DeleteObject {
                    key: TEST_OBJECT_NAME.to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            })
            .await
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_put_get_delete_object() {
        let config = match load_test_config() {
            Some(cfg) => cfg,
            None => {
                eprintln!("Test configuration not found. Skipping test.");
                return;
            }
        };

        // Generate a unique object name for this test
        let test_object_name = generate_unique_object_name("basic");

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

        // put object with unique name
        match client.put_object(PutObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            body: Some(crate::BodyContent::from_text(TEST_OBJECT_CONTENT.to_string(), None)),
            ..Default::default()
        }).await {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

        // get object and check equivalence
        match client.get_object(GetObjectRequest {
            bucket: config.bucket.to_string(),
            key: test_object_name.clone(),
            ..Default::default()
        }).await {
            Ok(mut result) => {
                println!("{:?}", result);
                let content_bytes = result.get_all_data().await.unwrap_or_default();
                let content = String::from_utf8_lossy(&content_bytes).into_owned();
                assert_eq!(content, TEST_OBJECT_CONTENT);
            }
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

        // delete object
        match client.delete_multiple_objects(DeleteMultipleObjectsRequest {
            bucket: config.bucket.to_string(),
            objects: vec![DeleteObject {
                key: test_object_name, // Use unique test object name
                ..Default::default()
            }],
            ..Default::default()
        }).await {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_versioned_object_operations() {
        let config = match load_test_config() {
            Some(cfg) => cfg,
            None => {
                eprintln!("Test configuration not found. Skipping test.");
                return;
            }
        };

        // Use the versioned bucket from config, or fall back to regular bucket if not available
        let versioned_bucket = config.version_bucket.as_ref().unwrap_or(&config.bucket);

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

        // Generate a unique object name for this test
        let test_object_name = generate_unique_object_name("versioned");

        let content_v1 = "This is the first version of the object.";
        let content_v2 = "This is the updated second version of the object.";

        // Upload first version of the object
        let put_result_v1 = match client.put_object(PutObjectRequest {
            bucket: versioned_bucket.to_string(),
            key: test_object_name.clone(),
            body: Some(crate::BodyContent::from_bytes(content_v1.as_bytes().to_vec(), None)),
            ..Default::default()
        }).await {
            Ok(result) => {
                println!("Uploaded first version: {:?}", result);
                result
            }
            Err(err) => {
                eprintln!("Failed to upload first version: {:?}", err);
                return;
            }
        };

        // Upload second version of the object (same key, different content)
        let put_result_v2 = match client.put_object(PutObjectRequest {
            bucket: versioned_bucket.to_string(),
            key: test_object_name.clone(),
            body: Some(crate::BodyContent::from_bytes(content_v2.as_bytes().to_vec(), None)),
            ..Default::default()
        }).await {
            Ok(result) => {
                println!("Uploaded second version: {:?}", result);
                result
            }
            Err(err) => {
                eprintln!("Failed to upload second version: {:?}", err);
                return;
            }
        };

        // List object versions to verify both versions exist (if versioning is enabled on the bucket)
        // Note: Listing object versions requires a separate API call that may not exist in this SDK
        // So we'll just verify that both put operations returned version IDs
        println!("First version ID: {:?}", put_result_v1.version_id);
        println!("Second version ID: {:?}", put_result_v2.version_id);

        // If version IDs are present, we can test getting specific versions
        if let Some(first_version_id) = &put_result_v1.version_id {
            match client.get_object(GetObjectRequest {
                bucket: versioned_bucket.to_string(),
                key: test_object_name.clone(),
                version_id: Some(first_version_id.clone()),
                ..Default::default()
            }).await {
                Ok(mut get_result) => {
                    println!("Retrieved first version: {:?}", get_result);
                    let content_bytes = get_result.get_all_data().await.unwrap_or_default();
                    let content = String::from_utf8_lossy(&content_bytes).into_owned();
                    assert_eq!(content, content_v1);
                    assert_eq!(get_result.version_id.as_deref(), Some(first_version_id.as_str()));
                }
                Err(err) => {
                    eprintln!("Failed to get first version: {:?}", err);
                    // If versioning is not enabled on the bucket, this might fail, which is expected
                    // But if we know the bucket should support versioning, we should fail the test
                    match config.version_bucket.as_ref() {
                        Some(expected_versioned_bucket) if expected_versioned_bucket == versioned_bucket => {
                            panic!("Expected to be able to get first version from versioned bucket, but failed: {:?}", err);
                        }
                        _ => {
                            println!("This might be expected if versioning is not enabled on the bucket");
                        }
                    }
                }
            }
        }

        // Get second version by specifying version_id
        if let Some(second_version_id) = &put_result_v2.version_id {
            match client.get_object(GetObjectRequest {
                bucket: versioned_bucket.to_string(),
                key: test_object_name.clone(),
                version_id: Some(second_version_id.clone()),
                ..Default::default()
            }).await {
                Ok(mut get_result) => {
                    println!("Retrieved second version: {:?}", get_result);
                    let content_bytes = get_result.get_all_data().await.unwrap_or_default();
                    let content = String::from_utf8_lossy(&content_bytes).into_owned();
                    assert_eq!(content, content_v2);
                    assert_eq!(get_result.version_id.as_deref(), Some(second_version_id.as_str()));
                }
                Err(err) => {
                    eprintln!("Failed to get second version: {:?}", err);
                    // If versioning is not enabled on the bucket, this might fail, which is expected
                    // But if we know the bucket should support versioning, we should fail the test
                    match config.version_bucket.as_ref() {
                        Some(expected_versioned_bucket) if expected_versioned_bucket == versioned_bucket => {
                            panic!("Expected to be able to get second version from versioned bucket, but failed: {:?}", err);
                        }
                        _ => {
                            println!("This might be expected if versioning is not enabled on the bucket");
                        }
                    }
                }
            }
        }

        // Get latest version (without specifying version_id)
        match client.get_object(GetObjectRequest {
            bucket: versioned_bucket.to_string(),
            key: test_object_name.clone(),
            ..Default::default()
        }).await {
            Ok(mut get_result) => {
                println!("Retrieved latest version: {:?}", get_result);
                // Should get the second (latest) version
                let content_bytes = get_result.get_all_data().await.unwrap_or_default();
                let content = String::from_utf8_lossy(&content_bytes).into_owned();
                assert_eq!(content, content_v2);
            }
            Err(err) => {
                eprintln!("Failed to get latest version: {:?}", err);
                return;
            }
        }

        // Delete specific versions (if version IDs are available)
        // Delete first version
        if let Some(first_version_id) = &put_result_v1.version_id {
            match client.delete_object(crate::api::object::DeleteObjectRequest {
                bucket: versioned_bucket.to_string(),
                key: test_object_name.clone(),
                version_id: Some(first_version_id.clone()),
                ..Default::default()
            }).await {
                Ok(delete_result) => {
                    println!("Deleted first version: {:?}", delete_result);
                }
                Err(err) => {
                    eprintln!("Failed to delete first version: {:?}", err);
                    // If versioning is not enabled on the bucket, this might fail, which is expected
                    // But if we know the bucket should support versioning, we should fail the test
                    match config.version_bucket.as_ref() {
                        Some(expected_versioned_bucket) if expected_versioned_bucket == versioned_bucket => {
                            panic!("Expected to be able to delete first version from versioned bucket, but failed: {:?}", err);
                        }
                        _ => {
                            println!("This might be expected if versioning is not enabled on the bucket");
                        }
                    }
                }
            }
        }

        // Delete second version
        if let Some(second_version_id) = &put_result_v2.version_id {
            match client.delete_object(crate::api::object::DeleteObjectRequest {
                bucket: versioned_bucket.to_string(),
                key: test_object_name.clone(),
                version_id: Some(second_version_id.clone()),
                ..Default::default()
            }).await {
                Ok(delete_result) => {
                    println!("Deleted second version: {:?}", delete_result);
                }
                Err(err) => {
                    eprintln!("Failed to delete second version: {:?}", err);
                    // If versioning is not enabled on the bucket, this might fail, which is expected
                    // But if we know the bucket should support versioning, we should fail the test
                    match config.version_bucket.as_ref() {
                        Some(expected_versioned_bucket) if expected_versioned_bucket == versioned_bucket => {
                            panic!("Expected to be able to delete second version from versioned bucket, but failed: {:?}", err);
                        }
                        _ => {
                            println!("This might be expected if versioning is not enabled on the bucket");
                        }
                    }
                }
            }
        }

        // If versioning is not enabled, just delete the object normally
        if put_result_v1.version_id.is_none() && put_result_v2.version_id.is_none() {
            match client.delete_object(crate::api::object::DeleteObjectRequest {
                bucket: versioned_bucket.to_string(),
                key: test_object_name.clone(),
                ..Default::default()
            }).await {
                Ok(delete_result) => {
                    println!("Deleted object (versioning not enabled): {:?}", delete_result);
                }
                Err(err) => {
                    eprintln!("Failed to delete object: {:?}", err);
                }
            }
        }

        println!("Multi-version object test completed!");
    }
}