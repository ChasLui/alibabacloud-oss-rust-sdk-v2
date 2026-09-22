mod http;

use std::time::Duration;

use reqwest::tls;
use url::Url;

pub use self::http::*;

pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
pub const DEFAULT_READ_WRITE_TIMEOUT: Duration = Duration::from_secs(10);
pub const DEFAULT_IDLE_CONNECTION_TIMEOUT: Duration = Duration::from_secs(50);
pub const DEFAULT_EXPECT_CONTINUE_TIMEOUT: Duration = Duration::from_secs(1);
pub const DEFAULT_KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_MAX_CONNECTIONS: usize = 100;
pub const DEFAULT_TLS_MIN_VERSION: tls::Version = tls::Version::TLS_1_2;

/// Configuration options for the HTTP transport.
///
/// # Note
///
/// All fields are optional for merging other configurations
#[derive(Clone)]
pub struct TransportConfig {
    /// The timeout for establishing a connection.
    pub connect_timeout: Option<Duration>,
    /// The timeout for read and write operations.
    pub read_write_timeout: Option<Duration>,
    /// The timeout for idle connections.
    pub idle_connection_timeout: Option<Duration>,
    /// The timeout for keeping the connection alive.
    pub keep_alive_timeout: Option<Duration>,
    /// Indicates whether redirects are enabled.
    pub enabled_redirect: Option<bool>,
    /// Indicates whether to skip verification of TLS certificates.
    pub insecure_skip_verify: Option<bool>,
    /// The maximum number of connections to keep open.
    pub max_connections: Option<usize>,
    /// The minimum version of TLS to use.
    pub tls_min_version: Option<tls::Version>,
    /// The HTTP and HTTPS proxy to use.
    pub all_proxy: Option<Url>,
    /// Indicates whether to use the proxy specified in the environment
    /// variables. Only valid when `all_proxy` is not set.
    pub use_env_proxy: Option<bool>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        TransportConfig {
            connect_timeout: Some(DEFAULT_CONNECT_TIMEOUT),
            read_write_timeout: Some(DEFAULT_READ_WRITE_TIMEOUT),
            idle_connection_timeout: Some(DEFAULT_IDLE_CONNECTION_TIMEOUT),
            keep_alive_timeout: Some(DEFAULT_KEEP_ALIVE_TIMEOUT),
            enabled_redirect: None,
            insecure_skip_verify: None,
            max_connections: None,
            tls_min_version: None,
            all_proxy: None,
            use_env_proxy: Some(true),
        }
    }
}

impl TransportConfig {
    /// Merge the configuration settings from another `TransportConfig` instance
    /// into the current instance.
    pub fn merge_in(&mut self, other: &TransportConfig) {
        if let Some(connect_timeout) = other.connect_timeout {
            self.connect_timeout = Some(connect_timeout);
        }
        if let Some(read_write_timeout) = other.read_write_timeout {
            self.read_write_timeout = Some(read_write_timeout);
        }
        if let Some(idle_connection_timeout) = other.idle_connection_timeout {
            self.idle_connection_timeout = Some(idle_connection_timeout);
        }
        if let Some(keep_alive_timeout) = other.keep_alive_timeout {
            self.keep_alive_timeout = Some(keep_alive_timeout);
        }
        if let Some(enabled_redirect) = other.enabled_redirect {
            self.enabled_redirect = Some(enabled_redirect);
        }
        if let Some(insecure_skip_verify) = other.insecure_skip_verify {
            self.insecure_skip_verify = Some(insecure_skip_verify);
        }
        if let Some(max_connections) = other.max_connections {
            self.max_connections = Some(max_connections);
        }
        if let Some(tls_min_version) = other.tls_min_version {
            self.tls_min_version = Some(tls_min_version);
        }
        if let Some(http_proxy) = &other.all_proxy {
            self.all_proxy = Some(http_proxy.clone());
        }
        if let Some(http_proxy_env) = other.use_env_proxy {
            self.use_env_proxy = Some(http_proxy_env);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_in() {
        let mut config1 = TransportConfig::default();

        // Modify some fields in config2
        let config2 = TransportConfig {
            connect_timeout: Some(Duration::from_secs(3)),
            read_write_timeout: Some(Duration::from_secs(6)),
            idle_connection_timeout: Some(Duration::from_secs(12)),
            keep_alive_timeout: Some(Duration::from_secs(15)),
            enabled_redirect: Some(true),
            insecure_skip_verify: Some(true),
            max_connections: Some(50),
            tls_min_version: Some(tls::Version::TLS_1_3),
            all_proxy: Some(Url::parse("http://proxy.example.com").unwrap()),
            use_env_proxy: Some(false),
        };

        // Merge config2 into config1
        config1.merge_in(&config2);

        // Check if the fields in config1 are updated correctly
        assert_eq!(config1.connect_timeout, Some(Duration::from_secs(3)));
        assert_eq!(config1.read_write_timeout, Some(Duration::from_secs(6)));
        assert_eq!(
            config1.idle_connection_timeout,
            Some(Duration::from_secs(12))
        );
        assert_eq!(config1.keep_alive_timeout, Some(Duration::from_secs(15)));
        assert_eq!(config1.enabled_redirect, Some(true));
        assert_eq!(config1.insecure_skip_verify, Some(true));
        assert_eq!(config1.max_connections, Some(50));
        assert_eq!(config1.tls_min_version, Some(tls::Version::TLS_1_3));
        assert_eq!(
            config1.all_proxy,
            Some(Url::parse("http://proxy.example.com").unwrap())
        );
        assert_eq!(config1.use_env_proxy, Some(false));
    }
}
