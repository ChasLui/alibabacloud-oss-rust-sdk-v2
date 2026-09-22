//! Addressing rules for agentic buckets and bucket spaces.
//!
//! Every agentic request carries a *prefix*, not a physical bucket name. The
//! SDK expands it to `{prefix}-{accountId}-{region}-{suffix}`, where the suffix
//! is `ab-apsr` for agentic buckets and `bs-apsr` for bucket spaces, and uses
//! that name both for signing and for the request host. Mirrors Go's
//! `agenticProvider` / `BucketSpaceHelper`.

use std::rc::Rc;

use url::Url;

use crate::client::{BucketNameResolver, Client, EndpointProvider};
use crate::config::Config;
use crate::utils::escape_path;
use crate::{ClientError, OperationInput};

/// The suffix that expands a prefix into an agentic bucket name.
pub const AGENTIC_BUCKET_SUFFIX: &str = "ab-apsr";

/// The suffix that expands a prefix into a bucket space name.
pub const BUCKET_SPACE_SUFFIX: &str = "bs-apsr";

/// The literal segment that replaces `{accountId}-{region}` in the short alias
/// host label.
const ALIAS_TOKEN: &str = "alias";

/// The maximum length of a DNS host label.
const MAX_HOST_LABEL_LEN: usize = 63;

/// How an agentic request places its bucket on the wire.
///
/// Mirrors Go's `UrlStyleVirtualHosted`, `UrlStyleVirtualHostedAlias` and
/// `UrlStylePath` for the agentic provider. It is separate from
/// [`crate::UrlStyleType`] because that type has no alias variant.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AgenticUrlStyle {
    /// The full name becomes the leftmost DNS label:
    /// `{prefix}-{accountId}-{region}-{suffix}.{endpoint}`.
    #[default]
    VirtualHosted,
    /// A short `{prefix}-alias-{suffix}` label routes the request through a
    /// wildcard domain; signing still uses the full name.
    VirtualHostedAlias,
    /// The full name is the first path segment and the endpoint host is used
    /// as-is.
    Path,
}

/// Expands a prefix into a physical name
/// `{prefix}-{accountId}-{region}-{suffix}`.
fn build_full_name(prefix: &str, account_id: &str, region: &str, suffix: &str) -> String {
    format!("{}-{}-{}-{}", prefix, account_id, region, suffix)
}

/// Builds the short alias host label `{prefix}-alias-{suffix}`.
fn build_alias_label(prefix: &str, suffix: &str) -> String {
    format!("{}-{}-{}", prefix, ALIAS_TOKEN, suffix)
}

/// Builds the `missing required field, {field}.` error Go raises from
/// `NewErrParamRequired`.
fn param_required(field: &str) -> Box<dyn std::error::Error + Send + Sync> {
    let message = format!("missing required field, {}.", field);
    Box::new(ClientError {
        code: "InvalidArgument".to_string(),
        message: message.clone(),
        err: Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            message,
        )),
    })
}

/// The `host[:port]` of an endpoint.
fn endpoint_authority(endpoint: &Url) -> String {
    match endpoint.host_str() {
        Some(host) => match endpoint.port() {
            Some(port) => format!("{}:{}", host, port),
            None => host.to_string(),
        },
        None => String::new(),
    }
}

/// Rejects a host label that cannot be a DNS label.
fn check_host_label(label: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if label.len() > MAX_HOST_LABEL_LEN {
        return Err(format!(
            "the host label {:?} exceeds the maximum length of {} characters",
            label, MAX_HOST_LABEL_LEN
        )
        .into());
    }
    Ok(())
}

/// Resolves the physical name and the request URL for agentic buckets.
///
/// One instance is wired as both the [`BucketNameResolver`] (signing) and the
/// [`EndpointProvider`] (request host), mirroring Go's `agenticProvider`.
#[derive(Debug, Clone)]
struct AgenticProvider {
    /// The resolved endpoint the request is sent to.
    endpoint: Option<Url>,
    /// The account that owns the bucket; required.
    account_id: String,
    /// The region that owns the bucket; required.
    region: String,
    /// `ab-apsr` for agentic buckets, `bs-apsr` for bucket spaces.
    suffix: &'static str,
    /// Where the bucket is placed on the wire.
    url_style: AgenticUrlStyle,
}

impl AgenticProvider {
    /// Expands a prefix into its physical name, validating that the account ID
    /// and region are present.
    fn resolve_bucket_name(
        &self,
        prefix: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if self.account_id.is_empty() {
            return Err(param_required("AccountId"));
        }
        if self.region.is_empty() {
            return Err(param_required("Region"));
        }
        Ok(build_full_name(
            prefix,
            &self.account_id,
            &self.region,
            self.suffix,
        ))
    }
}

impl BucketNameResolver for AgenticProvider {
    /// The physical name is what the request is signed with, whatever the
    /// addressing style. A request without a bucket resolves to an empty name
    /// rather than an error, because the account and region are only required
    /// once a bucket is addressed.
    fn build_bucket_name(
        &self,
        input: &OperationInput,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match &input.bucket {
            Some(prefix) => self.resolve_bucket_name(prefix),
            None => Ok(String::new()),
        }
    }
}

impl EndpointProvider for AgenticProvider {
    /// Builds the full request URL (without the query string).
    fn build_url(
        &self,
        input: &OperationInput,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let endpoint = match &self.endpoint {
            Some(endpoint) => endpoint,
            None => return Ok(String::new()),
        };

        let authority = endpoint_authority(endpoint);
        let mut host = authority.clone();
        let mut paths: Vec<String> = Vec::new();

        if let Some(prefix) = &input.bucket {
            let full_name = self.resolve_bucket_name(prefix)?;
            match self.url_style {
                AgenticUrlStyle::VirtualHosted => {
                    check_host_label(&full_name)?;
                    host = format!("{}.{}", full_name, authority);
                }
                AgenticUrlStyle::VirtualHostedAlias => {
                    let label = build_alias_label(prefix, self.suffix);
                    check_host_label(&label)?;
                    host = format!("{}.{}", label, authority);
                }
                AgenticUrlStyle::Path => {
                    paths.push(full_name);
                    if input.key.is_none() {
                        // Keep the trailing slash that marks a bucket-level
                        // request.
                        paths.push(String::new());
                    }
                }
            }
        }

        if let Some(key) = &input.key {
            paths.push(escape_path(key, false));
        }

        Ok(format!(
            "{}://{}/{}",
            endpoint.scheme(),
            host,
            paths.join("/")
        ))
    }
}

/// Builds physical bucket space names for use with a plain [`Client`].
///
/// Mirrors Go's `BucketSpaceHelper` / Java's `BucketSpaceHelper`.
#[derive(Debug, Clone, Default)]
pub struct BucketSpaceHelper {
    account_id: String,
    region: String,
}

impl BucketSpaceHelper {
    /// Creates a helper from the account ID and region in `config`.
    pub fn new(config: &Config) -> Self {
        Self {
            account_id: config.account_id.clone().unwrap_or_default(),
            region: config.region.clone().unwrap_or_default(),
        }
    }

    /// Builds `{prefix}-{accountId}-{region}-bs-apsr` from a short prefix.
    pub fn to_bucket_name(&self, prefix: &str) -> String {
        build_full_name(prefix, &self.account_id, &self.region, BUCKET_SPACE_SUFFIX)
    }
}

/// Installs the agentic resolver and endpoint provider on an already-resolved
/// client.
fn install_agentic_provider(client: &mut Client, suffix: &'static str, url_style: AgenticUrlStyle) {
    let options = client.options_mut();
    let provider = Rc::new(AgenticProvider {
        endpoint: options.endpoint.clone(),
        account_id: options.account_id.clone().unwrap_or_default(),
        region: options.region.clone(),
        suffix,
        url_style,
    });
    options.bucket_name_resolver = Some(provider.clone());
    options.endpoint_provider = Some(provider);
}

impl Client {
    /// Creates a client for the agentic bucket APIs.
    ///
    /// Every request's `bucket` field is a prefix that the client expands to
    /// `{prefix}-{accountId}-{region}-ab-apsr`, so `Config::with_account_id`
    /// and `Config::with_region` are both required. Mirrors Go's
    /// `NewAgenticBucketClient`.
    pub fn new_agentic(config: &Config) -> Client {
        let url_style = match config.use_path_style {
            Some(true) => AgenticUrlStyle::Path,
            _ => AgenticUrlStyle::VirtualHosted,
        };
        Self::new_agentic_with_url_style(config, url_style)
    }

    /// Creates a client for the agentic bucket APIs with an explicit
    /// addressing style.
    ///
    /// Use [`AgenticUrlStyle::VirtualHostedAlias`] to route through a wildcard
    /// domain with the short `{prefix}-alias-ab-apsr` host label.
    pub fn new_agentic_with_url_style(config: &Config, url_style: AgenticUrlStyle) -> Client {
        let mut client = Client::new(config);
        install_agentic_provider(&mut client, AGENTIC_BUCKET_SUFFIX, url_style);
        client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACCOUNT_ID: &str = "1234567890123456";
    const REGION: &str = "cn-hangzhou";

    fn endpoint() -> Url {
        Url::parse("https://oss-cn-hangzhou.aliyuncs.com").unwrap()
    }

    fn provider(url_style: AgenticUrlStyle) -> AgenticProvider {
        AgenticProvider {
            endpoint: Some(endpoint()),
            account_id: ACCOUNT_ID.to_string(),
            region: REGION.to_string(),
            suffix: AGENTIC_BUCKET_SUFFIX,
            url_style,
        }
    }

    fn bucket_input(bucket: &str) -> OperationInput {
        OperationInput {
            bucket: Some(bucket.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_build_bucket_name_expands_prefix() {
        let p = provider(AgenticUrlStyle::VirtualHosted);

        assert_eq!(
            p.build_bucket_name(&bucket_input("my-agentic")).unwrap(),
            "my-agentic-1234567890123456-cn-hangzhou-ab-apsr"
        );
        // A request without a bucket has nothing to resolve.
        assert_eq!(p.build_bucket_name(&OperationInput::default()).unwrap(), "");
    }

    #[test]
    fn test_build_bucket_name_bucket_space_suffix() {
        let p = AgenticProvider {
            suffix: BUCKET_SPACE_SUFFIX,
            ..provider(AgenticUrlStyle::VirtualHosted)
        };

        assert_eq!(
            p.build_bucket_name(&bucket_input("my-sandbox")).unwrap(),
            "my-sandbox-1234567890123456-cn-hangzhou-bs-apsr"
        );
    }

    #[test]
    fn test_missing_account_id_and_region_are_errors() {
        let input = bucket_input("my-agentic");

        let no_account = AgenticProvider {
            account_id: String::new(),
            ..provider(AgenticUrlStyle::VirtualHosted)
        };
        let err = no_account
            .build_bucket_name(&input)
            .unwrap_err()
            .to_string();
        assert!(err.contains("AccountId"), "unexpected error: {}", err);
        let err = no_account.build_url(&input).unwrap_err().to_string();
        assert!(err.contains("AccountId"), "unexpected error: {}", err);

        let no_region = AgenticProvider {
            region: String::new(),
            ..provider(AgenticUrlStyle::VirtualHosted)
        };
        let err = no_region.build_bucket_name(&input).unwrap_err().to_string();
        assert!(err.contains("Region"), "unexpected error: {}", err);

        // Without a bucket the account and region are not needed.
        assert_eq!(
            no_account
                .build_bucket_name(&OperationInput::default())
                .unwrap(),
            ""
        );
    }

    #[test]
    fn test_build_url_virtual_hosted() {
        let p = provider(AgenticUrlStyle::VirtualHosted);

        assert_eq!(
            p.build_url(&bucket_input("my-agentic")).unwrap(),
            "https://my-agentic-1234567890123456-cn-hangzhou-ab-apsr.oss-cn-hangzhou.aliyuncs.com/"
        );
        // Without a bucket the endpoint host is used as-is.
        assert_eq!(
            p.build_url(&OperationInput::default()).unwrap(),
            "https://oss-cn-hangzhou.aliyuncs.com/"
        );
        // The key is appended, escaped.
        assert_eq!(
            p.build_url(&OperationInput {
                bucket: Some("my-agentic".to_string()),
                key: Some("dir/a b.txt".to_string()),
                ..Default::default()
            })
            .unwrap(),
            "https://my-agentic-1234567890123456-cn-hangzhou-ab-apsr.oss-cn-hangzhou.aliyuncs.com/dir/a%20b.txt"
        );
    }

    #[test]
    fn test_build_url_path_style() {
        let p = provider(AgenticUrlStyle::Path);

        assert_eq!(
            p.build_url(&bucket_input("my-agentic")).unwrap(),
            "https://oss-cn-hangzhou.aliyuncs.com/my-agentic-1234567890123456-cn-hangzhou-ab-apsr/"
        );
        assert_eq!(
            p.build_url(&OperationInput {
                bucket: Some("my-agentic".to_string()),
                key: Some("test.txt".to_string()),
                ..Default::default()
            })
            .unwrap(),
            "https://oss-cn-hangzhou.aliyuncs.com/my-agentic-1234567890123456-cn-hangzhou-ab-apsr/test.txt"
        );
        assert_eq!(
            p.build_url(&OperationInput::default()).unwrap(),
            "https://oss-cn-hangzhou.aliyuncs.com/"
        );
    }

    #[test]
    fn test_build_url_alias_style_signs_with_the_full_name() {
        let p = AgenticProvider {
            suffix: BUCKET_SPACE_SUFFIX,
            url_style: AgenticUrlStyle::VirtualHostedAlias,
            endpoint: Some(Url::parse("https://abc.com").unwrap()),
            ..provider(AgenticUrlStyle::VirtualHostedAlias)
        };

        let input = OperationInput {
            bucket: Some("my-sandbox".to_string()),
            key: Some("test.txt".to_string()),
            ..Default::default()
        };
        // The host carries the short label ...
        assert_eq!(
            p.build_url(&input).unwrap(),
            "https://my-sandbox-alias-bs-apsr.abc.com/test.txt"
        );
        // ... while the signing name stays the physical one.
        assert_eq!(
            p.build_bucket_name(&input).unwrap(),
            "my-sandbox-1234567890123456-cn-hangzhou-bs-apsr"
        );
    }

    #[test]
    fn test_host_label_length_limit() {
        // full name = prefix + 37 characters; the DNS limit is 63.
        let ok_prefix = "a".repeat(26);
        assert_eq!(
            provider(AgenticUrlStyle::VirtualHosted)
                .build_url(&bucket_input(&ok_prefix))
                .unwrap()
                .len(),
            "https://".len() + 63 + ".oss-cn-hangzhou.aliyuncs.com/".len()
        );

        let long_prefix = "a".repeat(27);
        let err = provider(AgenticUrlStyle::VirtualHosted)
            .build_url(&bucket_input(&long_prefix))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("exceeds the maximum length of 63 characters"),
            "unexpected error: {}",
            err
        );

        // Path style has no DNS label limit.
        assert!(provider(AgenticUrlStyle::Path)
            .build_url(&bucket_input(&long_prefix))
            .is_ok());

        // The alias label is shorter than the full name, so a prefix that is
        // too long for virtual-hosted style still fits.
        let alias = provider(AgenticUrlStyle::VirtualHostedAlias);
        assert!(alias.build_url(&bucket_input(&long_prefix)).is_ok());
        assert!(alias
            .build_url(&bucket_input(&"a".repeat(50)))
            .unwrap_err()
            .to_string()
            .contains("exceeds the maximum length of 63 characters"));
    }

    #[test]
    fn test_bucket_space_helper() {
        let helper = BucketSpaceHelper::new(
            &Config::default()
                .with_account_id(ACCOUNT_ID)
                .with_region(REGION),
        );

        assert_eq!(
            helper.to_bucket_name("my-sandbox"),
            "my-sandbox-1234567890123456-cn-hangzhou-bs-apsr"
        );
        assert_eq!(
            BucketSpaceHelper::default().to_bucket_name("my-sandbox"),
            "my-sandbox---bs-apsr"
        );
    }

    #[test]
    fn test_new_agentic_installs_the_provider() {
        let client = Client::new_agentic(
            &Config::default()
                .with_region(REGION)
                .with_account_id(ACCOUNT_ID),
        );

        assert!(client.options.bucket_name_resolver.is_some());
        assert!(client.options.endpoint_provider.is_some());
        assert_eq!(
            client
                .options
                .bucket_name_resolver
                .as_ref()
                .unwrap()
                .build_bucket_name(&bucket_input("my-agentic"))
                .unwrap(),
            "my-agentic-1234567890123456-cn-hangzhou-ab-apsr"
        );
    }

    #[test]
    fn test_new_agentic_path_style_from_config() {
        let client = Client::new_agentic(
            &Config::default()
                .with_region(REGION)
                .with_account_id(ACCOUNT_ID)
                .with_use_path_style(true),
        );

        assert_eq!(
            client
                .options
                .endpoint_provider
                .as_ref()
                .unwrap()
                .build_url(&bucket_input("my-agentic"))
                .unwrap(),
            "https://oss-cn-hangzhou.aliyuncs.com/my-agentic-1234567890123456-cn-hangzhou-ab-apsr/"
        );
    }
}
