use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};

use super::put_cname::{effective_cname_configuration, BucketCnameConfiguration, Cname};
use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationOutput, BodyContent, OperationInput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct DeleteCnameRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The request body schema.
    pub bucket_cname_configuration: BucketCnameConfiguration,

    pub common: RequestCommon,
}

#[derive(Debug, Default, OssResultModel)]
pub struct DeleteCnameResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Deletes a CNAME record that is mapped to a bucket.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DeleteCnameRequest` containing the bucket name and
    ///   the CNAME record to delete.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    /// #     BucketCnameConfiguration, Cname, DeleteCnameRequest,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DeleteCnameRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     bucket_cname_configuration: BucketCnameConfiguration {
    ///         cname: Some(Cname {
    ///             domain: Some("example.com".to_string()),
    ///             ..Default::default()
    ///         }),
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.delete_cname(&request).await {
    ///     Ok(result) => {
    ///         println!("Cname deleted: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to delete cname: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn delete_cname(
        &self,
        request: &DeleteCnameRequest,
    ) -> Result<DeleteCnameResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DeleteCname".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("cname", ""), ("comp", "delete")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        input.op_metadata.set(
            crate::signer::SUB_RESOURCE,
            std::rc::Rc::new(vec!["cname".to_string(), "comp".to_string()]),
        );

        let effective_config =
            effective_cname_configuration(&request.bucket_cname_configuration);
        let xml_body =
            quick_xml::se::to_string_with_root("BucketCnameConfiguration", &effective_config)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = DeleteCnameResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::{generate_unique_bucket_name, load_test_config};
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_cname() {
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
                .with_signature_version(SignatureVersionType::V4),
        );

        let bucket_name = generate_unique_bucket_name("del-cname-test");

        let created = client
            .create_bucket(&crate::api::bucket::CreateBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
        assert!(created.is_ok(), "create_bucket failed: {:?}", created.err());

        // A fresh bucket has no CNAME record; the server is expected to
        // reject the deletion.
        let result = client
            .delete_cname(&DeleteCnameRequest {
                bucket: bucket_name.clone(),
                bucket_cname_configuration: BucketCnameConfiguration {
                    cname: Some(Cname {
                        domain: Some("nonexistent-domain-for-test.example.com".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;
        assert!(
            result.is_err(),
            "delete_cname without a mapped record should fail"
        );

        let _ = client
            .delete_bucket(&crate::api::bucket::DeleteBucketRequest {
                bucket: bucket_name.clone(),
                ..Default::default()
            })
            .await;
    }
}
