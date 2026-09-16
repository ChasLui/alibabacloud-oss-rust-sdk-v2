use crate::credential::{Credentials, CredentialsProvider};

/// Provider that always returns "anonymous" credentials.
#[derive(Default)]
pub struct AnonymousCredentialsProvider;

impl AnonymousCredentialsProvider {
    pub fn new() -> Self {
        AnonymousCredentialsProvider
    }
}

#[async_trait::async_trait]
impl CredentialsProvider for AnonymousCredentialsProvider {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    /// Retrieves the credentials.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alibabacloud_oss_sdk_rust_v2::credential::providers::AnonymousCredentialsProvider;
    /// # use alibabacloud_oss_sdk_rust_v2::credential::{Credentials, CredentialsProvider};
    /// #
    /// # tokio_test::block_on(async {
    /// let provider = AnonymousCredentialsProvider;
    ///
    /// // Retrieve the credentials
    /// let credentials = provider.get_credentials().await;
    ///
    /// // Assert that the credentials are as expected
    /// assert!(credentials.is_ok());
    /// let credentials = credentials.unwrap();
    /// assert_eq!(credentials.access_key_id, "");
    /// assert_eq!(credentials.access_key_secret, "");
    /// # })
    /// ```
    async fn get_credentials(
        &self,
    ) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Credentials::default())
    }
}

#[cfg(test)]
mod tests {
    use tokio;

    use super::*;

    #[tokio::test]
    async fn test_anonymous_credentials_provider() {
        let provider = AnonymousCredentialsProvider::new();
        let credentials = provider.get_credentials().await.unwrap();

        assert!(credentials.access_key_id.is_empty());
        assert!(credentials.access_key_secret.is_empty());
        assert!(credentials.security_token.is_empty());
        assert!(credentials.expires.is_none());
    }
}
