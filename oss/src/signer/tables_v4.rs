use crate::signer::{Signer, SignerV4, SigningContext};

/// V4 signing for the table bucket product.
///
/// The only difference from [`SignerV4`] is the canonical URI, which this
/// product builds itself:
///
/// * with a bucket ARN it is `/{arn}/{key}`, where the key is unescaped first;
/// * without one it is `/acs:osstables:{region}::bucket//{key}`, the shape a
///   bucket-less request such as `CreateTableBucket` signs.
///
/// Both are exactly what `SignerV4` builds from a bucket and a key, so the
/// bucket and key are rewritten for the duration of the signing call and
/// restored afterwards. Mirrors Go's `SignerTablesV4.buildCanonicalUri`.
pub struct SignerTablesV4;

/// The bucket stand-in of a request that addresses no table bucket, i.e. the
/// `acs:osstables:{region}::bucket/` that `SignerV4` wraps in slashes.
fn bucket_less_arn(region: &str) -> String {
    format!("acs:osstables:{}::bucket/", region)
}

impl Signer for SignerTablesV4 {
    fn sign(
        &self,
        signing_ctx: &mut SigningContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request_bucket = signing_ctx.bucket.clone();
        let request_key = signing_ctx.key.clone();

        if request_bucket.is_none() {
            let region = signing_ctx.region.clone().unwrap_or_default();
            signing_ctx.bucket = Some(bucket_less_arn(&region));
        }
        if let Some(key) = &request_key {
            signing_ctx.key = Some(
                urlencoding::decode(key)
                    .map(|decoded| decoded.into_owned())
                    .unwrap_or_else(|_| key.clone()),
            );
        }

        let result = SignerV4.sign(signing_ctx);

        signing_ctx.bucket = request_bucket;
        signing_ctx.key = request_key;
        result
    }

    fn is_signed_header(&self, additional_headers: &[String], header: &str) -> bool {
        SignerV4.is_signed_header(additional_headers, header)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::*;
    use crate::credential::Credentials;
    use crate::{HEADER_OSS_CONTENT_SHA256, HTTP_HEADER_AUTHORIZATION};

    const BUCKET_ARN: &str = "acs:osstables:cn-hangzhou:1234567890123456:bucket/demo-bucket";

    fn context(bucket: Option<&str>, key: Option<&str>) -> SigningContext {
        SigningContext {
            product: Some("osstables".to_string()),
            region: Some("cn-hangzhou".to_string()),
            bucket: bucket.map(|b| b.to_string()),
            key: key.map(|k| k.to_string()),
            request: Some(
                reqwest::Client::new()
                    .request(
                        http::Method::GET,
                        "https://cn-hangzhou.oss-tables.aliyuncs.com/buckets",
                    )
                    .build()
                    .unwrap(),
            ),
            credentials: Some(Credentials {
                access_key_id: "ak".to_string(),
                access_key_secret: "sk".to_string(),
                security_token: String::new(),
                expires: None,
            }),
            time: Some(UNIX_EPOCH + Duration::from_secs(1_700_000_000)),
            ..Default::default()
        }
    }

    fn authorization(ctx: &SigningContext) -> String {
        ctx.request
            .as_ref()
            .expect("request")
            .headers()
            .get(HTTP_HEADER_AUTHORIZATION)
            .expect("authorization header")
            .to_str()
            .unwrap()
            .to_string()
    }

    /// `SignerV4` renders `/{bucket}/{key}` (see its own tests), so handing it
    /// the ARN and the unescaped key must reproduce the product's canonical
    /// URI. Signing both ways and comparing signatures proves the rewrite.
    #[test]
    fn test_signs_the_bucket_arn_path() {
        let mut tables = context(Some(BUCKET_ARN), Some("buckets%2Fabc"));
        SignerTablesV4.sign(&mut tables).expect("signing");

        let mut expected = context(Some(BUCKET_ARN), Some("buckets/abc"));
        SignerV4.sign(&mut expected).expect("signing");

        assert_eq!(authorization(&tables), authorization(&expected));
    }

    /// Without a bucket the canonical URI is
    /// `/acs:osstables:{region}::bucket//{key}`.
    #[test]
    fn test_signs_the_bucket_less_arn_path() {
        let mut tables = context(None, Some("buckets"));
        SignerTablesV4.sign(&mut tables).expect("signing");

        let mut expected = context(Some("acs:osstables:cn-hangzhou::bucket/"), Some("buckets"));
        SignerV4.sign(&mut expected).expect("signing");

        assert_eq!(authorization(&tables), authorization(&expected));
    }

    /// The rewrite must not leak: callers still see what they passed.
    #[test]
    fn test_sign_restores_the_request_bucket_and_key() {
        let mut signing_ctx = context(Some(BUCKET_ARN), Some("buckets%2Fabc"));

        SignerTablesV4.sign(&mut signing_ctx).expect("signing");

        assert_eq!(signing_ctx.bucket.as_deref(), Some(BUCKET_ARN));
        assert_eq!(signing_ctx.key.as_deref(), Some("buckets%2Fabc"));
        assert!(authorization(&signing_ctx).starts_with("OSS4-HMAC-SHA256 Credential=ak/"));
        assert!(signing_ctx
            .request
            .as_ref()
            .unwrap()
            .headers()
            .contains_key(HEADER_OSS_CONTENT_SHA256));
    }

    #[test]
    fn test_is_signed_header_delegates_to_v4() {
        assert!(SignerTablesV4.is_signed_header(&[], "x-oss-date"));
        assert!(SignerTablesV4.is_signed_header(&["x-custom".to_string()], "x-custom"));
        assert!(!SignerTablesV4.is_signed_header(&[], "x-custom"));
    }
}
