use std::env;

use crate::credential::{Credentials, CredentialsProvider};
use crate::{ENV_OSS_ACCESS_KEY_ID, ENV_OSS_ACCESS_KEY_SECRET, ENV_OSS_SESSION_TOKEN};

/// Provides credentials from environment variables.
pub struct EnvironmentVariableCredentialsProvider;

impl EnvironmentVariableCredentialsProvider {
    /// Creates a new instance of `EnvironmentVariableCredentialsProvider`.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> impl CredentialsProvider {
        EnvironmentVariableCredentialsProvider
    }
}

#[async_trait::async_trait]
impl CredentialsProvider for EnvironmentVariableCredentialsProvider {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    /// Retrieves the credentials from environment variables.
    async fn get_credentials(
        &self,
    ) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
        let id = env::var(ENV_OSS_ACCESS_KEY_ID).ok();
        let secret = env::var(ENV_OSS_ACCESS_KEY_SECRET).ok();

        match (id, secret) {
            (Some(id), Some(secret)) => Ok(Credentials {
                access_key_id: id,
                access_key_secret: secret,
                security_token: env::var(ENV_OSS_SESSION_TOKEN).ok().unwrap_or_default(),
                ..Default::default()
            }),
            _ => Err(format!(
                "{} or {} is empty.",
                ENV_OSS_ACCESS_KEY_ID, ENV_OSS_ACCESS_KEY_SECRET
            )
            .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_credentials_with_valid_env_vars() {
        async {
            env::set_var(ENV_OSS_ACCESS_KEY_ID, "valid_access_key_id");
            env::set_var(ENV_OSS_ACCESS_KEY_SECRET, "valid_access_key_secret");
            env::set_var(ENV_OSS_SESSION_TOKEN, "valid_session_token");
        }
        .await; // Wait for the environment variables to be set

        let provider = EnvironmentVariableCredentialsProvider::new();

        let result = provider.get_credentials().await;

        assert!(result.is_ok());
        let credentials = result.unwrap();
        assert_eq!(credentials.access_key_id, "valid_access_key_id");
        assert_eq!(credentials.access_key_secret, "valid_access_key_secret");
        assert_eq!(credentials.security_token, "valid_session_token");

        async {
            env::remove_var(ENV_OSS_ACCESS_KEY_ID);
            env::remove_var(ENV_OSS_ACCESS_KEY_SECRET);
            env::remove_var(ENV_OSS_SESSION_TOKEN);
        }
        .await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_credentials_with_missing_env_vars() {
        // Clear any existing environment variables
        async {
            env::remove_var(ENV_OSS_ACCESS_KEY_ID);
            env::remove_var(ENV_OSS_ACCESS_KEY_SECRET);
            env::remove_var(ENV_OSS_SESSION_TOKEN);
        }
        .await;

        let provider = EnvironmentVariableCredentialsProvider::new();

        let result = provider.get_credentials().await;

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            format!(
                "{} or {} is empty.",
                ENV_OSS_ACCESS_KEY_ID, ENV_OSS_ACCESS_KEY_SECRET
            )
        );
    }
}
