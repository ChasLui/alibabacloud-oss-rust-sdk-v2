use crate::signer::{Signer, SignerV4, SigningContext};

/// V4 signing for the vector bucket product.
///
/// The only difference from [`SignerV4`] is the canonical URI: vectors signs an
/// ARN path, `/acs:ossvector:{region}:{account_id}:{bucket}/{key}`, instead of
/// the request path. That is exactly the shape `SignerV4` builds from a bucket
/// and a key, so the bucket is swapped for its ARN form for the duration of the
/// signing call and restored afterwards. Mirrors Go's `SignerVectorsV4`.
pub struct SignerVectorsV4 {
    account_id: String,
}

impl SignerVectorsV4 {
    pub fn new(account_id: &str) -> Self {
        SignerVectorsV4 {
            account_id: account_id.to_string(),
        }
    }

    /// The bucket segment of the ARN path.
    ///
    /// Without a bucket the ARN still carries the region and an empty account
    /// ID, i.e. `acs:ossvector:{region}::`.
    pub(crate) fn arn_bucket(&self, signing_ctx: &SigningContext) -> String {
        let region = signing_ctx.region.as_deref().unwrap_or_default();
        match signing_ctx.bucket.as_deref() {
            Some(bucket) => format!("acs:ossvector:{}:{}:{}", region, self.account_id, bucket),
            None => format!("acs:ossvector:{}::", region),
        }
    }
}

impl Signer for SignerVectorsV4 {
    fn sign(
        &self,
        signing_ctx: &mut SigningContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let arn_bucket = self.arn_bucket(signing_ctx);
        let request_bucket = signing_ctx.bucket.replace(arn_bucket);
        let result = SignerV4.sign(signing_ctx);
        signing_ctx.bucket = request_bucket;
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
    use super::*;
    use crate::credential::Credentials;
    use crate::HTTP_HEADER_AUTHORIZATION;

    fn context(bucket: Option<&str>) -> SigningContext {
        SigningContext {
            region: Some("cn-hangzhou".to_string()),
            product: Some("oss".to_string()),
            bucket: bucket.map(|b| b.to_string()),
            key: Some("k".to_string()),
            request: Some(
                reqwest::Client::new()
                    .request(http::Method::GET, "https://example.com/")
                    .build()
                    .unwrap(),
            ),
            credentials: Some(Credentials {
                access_key_id: "ak".to_string(),
                access_key_secret: "sk".to_string(),
                security_token: String::new(),
                expires: None,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_arn_bucket_with_bucket() {
        let signer = SignerVectorsV4::new("123");
        assert_eq!(
            signer.arn_bucket(&context(Some("my-vector"))),
            "acs:ossvector:cn-hangzhou:123:my-vector"
        );
    }

    #[test]
    fn test_arn_bucket_without_bucket() {
        let signer = SignerVectorsV4::new("123");
        assert_eq!(
            signer.arn_bucket(&context(None)),
            "acs:ossvector:cn-hangzhou::"
        );
    }

    /// The bucket swap must not leak: callers still see the name they passed.
    #[test]
    fn test_sign_restores_the_request_bucket() {
        let mut signing_ctx = context(Some("my-vector"));

        SignerVectorsV4::new("123")
            .sign(&mut signing_ctx)
            .expect("signing must succeed");

        assert_eq!(signing_ctx.bucket.as_deref(), Some("my-vector"));
        let authorization = signing_ctx
            .request
            .expect("request")
            .headers()
            .get(HTTP_HEADER_AUTHORIZATION)
            .expect("authorization header")
            .to_str()
            .unwrap()
            .to_string();
        assert!(authorization.starts_with("OSS4-HMAC-SHA256 Credential=ak/"));
    }
}
