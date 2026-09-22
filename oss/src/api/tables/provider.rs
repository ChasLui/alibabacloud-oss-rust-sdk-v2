use std::rc::Rc;

use url::Url;

use crate::client::{Client, EndpointProvider};
use crate::config::Config;
use crate::signer::tables_v4::SignerTablesV4;
use crate::utils::is_valid_region;
use crate::{OperationInput, UrlStyleType};

/// The product name that participates in the V4 signing scope.
pub const TABLES_PRODUCT: &str = "osstables";

/// The bucket name segment of a table bucket ARN, i.e. the part after
/// `bucket/` in `acs:osstables:{region}:{account}:bucket/{name}`.
fn bucket_name(table_bucket_arn: &str) -> &str {
    table_bucket_arn
        .split(':')
        .nth(4)
        .and_then(|resource| resource.split('/').nth(1))
        .unwrap_or_default()
}

/// The account ID segment of a table bucket ARN.
fn bucket_account_id(table_bucket_arn: &str) -> &str {
    table_bucket_arn.split(':').nth(3).unwrap_or_default()
}

/// Builds request URLs for the table bucket product.
///
/// A table bucket is addressed by its ARN rather than by a bare name:
/// virtual-hosted requests go to `{name}-{account}.{endpoint}`, while every
/// other style keeps the endpoint host. The key is already escaped by the
/// operation, so it is placed on the path verbatim. Mirrors Go's
/// `tables.endpointProvider`.
#[derive(Debug, Clone)]
pub struct TablesEndpointProvider {
    endpoint: Url,
    url_style: UrlStyleType,
}

impl TablesEndpointProvider {
    pub fn new(endpoint: Url, url_style: UrlStyleType) -> Self {
        TablesEndpointProvider {
            endpoint,
            url_style,
        }
    }
}

impl EndpointProvider for TablesEndpointProvider {
    fn build_url(
        &self,
        input: &OperationInput,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let endpoint_host = match self.endpoint.port() {
            Some(port) => format!("{}:{}", self.endpoint.host_str().unwrap_or_default(), port),
            None => self.endpoint.host_str().unwrap_or_default().to_string(),
        };

        let host = match (&input.bucket, &self.url_style) {
            (Some(arn), UrlStyleType::VirtualHosted) => format!(
                "{}-{}.{}",
                bucket_name(arn),
                bucket_account_id(arn),
                endpoint_host
            ),
            _ => endpoint_host,
        };

        let path = input.key.clone().unwrap_or_default();

        Ok(format!("{}://{}/{}", self.endpoint.scheme(), host, path))
    }
}

impl Client {
    /// Creates a client for the table bucket APIs.
    ///
    /// The endpoint defaults to `{region}.oss-tables.aliyuncs.com` (or the
    /// internal variant) when none is configured, and requests are signed with
    /// [`SignerTablesV4`] under the `osstables` product. Mirrors Go's
    /// `NewTablesClient`.
    pub fn new_tables(config: &Config) -> Self {
        let mut product_config = config.clone();
        let region = product_config.region.clone().unwrap_or_default();
        if product_config.endpoint.is_none() && is_valid_region(&region) {
            product_config.endpoint =
                Some(if product_config.use_internal_endpoint.unwrap_or(false) {
                    format!("{}-internal.oss-tables.aliyuncs.com", region)
                } else {
                    format!("{}.oss-tables.aliyuncs.com", region)
                });
        }
        if product_config.user_agent.is_none() {
            product_config.user_agent = Some("tables-client".to_string());
        }

        let mut client = Client::new(&product_config);

        let endpoint = client.options_mut().endpoint.clone();
        let url_style = client.options_mut().url_style.clone();
        if let Some(endpoint) = endpoint {
            client.options_mut().endpoint_provider =
                Some(Rc::new(TablesEndpointProvider::new(endpoint, url_style)));
        }
        client.options_mut().product = TABLES_PRODUCT.to_string();
        client.options_mut().signer = Some(Rc::new(SignerTablesV4));

        client
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::EndpointProvider;

    const BUCKET_ARN: &str = "acs:osstables:cn-hangzhou:1234567890123456:bucket/demo-bucket";

    fn input(bucket: Option<&str>, key: Option<&str>) -> OperationInput {
        OperationInput {
            bucket: bucket.map(|b| b.to_string()),
            key: key.map(|k| k.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_build_url_virtual_hosted() {
        let provider = TablesEndpointProvider::new(
            Url::parse("https://cn-hangzhou.oss-tables.aliyuncs.com").unwrap(),
            UrlStyleType::VirtualHosted,
        );

        assert_eq!(
            provider
                .build_url(&input(Some(BUCKET_ARN), Some("buckets/abc")))
                .unwrap(),
            "https://demo-bucket-1234567890123456.cn-hangzhou.oss-tables.aliyuncs.com/buckets/abc"
        );
    }

    #[test]
    fn test_build_url_path_style_keeps_the_endpoint_host() {
        let provider = TablesEndpointProvider::new(
            Url::parse("https://cn-hangzhou.oss-tables.aliyuncs.com").unwrap(),
            UrlStyleType::Path,
        );

        assert_eq!(
            provider
                .build_url(&input(Some(BUCKET_ARN), Some("buckets/abc")))
                .unwrap(),
            "https://cn-hangzhou.oss-tables.aliyuncs.com/buckets/abc"
        );
    }

    #[test]
    fn test_build_url_without_bucket() {
        let provider = TablesEndpointProvider::new(
            Url::parse("https://cn-hangzhou.oss-tables.aliyuncs.com").unwrap(),
            UrlStyleType::VirtualHosted,
        );

        assert_eq!(
            provider.build_url(&input(None, Some("buckets"))).unwrap(),
            "https://cn-hangzhou.oss-tables.aliyuncs.com/buckets"
        );
    }

    #[test]
    fn test_new_tables_defaults_the_endpoint_from_the_region() {
        let mut client = Client::new_tables(&Config::default().with_region("cn-hangzhou"));

        assert_eq!(
            client.options_mut().endpoint.as_ref().unwrap().host_str(),
            Some("cn-hangzhou.oss-tables.aliyuncs.com")
        );
        assert_eq!(client.options_mut().product, TABLES_PRODUCT);
    }

    #[test]
    fn test_new_tables_uses_the_internal_endpoint_when_requested() {
        let mut client = Client::new_tables(
            &Config::default()
                .with_region("cn-hangzhou")
                .with_use_internal_endpoint(true),
        );

        assert_eq!(
            client.options_mut().endpoint.as_ref().unwrap().host_str(),
            Some("cn-hangzhou-internal.oss-tables.aliyuncs.com")
        );
    }
}
