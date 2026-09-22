//! Bucket Management Example - Demonstrates OSS bucket management operations
//!
//! Features include:
//! - Create bucket
//! - Get bucket information
//! - Set bucket ACL
//! - Get bucket ACL
//! - List objects in bucket
//!
//! Run command:
//! ```bash
//! cargo run --example 03_bucket_management
//! ```

use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    GetBucketAclRequest, GetBucketInfoRequest, ListObjectsV2Request,
};
use alibabacloud_oss_sdk_rust_v2::api::object::{DeleteObjectRequest, PutObjectRequest};
use alibabacloud_oss_sdk_rust_v2::client::Client;
use alibabacloud_oss_sdk_rust_v2::config::Config;
use alibabacloud_oss_sdk_rust_v2::credential::providers::StaticCredentialsProvider;
use alibabacloud_oss_sdk_rust_v2::BodyContent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let access_key_id =
        std::env::var("ACCESS_KEY_ID").unwrap_or_else(|_| "your-access-key-id".to_string());
    let access_key_secret =
        std::env::var("ACCESS_KEY_SECRET").unwrap_or_else(|_| "your-access-key-secret".to_string());
    let region = std::env::var("OSS_REGION").unwrap_or_else(|_| "cn-hangzhou".to_string());

    println!("OSS Bucket Management Example");
    println!("===========================================");

    let config = Config::default()
        .with_region(&region)
        .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
            &access_key_id,
            &access_key_secret,
            &[],
        )))
        .with_signature_version(alibabacloud_oss_sdk_rust_v2::SignatureVersionType::V4);

    let client = Client::new(&config);

    // Note: The following examples use a test bucket, modify according to your
    // situation
    let test_bucket =
        std::env::var("OSS_BUCKET").unwrap_or_else(|_| "your-test-bucket".to_string());

    // Example 1: Get bucket information
    println!("\n1. Get Bucket Information");
    let info_request = GetBucketInfoRequest {
        bucket: test_bucket.clone(),
        ..Default::default()
    };

    match client.get_bucket_info(&info_request).await {
        Ok(result) => {
            println!("   Bucket Information:");
            println!("      Name: {:?}", result.bucket_info.name);
            println!("      Location: {:?}", result.bucket_info.location);
            println!(
                "      Storage class: {:?}",
                result.bucket_info.storage_class
            );
            println!(
                "      Creation date: {:?}",
                result.bucket_info.creation_date
            );
            println!(
                "      Extranet endpoint: {:?}",
                result.bucket_info.extranet_endpoint
            );
            println!(
                "      Intranet endpoint: {:?}",
                result.bucket_info.intranet_endpoint
            );
        }
        Err(e) => {
            println!("   Failed to get bucket information: {}", e);
        }
    }

    // Example 2: Get bucket ACL
    println!("\n2. Get Bucket ACL (Access Control List)");
    let acl_request = GetBucketAclRequest {
        bucket: test_bucket.clone(),
        ..Default::default()
    };

    match client.get_bucket_acl(&acl_request).await {
        Ok(result) => {
            println!("   Bucket ACL: {:?}", result.acl);
        }
        Err(e) => {
            println!("   Failed to get ACL: {}", e);
        }
    }

    // Example 3: Set bucket ACL (use with caution, may require specific
    // permissions)
    println!("\n3. Set Bucket ACL (Demo only, skipped in actual run)");
    println!("   To set ACL, uncomment the code and ensure you have sufficient permissions");

    // Uncomment the code below to actually execute ACL setting
    /*
    let put_acl_request = PutBucketAclRequest {
        bucket: test_bucket.clone(),
        acl: Some(alibabacloud_oss_sdk_rust_v2::AclType::Private), // Private
        ..Default::default()
    };

    match client.put_bucket_acl(&put_acl_request).await {
        Ok(result) => {
            println!("   ACL set successfully");
        }
        Err(e) => {
            println!("   Failed to set ACL: {}", e);
        }
    }
    */
    println!("   Skipping ACL setting");

    // Example 4: List objects in bucket
    println!("\n4. List Objects in Bucket");

    // Create some test objects first
    println!("   Creating test objects...");
    for i in 1..=3 {
        let key = format!("examples/bucket-mgmt/test-object-{}.txt", i);
        let content = format!("This is test object number {}", i);

        let put_request = PutObjectRequest {
            bucket: test_bucket.clone(),
            key: key.clone(),
            body: Some(BodyContent::from_text(content, None)),
            ..Default::default()
        };

        if client.put_object(put_request).await.is_ok() {
            println!("   Created test object: {}", key);
        }
    }

    // List objects
    let list_request = ListObjectsV2Request {
        bucket: test_bucket.clone(),
        prefix: Some("examples/bucket-mgmt/".to_string()),
        max_keys: Some(10),
        ..Default::default()
    };

    match client.list_objects_v2(list_request).await {
        Ok(result) => {
            println!("   Object list:");
            println!("      Total: {}", result.contents.len());
            for obj in &result.contents {
                println!("         - {:?} (Size: {} bytes)", obj.key, obj.size);
            }

            // Clean up test objects
            println!("\n   Cleaning up test objects...");
            for obj in &result.contents {
                let delete_request = DeleteObjectRequest {
                    bucket: test_bucket.clone(),
                    key: obj.key.clone().unwrap_or_default(),
                    ..Default::default()
                };

                if client.delete_object(delete_request).await.is_ok() {
                    println!("      Deleted: {:?}", obj.key);
                }
            }
        }
        Err(e) => {
            println!("   Failed to list objects: {}", e);
        }
    }

    // Example 5: Create bucket (requires specific permissions)
    println!("\n5. Create New Bucket (Demo Only)");
    let new_bucket_name = format!("test-bucket-{}", chrono::Utc::now().timestamp());
    println!("   Suggested bucket name: {}", new_bucket_name);
    println!("   Skipping actual creation (requires appropriate permissions)");

    // Uncomment the code below to actually create bucket
    /*
    let create_request = CreateBucketRequest {
        bucket: new_bucket_name.clone(),
        storage_class: Some(alibabacloud_oss_sdk_rust_v2::StorageClass::Standard),
        ..Default::default()
    };

    match client.create_bucket(&create_request).await {
        Ok(result) => {
            println!("   Successfully created bucket: {}", new_bucket_name);
        }
        Err(e) => {
            println!("   Failed to create bucket: {}", e);
        }
    }
    */

    println!("\n===========================================");
    println!("Bucket management example completed!");

    Ok(())
}
