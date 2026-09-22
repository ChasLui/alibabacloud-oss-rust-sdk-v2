use std::rc::Rc;

use url::Url;

use crate::client::{Client, EndpointProvider};
use crate::config::Config;
use crate::signer::vectors_v4::SignerVectorsV4;
use crate::utils::escape_path;
use crate::{OperationInput, UrlStyleType};

/// Builds request URLs for the vector bucket product.
///
/// A vector bucket is addressed as `{bucket}-{account_id}.{endpoint}`, not as
/// `{bucket}.{endpoint}`. Mirrors Go's `vectors.endpointProvider`.
#[derive(Debug, Clone)]
pub struct VectorsEndpointProvider {
    endpoint: Url,
    account_id: String,
    url_style: UrlStyleType,
}

impl VectorsEndpointProvider {
    pub fn new(endpoint: Url, account_id: &str, url_style: UrlStyleType) -> Self {
        VectorsEndpointProvider {
            endpoint,
            account_id: account_id.to_string(),
            url_style,
        }
    }
}

impl EndpointProvider for VectorsEndpointProvider {
    fn build_url(
        &self,
        input: &OperationInput,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let endpoint_host = match self.endpoint.port() {
            Some(port) => format!("{}:{}", self.endpoint.host_str().unwrap_or_default(), port),
            None => self.endpoint.host_str().unwrap_or_default().to_string(),
        };

        let mut paths: Vec<String> = Vec::new();
        let host = match &input.bucket {
            None => endpoint_host,
            Some(bucket) => match self.url_style {
                UrlStyleType::Path => {
                    paths.push(bucket.clone());
                    if input.key.is_none() {
                        paths.push(String::new());
                    }
                    endpoint_host
                }
                _ => format!("{}-{}.{}", bucket, self.account_id, endpoint_host),
            },
        };

        if let Some(key) = &input.key {
            paths.push(escape_path(key, false));
        }

        Ok(format!(
            "{}://{}/{}",
            self.endpoint.scheme(),
            host,
            paths.join("/")
        ))
    }
}

impl Client {
    /// Creates a client for the vector bucket APIs.
    ///
    /// The endpoint defaults to `{region}.oss-vectors.aliyuncs.com` (or the
    /// internal variant) when none is configured, the bucket is addressed with
    /// the account ID, and requests are signed with
    /// [`SignerVectorsV4`]. Mirrors Go's `NewVectorsClient`.
    pub fn new_vectors(config: &Config) -> Self {
        let mut product_config = config.clone();
        let region = product_config.region.clone().unwrap_or_default();
        if product_config.endpoint.is_none() && !region.is_empty() {
            product_config.endpoint =
                Some(if product_config.use_internal_endpoint.unwrap_or(false) {
                    format!("{}-internal.oss-vectors.aliyuncs.com", region)
                } else {
                    format!("{}.oss-vectors.aliyuncs.com", region)
                });
        }
        if product_config.user_agent.is_none() {
            product_config.user_agent = Some("vectors-client".to_string());
        }

        let account_id = product_config.account_id.clone().unwrap_or_default();
        let mut client = Client::new(&product_config);

        let endpoint = client.options_mut().endpoint.clone();
        let url_style = client.options_mut().url_style.clone();
        if let Some(endpoint) = endpoint {
            client.options_mut().endpoint_provider = Some(Rc::new(VectorsEndpointProvider::new(
                endpoint,
                &account_id,
                url_style,
            )));
        }
        client.options_mut().signer = Some(Rc::new(SignerVectorsV4::new(&account_id)));

        client
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::EndpointProvider;

    fn input(bucket: Option<&str>, key: Option<&str>) -> OperationInput {
        OperationInput {
            bucket: bucket.map(|b| b.to_string()),
            key: key.map(|k| k.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_build_url_virtual_hosted() {
        let provider = VectorsEndpointProvider::new(
            Url::parse("https://cn-hangzhou.oss-vectors.aliyuncs.com").unwrap(),
            "123",
            UrlStyleType::VirtualHosted,
        );

        assert_eq!(
            provider.build_url(&input(Some("my-vector"), None)).unwrap(),
            "https://my-vector-123.cn-hangzhou.oss-vectors.aliyuncs.com/"
        );
    }

    #[test]
    fn test_build_url_path_style() {
        let provider = VectorsEndpointProvider::new(
            Url::parse("https://cn-hangzhou.oss-vectors.aliyuncs.com").unwrap(),
            "123",
            UrlStyleType::Path,
        );

        assert_eq!(
            provider
                .build_url(&input(Some("my-vector"), Some("index/1")))
                .unwrap(),
            "https://cn-hangzhou.oss-vectors.aliyuncs.com/my-vector/index/1"
        );
    }

    #[test]
    fn test_build_url_without_bucket() {
        let provider = VectorsEndpointProvider::new(
            Url::parse("https://cn-hangzhou.oss-vectors.aliyuncs.com").unwrap(),
            "123",
            UrlStyleType::VirtualHosted,
        );

        assert_eq!(
            provider.build_url(&input(None, None)).unwrap(),
            "https://cn-hangzhou.oss-vectors.aliyuncs.com/"
        );
    }

    #[test]
    fn test_new_vectors_defaults_the_endpoint_from_the_region() {
        let mut client = Client::new_vectors(
            &Config::default()
                .with_region("cn-hangzhou")
                .with_account_id("123"),
        );

        assert_eq!(
            client.options_mut().endpoint.as_ref().unwrap().host_str(),
            Some("cn-hangzhou.oss-vectors.aliyuncs.com")
        );
    }
}
