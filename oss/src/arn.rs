//! Alibaba Cloud Resource Name (ARN) parsing and building.
//!
//! An ARN is written `acs:{service}:{region}:{account_id}:{resource}`. The
//! resource itself is `{type}:{name}:{qualifier}`, where the separator may also
//! be `/`. Mirrors the Go SDK's `arn` package.

use std::fmt;

/// Reasons an ARN or its resource cannot be built or parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArnError {
    /// The service namespace is missing or blank.
    BlankService,
    /// The resource is missing or blank.
    BlankResource,
    /// The string is not a well-formed ARN.
    Malformed(String),
    /// The resource is not a bucket resource.
    NotBucketResource(String),
}

impl fmt::Display for ArnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArnError::BlankService => write!(f, "service must not be blank"),
            ArnError::BlankResource => write!(f, "resource must not be blank"),
            ArnError::Malformed(message) => write!(f, "{}", message),
            ArnError::NotBucketResource(resource) => {
                write!(f, "{} is not bucket resource", resource)
            }
        }
    }
}

impl std::error::Error for ArnError {}

/// The resource portion of an ARN, e.g. `bucket:my-bucket` or
/// `object:my-bucket/my-object:version`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ArnResource {
    /// The resource type, absent when the resource has no type prefix.
    pub resource_type: Option<String>,
    /// The resource name, as written.
    pub resource: String,
    /// The optional qualifier that follows the resource name.
    pub qualifier: Option<String>,
}

impl ArnResource {
    /// Parses a resource from its string form.
    pub fn from_string(resource: &str) -> Result<Self, ArnError> {
        let splitter_position = match resource.find([':', '/']) {
            // No separator, or the string starts with one: the whole string is
            // the resource name and there is no type.
            None => return ArnResourceBuilder::new().resource(resource).build(),
            Some(0) => return ArnResourceBuilder::new().resource(resource).build(),
            Some(position) => position,
        };

        let splitter = resource.as_bytes()[splitter_position] as char;
        let resource_type = &resource[..splitter_position];
        let mut rest = resource[splitter_position + 1..].splitn(2, splitter);

        let builder = ArnResourceBuilder::new().resource_type(resource_type);
        let name = rest.next().unwrap_or_default();
        match rest.next() {
            Some(qualifier) => builder.resource(name).qualifier(qualifier).build(),
            None => builder.resource(name).build(),
        }
    }

    /// Ensures the resource is a bucket resource, i.e. `bucket:{name}` with no
    /// qualifier.
    pub fn ensure_bucket_resource(&self) -> Result<(), ArnError> {
        if self.resource_type.as_deref() != Some("bucket")
            || self.resource.trim().is_empty()
            || self.qualifier.is_some()
        {
            return Err(ArnError::NotBucketResource(self.to_string()));
        }
        Ok(())
    }

    /// Returns a builder seeded with this resource.
    pub fn to_builder(&self) -> ArnResourceBuilder {
        ArnResourceBuilder::new()
            .resource_type_opt(self.resource_type.clone())
            .resource(&self.resource)
            .qualifier_opt(self.qualifier.clone())
    }
}

impl fmt::Display for ArnResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(resource_type) = &self.resource_type {
            write!(f, "{}", resource_type)?;
        }
        write!(f, ":{}:", self.resource)?;
        if let Some(qualifier) = &self.qualifier {
            write!(f, "{}", qualifier)?;
        }
        Ok(())
    }
}

/// Builds an [`ArnResource`].
#[derive(Debug, Default, Clone)]
pub struct ArnResourceBuilder {
    resource_type: Option<String>,
    resource: String,
    qualifier: Option<String>,
}

impl ArnResourceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resource_type(mut self, resource_type: &str) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self
    }

    pub fn resource_type_opt(mut self, resource_type: Option<String>) -> Self {
        self.resource_type = resource_type;
        self
    }

    pub fn resource(mut self, resource: &str) -> Self {
        self.resource = resource.to_string();
        self
    }

    pub fn qualifier(mut self, qualifier: &str) -> Self {
        self.qualifier = Some(qualifier.to_string());
        self
    }

    pub fn qualifier_opt(mut self, qualifier: Option<String>) -> Self {
        self.qualifier = qualifier;
        self
    }

    pub fn build(self) -> Result<ArnResource, ArnError> {
        if self.resource.trim().is_empty() {
            return Err(ArnError::BlankResource);
        }
        Ok(ArnResource {
            resource_type: self.resource_type,
            resource: self.resource,
            qualifier: self.qualifier,
        })
    }
}

/// An Alibaba Cloud Resource Name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Arn {
    /// The service namespace that identifies the product.
    pub service: String,
    /// The region the resource resides in, when the ARN carries one.
    pub region: Option<String>,
    /// The ID of the account that owns the resource, when the ARN carries one.
    pub account_id: Option<String>,
    /// The resource, as written.
    pub resource: String,
    /// The parsed resource.
    pub arn_resource: ArnResource,
}

impl Arn {
    /// Parses an ARN, failing on malformed input.
    pub fn parse(arn: &str) -> Result<Self, ArnError> {
        match Arn::parse_inner(arn, true)? {
            Some(arn) => Ok(arn),
            None => Err(ArnError::Malformed("ARN parsing failed".to_string())),
        }
    }

    /// Parses an ARN, returning `None` when the string is not an ARN at all.
    ///
    /// A string that is shaped like an ARN but carries invalid parts is still
    /// an error.
    pub fn try_parse(arn: &str) -> Result<Option<Self>, ArnError> {
        Arn::parse_inner(arn, false)
    }

    fn parse_inner(arn: &str, throw_on_error: bool) -> Result<Option<Self>, ArnError> {
        if arn.is_empty() {
            return Ok(None);
        }

        let malformed = |message: &str| -> Result<Option<Self>, ArnError> {
            if throw_on_error {
                Err(ArnError::Malformed(message.to_string()))
            } else {
                Ok(None)
            }
        };

        let Some(rest) = arn.strip_prefix("acs:") else {
            return malformed("malformed ARN - doesn't start with 'acs:'");
        };

        let mut parts = rest.splitn(4, ':');
        let service = parts.next().unwrap_or_default();
        let Some(region) = parts.next() else {
            return malformed("malformed ARN - no region specified");
        };
        let Some(account_id) = parts.next() else {
            return malformed("malformed ARN - no account specified");
        };
        let Some(resource) = parts.next() else {
            return malformed("malformed ARN - no resource specified");
        };
        if resource.is_empty() {
            return malformed("malformed ARN - no resource specified");
        }

        Ok(Some(
            ArnBuilder::new()
                .service(service)
                .region_opt(non_empty(region))
                .account_id_opt(non_empty(account_id))
                .resource(resource)
                .build()?,
        ))
    }

    /// Returns a builder seeded with this ARN.
    pub fn to_builder(&self) -> ArnBuilder {
        ArnBuilder::new()
            .service(&self.service)
            .region_opt(self.region.clone())
            .account_id_opt(self.account_id.clone())
            .resource(&self.resource)
    }
}

impl fmt::Display for Arn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "acs:{}:{}:{}:{}",
            self.service,
            self.region.as_deref().unwrap_or_default(),
            self.account_id.as_deref().unwrap_or_default(),
            self.resource
        )
    }
}

impl std::str::FromStr for Arn {
    type Err = ArnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Arn::parse(s)
    }
}

/// Builds an [`Arn`].
#[derive(Debug, Default, Clone)]
pub struct ArnBuilder {
    service: String,
    region: Option<String>,
    account_id: Option<String>,
    resource: String,
}

impl ArnBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn service(mut self, service: &str) -> Self {
        self.service = service.to_string();
        self
    }

    pub fn region(mut self, region: &str) -> Self {
        self.region = Some(region.to_string());
        self
    }

    pub fn region_opt(mut self, region: Option<String>) -> Self {
        self.region = region;
        self
    }

    pub fn account_id(mut self, account_id: &str) -> Self {
        self.account_id = Some(account_id.to_string());
        self
    }

    pub fn account_id_opt(mut self, account_id: Option<String>) -> Self {
        self.account_id = account_id;
        self
    }

    pub fn resource(mut self, resource: &str) -> Self {
        self.resource = resource.to_string();
        self
    }

    pub fn build(self) -> Result<Arn, ArnError> {
        if self.service.trim().is_empty() {
            return Err(ArnError::BlankService);
        }
        if self.resource.trim().is_empty() {
            return Err(ArnError::BlankResource);
        }

        let arn_resource = ArnResource::from_string(&self.resource)?;

        Ok(Arn {
            service: self.service,
            region: self.region,
            account_id: self.account_id,
            resource: self.resource,
            arn_resource,
        })
    }
}

fn non_empty(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_arn_round_trip() {
        let arn = Arn::parse("acs:oss:cn-hangzhou:1234567890:bucket:my-bucket").unwrap();

        assert_eq!(arn.service, "oss");
        assert_eq!(arn.region.as_deref(), Some("cn-hangzhou"));
        assert_eq!(arn.account_id.as_deref(), Some("1234567890"));
        assert_eq!(arn.resource, "bucket:my-bucket");
        assert_eq!(
            arn.arn_resource,
            ArnResource {
                resource_type: Some("bucket".to_string()),
                resource: "my-bucket".to_string(),
                qualifier: None,
            }
        );
        assert_eq!(
            arn.to_string(),
            "acs:oss:cn-hangzhou:1234567890:bucket:my-bucket"
        );
    }

    #[test]
    fn test_parse_arn_with_empty_region_and_account() {
        let arn = Arn::parse("acs:oss:::bucket:my-bucket").unwrap();

        assert_eq!(arn.region, None);
        assert_eq!(arn.account_id, None);
        assert_eq!(arn.to_string(), "acs:oss:::bucket:my-bucket");
    }

    #[test]
    fn test_try_parse_returns_none_for_non_arn() {
        assert_eq!(Arn::try_parse("not-an-arn").unwrap(), None);
        assert_eq!(Arn::try_parse("").unwrap(), None);
        // Shaped like an ARN but missing parts is still an error.
        assert!(Arn::parse("acs:oss").is_err());
    }

    #[test]
    fn test_resource_with_qualifier() {
        let arn = Arn::parse("acs:oss:cn-hangzhou:123:object:my-bucket/key:version-1").unwrap();

        assert_eq!(
            arn.arn_resource,
            ArnResource {
                resource_type: Some("object".to_string()),
                resource: "my-bucket/key".to_string(),
                qualifier: Some("version-1".to_string()),
            }
        );
        assert_eq!(
            arn.arn_resource.to_string(),
            "object:my-bucket/key:version-1"
        );
    }

    #[test]
    fn test_resource_without_type() {
        let arn = Arn::parse("acs:oss:cn-hangzhou:123:my-bucket").unwrap();

        assert_eq!(arn.arn_resource.resource_type, None);
        assert_eq!(arn.arn_resource.resource, "my-bucket");
    }

    #[test]
    fn test_ensure_bucket_resource() {
        let bucket = Arn::parse("acs:oss:cn-hangzhou:123:bucket:my-bucket").unwrap();
        assert!(bucket.arn_resource.ensure_bucket_resource().is_ok());

        let with_qualifier =
            Arn::parse("acs:oss:cn-hangzhou:123:bucket:my-bucket:version-1").unwrap();
        assert_eq!(
            with_qualifier.arn_resource.ensure_bucket_resource(),
            Err(ArnError::NotBucketResource(
                "bucket:my-bucket:version-1".to_string()
            ))
        );

        let object = Arn::parse("acs:oss:cn-hangzhou:123:object:my-bucket/key").unwrap();
        assert!(object.arn_resource.ensure_bucket_resource().is_err());
    }

    #[test]
    fn test_builder_rejects_blank_parts() {
        assert_eq!(
            ArnBuilder::new().service(" ").resource("bucket:b").build(),
            Err(ArnError::BlankService)
        );
        assert_eq!(
            ArnBuilder::new().service("oss").resource("  ").build(),
            Err(ArnError::BlankResource)
        );
    }
}
