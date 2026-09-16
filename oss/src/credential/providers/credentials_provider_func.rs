use std::sync::Arc;

use crate::credential::{Credentials, CredentialsProvider};

/// A function-based credentials provider.
pub struct CredentialsProviderFunc<F>
where
    F: Fn() -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> + Sync + Send,
{
    pub func: Arc<F>,
}

#[async_trait::async_trait]
impl<F> CredentialsProvider for CredentialsProviderFunc<F>
where
    F: Fn() -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>>
        + Sync
        + Send
        + 'static,
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    /// Retrieves the credentials.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use std::sync::Arc;
    /// #
    /// # use alibabacloud_oss_sdk_rust_v2::credential::{
    /// #     Credentials, CredentialsProvider, CredentialsProviderFunc,
    /// # };
    /// #
    /// # tokio_test::block_on(async {
    /// // Define a function that returns a result with credentials
    /// let func = || -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
    ///     Ok(Credentials {
    ///         access_key_id: "ACCESS_KEY".to_string(),
    ///         access_key_secret: "SECRET_KEY".to_string(),
    ///         ..Default::default()
    ///     })
    /// };
    ///
    /// // Create a credentials provider using the function
    /// let provider = CredentialsProviderFunc {
    ///     func: Arc::new(func),
    /// };
    ///
    /// // Retrieve the credentials
    /// let credentials = provider.get_credentials().await;
    ///
    /// // Assert that the credentials are as expected
    /// assert!(credentials.is_ok());
    /// let credentials = credentials.unwrap();
    /// assert_eq!(credentials.access_key_id, "ACCESS_KEY");
    /// assert_eq!(credentials.access_key_secret, "SECRET_KEY");
    /// # })
    /// ```
    async fn get_credentials(
        &self,
    ) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
        (self.func)()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_credentials_provider_func() {
        // Define a function that returns a result with credentials
        let func = || -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Credentials {
                access_key_id: "ACCESS_KEY".to_string(),
                access_key_secret: "SECRET_KEY".to_string(),
                ..Default::default()
            })
        };

        // Create a credentials provider using the function
        let provider = CredentialsProviderFunc {
            func: Arc::new(func),
        };

        // Retrieve the credentials
        let credentials = provider.get_credentials().await;

        // Assert that the credentials are as expected
        assert!(credentials.is_ok());
        let credentials = credentials.unwrap();
        assert_eq!(credentials.access_key_id, "ACCESS_KEY");
        assert_eq!(credentials.access_key_secret, "SECRET_KEY");
    }
}
