mod anonymous_credentials_provider;
mod credentials_provider_func;
mod ecs_role_credentials_provider;
mod environment_credentials_provider;
mod fetcher_credentials_provider;
mod process_credentials_provider;
mod static_credentials_provider;

pub use self::anonymous_credentials_provider::*;
pub use self::credentials_provider_func::*;
pub use self::ecs_role_credentials_provider::*;
pub use self::environment_credentials_provider::*;
pub use self::fetcher_credentials_provider::*;
pub use self::process_credentials_provider::*;
pub use self::static_credentials_provider::*;

/// A trait for providing credentials to authenticate.
#[async_trait::async_trait]
pub trait CredentialsProvider: Sync + Send + std::any::Any {
    /// Downcast support: `type_id()` on `&dyn CredentialsProvider` returns the trait
    /// object's own TypeId (std blanket impl), so concrete-type checks must go
    /// through this method.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Asynchronously retrieves the credentials.
    ///
    /// # Returns
    ///
    /// - `Ok(credentials)`: If the credentials are successfully retrieved.
    /// - `Err(error)`: If an error occurs while retrieving the credentials.
    async fn get_credentials(
        &self,
    ) -> Result<super::Credentials, Box<dyn std::error::Error + Send + Sync>>;
}

#[cfg(test)]
mod tests_credentials_provider_trait {
    use tokio;

    use super::*;
    use crate::credential::Credentials;

    #[tokio::test]
    async fn test_get_credentials_success() {
        // Create a mock implementation of the CredentialsProvider trait
        struct MockCredentialsProvider;
        #[async_trait::async_trait]
        impl CredentialsProvider for MockCredentialsProvider {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
            async fn get_credentials(
                &self,
            ) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
                // Return a dummy set of credentials
                Ok(Credentials {
                    access_key_id: "dummy_access_key_id".to_string(),
                    access_key_secret: "dummy_access_key_secret".to_string(),
                    security_token: "dummy_security_token".to_string(),
                    expires: None,
                })
            }
        }

        // Create an instance of the mock credentials provider
        let credentials_provider = MockCredentialsProvider;

        // Call the get_credentials function and assert that it returns the expected
        // credentials
        let result = credentials_provider.get_credentials().await;
        assert!(result.is_ok());
        let credentials = result.unwrap();
        assert_eq!(credentials.access_key_id, "dummy_access_key_id");
        assert_eq!(credentials.access_key_secret, "dummy_access_key_secret");
        assert_eq!(credentials.security_token, "dummy_security_token");
        assert_eq!(credentials.expires, None);
    }

    #[tokio::test]
    async fn test_get_credentials_error() {
        // Create a mock implementation of the CredentialsProvider trait
        struct MockCredentialsProvider;
        #[async_trait::async_trait]
        impl CredentialsProvider for MockCredentialsProvider {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
            async fn get_credentials(
                &self,
            ) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
                // Return an error indicating that credentials retrieval failed
                Err("Failed to retrieve credentials".into())
            }
        }

        // Create an instance of the mock credentials provider
        let credentials_provider = MockCredentialsProvider;

        // Call the get_credentials function and assert that it returns an error
        let result = credentials_provider.get_credentials().await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.to_string(), "Failed to retrieve credentials");
    }
}
