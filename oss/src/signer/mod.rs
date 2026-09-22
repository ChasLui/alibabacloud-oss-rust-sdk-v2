pub mod tables_v4;
pub mod v1;
pub mod v4;
pub mod vectors_v4;

pub use v1::*;
pub use v4::*;

mod utils;

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use chrono::TimeDelta;

use crate::credential::Credentials;

pub const SUB_RESOURCE: &str = "SubResource";
pub const SIGN_TIME: &str = "SignTime";

/// Stamps the current time shifted by a signed clock correction.
///
/// `SystemTime` only adds/subtracts an unsigned `Duration`, so a negative
/// correction has to be applied by subtracting its magnitude. Mirrors Go
/// `time.Now().Add(signingCtx.ClockOffset)`.
pub(crate) fn now_with_offset(offset: TimeDelta) -> SystemTime {
    let now = SystemTime::now();
    let magnitude = offset.abs().to_std().unwrap_or(Duration::ZERO);
    if offset < TimeDelta::zero() {
        now - magnitude
    } else {
        now + magnitude
    }
}

// Common
pub(crate) const DEFAULT_EXPIRES_DURATION: Duration = Duration::from_secs(15 * 60);

// v1
const SECURITY_TOKEN_QUERY: &str = "security-token";
const EXPIRES_QUERY: &str = "Expires";
const ACCESS_KEY_ID_QUERY: &str = "OSSAccessKeyId";
const SIGNATURE_QUERY: &str = "Signature";

// v4
const ISO8601_DATETIME_FORMAT: &str = "%Y%m%dT%H%M%SZ";
const ISO8601_DATE_FORMAT: &str = "%Y%m%d";
const ALGORITHM_V4: &str = "OSS4-HMAC-SHA256";
const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";

#[derive(Default, Debug)]
pub struct SigningContext {
    // input
    pub product: Option<String>,
    pub region: Option<String>,
    pub bucket: Option<String>,
    pub key: Option<String>,
    pub request: Option<reqwest::Request>,
    pub sub_resource: Vec<String>,
    pub additional_headers: Vec<String>,
    pub credentials: Option<Credentials>,
    pub auth_method_query: bool,
    // input and output
    pub time: Option<SystemTime>,
    /// Signed clock correction applied when stamping `time`; also carries the
    /// correction learned from a `RequestTimeTooSkewed` response back to the
    /// client. Mirrors Go `signer.SigningContext.ClockOffset`.
    pub clock_offset: TimeDelta,
    // output
    pub signed_headers: HashMap<String, String>,
    pub string_to_sign: String,
    // for test
    pub sign_time: Option<SystemTime>,
}

/// The trait for signers.
pub trait Signer {
    /// Signs the request using the provided signing context.
    fn sign(
        &self,
        ctx: &mut SigningContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Checks whether `header` participates in the signature, i.e. it is a
    /// default signed header or listed in `additional_headers`.
    fn is_signed_header(&self, additional_headers: &[String], header: &str) -> bool;

    /// Returns the signer as [`std::any::Any`] for concrete type checks.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// A no-op signer implementation.
pub struct NopSigner;

impl Signer for NopSigner {
    fn sign(
        &self,
        _ctx: &mut SigningContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    fn is_signed_header(&self, _additional_headers: &[String], _header: &str) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nop_signer() {
        let mut ctx = SigningContext::default();
        let signer = NopSigner;
        let result = signer.sign(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_signing_context_default() {
        let ctx = SigningContext::default();
        assert_eq!(ctx.product, None);
        assert_eq!(ctx.region, None);
        assert_eq!(ctx.bucket, None);
        assert_eq!(ctx.key, None);
        assert!(ctx.request.is_none());
        assert_eq!(ctx.sub_resource, Vec::<String>::new());
        assert_eq!(ctx.additional_headers, Vec::<String>::new());
        assert_eq!(ctx.credentials, None);
        assert!(!ctx.auth_method_query);
        assert_eq!(ctx.time, None);
        assert_eq!(ctx.clock_offset, TimeDelta::zero());
        assert_eq!(ctx.signed_headers, HashMap::<String, String>::new());
        assert_eq!(ctx.string_to_sign, String::new());
        assert_eq!(ctx.sign_time, None);
    }
}
