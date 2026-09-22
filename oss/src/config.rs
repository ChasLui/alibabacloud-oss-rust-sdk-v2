use std::net::IpAddr;
use std::rc::Rc;
use std::time::Duration;

use crate::credential::CredentialsProvider;
use crate::log::{LogLevel, LogPrinter};
use crate::retry::Retryer;
use crate::{SignatureVersionType, ENV_OSS_SDK_LOG_LEVEL};

#[derive(Default, Clone)]
pub struct Config {
    pub region: Option<String>,
    pub endpoint: Option<String>,
    pub retry_max_attempts: Option<u32>,
    pub retryer: Option<Rc<dyn Retryer>>,
    pub http_client: Option<reqwest::Client>,
    pub credentials_provider: Option<Rc<dyn CredentialsProvider>>,
    pub use_path_style: Option<bool>,
    pub use_cname: Option<bool>,
    pub connect_timeout: Option<Duration>,
    pub read_write_timeout: Option<Duration>,
    /// Maximum number of idle connections kept per host.
    pub max_connections: Option<usize>,
    pub insecure_skip_verify: Option<bool>,
    pub enabled_redirect: Option<bool>,
    pub proxy_host: Option<String>,
    pub proxy_from_environment: Option<bool>,
    /// in KBps
    pub upload_bandwidth_limit: Option<u64>,
    /// in KBps
    pub download_bandwidth_limit: Option<u64>,
    pub signature_version: Option<SignatureVersionType>,
    pub log_level: Option<LogLevel>,
    pub log_printer: Option<Rc<dyn LogPrinter>>,
    pub disable_ssl: Option<bool>,
    pub use_dual_stack_endpoint: Option<bool>,
    pub use_accelerate_endpoint: Option<bool>,
    pub use_internal_endpoint: Option<bool>,
    /// Cloud box ID. When set, it becomes the region and switches signing to
    /// the cloud box product.
    pub cloud_box_id: Option<String>,
    /// Derives the cloud box ID from a cloud box endpoint.
    pub enable_auto_detect_cloud_box_id: Option<bool>,
    /// Account ID. Products such as vectors and agentic buckets address a
    /// bucket by a name derived from it.
    pub account_id: Option<String>,
    pub disable_upload_crc64_check: Option<bool>,
    pub disable_download_crc64_check: Option<bool>,
    pub additional_headers: Vec<String>,
    /// Headers sent with every request, unless the operation sets the same
    /// header itself.
    pub default_request_headers: Vec<(String, String)>,
    pub user_agent: Option<String>,
    /// Local address outgoing connections are bound to.
    pub bind_address: Option<IpAddr>,
}

impl Config {
    pub fn new() -> Self {
        Config::default()
    }

    pub fn load_default_config() -> Self {
        let mut config = Config::new();
        // load from env
        if let Ok(log_level_string) = std::env::var(ENV_OSS_SDK_LOG_LEVEL) {
            let log_level = LogLevel::from(log_level_string.as_str());
            if log_level != LogLevel::Off {
                config.log_level = Some(log_level);
            }
        }
        config
    }

    pub fn with_region(mut self, region: &str) -> Self {
        self.region = Some(region.to_string());
        self
    }

    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = Some(endpoint.to_string());
        self
    }

    pub fn with_retry_max_attempts(mut self, retry_max_attempts: u32) -> Self {
        self.retry_max_attempts = Some(retry_max_attempts);
        self
    }

    pub fn with_retryer(mut self, retryer: Rc<dyn Retryer>) -> Self {
        self.retryer = Some(retryer);
        self
    }

    pub fn with_http_client(mut self, http_client: reqwest::Client) -> Self {
        self.http_client = Some(http_client);
        self
    }

    pub fn with_credentials_provider(
        mut self,
        credentials_provider: Rc<dyn CredentialsProvider>,
    ) -> Self {
        self.credentials_provider = Some(credentials_provider);
        self
    }

    pub fn with_use_path_style(mut self, use_path_style: bool) -> Self {
        self.use_path_style = Some(use_path_style);
        self
    }

    pub fn with_use_cname(mut self, use_cname: bool) -> Self {
        self.use_cname = Some(use_cname);
        self
    }

    pub fn with_connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = Some(connect_timeout);
        self
    }

    pub fn with_read_write_timeout(mut self, read_write_timeout: Duration) -> Self {
        self.read_write_timeout = Some(read_write_timeout);
        self
    }

    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = Some(max_connections);
        self
    }

    pub fn with_insecure_skip_verify(mut self, insecure_skip_verify: bool) -> Self {
        self.insecure_skip_verify = Some(insecure_skip_verify);
        self
    }

    pub fn with_enabled_redirect(mut self, enabled_redirect: bool) -> Self {
        self.enabled_redirect = Some(enabled_redirect);
        self
    }

    pub fn with_proxy_host(mut self, proxy_host: &str) -> Self {
        self.proxy_host = Some(proxy_host.to_string());
        self
    }

    pub fn with_proxy_from_environment(mut self, proxy_from_environment: bool) -> Self {
        self.proxy_from_environment = Some(proxy_from_environment);
        self
    }

    pub fn with_upload_bandwidth_limit(mut self, upload_bandwidth_limit: u64) -> Self {
        self.upload_bandwidth_limit = Some(upload_bandwidth_limit);
        self
    }

    pub fn with_download_bandwidth_limit(mut self, download_bandwidth_limit: u64) -> Self {
        self.download_bandwidth_limit = Some(download_bandwidth_limit);
        self
    }

    pub fn with_signature_version(mut self, signature_version: SignatureVersionType) -> Self {
        self.signature_version = Some(signature_version);
        self
    }

    pub fn with_log_level(mut self, log_level: LogLevel) -> Self {
        self.log_level = Some(log_level);
        self
    }

    pub fn with_log_printer(mut self, log_printer: Rc<dyn LogPrinter>) -> Self {
        self.log_printer = Some(log_printer);
        self
    }

    pub fn with_disable_ssl(mut self, disable_ssl: bool) -> Self {
        self.disable_ssl = Some(disable_ssl);
        self
    }

    pub fn with_use_dual_stack_endpoint(mut self, use_dual_stack_endpoint: bool) -> Self {
        self.use_dual_stack_endpoint = Some(use_dual_stack_endpoint);
        self
    }

    pub fn with_use_accelerate_endpoint(mut self, use_accelerate_endpoint: bool) -> Self {
        self.use_accelerate_endpoint = Some(use_accelerate_endpoint);
        self
    }

    pub fn with_use_internal_endpoint(mut self, use_internal_endpoint: bool) -> Self {
        self.use_internal_endpoint = Some(use_internal_endpoint);
        self
    }

    pub fn with_cloud_box_id(mut self, cloud_box_id: &str) -> Self {
        self.cloud_box_id = Some(cloud_box_id.to_string());
        self
    }

    pub fn with_enable_auto_detect_cloud_box_id(
        mut self,
        enable_auto_detect_cloud_box_id: bool,
    ) -> Self {
        self.enable_auto_detect_cloud_box_id = Some(enable_auto_detect_cloud_box_id);
        self
    }

    pub fn with_account_id(mut self, account_id: &str) -> Self {
        self.account_id = Some(account_id.to_string());
        self
    }

    pub fn with_disable_upload_crc64_check(mut self, disable_upload_crc64_check: bool) -> Self {
        self.disable_upload_crc64_check = Some(disable_upload_crc64_check);
        self
    }

    pub fn with_disable_download_crc64_check(mut self, disable_download_crc64_check: bool) -> Self {
        self.disable_download_crc64_check = Some(disable_download_crc64_check);
        self
    }

    pub fn with_additional_headers(mut self, additional_headers: Vec<String>) -> Self {
        self.additional_headers = additional_headers;
        self
    }

    pub fn with_default_request_headers(
        mut self,
        default_request_headers: Vec<(String, String)>,
    ) -> Self {
        self.default_request_headers = default_request_headers;
        self
    }

    pub fn with_bind_address(mut self, bind_address: IpAddr) -> Self {
        self.bind_address = Some(bind_address);
        self
    }

    pub fn with_user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = Some(user_agent.to_string());
        self
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::StandardLogPrinter;

    #[test]
    fn test_new() {
        let config = Config::new();
        assert!(config.region.is_none());
        assert!(config.endpoint.is_none());
        assert!(config.retry_max_attempts.is_none());
        assert!(config.retryer.is_none());
        assert!(config.http_client.is_none());
        assert!(config.credentials_provider.is_none());
        assert!(config.use_path_style.is_none());
        assert!(config.use_cname.is_none());
        assert!(config.connect_timeout.is_none());
        assert!(config.read_write_timeout.is_none());
        assert!(config.insecure_skip_verify.is_none());
        assert!(config.enabled_redirect.is_none());
        assert!(config.proxy_host.is_none());
        assert!(config.proxy_from_environment.is_none());
        assert!(config.upload_bandwidth_limit.is_none());
        assert!(config.download_bandwidth_limit.is_none());
        assert!(config.signature_version.is_none());
        assert!(config.log_level.is_none());
        assert!(config.log_printer.is_none());
        assert!(config.disable_ssl.is_none());
        assert!(config.use_dual_stack_endpoint.is_none());
        assert!(config.use_accelerate_endpoint.is_none());
        assert!(config.use_internal_endpoint.is_none());
        assert!(config.cloud_box_id.is_none());
        assert!(config.enable_auto_detect_cloud_box_id.is_none());
        assert!(config.disable_upload_crc64_check.is_none());
        assert!(config.disable_download_crc64_check.is_none());
        assert!(config.additional_headers.is_empty());
        assert!(config.default_request_headers.is_empty());
        assert!(config.user_agent.is_none());
        assert!(config.bind_address.is_none());
    }

    #[test]
    fn test_load_default_config_with_log_level() {
        std::env::set_var(ENV_OSS_SDK_LOG_LEVEL, "debug");
        let config = Config::load_default_config();
        assert_eq!(config.log_level, Some(LogLevel::Debug));
        std::env::remove_var(ENV_OSS_SDK_LOG_LEVEL);
    }

    #[test]
    fn test_with_region() {
        let config = Config::new().with_region("hangzhou");
        assert_eq!(config.region, Some("hangzhou".to_string()));
    }

    #[test]
    fn test_with_endpoint() {
        let config = Config::new().with_endpoint("oss-cn-hangzhou.aliyuncs.com");
        assert_eq!(
            config.endpoint,
            Some("oss-cn-hangzhou.aliyuncs.com".to_string())
        );
    }

    #[test]
    fn test_with_retry_max_attempts() {
        let config = Config::new().with_retry_max_attempts(3);
        assert_eq!(config.retry_max_attempts, Some(3));
    }

    #[test]
    fn test_with_retryer() {
        let config = Config::new().with_retryer(Rc::new(crate::retry::NopRetryer::new()));
        assert!(config.retryer.is_some());
    }

    #[test]
    fn test_with_http_client() {
        let config = Config::new().with_http_client(reqwest::Client::new());
        assert!(config.http_client.is_some());
    }

    #[test]
    fn test_with_credentials_provider() {
        let config = Config::new()
            .with_credentials_provider(Rc::new(crate::credential::AnonymousCredentialsProvider));
        assert!(config.credentials_provider.is_some());
    }

    #[test]
    fn test_with_use_path_style() {
        let config = Config::new().with_use_path_style(true);
        assert_eq!(config.use_path_style, Some(true));
    }

    #[test]
    fn test_with_use_cname() {
        let config = Config::new().with_use_cname(true);
        assert_eq!(config.use_cname, Some(true));
    }

    #[test]
    fn test_with_connect_timeout() {
        let config = Config::new().with_connect_timeout(Duration::from_secs(30));
        assert_eq!(config.connect_timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_with_max_connections() {
        let config = Config::new().with_max_connections(64);
        assert_eq!(config.max_connections, Some(64));
    }

    #[test]
    fn test_with_read_write_timeout() {
        let config = Config::new().with_read_write_timeout(Duration::from_secs(30));
        assert_eq!(config.read_write_timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_with_insecure_skip_verify() {
        let config = Config::new().with_insecure_skip_verify(true);
        assert_eq!(config.insecure_skip_verify, Some(true));
    }

    #[test]
    fn test_with_enabled_redirect() {
        let config = Config::new().with_enabled_redirect(true);
        assert_eq!(config.enabled_redirect, Some(true));
    }

    #[test]
    fn test_with_proxy_host() {
        let config = Config::new().with_proxy_host("127.0.0.1:8080");
        assert_eq!(config.proxy_host, Some("127.0.0.1:8080".to_string()));
    }

    #[test]
    fn test_with_proxy_from_environment() {
        let config = Config::new().with_proxy_from_environment(true);
        assert_eq!(config.proxy_from_environment, Some(true));
    }

    #[test]
    fn test_with_upload_bandwidth_limit() {
        let config = Config::new().with_upload_bandwidth_limit(1024);
        assert_eq!(config.upload_bandwidth_limit, Some(1024));
    }
    #[test]
    fn test_with_download_bandwidth_limit() {
        let config = Config::new().with_download_bandwidth_limit(1024);
        assert_eq!(config.download_bandwidth_limit, Some(1024));
    }

    #[test]
    fn test_with_signature_version() {
        let config = Config::new().with_signature_version(SignatureVersionType::V4);
        assert_eq!(config.signature_version, Some(SignatureVersionType::V4));
    }

    #[test]
    fn test_with_log_level() {
        let config = Config::new().with_log_level(LogLevel::Debug);
        assert_eq!(config.log_level, Some(LogLevel::Debug));
    }

    #[test]
    fn test_with_log_printer() {
        let config = Config::new().with_log_printer(Rc::new(StandardLogPrinter::default()));
        assert!(config.log_printer.is_some());
    }

    #[test]
    fn test_with_disable_ssl() {
        let config = Config::new().with_disable_ssl(true);
        assert_eq!(config.disable_ssl, Some(true));
    }

    #[test]
    fn test_with_use_dual_stack_endpoint() {
        let config = Config::new().with_use_dual_stack_endpoint(true);
        assert_eq!(config.use_dual_stack_endpoint, Some(true));
    }

    #[test]
    fn test_with_use_accelerate_endpoint() {
        let config = Config::new().with_use_accelerate_endpoint(true);
        assert_eq!(config.use_accelerate_endpoint, Some(true));
    }

    #[test]
    fn test_with_use_internal_endpoint() {
        let config = Config::new().with_use_internal_endpoint(true);
        assert_eq!(config.use_internal_endpoint, Some(true));
    }

    #[test]
    fn test_with_disable_upload_crc64_check() {
        let config = Config::new().with_disable_upload_crc64_check(true);
        assert_eq!(config.disable_upload_crc64_check, Some(true));
    }

    #[test]
    fn test_with_disable_download_crc64_check() {
        let config = Config::new().with_disable_download_crc64_check(true);
        assert_eq!(config.disable_download_crc64_check, Some(true));
    }

    #[test]
    fn test_with_additional_headers() {
        let headers = vec![
            "Content-Type: application/json".to_string(),
            "X-Custom-Header: value".to_string(),
        ];
        let config = Config::new().with_additional_headers(headers.clone());
        assert_eq!(config.additional_headers, headers);
        // Add assertions for other fields
    }

    #[test]
    fn test_with_user_agent() {
        let config = Config::new().with_user_agent("MyApp/1.0");
        assert_eq!(config.user_agent, Some("MyApp/1.0".to_string()));
        // Add assertions for other fields
    }

    #[test]
    fn test_with_default_request_headers() {
        let config = Config::new()
            .with_default_request_headers(vec![("x-custom".to_string(), "value".to_string())]);
        assert_eq!(
            config.default_request_headers,
            vec![("x-custom".to_string(), "value".to_string())]
        );
    }

    #[test]
    fn test_with_bind_address() {
        let address: IpAddr = "10.0.0.5".parse().unwrap();
        let config = Config::new().with_bind_address(address);
        assert_eq!(config.bind_address, Some(address));
    }
}
