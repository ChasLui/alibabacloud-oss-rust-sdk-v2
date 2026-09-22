use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tokio::sync::RwLock;

use crate::credential::{Credentials, CredentialsProvider};

const DEFAULT_EXPIRED_FACTOR: f64 = 0.8;
const DEFAULT_REFRESH_DURATION: Duration = Duration::from_secs(120);

#[async_trait::async_trait]
#[allow(async_fn_in_trait)] // `async_trait` rewrites the signature; the lint fires on pre-expansion source
/// Trait for fetching credentials.
pub trait CredentialsFetcher: Send + Sync {
    /// Fetches the credentials asynchronously.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the fetched `Credentials` or an error.
    async fn fetch(&self) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>>;
}

/// Options for the `CredentialsFetcher` struct.
pub struct CredentialsFetcherOptions {
    /// The factor by which the credentials are considered expired.
    pub expired_factor: f64,
    /// The duration after which the credentials should be refreshed.
    pub refresh_duration: Duration,
}

impl Default for CredentialsFetcherOptions {
    fn default() -> Self {
        CredentialsFetcherOptions {
            expired_factor: DEFAULT_EXPIRED_FACTOR,
            refresh_duration: DEFAULT_REFRESH_DURATION,
        }
    }
}

#[derive(Clone)]
/// A provider for fetcher credentials used for authentication.
pub struct CredentialsFetcherProvider {
    /// The fetcher credentials used for authentication.
    fetcher_credentials: Arc<RwLock<Option<FetcherCredentials>>>,
    /// The fetcher used to retrieve the credentials.
    fetcher: Arc<dyn CredentialsFetcher + Send + Sync>,
    /// The factor by which the credentials are considered expired.
    expired_factor: f64,
    /// The duration after which the credentials should be refreshed.
    refresh_duration: Duration,
}

#[derive(Clone, PartialEq, Eq)]
/// Represents the credentials fetched by the `FetcherCredentialsProvider`.
pub struct FetcherCredentials {
    /// The fetched credentials.
    pub credentials: Credentials,
    /// The duration of the expiry window for the credentials.
    pub expiry_window: Duration,
}

impl CredentialsFetcherProvider {
    /// Creates a new `CredentialsFetcherProvider` with the given fetcher and
    /// options.
    ///
    /// # Arguments
    ///
    /// * `fetcher` - An Arc<dyn CredentialsFetcher + Send + Sync> representing
    ///   the fetcher implementation.
    /// * `opt_fns` - A closure that takes a mutable reference to
    ///   `CredentialsFetcherOptions` and allows configuring the options for the
    ///   provider.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::sync::Arc;
    /// # use std::time::{Duration, SystemTime};
    /// #
    /// # use alibabacloud_oss_sdk_rust_v2::credential::{
    /// #     Credentials, CredentialsFetcher, CredentialsFetcherProvider, CredentialsProvider,
    /// # };
    /// # use tokio::sync::RwLock;
    /// #
    /// struct MockCredentialsFetcher;
    ///
    /// #[async_trait::async_trait]
    /// impl CredentialsFetcher for MockCredentialsFetcher {
    ///     async fn fetch(&self) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
    ///         // Return some mock credentials for testing
    ///         Ok(Credentials {
    ///             access_key_id: "mock_access_key".to_string(),
    ///             access_key_secret: "mock_secret_key".to_string(),
    ///             expires: Some(SystemTime::now() + Duration::from_secs(3600)),
    ///             ..Default::default()
    ///         })
    ///     }
    /// }
    ///
    /// let fetcher = Arc::new(MockCredentialsFetcher);
    /// let provider = CredentialsFetcherProvider::new(fetcher.clone(), |options| {
    ///     options.expired_factor = 0.9;
    ///     options.refresh_duration = Duration::from_secs(180);
    /// });
    /// ```
    pub fn new<F>(fetcher: Arc<dyn CredentialsFetcher + Send + Sync>, opt_fns: F) -> Self
    where
        F: FnOnce(&mut CredentialsFetcherOptions),
    {
        let mut options = CredentialsFetcherOptions::default();
        opt_fns(&mut options);

        CredentialsFetcherProvider {
            fetcher_credentials: Arc::new(RwLock::new(None)),
            fetcher,
            expired_factor: options.expired_factor,
            refresh_duration: options.refresh_duration,
        }
    }

    /// Fetches the credentials from the fetcher and updates the cache.
    ///
    /// Returns the fetched credentials on success, or an error if fetching
    /// fails.
    async fn fetch(&self) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
        let fetched_credentials = self.fetcher.fetch().await?;

        self.update_credentials(fetched_credentials.clone()).await;

        Ok(fetched_credentials)
    }

    /// Updates the credentials in the cache with the given credentials.
    ///
    /// # Arguments
    ///
    /// * `credentials` - The new credentials to update the cache with.
    async fn update_credentials(&self, credentials: Credentials) {
        let mut expiry_window =
            if let Some(fetcher_credentials) = self.fetcher_credentials.read().await.as_ref() {
                // Use the expiry window from the current fetcher credentials
                fetcher_credentials.expiry_window
            } else {
                // No fetcher credentials set
                Duration::from_secs(0)
            };

        if let Some(expires) = credentials.expires {
            // Expires has been set
            if let Ok(time_left) = expires.duration_since(SystemTime::now()) {
                if time_left > self.refresh_duration {
                    // Make sure the expiry window is not longer than the
                    // refresh duration
                    expiry_window =
                        Duration::from_secs_f64(time_left.as_secs_f64() * self.expired_factor);
                }
            }
        }

        let fetcher_credentials = FetcherCredentials {
            credentials,
            expiry_window,
        };

        *self.fetcher_credentials.write().await = Some(fetcher_credentials);
    }

    /// Updates the expiry window of the credentials in the cache.
    ///
    /// This method is called when the credentials are soon to expire and a new
    /// fetch is not possible.
    async fn update_expiry_window(&self) {
        let mut fetcher_credentials_option = self.fetcher_credentials.write().await;

        if let Some(fetcher_credentials) = fetcher_credentials_option.as_mut() {
            if fetcher_credentials.expiry_window > self.refresh_duration {
                fetcher_credentials.expiry_window -= self.refresh_duration;
            }
        }
    }

    /// Checks if the given fetcher credentials are expired.
    ///
    /// Returns `true` if the credentials are expired, otherwise `false`.
    ///
    /// # Arguments
    ///
    /// * `fetcher_credentials` - The fetcher credentials to check for
    ///   expiration.
    async fn is_expired(&self, fetcher_credentials: &FetcherCredentials) -> bool {
        if let Some(expiration) = fetcher_credentials.credentials.expires {
            return SystemTime::now() >= expiration;
        }
        // No expiration time
        false
    }

    /// Checks if the given fetcher credentials are soon to expire.
    ///
    /// Returns `true` if the credentials are soon to expire, otherwise `false`.
    ///
    /// # Arguments
    ///
    /// * `fetcher_credentials` - The fetcher credentials to check for soon
    ///   expiration.
    async fn is_soon_expire(&self, fetcher_credentials: &FetcherCredentials) -> bool {
        if let Some(expiration) = fetcher_credentials.credentials.expires {
            return SystemTime::now() + fetcher_credentials.expiry_window >= expiration;
        }
        // No expiration time
        false
    }
}

#[async_trait::async_trait]
impl CredentialsProvider for CredentialsFetcherProvider {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    /// Returns the credentials from the cache if they are valid, otherwise
    /// fetches new credentials and updates the cache.
    ///
    /// Returns the valid credentials on success, or an error if fetching fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::sync::Arc;
    /// # use std::time::{Duration, SystemTime};
    /// #
    /// # use alibabacloud_oss_sdk_rust_v2::credential::{
    /// #     Credentials, CredentialsFetcher, CredentialsFetcherProvider, CredentialsProvider,
    /// # };
    /// # use tokio::sync::RwLock;
    /// #
    /// struct MockCredentialsFetcher;
    ///
    /// #[async_trait::async_trait]
    /// impl CredentialsFetcher for MockCredentialsFetcher {
    ///     async fn fetch(&self) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
    ///         // Return some mock credentials for testing
    ///         Ok(Credentials {
    ///             access_key_id: "mock_access_key".to_string(),
    ///             access_key_secret: "mock_secret_key".to_string(),
    ///             expires: Some(SystemTime::now() + Duration::from_secs(3600)),
    ///             ..Default::default()
    ///         })
    ///     }
    /// }
    ///
    /// # tokio_test::block_on(async {
    /// let fetcher = Arc::new(MockCredentialsFetcher);
    /// let provider = CredentialsFetcherProvider::new(fetcher.clone(), |_| {});
    ///
    /// let credentials = provider.get_credentials().await;
    /// # })
    /// ```
    async fn get_credentials(
        &self,
    ) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
        let credentials_option = self.fetcher_credentials.read().await.clone();

        if let Some(fetcher_credentials) = credentials_option {
            if !self.is_expired(&fetcher_credentials).await {
                if self.is_soon_expire(&fetcher_credentials).await {
                    // Soon to expire credentials
                    if let Ok(fetched_credentials) = self.fetch().await {
                        self.update_credentials(fetched_credentials).await;
                    } else {
                        self.update_expiry_window().await;
                    }
                    return Ok(fetcher_credentials.credentials);
                } else {
                    // Normal credentials
                    return Ok(fetcher_credentials.credentials);
                }
            }
        }
        // No credentials or expired credentials
        let fetched = self.fetcher.fetch().await?;
        self.update_credentials(fetched.clone()).await;
        Ok(fetched)
    }
}

pub fn new_credentials_fetcher_provider(
    fetcher: Arc<dyn CredentialsFetcher + Send + Sync>,
    refresh_duration: Option<Duration>,
    expired_factor: Option<f64>,
) -> impl CredentialsProvider {
    let opt_fn = |options: &mut CredentialsFetcherOptions| {
        options.refresh_duration = refresh_duration.unwrap_or(DEFAULT_REFRESH_DURATION);
        options.expired_factor = expired_factor.unwrap_or(DEFAULT_EXPIRED_FACTOR);
    };
    CredentialsFetcherProvider::new(fetcher, opt_fn)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCredentialsFetcher;

    #[async_trait::async_trait]
    impl CredentialsFetcher for MockCredentialsFetcher {
        async fn fetch(&self) -> Result<Credentials, Box<dyn std::error::Error + Send + Sync>> {
            // Return some mock credentials for testing
            Ok(Credentials {
                access_key_id: "mock_access_key".to_string(),
                access_key_secret: "mock_secret_key".to_string(),
                expires: Some(SystemTime::now() + Duration::from_secs(3600)),
                ..Default::default()
            })
        }
    }

    #[tokio::test]
    async fn test_fetch() {
        let fetcher = Arc::new(MockCredentialsFetcher);
        let provider = CredentialsFetcherProvider::new(fetcher.clone(), |_| {});

        let credentials = provider.fetch().await.ok().unwrap();

        assert_eq!(credentials.access_key_id, "mock_access_key");
        assert_eq!(credentials.access_key_secret, "mock_secret_key");
    }

    #[tokio::test]
    async fn test_get_credentials() {
        let fetcher = Arc::new(MockCredentialsFetcher);
        let provider = CredentialsFetcherProvider::new(fetcher.clone(), |_| {});

        let credentials = provider.get_credentials().await.ok().unwrap();

        assert_eq!(credentials.access_key_id, "mock_access_key");
        assert_eq!(credentials.access_key_secret, "mock_secret_key");
    }

    #[tokio::test]
    async fn test_update_credentials() {
        let fetcher = Arc::new(MockCredentialsFetcher);
        let provider = CredentialsFetcherProvider::new(fetcher.clone(), |_| {});

        provider
            .update_credentials(Credentials {
                access_key_id: "new_access_key".to_string(),
                access_key_secret: "new_secret_key".to_string(),
                expires: Some(SystemTime::now() + Duration::from_secs(3600)),
                ..Default::default()
            })
            .await;

        let credentials = provider.fetcher_credentials.read().await;
        let fetched_credentials = credentials.as_ref().unwrap();

        assert_eq!(
            fetched_credentials.credentials.access_key_id,
            "new_access_key"
        );
        assert_eq!(
            fetched_credentials.credentials.access_key_secret,
            "new_secret_key"
        );
    }

    #[tokio::test]
    async fn test_update_and_get_credentials() {
        let fetcher = Arc::new(MockCredentialsFetcher);
        let provider = CredentialsFetcherProvider::new(fetcher.clone(), |_| {});

        provider
            .update_credentials(Credentials {
                access_key_id: "new_access_key".to_string(),
                access_key_secret: "new_secret_key".to_string(),
                expires: Some(SystemTime::now() + Duration::from_secs(3600)),
                ..Default::default()
            })
            .await;

        let credentials = provider.get_credentials().await.ok().unwrap();

        assert_eq!(credentials.access_key_id, "new_access_key");
        assert_eq!(credentials.access_key_secret, "new_secret_key");
    }

    #[tokio::test]
    async fn test_update_expiry_window() {
        let fetcher = Arc::new(MockCredentialsFetcher);
        let provider = CredentialsFetcherProvider::new(fetcher.clone(), |_| {});

        let init_expiry_window = Duration::from_secs(3600);

        provider
            .update_credentials(Credentials {
                expires: Some(SystemTime::now() + init_expiry_window),
                ..Default::default()
            })
            .await;

        for i in
            0..Duration::from_secs_f64(init_expiry_window.as_secs_f64() * DEFAULT_EXPIRED_FACTOR)
                .as_secs()
                / DEFAULT_REFRESH_DURATION.as_secs()
        {
            use std::ops::Mul;

            provider.update_expiry_window().await;

            let credentials = provider.fetcher_credentials.read().await;
            let fetched_credentials = credentials.as_ref().unwrap();

            assert!(
                fetched_credentials.expiry_window
                    <= Duration::from_secs_f64(
                        init_expiry_window.as_secs_f64() * DEFAULT_EXPIRED_FACTOR
                    ) - DEFAULT_REFRESH_DURATION.mul(i.try_into().unwrap())
            );
        }
    }

    #[tokio::test]
    async fn test_is_expired() {
        let fetcher = MockCredentialsFetcher;
        let provider = CredentialsFetcherProvider::new(Arc::new(fetcher), |_| {});

        let credentials = Credentials {
            expires: Some(SystemTime::now() - Duration::from_secs(3600)),
            ..Default::default()
        };

        let fetcher_credentials = FetcherCredentials {
            credentials,
            expiry_window: Duration::from_secs(120),
        };

        assert!(provider.is_expired(&fetcher_credentials).await);
    }

    #[tokio::test]
    async fn test_is_soon_expire() {
        let fetcher = MockCredentialsFetcher;
        let provider = CredentialsFetcherProvider::new(Arc::new(fetcher), |_| {});

        let credentials = Credentials {
            expires: Some(SystemTime::now() + Duration::from_secs(100)),
            ..Default::default()
        };

        let fetcher_credentials = FetcherCredentials {
            credentials,
            expiry_window: Duration::from_secs(120),
        };

        assert!(provider.is_soon_expire(&fetcher_credentials).await);
    }
}
