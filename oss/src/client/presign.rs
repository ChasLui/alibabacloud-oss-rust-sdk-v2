//! Presign support: generate pre-signed URLs for a fixed set of operations
//! without sending any HTTP request.

use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use super::{apply_operation_metadata, apply_operation_opt, Client, ClientOptions};
use crate::api::object::{
    AbortMultipartUploadRequest, CompleteMultipartUploadRequest, GetObjectRequest,
    HeadObjectRequest, InitiateMultipartUploadRequest, PutObjectRequest, UploadPartRequest,
};
use crate::signer::{DEFAULT_EXPIRES_DURATION, SIGN_TIME};
use crate::utils::modify_request;
use crate::{AuthMethodType, OperationInput};

/// Options for [`Client::presign`].
#[derive(Debug, Default)]
pub struct PresignOptions {
    /// The expiration duration for the generated presign url.
    pub expires: Option<Duration>,

    /// The expiration time for the generated presign url.
    /// Takes precedence over `expires` when both are set.
    pub expiration: Option<SystemTime>,
}

/// The result of [`Client::presign`].
#[derive(Debug, Default)]
pub struct PresignResult {
    /// The HTTP method of the pre-signed request.
    pub method: String,

    /// The pre-signed URL.
    pub url: String,

    /// The expiration time of the pre-signed URL. When neither
    /// [`PresignOptions::expires`] nor [`PresignOptions::expiration`] is set,
    /// this is the signing time plus the signer default (900 seconds).
    pub expiration: Option<SystemTime>,

    /// The headers that participate in the signature. Callers must send these
    /// headers unchanged when using the pre-signed URL.
    pub signed_headers: HashMap<String, String>,
}

/// A request type that can be pre-signed.
///
/// Implemented only for the request types supported by the official Go SDK
/// v2 presign: GetObject, PutObject, HeadObject, InitiateMultipartUpload,
/// UploadPart, CompleteMultipartUpload and AbortMultipartUpload.
pub trait PresignRequest {
    /// Builds the base [OperationInput] for the operation, merging the
    /// request's header and query fields.
    fn to_operation_input(
        &self,
    ) -> Result<OperationInput, Box<dyn std::error::Error + Send + Sync>>;
}

macro_rules! impl_presign_request {
    ($request:ty, $op_name:literal, $method:expr) => {
        impl PresignRequest for $request {
            fn to_operation_input(
                &self,
            ) -> Result<OperationInput, Box<dyn std::error::Error + Send + Sync>> {
                let mut input = OperationInput {
                    op_name: $op_name.to_string(),
                    method: $method,
                    bucket: Some(self.bucket.clone()),
                    key: Some(self.key.clone()),
                    ..Default::default()
                };
                modify_request(&mut input, self.header_map(), self.query_map(), vec![])?;
                Ok(input)
            }
        }
    };
}

impl_presign_request!(GetObjectRequest, "GetObject", http::Method::GET);
impl_presign_request!(PutObjectRequest, "PutObject", http::Method::PUT);
impl_presign_request!(HeadObjectRequest, "HeadObject", http::Method::HEAD);
impl_presign_request!(UploadPartRequest, "UploadPart", http::Method::PUT);
impl_presign_request!(
    CompleteMultipartUploadRequest,
    "CompleteMultipartUpload",
    http::Method::POST
);
impl_presign_request!(
    AbortMultipartUploadRequest,
    "AbortMultipartUpload",
    http::Method::DELETE
);

impl PresignRequest for InitiateMultipartUploadRequest {
    fn to_operation_input(
        &self,
    ) -> Result<OperationInput, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "InitiateMultipartUpload".to_string(),
            method: http::Method::POST,
            bucket: Some(self.bucket.clone()),
            key: Some(self.key.clone()),
            parameters: [("uploads", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };
        modify_request(&mut input, self.header_map(), self.query_map(), vec![])?;
        Ok(input)
    }
}

impl Client {
    /// Generates a pre-signed URL for the given request without sending any
    /// HTTP request.
    ///
    /// The request is signed with the query authentication method. For the V4
    /// signer the expiration must not be later than seven days from now.
    ///
    /// # Arguments
    ///
    /// * `request` - One of the request types implementing [PresignRequest].
    /// * `options` - Optional [PresignOptions] controlling the expiration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::GetObjectRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::{Client, PresignOptions};
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let result = client
    ///     .presign(
    ///         &GetObjectRequest {
    ///             bucket: "my-bucket".to_string(),
    ///             key: "my-object".to_string(),
    ///             ..Default::default()
    ///         },
    ///         None,
    ///     )
    ///     .await
    ///     .unwrap();
    /// println!("presigned url: {}", result.url);
    /// # })
    /// ```
    pub async fn presign<R: PresignRequest>(
        &self,
        request: &R,
        options: Option<&PresignOptions>,
    ) -> Result<PresignResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = request.to_operation_input()?;

        // Expiration: explicit expiration wins over expires duration. When
        // neither is set, no SIGN_TIME is set and the signer default applies.
        let now = SystemTime::now();
        let expiration = options.and_then(|o| match (&o.expiration, &o.expires) {
            (Some(expiration), _) => Some(*expiration),
            (None, Some(expires)) => Some(now + *expires),
            (None, None) => None,
        });
        if let Some(expiration) = expiration {
            input.op_metadata.set(SIGN_TIME, Rc::new(expiration));
        }

        // Assemble options like invoke_operation_inner does, with the query
        // auth method forced.
        let mut opts = self.options.clone();
        let modified_options = ClientOptions {
            auth_method: Some(AuthMethodType::Query),
            ..Default::default()
        };
        apply_operation_opt(&mut opts, &modified_options);
        apply_operation_metadata(&input, &mut opts);

        let mut signing_ctx = self.build_signing_context(input, Some(&opts)).await?;
        self.sign_request(&mut signing_ctx, Some(&opts)).await?;

        // Backfill the expiration: the signers do not always write the time
        // back (V4 leaves it None when no SIGN_TIME was set).
        let expiration = signing_ctx
            .time
            .or_else(|| Some(SystemTime::now() + DEFAULT_EXPIRES_DURATION));

        let signer = opts.signer.as_ref().expect("Signer not set");

        let is_v4_signer = {
            use crate::signer::v4::SignerV4;
            signer.as_ref().as_any().type_id() == std::any::TypeId::of::<SignerV4>()
        };
        if is_v4_signer {
            if let Some(expiration) = expiration {
                if expiration > SystemTime::now() + Duration::from_secs(604800) {
                    return Err("expires should be not greater than 604800(seven days)".into());
                }
            }
        }

        let request = signing_ctx.request.as_ref().expect("Request not set");

        // Collect the headers that participate in the signature.
        let mut signed_headers = HashMap::new();
        for (name, value) in request.headers() {
            if signer.is_signed_header(&opts.additional_headers, name.as_str()) {
                signed_headers.insert(
                    name.as_str().to_string(),
                    value.to_str().unwrap_or_default().to_string(),
                );
            }
        }

        Ok(PresignResult {
            method: request.method().as_str().to_string(),
            url: request.url().to_string(),
            expiration,
            signed_headers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::SignatureVersionType;

    fn test_client() -> Client {
        Client::new(
            &Config::default()
                .with_region("cn-hangzhou")
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    "test-ak",
                    "test-sk",
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V4),
        )
    }

    #[tokio::test]
    async fn test_presign_get_object_default() {
        let client = test_client();
        let result = client
            .presign(
                &GetObjectRequest {
                    bucket: "my-bucket".to_string(),
                    key: "my-object".to_string(),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        assert_eq!(result.method, "GET");
        assert!(result
            .url
            .contains("x-oss-signature-version=OSS4-HMAC-SHA256"));
        let default_expires: u64 = presigned_url_expires(&result.url);
        assert!(
            (895..=900).contains(&default_expires),
            "expires={}",
            default_expires
        );
        assert!(result.url.contains("x-oss-signature="));
        assert!(result.url.contains("my-bucket"));
        assert!(result.url.contains("my-object"));

        // expiration defaults to signing time + 900s
        let expiration = result.expiration.unwrap();
        let now = SystemTime::now();
        assert!(expiration > now);
        assert!(expiration <= now + Duration::from_secs(960));
    }

    #[tokio::test]
    async fn test_presign_get_object_expires() {
        let client = test_client();
        let options = PresignOptions {
            expires: Some(Duration::from_secs(60)),
            ..Default::default()
        };
        let result = client
            .presign(
                &GetObjectRequest {
                    bucket: "my-bucket".to_string(),
                    key: "my-object".to_string(),
                    ..Default::default()
                },
                Some(&options),
            )
            .await
            .unwrap();

        // The signer truncates sub-second precision, so 60s may surface as 59
        let expires_value: u64 = presigned_url_expires(&result.url);
        assert!(
            (55..=60).contains(&expires_value),
            "expires={}",
            expires_value
        );
        let expiration = result.expiration.unwrap();
        let now = SystemTime::now();
        assert!(expiration > now + Duration::from_secs(50));
        assert!(expiration <= now + Duration::from_secs(120));
    }

    #[tokio::test]
    async fn test_presign_put_object_signed_headers() {
        let client = test_client();
        let result = client
            .presign(
                &PutObjectRequest {
                    bucket: "my-bucket".to_string(),
                    key: "my-object".to_string(),
                    content_type: Some("text/plain".to_string()),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        assert_eq!(result.method, "PUT");
        // content-type is a default signed header and must be reported
        assert_eq!(
            signed_header_value(&result, "content-type"),
            Some("text/plain")
        );
    }

    #[tokio::test]
    async fn test_presign_expiration_over_seven_days() {
        let client = test_client();
        let options = PresignOptions {
            expiration: Some(SystemTime::now() + Duration::from_secs(8 * 24 * 3600)),
            ..Default::default()
        };
        let result = client
            .presign(
                &GetObjectRequest {
                    bucket: "my-bucket".to_string(),
                    key: "my-object".to_string(),
                    ..Default::default()
                },
                Some(&options),
            )
            .await;

        match result {
            Err(err) => assert!(err
                .to_string()
                .contains("expires should be not greater than 604800")),
            Ok(_) => panic!("presign should fail when expiration exceeds seven days"),
        }
    }

    #[tokio::test]
    async fn test_presign_initiate_multipart_upload() {
        let client = test_client();
        let result = client
            .presign(
                &InitiateMultipartUploadRequest {
                    bucket: "my-bucket".to_string(),
                    key: "my-object".to_string(),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        assert_eq!(result.method, "POST");
        assert!(result.url.contains("uploads"));
        assert!(result
            .url
            .contains("x-oss-signature-version=OSS4-HMAC-SHA256"));
    }

    fn presigned_url_expires(url: &str) -> u64 {
        url.split('?')
            .nth(1)
            .unwrap_or("")
            .split('&')
            .find_map(|pair| pair.strip_prefix("x-oss-expires="))
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| panic!("x-oss-expires missing/unparsable: {}", url))
    }
    fn signed_header_value<'a>(result: &'a PresignResult, name: &str) -> Option<&'a str> {
        result
            .signed_headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}
