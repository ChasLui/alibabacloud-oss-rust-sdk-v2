use std::collections::HashSet;
use std::time::SystemTime;

use base64::{self, Engine};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use lazy_static::lazy_static;
use sha1::Sha1;
use url::Url;

use super::utils::{get_query, set_query};
use super::{
    Signer, SigningContext, ACCESS_KEY_ID_QUERY, DEFAULT_EXPIRES_DURATION, EXPIRES_QUERY,
    SECURITY_TOKEN_QUERY, SIGNATURE_QUERY,
};
use crate::{
    HEADER_OSS_PREFIX, HEADER_OSS_SECURITY_TOKEN, HTTP_HEADER_AUTHORIZATION,
    HTTP_HEADER_CONTENT_MD5, HTTP_HEADER_CONTENT_TYPE, HTTP_HEADER_DATE,
};

lazy_static! {
    static ref REQUIRED_SIGNED_PARAMETERS: HashSet<&'static str> = [
        "acl",
        "bucketInfo",
        "location",
        "stat",
        "delete",
        "append",
        "tagging",
        "objectMeta",
        "uploads",
        "uploadId",
        "partNumber",
        "security-token",
        "position",
        "response-content-type",
        "response-content-language",
        "response-expires",
        "response-cache-control",
        "response-content-disposition",
        "response-content-encoding",
        "restore",
        "callback",
        "callback-var",
        "versions",
        "versioning",
        "versionId",
        "sequential",
        "continuation-token",
        "regionList",
        "cloudboxes",
        "symlink",
    ]
    .iter()
    .cloned()
    .collect();
}

/// This module provides the implementation of the SignerV1 struct, which is
/// responsible for signing requests using the OSS (Object Storage Service) V1
/// authentication method.
///
/// The SignerV1 struct implements the Signer trait, which defines the sign
/// method for signing requests. It also provides methods for generating the
/// authorization header and the authorization query string.
///
/// The signing process involves calculating a string to sign based on the
/// request information and the signing context, and then using HMAC-SHA1 to
/// generate a signature from the string to sign and the access key secret.
///
/// The SignerV1 struct also provides utility methods for checking if a header
/// is a signed header and for determining if a given key is a sub-resource.
pub struct SignerV1;

impl SignerV1 {
    /// Checks if a given key is a sub-resource.
    ///
    /// # Arguments
    ///
    /// * `list` - A list of sub-resources.
    /// * `key` - The key to check.
    ///
    /// # Returns
    ///
    /// Returns true if the key is a sub-resource, false otherwise.
    fn is_sub_resource(list: &[String], key: &str) -> bool {
        list.contains(&key.to_string())
    }

    /// Calculates the string to sign for the request based on the date and the
    /// signing context.
    ///
    /// # Arguments
    ///
    /// * `date` - The date string in RFC2822 format.
    /// * `signing_ctx` - The signing context containing the request and other
    ///   information.
    ///
    /// # Returns
    ///
    /// Returns the string to sign.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let date = "Tue, 21 Sep 2021 10:00:00 GMT";
    /// let signing_ctx = SigningContext { ... };
    /// let string_to_sign = signer_v1.calc_string_to_sign(date, &signing_ctx);
    /// ```
    ///
    /// # Note
    ///
    /// This method follows the algorithm described in the [Aliyun OSS documentation](https://help.aliyun.com/oss/developer-reference/include-signatures-in-the-authorization-header) for including signatures in the authorization header.
    fn calc_string_to_sign(&self, date: &str, signing_ctx: &SigningContext) -> String {
        let request = signing_ctx.request.as_ref().expect("Request is None");
        let content_md5 = request
            .headers()
            .get(HTTP_HEADER_CONTENT_MD5)
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or("");
        let content_type = request
            .headers()
            .get(HTTP_HEADER_CONTENT_TYPE)
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or_default();

        let mut headers: Vec<String> = request
            .headers()
            .iter()
            .filter_map(|(k, _)| {
                let lowercase_key = k.as_str().to_lowercase();
                if lowercase_key.starts_with(HEADER_OSS_PREFIX.to_lowercase().as_str()) {
                    Some(lowercase_key)
                } else {
                    None
                }
            })
            .collect();
        headers.sort();

        let header_items: Vec<String> = headers
            .iter()
            .map(|k| {
                let header_values: Vec<String> = request
                    .headers()
                    .get_all(k)
                    .iter()
                    .map(|v| v.to_str().unwrap_or("").trim().to_string())
                    .collect();
                format!("{}:{}\n", k, header_values.join(","))
            })
            .collect();

        let canonicalized_oss_headers = header_items.join("");

        let query_pairs: Vec<(String, String)> = request
            .url()
            .query_pairs()
            .filter_map(|(k, v)| {
                let key = k.into_owned();
                if REQUIRED_SIGNED_PARAMETERS.contains(key.as_str())
                    || key.starts_with(HEADER_OSS_PREFIX)
                    || Self::is_sub_resource(&signing_ctx.sub_resource, &key)
                {
                    Some((key, v.into_owned()))
                } else {
                    None
                }
            })
            .collect();

        let mut params: Vec<String> = query_pairs.iter().map(|(k, _)| k.clone()).collect();
        params.sort();

        let param_items: Vec<String> = params
            .iter()
            .map(|k| {
                let v = query_pairs
                    .iter()
                    .find(|(key, _)| key == k)
                    .map(|(_, value)| value.clone())
                    .unwrap_or_default();
                if !v.is_empty() {
                    format!("{}={}", k, v)
                } else {
                    k.to_string()
                }
            })
            .collect();
        let sub_resource = param_items.join("&");

        let mut canonicalized_resource = "/".to_string();
        if let Some(bucket) = &signing_ctx.bucket {
            canonicalized_resource += format!("{}/", bucket).as_str();
        }
        if let Some(key) = &signing_ctx.key {
            canonicalized_resource += key;
        }
        if !sub_resource.is_empty() {
            canonicalized_resource += format!("?{}", sub_resource).as_str();
        }

        format!(
            "{}\n{}\n{}\n{}\n{}{}", // No LF between oss headers and resource
            request.method(),
            content_md5,
            content_type,
            date.replace("+0000", "GMT"),
            canonicalized_oss_headers,
            canonicalized_resource
        )
    }

    /// Generates the authorization header for the request based on the signing
    /// context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The signing context containing the request and other
    ///   information.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the authorization header is generated successfully, or
    /// an error if there is a failure.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut signing_ctx = SigningContext { ... };
    /// signer_v1.auth_header(&mut signing_ctx)?;
    /// ```
    ///
    /// # Note
    ///
    /// This method calculates the string to sign, generates the signature using
    /// HMAC-SHA1, and adds the authorization header to the request.
    fn auth_header(
        &self,
        ctx: &mut SigningContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cred = ctx.credentials.as_ref().expect("Credentials is None");

        if ctx.time.is_none() {
            ctx.time = Some(SystemTime::now() + ctx.clock_offset);
        }
        let datetime: DateTime<Utc> = DateTime::from(ctx.time.expect("Time is None"));
        let datetime_string = datetime.to_rfc2822();

        let string_to_sign = self.calc_string_to_sign(datetime_string.as_str(), ctx);
        ctx.string_to_sign.clone_from(&string_to_sign);

        let mut hmac = Hmac::<Sha1>::new_from_slice(cred.access_key_secret.as_bytes())?;
        hmac.update(string_to_sign.as_bytes());
        let signature =
            base64::engine::general_purpose::STANDARD.encode(hmac.finalize().into_bytes());

        let request = ctx.request.as_mut().expect("Request is None");

        request
            .headers_mut()
            .insert(HTTP_HEADER_DATE, datetime_string.parse()?);

        if !cred.security_token.is_empty() {
            request
                .headers_mut()
                .insert(HEADER_OSS_SECURITY_TOKEN, cred.security_token.parse()?);
        }
        request.headers_mut().insert(
            HTTP_HEADER_AUTHORIZATION,
            format!("OSS {}:{}", cred.access_key_id, signature).parse()?,
        );

        Ok(())
    }

    /// Generates the authorization query string for the request based on the
    /// signing context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The signing context containing the request and other
    ///   information.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the authorization query string is generated
    /// successfully, or an error if there is a failure.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut signing_ctx = SigningContext { ... };
    /// signer_v1.auth_query(&mut signing_ctx)?;
    /// ```
    ///
    /// # Note
    ///
    /// This method calculates the string to sign, generates the signature using
    /// HMAC-SHA1, and adds the authorization query parameters to the request
    /// URL.
    fn auth_query(
        &self,
        ctx: &mut SigningContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cred = ctx.credentials.as_ref().expect("Credentials is None");

        if ctx.time.is_none() {
            ctx.time = Some(SystemTime::now() + DEFAULT_EXPIRES_DURATION);
        }
        let datetime = format!(
            "{}",
            ctx.time
                .expect("Time is None")
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs()
        );

        let mut query = get_query(ctx.request.as_ref().expect("Request is None"));
        if !cred.security_token.is_empty() {
            query.insert(
                SECURITY_TOKEN_QUERY.to_string(),
                cred.security_token.clone(),
            );
            // Add "security-token=xxx" to `ctx.request.url.query` for later signature
            // calculation
            set_query(ctx.request.as_mut().expect("Request is None"), &query)
        }

        let string_to_sign = self.calc_string_to_sign(&datetime, ctx);
        ctx.string_to_sign.clone_from(&string_to_sign);

        let mut hmac = Hmac::<Sha1>::new_from_slice(cred.access_key_secret.as_bytes())?;
        hmac.update(string_to_sign.as_bytes());
        let signature =
            base64::engine::general_purpose::STANDARD.encode(hmac.finalize().into_bytes());

        query.insert(EXPIRES_QUERY.to_string(), datetime);
        query.insert(ACCESS_KEY_ID_QUERY.to_string(), cred.access_key_id.clone());
        query.insert(SIGNATURE_QUERY.to_string(), signature);

        let url = Url::parse_with_params(
            ctx.request
                .as_ref()
                .expect("Request is None")
                .url()
                .as_str(),
            query.iter(),
        )?;

        *ctx.request.as_mut().expect("Request is None").url_mut() = url;

        Ok(())
    }

}

impl Signer for SignerV1 {
    /// Signs the request using the OSS V1 authentication method.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The signing context containing the request and other
    ///   information.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the request is signed successfully, or an error if
    /// there is a failure.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut signing_ctx = SigningContext { ... };
    /// signer_v1.sign(&mut signing_ctx)?;
    /// ```
    ///
    /// # Note
    ///
    /// This method first checks if the signing context has valid credentials
    /// and a request. It then calls either the auth_header or auth_query
    /// method based on the auth_method_query flag in the signing context.
    fn sign(
        &self,
        ctx: &mut SigningContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(credentials) = ctx.credentials.as_ref() {
            if !credentials.has_keys() {
                return Err("SigningContext.Credentials is empty.".into());
            }
        } else {
            return Err("SigningContext.Credentials is None.".into());
        }
        if ctx.request.is_none() {
            return Err("SigningContext.Request is None.".into());
        }

        if ctx.auth_method_query {
            self.auth_query(ctx)
        } else {
            self.auth_header(ctx)
        }
    }

    fn is_signed_header(&self, additional_headers: &[String], header: &str) -> bool {
        let lowercase_header = header.to_lowercase();
        lowercase_header.starts_with(HEADER_OSS_PREFIX.to_lowercase().as_str())
            || lowercase_header == HTTP_HEADER_DATE.to_lowercase()
            || lowercase_header == HTTP_HEADER_CONTENT_TYPE.to_lowercase()
            || lowercase_header == HTTP_HEADER_CONTENT_MD5.to_lowercase()
            || additional_headers
                .iter()
                .any(|h| h.eq_ignore_ascii_case(header))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use http::Method;
    use reqwest::Request;

    use super::*;
    use crate::credential::{self, Credentials, CredentialsProvider};
    use crate::*;

    const OSS_META_AUTHOR_HEADER: &str = "x-oss-meta-author";
    const OSS_META_MAGIC_HEADER: &str = "x-oss-meta-magic";

    #[test]
    fn test_is_sub_resource() {
        let list = vec![
            "acl".to_string(),
            "location".to_string(),
            "tagging".to_string(),
        ];
        assert!(SignerV1::is_sub_resource(&list, "acl"));
        assert!(!SignerV1::is_sub_resource(&list, "bucketInfo"));
        assert!(SignerV1::is_sub_resource(&list, "tagging"));
    }

    async fn get_cred(id: &str, secret: &str, tokens: &[&str]) -> Credentials {
        let credentials_provider = credential::StaticCredentialsProvider::new(id, secret, tokens);
        credentials_provider.get_credentials().await.unwrap()
    }

    #[tokio::test]
    async fn test_auth_header_case_1() {
        let cred = get_cred("ak", "sk", &[]).await;

        let mut request = Request::new(
            Method::PUT,
            "http://examplebucket.oss-cn-hangzhou.aliyuncs.com"
                .parse()
                .unwrap(),
        );
        request.headers_mut().insert(
            HTTP_HEADER_CONTENT_MD5,
            "eB5eJF1ptWaXm4bijSPyxw==".parse().unwrap(),
        );
        request
            .headers_mut()
            .insert(HTTP_HEADER_CONTENT_TYPE, "text/html".parse().unwrap());
        request
            .headers_mut()
            .insert(OSS_META_AUTHOR_HEADER, "alice".parse().unwrap());
        request
            .headers_mut()
            .insert(OSS_META_MAGIC_HEADER, "abracadabra".parse().unwrap());
        request.headers_mut().insert(
            HEADER_OSS_DATE,
            "Wed, 28 Dec 2022 10:27:41 GMT".parse().unwrap(),
        );

        let sign_datetime = DateTime::parse_from_rfc2822("Wed, 28 Dec 2022 10:27:41 GMT").unwrap();
        let sign_time: SystemTime = sign_datetime.into();
        let mut sign_ctx = SigningContext {
            bucket: Some("examplebucket".into()),
            key: Some("nelson".into()),
            request: request.try_clone().unwrap().into(),
            credentials: cred.into(),
            time: sign_time.into(),
            ..Default::default()
        };

        let signer = SignerV1 {};
        let _ = signer.sign(&mut sign_ctx);

        let sign_to_string = "PUT\neB5eJF1ptWaXm4bijSPyxw==\ntext/html\nWed, 28 Dec 2022 10:27:41 \
                              GMT\nx-oss-date:Wed, 28 Dec 2022 10:27:41 \
                              GMT\nx-oss-meta-author:alice\nx-oss-meta-magic:abracadabra\n/\
                              examplebucket/nelson";
        assert_eq!(sign_to_string, sign_ctx.string_to_sign);
        assert_eq!(sign_time, sign_ctx.time.unwrap());

        assert_eq!(
            "OSS ak:kSHKmLxlyEAKtZPkJhG9bZb5k7M=",
            sign_ctx
                .request
                .unwrap()
                .headers()
                .get(HTTP_HEADER_AUTHORIZATION.to_lowercase())
                .unwrap()
                .to_str()
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_auth_header_case_2() {
        let cred = get_cred("ak", "sk", &[]).await;

        let mut request = Request::new(
            Method::PUT,
            "http://examplebucket.oss-cn-hangzhou.aliyuncs.com/?acl"
                .parse()
                .unwrap(),
        );
        request.headers_mut().insert(
            HTTP_HEADER_CONTENT_MD5,
            "eB5eJF1ptWaXm4bijSPyxw==".parse().unwrap(),
        );
        request
            .headers_mut()
            .insert(HTTP_HEADER_CONTENT_TYPE, "text/html".parse().unwrap());
        request
            .headers_mut()
            .insert(OSS_META_AUTHOR_HEADER, "alice".parse().unwrap());
        request
            .headers_mut()
            .insert(OSS_META_MAGIC_HEADER, "abracadabra".parse().unwrap());
        request.headers_mut().insert(
            HEADER_OSS_DATE,
            "Wed, 28 Dec 2022 10:27:41 GMT".parse().unwrap(),
        );

        let sign_datetime = DateTime::parse_from_rfc2822("Wed, 28 Dec 2022 10:27:41 GMT").unwrap();
        let sign_time: SystemTime = sign_datetime.into();
        let mut sign_ctx = SigningContext {
            bucket: Some("examplebucket".into()),
            key: Some("nelson".into()),
            request: request.try_clone().unwrap().into(),
            credentials: cred.into(),
            time: sign_time.into(),
            ..Default::default()
        };

        let _ = SignerV1 {}.sign(&mut sign_ctx);

        let sign_to_string = "PUT\neB5eJF1ptWaXm4bijSPyxw==\ntext/html\nWed, 28 Dec 2022 10:27:41 \
                              GMT\nx-oss-date:Wed, 28 Dec 2022 10:27:41 \
                              GMT\nx-oss-meta-author:alice\nx-oss-meta-magic:abracadabra\n/\
                              examplebucket/nelson?acl";
        assert_eq!(sign_to_string, sign_ctx.string_to_sign);
        assert_eq!(sign_time, sign_ctx.time.unwrap());
        assert_eq!(
            "OSS ak:/afkugFbmWDQ967j1vr6zygBLQk=",
            sign_ctx
                .request
                .unwrap()
                .headers()
                .get(HTTP_HEADER_AUTHORIZATION.to_lowercase())
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_auth_header_case_3() {
        let cred = get_cred("ak", "sk", &[]).await;

        let mut request = Request::new(
            Method::GET,
            "http://examplebucket.oss-cn-hangzhou.aliyuncs.com/?resourceGroup&non-resousce=null"
                .parse()
                .unwrap(),
        );
        request.headers_mut().insert(
            HEADER_OSS_DATE,
            "Wed, 28 Dec 2022 10:27:41 GMT".parse().unwrap(),
        );

        let sign_datetime = DateTime::parse_from_rfc2822("Wed, 28 Dec 2022 10:27:41 GMT").unwrap();
        let sign_time: SystemTime = sign_datetime.into();
        let sign_ctx = &mut SigningContext {
            bucket: Some("examplebucket".into()),
            request: request.try_clone().unwrap().into(),
            credentials: cred.into(),
            time: sign_time.into(),
            sub_resource: vec!["resourceGroup".to_string()],
            ..Default::default()
        };

        let _ = SignerV1 {}.sign(sign_ctx);

        let sign_to_string = "GET\n\n\nWed, 28 Dec 2022 10:27:41 GMT\nx-oss-date:Wed, 28 Dec 2022 \
                              10:27:41 GMT\n/examplebucket/?resourceGroup";
        assert_eq!(sign_to_string, sign_ctx.string_to_sign);
        assert_eq!(sign_time, sign_ctx.time.unwrap());
        assert_eq!(
            "OSS ak:vkQmfuUDyi1uDi3bKt67oemssIs=",
            sign_ctx
                .request
                .as_ref()
                .unwrap()
                .headers()
                .get(HTTP_HEADER_AUTHORIZATION.to_lowercase())
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_auth_query_case_1() {
        let cred = get_cred("ak", "sk", &[]).await;

        let request = Request::new(
            Method::GET,
            "http://bucket.oss-cn-hangzhou.aliyuncs.com/key?versionId=versionId"
                .parse()
                .unwrap(),
        );

        let sign_datetime = DateTime::parse_from_rfc2822("Sun, 12 Nov 2023 16:43:40 GMT").unwrap();
        let sign_time: SystemTime = sign_datetime.into();
        let sign_ctx = &mut SigningContext {
            bucket: Some("bucket".into()),
            key: Some("key".into()),
            request: request.try_clone().unwrap().into(),
            credentials: cred.into(),
            time: sign_time.into(),
            auth_method_query: true,
            ..Default::default()
        };

        let _ = SignerV1 {}.sign(sign_ctx);

        // http://bucket.oss-cn-hangzhou.aliyuncs.com/key?Expires=1699807420&OSSAccessKeyId=ak&Signature=dcLTea%2BYh9ApirQ8o8dOPqtvJXQ%3D&versionId=versionId
        let sign_url = sign_ctx.request.as_ref().unwrap().url();
        assert_eq!(
            sign_url.domain().unwrap(),
            "bucket.oss-cn-hangzhou.aliyuncs.com"
        );
        assert_eq!(sign_url.path(), "/key");
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == EXPIRES_QUERY)
                .unwrap()
                .1,
            "1699807420"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == ACCESS_KEY_ID_QUERY)
                .unwrap()
                .1,
            "ak"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == SIGNATURE_QUERY)
                .unwrap()
                .1,
            "dcLTea+Yh9ApirQ8o8dOPqtvJXQ="
        );
    }

    #[tokio::test]
    async fn test_auth_query_case_2() {
        let cred = get_cred("ak", "sk", &["token"]).await;

        let request = Request::new(
            Method::GET,
            "http://bucket.oss-cn-hangzhou.aliyuncs.com/key%2B123?versionId=versionId"
                .parse()
                .unwrap(),
        );

        let sign_datetime = DateTime::parse_from_rfc2822("Sun, 12 Nov 2023 16:56:44 GMT").unwrap();
        let sign_time: SystemTime = sign_datetime.into();
        let sign_ctx = &mut SigningContext {
            bucket: Some("bucket".into()),
            key: Some("key+123".into()),
            request: request.try_clone().unwrap().into(),
            credentials: cred.into(),
            time: sign_time.into(),
            auth_method_query: true,
            ..Default::default()
        };

        let _ = SignerV1 {}.sign(sign_ctx);

        // http://bucket.oss-cn-hangzhou.aliyuncs.com/key%2B123?Expires=1699808204&OSSAccessKeyId=ak&Signature=jzKYRrM5y6Br0dRFPaTGOsbrDhY%3D&security-token=token&versionId=versionI
        let sign_url = sign_ctx.request.as_ref().unwrap().url();
        assert_eq!(
            sign_url.domain().unwrap(),
            "bucket.oss-cn-hangzhou.aliyuncs.com"
        );
        assert_eq!(sign_url.path(), "/key%2B123");
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == EXPIRES_QUERY)
                .unwrap()
                .1,
            "1699808204"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == ACCESS_KEY_ID_QUERY)
                .unwrap()
                .1,
            "ak"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == SECURITY_TOKEN_QUERY)
                .unwrap()
                .1,
            "token"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == SIGNATURE_QUERY)
                .unwrap()
                .1,
            "jzKYRrM5y6Br0dRFPaTGOsbrDhY="
        );
    }

    #[test]
    fn test_invalid_argument() {
        let signer = SignerV1;

        let mut sign_ctx = SigningContext {
            credentials: None,
            ..Default::default()
        };
        let err = signer.sign(&mut sign_ctx).unwrap_err();
        assert!(err
            .to_string()
            .contains("SigningContext.Credentials is None"));

        sign_ctx.credentials = Some(Credentials {
            access_key_id: "".to_string(),
            access_key_secret: "sk".to_string(),
            ..Default::default()
        });
        let err = signer.sign(&mut sign_ctx).unwrap_err();
        assert!(err
            .to_string()
            .contains("SigningContext.Credentials is empty"));

        sign_ctx.credentials = Some(Credentials {
            access_key_id: "ak".to_string(),
            access_key_secret: "sk".to_string(),
            ..Default::default()
        });
        let err = signer.sign(&mut sign_ctx).unwrap_err();
        assert!(err.to_string().contains("SigningContext.Request is None"));
    }
}
