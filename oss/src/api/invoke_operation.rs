use crate::client::{Client, ClientOptions};
use crate::{OperationInput, OperationOutput};

impl Client {
    /// Performs a raw operation on the client.
    ///
    /// This method allows you to perform a raw operation on the client by
    /// providing an `OperationInput` and a list of optional functions that
    /// can modify the `ClientOptions`.
    ///
    /// # Arguments
    ///
    /// * `input` - The `OperationInput` containing the necessary parameters for
    ///   the operation.
    /// * `opt_fns` - A list of optional functions that can modify the
    ///   `ClientOptions`.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `OperationOutput` if the operation is
    /// successful, or a boxed `dyn std::error::Error` if an error occurs
    /// during the operation.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::RequestCommon;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// # use alibabacloud_oss_sdk_rust_v2::OperationInput;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let input = OperationInput::default();
    ///
    /// let result = client.invoke_operation(input, vec![]).await; // 移除 & 符号
    /// match result {
    ///     Ok(output) => {
    ///         // Handle successful operation output
    ///     }
    ///     Err(err) => {
    ///         // Handle error
    ///     }
    /// }
    /// # })
    /// ```
    #[allow(clippy::type_complexity)]
    pub async fn invoke_operation(
        &self,
        input: OperationInput,
        opt_fns: Vec<fn(&mut ClientOptions)>,
    ) -> Result<OperationOutput, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(err) = input.validate().err() {
            return Err(err);
        }
        // let mut input = input.clone();
        // extend_headers_case_insensitive(&mut input.headers,
        // &request_common.headers); extend_headers_case_insensitive(&
        // mut input.parameters, &request_common.parameters);

        let output = self.invoke_operation_inner(input, opt_fns).await?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use http::Method;

    use super::*;
    use crate::client::BodyDataReader;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::test_utils::{generate_unique_object_name, load_test_config};
    use crate::{DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

    #[tokio::test]
    #[serial_test::serial]
    async fn test_invoke_operation_comprehensive_workflow() {
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
                .with_log_level(LogLevel::Debug),
        );

        // Generate a unique object name for this test
        let object_name = generate_unique_object_name("invoke-operation-test");

        // 1. Upload object using invoke_operation
        println!("Step 1: Uploading object using invoke_operation");
        let content = "Hello from invoke_operation!";
        let content_bytes = content.as_bytes();

        let input = OperationInput {
            op_name: "PutObject".to_string(),
            method: Method::PUT,
            bucket: Some(config.bucket.to_string()),
            key: Some(object_name.clone()),
            body: Some(crate::BodyContent::from_bytes(content_bytes.to_vec(), None)),
            headers: [
                (
                    HTTP_HEADER_CONTENT_TYPE.to_string(),
                    DEFAULT_CONTENT_TYPE.to_string(),
                ),
                (
                    "Content-Length".to_string(),
                    content_bytes.len().to_string(),
                ), // Add content length
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
            ..Default::default()
        };

        match client.invoke_operation(input, vec![]).await {
            // 移除 & 符号
            Ok(output) => {
                println!(
                    "Object uploaded successfully via invoke_operation. Status: {}",
                    output.status.as_u16()
                );
                assert!(output.status.is_success());
            }
            Err(err) => panic!("Upload via invoke_operation failed: {:?}", err),
        }

        // 2. List objects using invoke_operation
        println!("Step 2: Listing objects using invoke_operation");
        let input = OperationInput {
            op_name: "ListObjectsV2".to_string(),
            method: Method::GET,
            bucket: Some(config.bucket.to_string()),
            key: None,
            parameters: [("list-type", "2".to_string())] // Proper parameter for list objects v2
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        match client.invoke_operation(input, vec![]).await {
            // 移除 & 符号
            Ok(mut output) => {
                println!(
                    "Objects listed successfully via invoke_operation. Status: {}",
                    output.status.as_u16()
                );
                assert!(output.status.is_success());

                // Read the response body to verify the object is there
                let body_data = output.get_all_data().await.unwrap_or_default();
                println!("List response length: {} bytes", body_data.len());
                let body_str = String::from_utf8_lossy(&body_data);
                if body_str.contains(&object_name) {
                    println!("Object '{}' found in list response", object_name);
                } else {
                    println!(
                        "Object '{}' may not be immediately visible in list",
                        object_name
                    );
                }
            }
            Err(err) => panic!("List via invoke_operation failed: {:?}", err),
        }

        // 3. Download object using invoke_operation
        println!("Step 3: Downloading object using invoke_operation");
        let input = OperationInput {
            op_name: "GetObject".to_string(),
            method: Method::GET,
            bucket: Some(config.bucket.to_string()),
            key: Some(object_name.clone()),
            ..Default::default()
        };

        match client.invoke_operation(input, vec![]).await {
            // 移除 & 符号
            Ok(mut output) => {
                println!(
                    "Object downloaded successfully via invoke_operation. Status: {}",
                    output.status.as_u16()
                );
                assert!(output.status.is_success());

                // Read the response body
                let body_data = output.get_all_data().await.unwrap_or_default();
                let downloaded_content = String::from_utf8_lossy(&body_data).into_owned(); // Convert to owned String
                println!("Downloaded content: '{}'", downloaded_content);
                assert_eq!(
                    downloaded_content, content,
                    "Downloaded content should match uploaded content"
                );
            }
            Err(err) => panic!("Download via invoke_operation failed: {:?}", err),
        }

        // 4. Delete object using invoke_operation
        println!("Step 4: Deleting object using invoke_operation");
        let input = OperationInput {
            op_name: "DeleteObject".to_string(),
            method: Method::DELETE,
            bucket: Some(config.bucket.to_string()),
            key: Some(object_name.clone()),
            ..Default::default()
        };

        match client.invoke_operation(input, vec![]).await {
            // 移除 & 符号
            Ok(output) => {
                println!(
                    "Object deleted successfully via invoke_operation. Status: {}",
                    output.status.as_u16()
                );
                assert!(output.status.is_success());
            }
            Err(err) => panic!("Delete via invoke_operation failed: {:?}", err),
        }

        println!("All operations completed successfully using invoke_operation!");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_invoke_operation_put_get_delete() {
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
                .with_log_level(LogLevel::Debug),
        );

        // Generate a unique object name for this test
        let object_name = generate_unique_object_name("invoke-op-simple");

        // Upload using invoke_operation
        let content = "Simple test content";
        let content_bytes = content.as_bytes();

        let input = OperationInput {
            op_name: "PutObject".to_string(),
            method: Method::PUT,
            bucket: Some(config.bucket.to_string()),
            key: Some(object_name.clone()),
            body: Some(crate::BodyContent::from_bytes(content_bytes.to_vec(), None)),
            headers: [
                (
                    HTTP_HEADER_CONTENT_TYPE.to_string(),
                    DEFAULT_CONTENT_TYPE.to_string(),
                ),
                (
                    "Content-Length".to_string(),
                    content_bytes.len().to_string(),
                ), // Add content length
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
            ..Default::default()
        };

        match client.invoke_operation(input, vec![]).await {
            // 移除 & 符号
            Ok(output) => {
                assert!(output.status.is_success());
            }
            Err(err) => panic!("Upload failed: {:?}", err),
        }

        // Download using invoke_operation
        let input = OperationInput {
            op_name: "GetObject".to_string(),
            method: Method::GET,
            bucket: Some(config.bucket.to_string()),
            key: Some(object_name.clone()),
            ..Default::default()
        };

        match client.invoke_operation(input, vec![]).await {
            // 移除 & 符号
            Ok(mut output) => {
                assert!(output.status.is_success());
                let body_data = output.get_all_data().await.unwrap_or_default();
                let downloaded_content = String::from_utf8_lossy(&body_data).into_owned(); // Convert to owned String
                assert_eq!(downloaded_content, content);
            }
            Err(err) => panic!("Download failed: {:?}", err),
        }

        // Delete using invoke_operation
        let input = OperationInput {
            op_name: "DeleteObject".to_string(),
            method: Method::DELETE,
            bucket: Some(config.bucket.to_string()),
            key: Some(object_name.clone()),
            ..Default::default()
        };

        match client.invoke_operation(input, vec![]).await {
            // 移除 & 符号
            Ok(output) => {
                assert!(output.status.is_success());
            }
            Err(err) => panic!("Delete failed: {:?}", err),
        }
    }
}

// /// Override input with
// fn extend_headers_case_insensitive(
//     base: &mut HashMap<String, String>,
//     new: &HashMap<String, String>,
// ) {
//     // {base.key.to_lowercase(): base.key}
//     let key_indices: HashMap<String, String> =
//         base.keys().map(|k| (k.to_lowercase(), k.clone())).collect();

//     new.iter().for_each(|(new_key, v)| {
//         let new_lower_key = new_key.to_lowercase();
//         if let Some(base_key) = key_indices.get(&new_lower_key) {
//             // base key exists, use base key
//             base.insert(base_key.clone(), v.clone());
//         } else {
//             // base key not exists, just insert
//             base.insert(new_key.clone(), v.clone());
//         }
//     });
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_extend_headers_case_insensitive() {
//         let mut base = [("key1", "value1"), ("KEY2", "value2"), ("key3",
// "value3")]             .iter()
//             .map(|(k, v)| (k.to_string(), v.to_string()))
//             .collect::<std::collections::HashMap<String, String>>();

//         let new = [("key1", "new value1"), ("key2", "new value2")]
//             .iter()
//             .map(|(k, v)| (k.to_string(), v.to_string()))
//             .collect::<std::collections::HashMap<String, String>>();

//         extend_headers_case_insensitive(&mut base, &new);

//         assert_eq!(
//             [
//                 ("key1", "new value1"),
//                 ("KEY2", "new value2"),
//                 ("key3", "value3")
//             ]
//             .iter()
//             .map(|(k, v)| (k.to_string(), v.to_string()))
//             .collect::<std::collections::HashMap<String, String>>(),
//             base
//         );
//     }
// }
