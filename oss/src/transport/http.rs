use std::env;
use std::sync::Arc;

use reqwest::{redirect, Client, ClientBuilder, Proxy};
use url::Url;

use super::{
    TransportConfig, DEFAULT_CONNECT_TIMEOUT, DEFAULT_IDLE_CONNECTION_TIMEOUT,
    DEFAULT_KEEP_ALIVE_TIMEOUT, DEFAULT_MAX_CONNECTIONS, DEFAULT_READ_WRITE_TIMEOUT,
    DEFAULT_TLS_MIN_VERSION,
};

/// Creates a new `ClientBuilder` for an HTTP client with the given
/// configuration and transport modifiers.
///
/// # Arguments
///
/// * `config` - The configuration for the HTTP client.
/// * `transport_modifiers` - A slice of transport modifiers that can modify the
///   `ClientBuilder`.
///
/// # Returns
///
/// A `ClientBuilder` configured with the provided options.
#[allow(clippy::type_complexity)]
pub fn new_http_client_builder(
    config: &TransportConfig,
    transport_modifiers: &[Arc<dyn Fn(&mut ClientBuilder) + Send + Sync>],
) -> ClientBuilder {
    let mut builder = Client::builder()
        .connect_timeout(config.connect_timeout.unwrap_or(DEFAULT_CONNECT_TIMEOUT))
        .timeout(
            config
                .read_write_timeout
                .unwrap_or(DEFAULT_READ_WRITE_TIMEOUT),
        )
        .pool_idle_timeout(
            config
                .idle_connection_timeout
                .unwrap_or(DEFAULT_IDLE_CONNECTION_TIMEOUT),
        )
        .http2_keep_alive_timeout(
            config
                .keep_alive_timeout
                .unwrap_or(DEFAULT_KEEP_ALIVE_TIMEOUT),
        )
        .danger_accept_invalid_certs(config.insecure_skip_verify.unwrap_or(false))
        .pool_max_idle_per_host(config.max_connections.unwrap_or(DEFAULT_MAX_CONNECTIONS))
        .min_tls_version(config.tls_min_version.unwrap_or(DEFAULT_TLS_MIN_VERSION));

    if let Some(enabled) = config.enabled_redirect {
        if enabled {
            builder = builder.redirect(redirect::Policy::default());
        }
    }

    if let Some(all_proxy) = &config.all_proxy {
        builder = builder.proxy(Proxy::all(all_proxy.as_str()).expect("Invalid proxy URL"));
    } else if let Some(env_proxy) = config.use_env_proxy {
        // Only when config.http_proxy is not set
        let get_proxy_url = |env_str: &str| -> Option<Url> {
            env::var(env_str.to_uppercase())
                .or_else(|_| env::var(env_str.to_lowercase()))
                .ok()
                .and_then(|proxy_str| Url::parse(&proxy_str).ok())
        };
        if env_proxy {
            if let Some(proxy_url) = get_proxy_url("all_proxy") {
                builder = builder.proxy(Proxy::all(proxy_url).expect("Invalid proxy URL"));
            } else {
                // Only when "all_proxy" is not set
                if let Some(proxy_url) = get_proxy_url("http_proxy") {
                    builder = builder.proxy(Proxy::http(proxy_url).expect("Invalid proxy URL"));
                }
                if let Some(proxy_url) = get_proxy_url("https_proxy") {
                    builder = builder.proxy(Proxy::https(proxy_url).expect("Invalid proxy URL"));
                }
            }
        }
    }

    if let Some(bind_address) = config.bind_address {
        builder = builder.local_address(bind_address);
    }

    for modifier in transport_modifiers {
        modifier(&mut builder);
    }

    builder
}

#[cfg(test)]
mod tests {

    #[test]
    #[ignore = "cannot inspect whether the proxy is set correctly in client"]
    fn test_all_proxy() {
        // let config = TransportConfig {
        //     all_proxy: Some(Url::parse("http://proxy.example.com").unwrap()),
        //     use_env_proxy: Some(false),
        //     ..Default::default()
        // };
        // let client = new_http_client_builder(&config,
        // &[]).build().unwrap();
    }
}
