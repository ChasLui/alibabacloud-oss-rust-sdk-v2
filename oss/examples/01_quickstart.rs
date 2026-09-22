//! Quickstart Example - Demonstrates basic OSS SDK functionality
//!
//! Before running this example, ensure:
//! 1. Environment variables ACCESS_KEY_ID and ACCESS_KEY_SECRET are set
//! 2. You have an available OSS Bucket
//!
//! Run command:
//! ```bash
//! cargo run --example 01_quickstart
//! ```

use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2::api::bucket::GetBucketInfoRequest;
use alibabacloud_oss_sdk_rust_v2::api::object::{GetObjectRequest, PutObjectRequest};
use alibabacloud_oss_sdk_rust_v2::client::Client;
use alibabacloud_oss_sdk_rust_v2::config::Config;
use alibabacloud_oss_sdk_rust_v2::credential::providers::StaticCredentialsProvider;
use alibabacloud_oss_sdk_rust_v2::BodyContent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get configuration from environment variables
    let access_key_id =
        std::env::var("ACCESS_KEY_ID").unwrap_or_else(|_| "your-access-key-id".to_string());
    let access_key_secret =
        std::env::var("ACCESS_KEY_SECRET").unwrap_or_else(|_| "your-access-key-secret".to_string());
    let region = std::env::var("OSS_REGION").unwrap_or_else(|_| "cn-hangzhou".to_string());
    let bucket = std::env::var("OSS_BUCKET").unwrap_or_else(|_| "your-bucket-name".to_string());

    println!("OSS SDK Rust Quickstart Example");
    println!("===========================================");
    println!("Region: {}", region);
    println!("Bucket: {}", bucket);
    println!();

    // 创建客户端配置
    let config = Config::default()
        .with_region(&region)
        .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
            &access_key_id,
            &access_key_secret,
            &[],
        )))
        .with_signature_version(alibabacloud_oss_sdk_rust_v2::SignatureVersionType::V4);

    // 创建客户端
    let client = Client::new(&config);

    // Example 1: Upload object
    println!("Example 1: Upload Object");
    let object_key = "examples/quickstart/hello-oss.txt";
    let content = "Hello, Alibaba Cloud OSS! 🎉";

    let put_request = PutObjectRequest {
        bucket: bucket.clone(),
        key: object_key.to_string(),
        body: Some(BodyContent::from_text(content.to_string(), None)),
        ..Default::default()
    };

    match client.put_object(put_request).await {
        Ok(result) => {
            println!("   Upload successful!");
            println!("   ETag: {:?}", result.common.headers.get("etag"));
            println!(
                "   Request ID: {}",
                result
                    .common
                    .headers
                    .get("x-oss-request-id")
                    .unwrap_or(&"N/A".to_string())
            );
        }
        Err(e) => {
            println!("   Upload failed: {}", e);
        }
    }
    println!();

    // Example 2: Download object
    println!("Example 2: Download Object");
    let get_request = GetObjectRequest {
        bucket: bucket.clone(),
        key: object_key.to_string(),
        ..Default::default()
    };

    match client.get_object(get_request).await {
        Ok(mut result) => {
            println!("   Download successful!");
            // Use BodyDataReader trait to read stream data
            use alibabacloud_oss_sdk_rust_v2::client::BodyDataReader;

            match result.get_all_data().await {
                Ok(data) => {
                    let content = String::from_utf8_lossy(&data);
                    println!("   Content: {}", content);
                    println!("   Size: {} bytes", data.len());
                }
                Err(e) => {
                    println!("   Failed to read content: {}", e);
                }
            }
        }
        Err(e) => {
            println!("   Download failed: {}", e);
        }
    }
    println!();

    // Example 3: Get bucket information
    println!("Example 3: Get Bucket Information");
    let info_request = GetBucketInfoRequest {
        bucket: bucket.clone(),
        ..Default::default()
    };

    match client.get_bucket_info(&info_request).await {
        Ok(result) => {
            println!("   Successfully retrieved bucket information!");
            println!("   Bucket name: {:?}", result.bucket_info.name);
            println!("   Creation date: {:?}", result.bucket_info.creation_date);
            println!("   Storage class: {:?}", result.bucket_info.storage_class);
            println!("   Location: {:?}", result.bucket_info.location);
        }
        Err(e) => {
            println!("   Failed to get bucket information: {}", e);
        }
    }
    println!();

    println!("===========================================");
    println!("Quickstart example completed!");

    Ok(())
}
