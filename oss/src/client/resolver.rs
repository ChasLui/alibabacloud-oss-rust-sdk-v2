use std::net::IpAddr;
use std::rc::Rc;
use std::str::FromStr;
#[allow(unused_imports)]
use std::sync::Arc;

#[allow(unused_imports)]
use url::Url;

use super::ClientOptions;
use crate::config::Config;
use crate::retry::Standard as StandardRetryer;
use crate::signer::{SignerV1, SignerV4};
use crate::transport::{self, TransportConfig};
#[allow(unused_imports)]
use crate::utils::{
    add_endpoint_scheme, endpoint_from_region, is_valid_region, BwTokenBucket, EndpointType,
    DEFAULT_USER_AGENT,
};
use crate::{
    FeatureFlagsType, SignatureVersionType, UrlStyleType, CLOUD_BOX_PRODUCT,
    DEFAULT_SIGNATURE_VERSION,
};

/// Resolves the endpoint based on the provided configuration and updates the
/// client options. If a custom endpoint is specified, it is used. Otherwise,
/// the endpoint is determined based on the region and other configuration
/// options. The resolved endpoint is then set in the `client_options` struct.
pub fn resolve_endpoint(config: &Config, client_options: &mut ClientOptions) {
    let disable_ssl = config.disable_ssl.unwrap_or(false);
    let mut endpoint = config.endpoint.clone().unwrap_or_default();
    let region = config.region.clone().unwrap_or_default();

    if !endpoint.is_empty() {
        endpoint = add_endpoint_scheme(&endpoint, disable_ssl);
    } else if is_valid_region(&region) {
        endpoint = endpoint_from_region(
            &region,
            disable_ssl,
            if config.use_internal_endpoint.unwrap_or(false) {
                EndpointType::Internal
            } else if config.use_dual_stack_endpoint.unwrap_or(false) {
                EndpointType::DualStack
            } else if config.use_accelerate_endpoint.unwrap_or(false) {
                EndpointType::Accelerate
            } else {
                EndpointType::Public
            },
        );
    }

    if !endpoint.is_empty() {
        if let Ok(parsed_url) = Url::from_str(&endpoint) {
            client_options.endpoint = Some(parsed_url);
        }
    }
}

/// Resolves the retryer based on the provided configuration and updates the
/// client options. If a retryer is not already set in the `client_options`, a
/// default `StandardRetryer` is created and set.
pub fn resolve_retryer(_config: &Config, client_options: &mut ClientOptions) {
    if client_options.retryer.is_none() {
        client_options.retryer = Some(Rc::new(StandardRetryer::new()));
    }
}

/// Resolves the HTTP client based on the provided configuration and updates the
/// client options. If an HTTP client is not already set in the
/// `client_options`, a new HTTP client is created and set using the provided
/// transport configuration.
pub fn resolve_http_client(config: &Config, client_options: &mut ClientOptions) {
    if client_options.http_client.is_some() {
        return;
    }

    let transport_config = TransportConfig {
        connect_timeout: config.connect_timeout,
        read_write_timeout: config.read_write_timeout,
        max_connections: config.max_connections,
        enabled_redirect: config.enabled_redirect,
        insecure_skip_verify: config.insecure_skip_verify,
        use_env_proxy: config.proxy_from_environment,
        all_proxy: config
            .proxy_host
            .as_ref()
            .map(|proxy_string| proxy_string.parse().expect("Invalid proxy string")),
        bind_address: config.bind_address,
        ..Default::default()
    };

    client_options.http_client = Some(
        transport::new_http_client_builder(&transport_config, &[])
            .build()
            .expect("Failed to create HTTP client"),
    );
}

/// Resolves the bandwidth limits into usable limiters.
///
/// A limit is configured in KBps and becomes a limiter in bytes per second.
/// The limiter is attached to the client's options rather than to raw sockets,
/// because reqwest does its own socket handling: the only place the SDK can
/// pace traffic is the body streams it hands to and receives from reqwest.
pub fn resolve_bandwidth_limit(config: &Config, client_options: &mut ClientOptions) {
    if let Some(limit) = config.upload_bandwidth_limit.filter(|limit| *limit > 0) {
        client_options.upload_bandwidth_limiter = Some(Arc::new(BwTokenBucket::new(limit * 1024)));
    }
    if let Some(limit) = config.download_bandwidth_limit.filter(|limit| *limit > 0) {
        client_options.download_bandwidth_limiter =
            Some(Arc::new(BwTokenBucket::new(limit * 1024)));
    }
}

/// Resolves the signer based on the provided configuration and updates the
/// client options. If a signer is not already set in the `client_options`, a
/// signer is selected based on the configured signature version.
pub fn resolve_signer(config: &Config, client_options: &mut ClientOptions) {
    if client_options.signer.is_some() {
        return;
    }

    match if let Some(config_version) = config.signature_version {
        config_version
    } else {
        DEFAULT_SIGNATURE_VERSION
    } {
        SignatureVersionType::V1 => client_options.signer = Some(Rc::new(SignerV1 {})),
        SignatureVersionType::V4 => client_options.signer = Some(Rc::new(SignerV4 {})),
    }
}

/// Resolves the URL style based on the provided configuration and updates the
/// client options. The URL style can be set to CName, Path, or VirtualHosted
/// based on the configuration options. If the endpoint is an IP address, the
/// URL style is set to Path.
pub fn resolve_url_style(config: &Config, client_options: &mut ClientOptions) {
    if config.use_cname.unwrap_or(false) {
        client_options.url_style = UrlStyleType::CName;
    } else if config.use_path_style.unwrap_or(false) {
        client_options.url_style = UrlStyleType::Path;
    } else {
        client_options.url_style = UrlStyleType::VirtualHosted;
    }

    // if the endpoint is ip, set to path-style
    if let Some(endpoint) = &client_options.endpoint {
        if let Some(host_str) = endpoint.host_str() {
            if host_str.parse::<IpAddr>().is_ok() {
                client_options.url_style = UrlStyleType::Path;
            }
        }
    }
}

/// Resolves the feature flags based on the provided configuration and updates
/// the client options. The feature flags control various features of the
/// client, such as CRC64 check for uploads and downloads.
pub fn resolve_feature_flags(config: &Config, client_options: &mut ClientOptions) {
    if config.disable_download_crc64_check.unwrap_or(false) {
        client_options.feature_flags &= !FeatureFlagsType::ENABLE_CRC64_CHECK_DOWNLOAD;
    }
    if config.disable_upload_crc64_check.unwrap_or(false) {
        client_options.feature_flags &= !FeatureFlagsType::ENABLE_CRC64_CHECK_UPLOAD;
    }
}

/// Resolves the cloud box settings.
///
/// A cloud box is addressed by its ID instead of a region, and signs with the
/// cloud box product. The ID can also be derived from the endpoint, which is
/// the only place a cloud box deployment announces itself. Mirrors Go's
/// `resolveCloudBox`.
pub fn resolve_cloud_box(config: &Config, client_options: &mut ClientOptions) {
    if let Some(cloud_box_id) = &config.cloud_box_id {
        client_options.region = cloud_box_id.clone();
        client_options.product = CLOUD_BOX_PRODUCT.to_string();
        return;
    }

    if !config.enable_auto_detect_cloud_box_id.unwrap_or(false) {
        return;
    }

    let Some(endpoint) = &client_options.endpoint else {
        return;
    };
    let Some(host) = endpoint.host_str() else {
        return;
    };

    // cb-***.{region}.oss-cloudbox.aliyuncs.com
    // cb-***.{region}.oss-cloudbox-control.aliyuncs.com
    if !(host.ends_with(".oss-cloudbox.aliyuncs.com")
        || host.ends_with(".oss-cloudbox-control.aliyuncs.com"))
    {
        return;
    }

    let keys: Vec<&str> = host.split('.').collect();
    if keys.len() != 5 || !keys[0].starts_with("cb-") {
        return;
    }

    client_options.region = keys[0].to_string();
    client_options.product = CLOUD_BOX_PRODUCT.to_string();
}

/// Builds the user agent string based on the provided configuration.
/// If a custom user agent is specified, it is appended to the default user
/// agent string.
pub fn build_user_agent(config: &Config) -> String {
    match config.user_agent.as_ref() {
        Some(user_agent) => format!("{}/{}", DEFAULT_USER_AGENT.clone(), user_agent),
        None => DEFAULT_USER_AGENT.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_endpoint_with_custom_endpoint() {
        let config = Config::default().with_endpoint("https://custom-endpoint.com");
        let mut client_options = ClientOptions::default();

        resolve_endpoint(&config, &mut client_options);

        assert_eq!(
            client_options.endpoint.unwrap().as_str(),
            "https://custom-endpoint.com/"
        );
    }

    #[test]
    fn test_resolve_endpoint_with_valid_region() {
        let config = Config::default().with_region("hangzhou");
        let mut client_options = ClientOptions::default();

        resolve_endpoint(&config, &mut client_options);

        assert_eq!(
            client_options.endpoint.unwrap().as_str(),
            "https://oss-hangzhou.aliyuncs.com/"
        );
    }

    #[test]
    fn test_resolve_retryer_with_existing_retryer() {
        let config = Config::default();
        let mut client_options = ClientOptions {
            retryer: Some(Rc::new(StandardRetryer::new())),
            ..Default::default()
        };

        resolve_retryer(&config, &mut client_options);

        assert!(client_options.retryer.is_some());
    }

    #[test]
    fn test_resolve_retryer_without_existing_retryer() {
        let config = Config::default();
        let mut client_options = ClientOptions::default();

        resolve_retryer(&config, &mut client_options);

        assert!(client_options.retryer.is_some());
    }

    #[test]
    fn test_resolve_http_client_with_existing_http_client() {
        let config = Config::default();
        let mut client_options = ClientOptions {
            http_client: Some(
                transport::new_http_client_builder(&TransportConfig::default(), &[])
                    .build()
                    .unwrap(),
            ),
            ..Default::default()
        };

        resolve_http_client(&config, &mut client_options);

        assert!(client_options.http_client.is_some());
    }

    #[test]
    fn test_resolve_http_client_without_existing_http_client() {
        let config = Config::default();
        let mut client_options = ClientOptions::default();

        resolve_http_client(&config, &mut client_options);

        assert!(client_options.http_client.is_some());
    }

    #[test]
    fn test_resolve_signer_with_existing_signer() {
        let config = Config::default();
        let mut client_options = ClientOptions {
            signer: Some(Rc::new(SignerV1 {})),
            ..Default::default()
        };

        resolve_signer(&config, &mut client_options);

        assert!(client_options.signer.is_some());
    }

    #[test]
    fn test_resolve_signer_without_existing_signer() {
        let config = Config::default();
        let mut client_options = ClientOptions::default();

        resolve_signer(&config, &mut client_options);

        assert!(client_options.signer.is_some());
    }

    #[test]
    fn test_resolve_url_style_with_cname() {
        let config = Config::default().with_use_cname(true);
        let mut client_options = ClientOptions::default();

        resolve_url_style(&config, &mut client_options);

        assert_eq!(client_options.url_style, UrlStyleType::CName);
    }

    #[test]
    fn test_resolve_url_style_with_path_style() {
        let config = Config::default().with_use_path_style(true);
        let mut client_options = ClientOptions::default();

        resolve_url_style(&config, &mut client_options);

        assert_eq!(client_options.url_style, UrlStyleType::Path);
    }

    #[test]
    fn test_resolve_url_style_with_virtual_hosted() {
        let config = Config::default();
        let mut client_options = ClientOptions::default();

        resolve_url_style(&config, &mut client_options);

        assert_eq!(client_options.url_style, UrlStyleType::VirtualHosted);
    }

    #[test]
    fn test_resolve_url_style_with_ip_endpoint() {
        let config = Config::default().with_endpoint("https://192.168.0.1");
        let mut client_options = ClientOptions::default();

        resolve_endpoint(&config, &mut client_options); // resolve endpoint
                                                        // first
        resolve_url_style(&config, &mut client_options);

        assert_eq!(client_options.url_style, UrlStyleType::Path);
    }

    #[test]
    fn test_resolve_feature_flags_with_disabled_download_crc64_check() {
        let config = Config::default().with_disable_download_crc64_check(true);
        let mut client_options = ClientOptions {
            feature_flags: FeatureFlagsType::ENABLE_CRC64_CHECK_DOWNLOAD,
            ..Default::default()
        };

        resolve_feature_flags(&config, &mut client_options);

        assert!(client_options.feature_flags.is_empty());
    }

    #[test]
    fn test_resolve_feature_flags_with_disabled_upload_crc64_check() {
        let config = Config::default().with_disable_upload_crc64_check(true);
        let mut client_options = ClientOptions {
            feature_flags: FeatureFlagsType::ENABLE_CRC64_CHECK_UPLOAD,
            ..Default::default()
        };

        resolve_feature_flags(&config, &mut client_options);

        assert!(client_options.feature_flags.is_empty());
    }

    #[test]
    fn test_build_user_agent_with_default_user_agent() {
        let config = Config::default();

        let user_agent = build_user_agent(&config);

        assert_eq!(user_agent, *DEFAULT_USER_AGENT);
    }

    #[test]
    fn test_build_user_agent_with_custom_user_agent() {
        let config = Config::default().with_user_agent("CustomAgent/1.0");

        let user_agent = build_user_agent(&config);

        assert!(user_agent.starts_with("alibabacloud-oss-sdk-rust-v2/"));
        assert!(user_agent.ends_with("/CustomAgent/1.0"));
    }

    #[test]
    fn test_resolve_cloud_box_from_config() {
        let config = Config::default().with_cloud_box_id("cb-12345");
        let mut client_options = ClientOptions::default();

        resolve_cloud_box(&config, &mut client_options);

        assert_eq!(client_options.region, "cb-12345");
        assert_eq!(client_options.product, CLOUD_BOX_PRODUCT);
    }

    #[test]
    fn test_resolve_cloud_box_detected_from_endpoint() {
        let config = Config::default().with_enable_auto_detect_cloud_box_id(true);
        let mut client_options = ClientOptions {
            endpoint: Some(
                Url::parse("https://cb-12345.cn-hangzhou.oss-cloudbox.aliyuncs.com").unwrap(),
            ),
            ..Default::default()
        };

        resolve_cloud_box(&config, &mut client_options);

        assert_eq!(client_options.region, "cb-12345");
        assert_eq!(client_options.product, CLOUD_BOX_PRODUCT);
    }

    #[test]
    fn test_resolve_cloud_box_ignores_ordinary_endpoint() {
        let config = Config::default().with_enable_auto_detect_cloud_box_id(true);
        let mut client_options = ClientOptions {
            endpoint: Some(Url::parse("https://oss-cn-hangzhou.aliyuncs.com").unwrap()),
            product: crate::DEFAULT_PRODUCT.to_string(),
            ..Default::default()
        };

        resolve_cloud_box(&config, &mut client_options);

        assert_eq!(client_options.region, "");
        assert_eq!(client_options.product, crate::DEFAULT_PRODUCT);
    }
}
