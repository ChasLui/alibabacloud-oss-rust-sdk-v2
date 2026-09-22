//! File Upload Example - Demonstrates various file upload methods
//!
//! Features include:
//! - Upload text content
//! - Upload binary data
//! - Set object metadata
//! - Multipart upload for large files
//!
//! Run command:
//! ```bash
//! cargo run --example 02_file_upload
//! ```

use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2::api::object::{
    CompleteMultipartUploadPart, CompleteMultipartUploadRequest, InitiateMultipartUploadRequest,
    PutObjectRequest, UploadPartRequest,
};
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
    let bucket = std::env::var("OSS_BUCKET").unwrap_or_else(|_| "your-bucket-name".to_string());

    println!("OSS File Upload Example");
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

    // Example 1: Upload text string
    println!("\n1. Upload Text String");
    let text_key = "examples/upload/hello.txt";
    let text_content = "这是一段简单的文本内容，用于测试 OSS 上传功能。";

    let put_request = PutObjectRequest {
        bucket: bucket.clone(),
        key: text_key.to_string(),
        body: Some(BodyContent::from_text(text_content.to_string(), None)),
        ..Default::default()
    };

    match client.put_object(put_request).await {
        Ok(result) => {
            println!(
                "   Text upload successful, ETag: {:?}",
                result.common.headers.get("etag")
            );
        }
        Err(e) => {
            println!("   Upload failed: {}", e);
        }
    }

    // Example 2: Upload byte array
    println!("\n2. Upload Byte Array");
    let binary_key = "examples/upload/data.bin";
    let binary_data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello" 的 ASCII 码

    let put_request = PutObjectRequest {
        bucket: bucket.clone(),
        key: binary_key.to_string(),
        body: Some(BodyContent::from_bytes(binary_data, None)),
        ..Default::default()
    };

    match client.put_object(put_request).await {
        Ok(result) => {
            println!(
                "   Binary data upload successful, ETag: {:?}",
                result.common.headers.get("etag")
            );
        }
        Err(e) => {
            println!("   Upload failed: {}", e);
        }
    }

    // Example 3: Upload with custom metadata
    println!("\n3. Upload with Custom Metadata");
    let metadata_key = "examples/upload/metadata.txt";
    let metadata_content = "File with metadata";

    let mut put_request = PutObjectRequest {
        bucket: bucket.clone(),
        key: metadata_key.to_string(),
        body: Some(BodyContent::from_text(metadata_content.to_string(), None)),
        ..Default::default()
    };

    // Add custom metadata via common.headers
    put_request
        .common
        .headers
        .insert("x-oss-meta-author".to_string(), "OSS SDK User".to_string());
    put_request
        .common
        .headers
        .insert("x-oss-meta-version".to_string(), "1.0.0".to_string());
    put_request.common.headers.insert(
        "x-oss-meta-description".to_string(),
        "Test file for metadata".to_string(),
    );

    match client.put_object(put_request).await {
        Ok(result) => {
            println!("   File with metadata upload successful");
            println!("   ETag: {:?}", result.common.headers.get("etag"));
        }
        Err(e) => {
            println!("   Upload failed: {}", e);
        }
    }

    // Example 4: Multipart upload (simulating large file)
    println!("\n4. Multipart Upload Example");
    let multipart_key = "examples/upload/large-file.bin";

    // Step 1: Initialize multipart upload
    let init_request = InitiateMultipartUploadRequest {
        bucket: bucket.clone(),
        key: multipart_key.to_string(),
        ..Default::default()
    };

    let upload_id = match client.initiate_multipart_upload(&init_request).await {
        Ok(result) => {
            let upload_id = result.upload_id.clone().unwrap_or_default();
            println!(
                "   Multipart upload initialization successful, UploadId: {:?}",
                upload_id
            );
            upload_id
        }
        Err(e) => {
            println!("   Initialization failed: {}", e);
            return Ok(());
        }
    };

    // Step 2: Upload multiple parts
    let mut parts = Vec::new();
    let part_data_1 = vec![1u8; 1024]; // 1KB data
    let part_data_2 = vec![2u8; 1024];

    // Upload part 1
    let upload_part_req_1 = UploadPartRequest {
        bucket: bucket.clone(),
        key: multipart_key.to_string(),
        upload_id: upload_id.clone(),
        part_number: 1,
        body: Some(BodyContent::from_bytes(part_data_1, None)),
        ..Default::default()
    };

    if let Ok(result) = client.upload_part(upload_part_req_1).await {
        println!(
            "   Part 1 upload successful, ETag: {:?}",
            result.common.headers.get("etag")
        );
        parts.push(CompleteMultipartUploadPart {
            part_number: 1,
            etag: result
                .common
                .headers
                .get("etag")
                .map(|s| s.to_string())
                .unwrap_or_default(),
        });
    }

    // Upload part 2
    let upload_part_req_2 = UploadPartRequest {
        bucket: bucket.clone(),
        key: multipart_key.to_string(),
        upload_id: upload_id.clone(),
        part_number: 2,
        body: Some(BodyContent::from_bytes(part_data_2, None)),
        ..Default::default()
    };

    if let Ok(result) = client.upload_part(upload_part_req_2).await {
        println!(
            "   Part 2 upload successful, ETag: {:?}",
            result.common.headers.get("etag")
        );
        parts.push(CompleteMultipartUploadPart {
            part_number: 2,
            etag: result
                .common
                .headers
                .get("etag")
                .map(|s| s.to_string())
                .unwrap_or_default(),
        });
    }

    // Step 3: Complete multipart upload
    let complete_request = CompleteMultipartUploadRequest {
        bucket: bucket.clone(),
        key: multipart_key.to_string(),
        upload_id,
        parts,
        ..Default::default()
    };

    match client.complete_multipart_upload(&complete_request).await {
        Ok(result) => {
            println!("   Multipart upload completed!");
            println!("   Final ETag: {:?}", result.common.headers.get("etag"));
            println!("   Object location: {:?}.{:?}", result.bucket, result.key);
        }
        Err(e) => {
            println!("   Failed to complete multipart upload: {}", e);
        }
    }

    println!("\n===========================================");
    println!("File upload example completed!");

    Ok(())
}
