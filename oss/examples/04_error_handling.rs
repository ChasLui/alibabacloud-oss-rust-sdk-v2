//! Error Handling Example - Demonstrates how to properly handle errors in OSS
//! SDK
//!
//! Features include:
//! - Capture and handle common errors
//! - Retry mechanisms
//! - Error classification and custom error handling
//!
//! Run command:
//! ```bash
//! cargo run --example 04_error_handling
//! ```

use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2::api::object::{GetObjectRequest, PutObjectRequest};
use alibabacloud_oss_sdk_rust_v2::client::Client;
use alibabacloud_oss_sdk_rust_v2::config::Config;
use alibabacloud_oss_sdk_rust_v2::credential::providers::StaticCredentialsProvider;
use alibabacloud_oss_sdk_rust_v2::BodyContent;

/// Custom error type
#[derive(Debug)]
#[allow(dead_code)] // the example shows the full error taxonomy, not only the
                    // variants it exercises
enum OssAppError {
    NotFound(String),
    PermissionDenied(String),
    NetworkError(String),
    InvalidInput(String),
    Other(String),
}

impl std::fmt::Display for OssAppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OssAppError::NotFound(msg) => write!(f, "Resource not found: {}", msg),
            OssAppError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            OssAppError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            OssAppError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            OssAppError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for OssAppError {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let access_key_id =
        std::env::var("ACCESS_KEY_ID").unwrap_or_else(|_| "your-access-key-id".to_string());
    let access_key_secret =
        std::env::var("ACCESS_KEY_SECRET").unwrap_or_else(|_| "your-access-key-secret".to_string());
    let region = std::env::var("OSS_REGION").unwrap_or_else(|_| "cn-hangzhou".to_string());
    let bucket = std::env::var("OSS_BUCKET").unwrap_or_else(|_| "your-bucket-name".to_string());

    println!("OSS Error Handling Example");
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

    // Example 1: Handle object not found error
    println!("\n1. Handle Object Not Found (404)");
    let non_existent_key = "examples/error-handling/this-file-does-not-exist.txt";
    let get_request = GetObjectRequest {
        bucket: bucket.clone(),
        key: non_existent_key.to_string(),
        ..Default::default()
    };

    match client.get_object(get_request).await {
        Ok(_) => {
            println!("   Object exists (this should not happen)");
        }
        Err(e) => {
            println!("   Captured error: {}", e);

            // Check if it's a 404 error
            let error_str = e.to_string();
            if error_str.contains("404") || error_str.contains("NoSuchKey") {
                println!("   This is a 404 error - object does not exist");

                // Convert to custom error type
                let app_error =
                    OssAppError::NotFound(format!("Object '{}' does not exist", non_existent_key));
                println!("   Application layer error: {}", app_error);
            }
        }
    }

    // Example 2: Handle invalid input error
    println!("\n2. Handle Invalid Input");

    // Try to upload empty content to invalid path
    let invalid_key = ""; // Empty object name is invalid
    let put_request = PutObjectRequest {
        bucket: bucket.clone(),
        key: invalid_key.to_string(),
        body: Some(BodyContent::from_text("test".to_string(), None)),
        ..Default::default()
    };

    match client.put_object(put_request).await {
        Ok(_) => {
            println!("   Upload successful (this should not happen)");
        }
        Err(e) => {
            println!("   Captured error: {}", e);
            println!("   This is an invalid input error");

            let app_error = OssAppError::InvalidInput("Object name cannot be empty".to_string());
            println!("   Application suggestion: {}", app_error);
        }
    }

    // Example 3: Use Result wrapper for error transformation
    println!("\n3. Error Wrapping and Propagation");

    fn upload_with_validation(
        _client: &Client,
        bucket: String,
        key: String,
        content: String,
    ) -> Result<String, OssAppError> {
        // Validate input
        if key.is_empty() {
            return Err(OssAppError::InvalidInput(
                "Object key cannot be empty".to_string(),
            ));
        }

        if content.is_empty() {
            return Err(OssAppError::InvalidInput(
                "Content cannot be empty".to_string(),
            ));
        }

        // Here we return a Result that will be completed in the future, we need
        // to handle it in async context For demonstration, we directly
        // return Ok
        Ok(format!(
            "Validation passed, preparing to upload: {}/{}",
            bucket, key
        ))
    }

    match upload_with_validation(
        &client,
        bucket.clone(),
        "valid-key".to_string(),
        "valid-content".to_string(),
    ) {
        Ok(msg) => println!("   {}", msg),
        Err(e) => println!("   {}", e),
    }

    // Example 4: Retry logic example
    println!("\n4. Retry Strategy Example");
    println!(
        "   SDK has built-in automatic retry mechanism, can be adjusted through configuration:"
    );
    println!("      - Maximum retry attempts");
    println!("      - Retry delay");
    println!("      - Retryable error types");

    // Configuration example (not actually executed)
    println!("\n   Configuration example code:");
    println!(
        r#"   let config = Config::default()
       .with_retry_max_attempts(3)           // Maximum 3 retries
       .with_retry_initial_interval(1.0)     // Initial interval 1 second
       .with_retry_max_interval(60.0);       // Maximum interval 60 seconds"#
    );

    // Example 5: Graceful error recovery
    println!("\n5. Graceful Error Recovery Strategy");

    let test_keys = vec![
        "examples/error-handling/existing-object.txt",
        "examples/error-handling/non-existing-1.txt",
        "examples/error-handling/non-existing-2.txt",
    ];

    let mut success_count = 0;
    let mut failure_count = 0;

    for key in test_keys {
        print!("   Processing {}: ", key);

        let get_request = GetObjectRequest {
            bucket: bucket.clone(),
            key: key.to_string(),
            ..Default::default()
        };

        match client.get_object(get_request).await {
            Ok(_) => {
                println!("Success");
                success_count += 1;
            }
            Err(e) => {
                println!("Failed ({})", e);
                failure_count += 1;

                // Can continue processing next instead of aborting entire
                // program
                continue;
            }
        }
    }

    println!("\n   Statistics:");
    println!("      Success: {} operations", success_count);
    println!("      Failure: {} operations", failure_count);
    println!("   Even if some operations fail, the program can continue running");

    println!("\n===========================================");
    println!("Error handling example completed!");
    println!();
    println!("Best Practices:");
    println!("   1. Always check operation results");
    println!("   2. Classify and handle specific error types");
    println!("   3. Implement appropriate retry mechanisms");
    println!("   4. Record detailed error information for debugging");
    println!("   5. Provide user-friendly error messages");

    Ok(())
}
