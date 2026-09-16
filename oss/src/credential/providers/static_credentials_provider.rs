use crate::credential::{Credentials, CredentialsProvider};

/// Struct equivalent to the StaticCredentialsProvider in Go
pub struct StaticCredentialsProvider {
    credentials: Credentials,
}

impl StaticCredentialsProvider {
    /// Function to create a new StaticCredentialsProvider. Rust does not
    /// support variadic parameters similar to Go, so we use Option to
    /// handle the optional token.
    ///
    /// # Arguments
    ///
    /// * `id` - The access key ID.
    /// * `secret` - The access key secret.
    /// * `tokens` - Optional security tokens.
    ///
    /// # Example
    ///
    /// ```
    /// # use alibabacloud_oss_sdk_rust_v2::credential::StaticCredentialsProvider;
    /// #
    /// let id = "test_id";
    /// let secret = "test_secret";
    /// let tokens = vec!["token1", "token2"];
    /// let provider = StaticCredentialsProvider::new(id, secret, &tokens);
    /// ```
    pub fn new(id: &str, secret: &str, tokens: &[&str]) -> Self {
        StaticCredentialsProvider {
            credentials: Credentials {
                access_key_id: id.to_string(),
                access_key_secret: secret.to_string(),
                security_token: tokens.first().map_or(String::new(), |t| t.to_string()), /* keep first token if there are multiple */
                ..Default::default()
            },
        }
    }
}

#[async_trait::async_trait]
impl CredentialsProvider for StaticCredentialsProvider {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    /// Asynchronously retrieves the credentials.
    ///
    /// # Returns
    ///
    /// A `Result` containing the retrieved `Credentials` or an error.
    async fn get_credentials(
        &self,
    ) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.credentials.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_credentials() {
        let id = "test_id";
        let secret = "test_secret";
        let tokens = vec!["token1", "token2"];
        let provider = StaticCredentialsProvider::new(id, secret, &tokens);

        let credentials = provider.get_credentials().await.unwrap();

        assert_eq!(credentials.access_key_id, id);
        assert_eq!(credentials.access_key_secret, secret);
        assert_eq!(credentials.security_token, tokens[0]);
    }

    #[tokio::test]
    async fn test_get_credentials_without_tokens() {
        let id = "test_id";
        let secret = "test_secret";
        let tokens = vec![];
        let provider = StaticCredentialsProvider::new(id, secret, &tokens);

        let credentials = provider.get_credentials().await.unwrap();

        assert_eq!(credentials.access_key_id, id);
        assert_eq!(credentials.access_key_secret, secret);
        assert_eq!(credentials.security_token, "");
    }
}
