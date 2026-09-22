//! Shared setup for the agentic integration tests.
//!
//! The integration tests only run when a real agentic account is reachable.
//! Besides the standard `test_config.json`, agentic requests need the account
//! ID and endpoint that make up the physical bucket name
//! `{prefix}-{accountId}-{region}-ab-apsr`; mirroring the Go SDK, those come
//! from `OSS_TEST_ACCOUNT_ID` and `OSS_TEST_ENDPOINT`.

use std::rc::Rc;

use crate::api::agentic::{CreateAgenticBucketRequest, DeleteAgenticBucketRequest};
use crate::client::Client;
use crate::config::Config;
use crate::credential::StaticCredentialsProvider;
use crate::test_utils::load_test_config;
use crate::SignatureVersionType;

/// The default agentic bucket prefix. Short enough that the resolved DNS host
/// label stays within 63 characters for any region.
const DEFAULT_BUCKET_PREFIX: &str = "sdk-oss-test-rust-ab";

/// Builds the client and bucket prefix for the agentic integration tests, or
/// `None` when the environment does not describe a real agentic account.
pub(crate) fn agentic_test_client() -> Option<(Client, String)> {
    let config = load_test_config()?;
    let account_id = non_empty_env("OSS_TEST_ACCOUNT_ID")?;
    let endpoint = non_empty_env("OSS_TEST_ENDPOINT")?;
    let prefix = non_empty_env("OSS_TEST_BUCKET_PREFIX")
        .unwrap_or_else(|| DEFAULT_BUCKET_PREFIX.to_string());

    let client = Client::new_agentic(
        &Config::default()
            .with_region(&config.region)
            .with_endpoint(&endpoint)
            .with_account_id(&account_id)
            .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                &config.access_key_id,
                &config.access_key_secret,
                &[],
            )))
            .with_signature_version(SignatureVersionType::V4),
    );

    Some((client, prefix))
}

fn non_empty_env(key: &str) -> Option<String> {
    match std::env::var(key) {
        Ok(value) if !value.is_empty() => Some(value),
        _ => None,
    }
}

/// Creates the agentic bucket the tests operate on. An existing bucket is not
/// an error: the tests share one prefix.
pub(crate) async fn ensure_agentic_bucket(client: &Client, prefix: &str) {
    let result = client
        .create_agentic_bucket(&CreateAgenticBucketRequest {
            bucket: prefix.to_string(),
            ..Default::default()
        })
        .await;
    if let Err(error) = &result {
        eprintln!("create_agentic_bucket skipped: {}", error);
    }
}

/// Deletes the agentic bucket the tests operate on, ignoring a bucket that is
/// already gone.
pub(crate) async fn cleanup_agentic_bucket(client: &Client, prefix: &str) {
    let result = client
        .delete_agentic_bucket(&DeleteAgenticBucketRequest {
            bucket: prefix.to_string(),
            ..Default::default()
        })
        .await;
    if let Err(error) = &result {
        eprintln!("delete_agentic_bucket skipped: {}", error);
    }
}
