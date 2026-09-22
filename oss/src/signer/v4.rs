use std::collections::HashSet;
use std::fmt::Write;
use std::time::Duration;

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use http::header::HeaderMap;
use md5::digest::InvalidLength;
use sha2::{Digest, Sha256};

use super::utils::{get_decoded_query_from_str, set_query};
use super::{
    Signer, SigningContext, ALGORITHM_V4, DEFAULT_EXPIRES_DURATION, ISO8601_DATETIME_FORMAT,
    ISO8601_DATE_FORMAT, UNSIGNED_PAYLOAD,
};
use crate::signer::utils::get_encoded_query_from_str;
use crate::utils::escape_path;
use crate::{
    HEADER_OSS_ADDITIONAL_HEADERS, HEADER_OSS_CONTENT_SHA256, HEADER_OSS_CREDENTIAL,
    HEADER_OSS_DATE, HEADER_OSS_EXPIRES, HEADER_OSS_PREFIX, HEADER_OSS_SECURITY_TOKEN,
    HEADER_OSS_SIGNATURE, HEADER_OSS_SIGNATURE_VERSION, HTTP_HEADER_AUTHORIZATION,
    HTTP_HEADER_CONTENT_MD5, HTTP_HEADER_CONTENT_TYPE, HTTP_HEADER_DATE,
};

fn is_default_signed_header(header: &str) -> bool {
    let lowercase_header = header.to_lowercase();
    lowercase_header.starts_with(HEADER_OSS_PREFIX.to_lowercase().as_str())
        || lowercase_header.to_lowercase() == HTTP_HEADER_CONTENT_TYPE.to_lowercase()
        || lowercase_header.to_lowercase() == HTTP_HEADER_CONTENT_MD5.to_lowercase()
}

fn get_common_additional_headers<'a>(
    headers: &HeaderMap,
    additional_headers: &'a [&'a str],
) -> Vec<String> {
    let mut keys = Vec::new();
    for key in additional_headers {
        let lowercase_key = key.to_lowercase();
        if is_default_signed_header(&lowercase_key) {
            // Skip default signed header
            continue;
        } else if headers.contains_key(&lowercase_key) {
            keys.push(lowercase_key);
        }
    }
    keys.sort();
    keys
}

pub struct SignerV4;

impl SignerV4 {
    fn calc_string_to_sign(&self, datetime: &str, scope: &str, canonical_request: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(canonical_request.as_bytes());
        let canonical_hash = hex::encode(hasher.finalize());

        format!(
            "{}\n{}\n{}\n{}",
            ALGORITHM_V4, datetime, scope, canonical_hash
        )
    }

    fn calc_canonical_request(&self, ctx: &SigningContext, additional_headers: &[&str]) -> String {
        let request = ctx.request.as_ref().expect("Request is None");

        // Canonical URI
        let mut uri = "/".to_string();
        if let Some(bucket) = ctx.bucket.as_ref() {
            uri += format!("{}/", bucket).as_str();
        }
        if let Some(key) = ctx.key.as_ref() {
            uri += key;
        }
        let canonical_uri = escape_path(&uri, false);

        // Canonical Query
        let query_str = request.url().query().unwrap_or_default();
        let query = get_encoded_query_from_str(query_str);
        let mut keys: Vec<&String> = query.keys().collect();
        keys.sort();
        let mut canonical_query = String::new();
        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                canonical_query.push('&');
            }
            canonical_query.push_str(key);
            if let Some(val) = query.get(*key) {
                if !val.is_empty() {
                    canonical_query.push('=');
                    canonical_query.push_str(val);
                }
            }
        }

        // Canonical Headers
        let mut add_headers_set = HashSet::new();
        for k in additional_headers {
            add_headers_set.insert(k.to_lowercase());
        }
        let mut headers = Vec::<(String, String)>::new();
        for (k, v) in request.headers().iter() {
            let lowercase_k = k.as_str().to_lowercase();
            if is_default_signed_header(&lowercase_k) || add_headers_set.contains(&lowercase_k) {
                headers.push((
                    lowercase_k,
                    v.to_str().expect("Non-ASCII chars found").to_string(),
                ));
            }
        }
        headers.sort_by(|a, b| a.0.cmp(&b.0));
        let canonical_headers = headers.iter().fold(String::new(), |mut output, (h, v)| {
            let _ = writeln!(&mut output, "{}:{}", h, v.trim());
            output
        });

        // Additional Headers
        let canonical_additional_headers = additional_headers.join(";");

        // Hashed Payload
        let hash_payload = request
            .headers()
            .get(HEADER_OSS_CONTENT_SHA256)
            .map(|v| v.to_str().expect("None-ASCII chars found").to_string())
            .unwrap_or_else(|| UNSIGNED_PAYLOAD.to_string());

        format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            request.method(),
            canonical_uri,
            canonical_query,
            canonical_headers,
            canonical_additional_headers,
            hash_payload
        )
    }

    fn build_scope(&self, date: &str, region: &str, product: &str) -> String {
        format!("{}/{}/{}/aliyun_v4_request", date, region, product)
    }

    fn calc_signature(
        &self,
        secret_key: &str,
        date: &str,
        region: &str,
        product: &str,
        string_to_sign: &str,
    ) -> Result<String, InvalidLength> {
        let mut hmac_key =
            Hmac::<Sha256>::new_from_slice(format!("aliyun_v4{}", secret_key).as_bytes())?;
        hmac_key.update(date.as_bytes());
        let h1_key = hmac_key.finalize().into_bytes();

        let mut hmac_key = Hmac::<Sha256>::new_from_slice(&h1_key)?;
        hmac_key.update(region.as_bytes());
        let h2_key = hmac_key.finalize().into_bytes();

        let mut hmac_key = Hmac::<Sha256>::new_from_slice(&h2_key)?;
        hmac_key.update(product.as_bytes());
        let h3_key = hmac_key.finalize().into_bytes();

        let mut hmac_key = Hmac::<Sha256>::new_from_slice(&h3_key)?;
        hmac_key.update("aliyun_v4_request".as_bytes());
        let h4_key = hmac_key.finalize().into_bytes();

        let mut hmac_key = Hmac::<Sha256>::new_from_slice(&h4_key)?;
        hmac_key.update(string_to_sign.as_bytes());
        let h5_key = hmac_key.finalize().into_bytes();

        Ok(hex::encode(h5_key))
    }

    fn auth_header(
        &self,
        ctx: &mut SigningContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = ctx.request.as_mut().ok_or("Request is None")?;
        let cred = ctx.credentials.as_ref().ok_or("Credentials is None")?;

        // Date
        if ctx.time.is_none() {
            ctx.time = Some(super::now_with_offset(ctx.clock_offset));
        }
        let utc_datetime: DateTime<Utc> = ctx.time.ok_or("Time is None")?.into();
        let datetime = utc_datetime.format(ISO8601_DATETIME_FORMAT).to_string();
        let date = utc_datetime.format(ISO8601_DATE_FORMAT).to_string();
        request
            .headers_mut()
            .insert(HEADER_OSS_DATE, datetime.parse()?);
        request
            .headers_mut()
            .insert(HTTP_HEADER_DATE, utc_datetime.to_rfc2822().parse()?);

        // Credentials information
        if !cred.security_token.is_empty() {
            request
                .headers_mut()
                .insert(HEADER_OSS_SECURITY_TOKEN, cred.security_token.parse()?);
        }

        // Other Headers
        request
            .headers_mut()
            .insert(HEADER_OSS_CONTENT_SHA256, UNSIGNED_PAYLOAD.parse()?);

        // Scope
        let region = ctx.region.as_deref().unwrap_or_default();
        let product = ctx.product.as_deref().unwrap_or_default();
        let scope = self.build_scope(&date, region, product);

        let additional_headers = ctx
            .additional_headers
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();
        let additional_headers =
            get_common_additional_headers(request.headers(), &additional_headers);

        // CanonicalRequest
        let canonical_request = self.calc_canonical_request(
            ctx,
            &additional_headers
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>(),
        );

        // StringToSign
        let string_to_sign = self.calc_string_to_sign(&datetime, &scope, &canonical_request);
        ctx.string_to_sign.clone_from(&string_to_sign);

        // Signature
        let signature = self.calc_signature(
            &cred.access_key_secret,
            &date,
            region,
            product,
            &string_to_sign,
        )?;

        // Authorization Header
        let additional_headers_str = if additional_headers.is_empty() {
            "".to_string()
        } else {
            format!("AdditionalHeaders={},", additional_headers.join(";"))
        };

        let authorization = format!(
            "{} Credential={}/{},{}Signature={}",
            ALGORITHM_V4, cred.access_key_id, scope, additional_headers_str, signature
        );
        ctx.request
            .as_mut()
            .unwrap()
            .headers_mut()
            .insert(HTTP_HEADER_AUTHORIZATION, authorization.parse()?);

        Ok(())
    }

    fn auth_query(
        &self,
        signing_ctx: &mut SigningContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = &mut signing_ctx.request.as_mut().unwrap();
        let cred = &signing_ctx.credentials.as_ref().unwrap();

        let datetime_now = match signing_ctx.sign_time {
            Some(sign_time) => sign_time.into(),
            None => Utc::now(),
        };
        let datetime = datetime_now.format(ISO8601_DATETIME_FORMAT).to_string();
        let date = datetime_now.format(ISO8601_DATE_FORMAT).to_string();
        let expires = match signing_ctx.time {
            Some(time) => time
                .duration_since(datetime_now.into())
                .unwrap_or(Duration::ZERO),
            None => DEFAULT_EXPIRES_DURATION,
        }
        .as_secs();

        // Scope
        let region = signing_ctx.region.as_deref().unwrap_or_default();
        let product = signing_ctx.product.as_deref().unwrap_or_default();
        let scope = self.build_scope(&date, region, product);

        let additional_headers = signing_ctx
            .additional_headers
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();
        let additional_headers =
            get_common_additional_headers(request.headers(), &additional_headers);

        // Credentials information
        let mut query = get_decoded_query_from_str(request.url().query().unwrap_or_default());
        if !cred.security_token.is_empty() {
            query.insert(
                HEADER_OSS_SECURITY_TOKEN.to_lowercase(),
                cred.security_token.clone(),
            );
        }
        query.insert(
            HEADER_OSS_SIGNATURE_VERSION.to_lowercase(),
            ALGORITHM_V4.to_string(),
        );
        query.insert(HEADER_OSS_DATE.to_lowercase(), datetime.clone());
        query.insert(HEADER_OSS_EXPIRES.to_lowercase(), expires.to_string());
        query.insert(
            HEADER_OSS_CREDENTIAL.to_lowercase(),
            format!("{}/{}", cred.access_key_id, scope),
        );
        if !additional_headers.is_empty() {
            query.insert(
                HEADER_OSS_ADDITIONAL_HEADERS.to_lowercase(),
                additional_headers.join(";"),
            );
        }

        set_query(signing_ctx.request.as_mut().unwrap(), &query);

        // CanonicalRequest
        let canonical_request = self.calc_canonical_request(
            signing_ctx,
            &additional_headers
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>(),
        );

        // StringToSign
        let string_to_sign = self.calc_string_to_sign(&datetime, &scope, &canonical_request);
        signing_ctx.string_to_sign.clone_from(&string_to_sign);

        // Signature
        let signature = self.calc_signature(
            &cred.access_key_secret,
            &date,
            region,
            product,
            &string_to_sign,
        )?;

        // Authorization Query
        query.insert(HEADER_OSS_SIGNATURE.to_lowercase(), signature);

        signing_ctx.request.as_mut().unwrap().url_mut().set_query(
            query
                .iter()
                .map(|(k, v)| {
                    if !v.is_empty() {
                        format!("{}={}", k, v)
                    } else {
                        k.to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join("&")
                .as_str()
                .into(),
        );

        Ok(())
    }
}

impl Signer for SignerV4 {
    fn sign(
        &self,
        signing_ctx: &mut SigningContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if signing_ctx
            .credentials
            .as_ref()
            .and_then(|credentials| credentials.has_keys().then_some(()))
            .is_none()
        {
            return Err("SigningContext.Credentials is null or empty.".into());
        }
        if signing_ctx.request.is_none() {
            return Err("SigningContext.Request is null.".into());
        }

        if signing_ctx.auth_method_query {
            self.auth_query(signing_ctx)
        } else {
            self.auth_header(signing_ctx)
        }
    }

    fn is_signed_header(&self, additional_headers: &[String], header: &str) -> bool {
        is_default_signed_header(&header.to_lowercase())
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
    use std::time::{Duration, UNIX_EPOCH};

    use http::Method;
    use reqwest::Request;

    use super::*;
    use crate::credential::{self, Credentials, CredentialsProvider};
    use crate::signer::utils::{get_query, set_query};
    use crate::*;

    async fn get_cred(id: &str, secret: &str, tokens: &[&str]) -> Credentials {
        let credentials_provider = credential::StaticCredentialsProvider::new(id, secret, tokens);
        credentials_provider.get_credentials().await.unwrap()
    }

    #[tokio::test]
    async fn test_v4_auth_header() {
        let cred = get_cred("ak", "sk", &[]).await;

        let mut request = Request::new(
            Method::PUT,
            "http://bucket.oss-cn-hangzhou.aliyuncs.com"
                .parse()
                .unwrap(),
        );
        request
            .headers_mut()
            .insert("x-oss-head1", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("abc", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("ZAbc", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("XYZ", "value".parse().unwrap());
        request
            .headers_mut()
            .insert(HTTP_HEADER_CONTENT_TYPE, "text/plain".parse().unwrap());
        request.headers_mut().insert(
            HEADER_OSS_CONTENT_SHA256,
            "UNSIGNED-PAYLOAD".parse().unwrap(),
        );

        let sign_time = UNIX_EPOCH + Duration::from_secs(1702743657);
        let mut sign_ctx = SigningContext {
            bucket: Some("bucket".into()),
            key: Some("1234+-/123/1.txt".into()),
            request: request.into(),
            credentials: cred.into(),
            product: Some("oss".into()),
            region: Some("cn-hangzhou".into()),
            time: sign_time.into(),
            ..Default::default()
        };

        let mut query = get_query(sign_ctx.request.as_ref().unwrap());
        query.insert("param1".into(), "value1".into());
        query.insert("+param1".into(), "value3".into());
        query.insert("|param1".into(), "value4".into());
        query.insert("+param2".into(), "".into());
        query.insert("|param2".into(), "".into());
        query.insert("param2".into(), "".into());
        set_query(sign_ctx.request.as_mut().unwrap(), &query);

        let _ = SignerV4 {}.sign(&mut sign_ctx);

        let auth_pat = "OSS4-HMAC-SHA256 \
                        Credential=ak/20231216/cn-hangzhou/oss/aliyun_v4_request,\
                        Signature=e21d18daa82167720f9b1047ae7e7f1ce7cb77a31e8203a7d5f4624fa0284afe";

        assert_eq!(
            auth_pat,
            sign_ctx
                .request
                .unwrap()
                .headers()
                .get(HTTP_HEADER_AUTHORIZATION)
                .unwrap()
                .to_str()
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_v4_auth_header_token() {
        let cred = get_cred("ak", "sk", &["token"]).await;

        let mut request = Request::new(
            Method::PUT,
            "http://bucket.oss-cn-hangzhou.aliyuncs.com"
                .parse()
                .unwrap(),
        );
        request
            .headers_mut()
            .insert("x-oss-head1", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("abc", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("ZAbc", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("XYZ", "value".parse().unwrap());
        request
            .headers_mut()
            .insert(HTTP_HEADER_CONTENT_TYPE, "text/plain".parse().unwrap());
        request.headers_mut().insert(
            HEADER_OSS_CONTENT_SHA256,
            "UNSIGNED-PAYLOAD".parse().unwrap(),
        );

        let sign_time = UNIX_EPOCH + Duration::from_secs(1702784856);
        let mut sign_ctx = SigningContext {
            bucket: Some("bucket".into()),
            key: Some("1234+-/123/1.txt".into()),
            request: request.into(),
            credentials: cred.into(),
            product: Some("oss".into()),
            region: Some("cn-hangzhou".into()),
            time: sign_time.into(),
            ..Default::default()
        };

        let mut query = get_query(sign_ctx.request.as_ref().unwrap());
        query.insert("param1".into(), "value1".into());
        query.insert("+param1".into(), "value3".into());
        query.insert("|param1".into(), "value4".into());
        query.insert("+param2".into(), "".into());
        query.insert("|param2".into(), "".into());
        query.insert("param2".into(), "".into());
        set_query(sign_ctx.request.as_mut().unwrap(), &query);

        let _ = SignerV4 {}.sign(&mut sign_ctx);

        let auth_pat = "OSS4-HMAC-SHA256 \
                        Credential=ak/20231217/cn-hangzhou/oss/aliyun_v4_request,\
                        Signature=b94a3f999cf85bcdc00d332fbd3734ba03e48382c36fa4d5af5df817395bd9ea";

        assert_eq!(
            auth_pat,
            sign_ctx
                .request
                .unwrap()
                .headers()
                .get(HTTP_HEADER_AUTHORIZATION)
                .unwrap()
                .to_str()
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_v4_auth_header_with_additional_headers_case_1() {
        let cred = get_cred("ak", "sk", &[]).await;

        let mut request = Request::new(
            Method::PUT,
            "http://bucket.oss-cn-hangzhou.aliyuncs.com"
                .parse()
                .unwrap(),
        );
        request
            .headers_mut()
            .insert("x-oss-head1", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("abc", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("ZAbc", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("XYZ", "value".parse().unwrap());
        request
            .headers_mut()
            .insert(HTTP_HEADER_CONTENT_TYPE, "text/plain".parse().unwrap());
        request.headers_mut().insert(
            HEADER_OSS_CONTENT_SHA256,
            "UNSIGNED-PAYLOAD".parse().unwrap(),
        );

        let sign_time = UNIX_EPOCH + Duration::from_secs(1702747512);
        let mut sign_ctx = SigningContext {
            bucket: Some("bucket".into()),
            key: Some("1234+-/123/1.txt".into()),
            request: request.into(),
            credentials: cred.into(),
            product: Some("oss".into()),
            region: Some("cn-hangzhou".into()),
            time: sign_time.into(),
            additional_headers: vec!["zAbc".into(), "abc".into()],
            ..Default::default()
        };

        let mut query = get_query(sign_ctx.request.as_ref().unwrap());
        query.insert("param1".into(), "value1".into());
        query.insert("+param1".into(), "value3".into());
        query.insert("|param1".into(), "value4".into());
        query.insert("+param2".into(), "".into());
        query.insert("|param2".into(), "".into());
        query.insert("param2".into(), "".into());
        set_query(sign_ctx.request.as_mut().unwrap(), &query);

        let _ = SignerV4 {}.sign(&mut sign_ctx);

        let auth_pat = "OSS4-HMAC-SHA256 \
                        Credential=ak/20231216/cn-hangzhou/oss/aliyun_v4_request,\
                        AdditionalHeaders=abc;zabc,\
                        Signature=4a4183c187c07c8947db7620deb0a6b38d9fbdd34187b6dbaccb316fa251212f";

        assert_eq!(
            auth_pat,
            sign_ctx
                .request
                .unwrap()
                .headers()
                .get(HTTP_HEADER_AUTHORIZATION)
                .unwrap()
                .to_str()
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_v4_auth_header_with_additional_headers_case_2() {
        let cred = get_cred("ak", "sk", &[]).await;

        let mut request = Request::new(
            Method::PUT,
            "http://bucket.oss-cn-hangzhou.aliyuncs.com"
                .parse()
                .unwrap(),
        );
        request
            .headers_mut()
            .insert("x-oss-head1", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("abc", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("ZAbc", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("XYZ", "value".parse().unwrap());
        request
            .headers_mut()
            .insert(HTTP_HEADER_CONTENT_TYPE, "text/plain".parse().unwrap());
        request.headers_mut().insert(
            HEADER_OSS_CONTENT_SHA256,
            "UNSIGNED-PAYLOAD".parse().unwrap(),
        );

        let sign_time = UNIX_EPOCH + Duration::from_secs(1702747512);
        let mut sign_ctx = SigningContext {
            bucket: Some("bucket".into()),
            key: Some("1234+-/123/1.txt".into()),
            request: request.into(),
            credentials: cred.into(),
            product: Some("oss".into()),
            region: Some("cn-hangzhou".into()),
            time: sign_time.into(),
            additional_headers: vec![
                "x-oss-no-exist".into(),
                "zAbc".into(),
                "x-oss-head1".into(),
                "abc".into(),
            ],
            ..Default::default()
        };

        let mut query = get_query(sign_ctx.request.as_ref().unwrap());
        query.insert("param1".into(), "value1".into());
        query.insert("+param1".into(), "value3".into());
        query.insert("|param1".into(), "value4".into());
        query.insert("+param2".into(), "".into());
        query.insert("|param2".into(), "".into());
        query.insert("param2".into(), "".into());
        set_query(sign_ctx.request.as_mut().unwrap(), &query);

        let _ = SignerV4 {}.sign(&mut sign_ctx);

        let auth_pat = "OSS4-HMAC-SHA256 \
                        Credential=ak/20231216/cn-hangzhou/oss/aliyun_v4_request,\
                        AdditionalHeaders=abc;zabc,\
                        Signature=4a4183c187c07c8947db7620deb0a6b38d9fbdd34187b6dbaccb316fa251212f";

        assert_eq!(
            auth_pat,
            sign_ctx
                .request
                .unwrap()
                .headers()
                .get(HTTP_HEADER_AUTHORIZATION)
                .unwrap()
                .to_str()
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_v4_auth_query() {
        let cred = get_cred("ak", "sk", &[]).await;

        let mut request = Request::new(
            Method::PUT,
            "http://bucket.oss-cn-hangzhou.aliyuncs.com"
                .parse()
                .unwrap(),
        );
        request
            .headers_mut()
            .insert("x-oss-head1", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("abc", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("ZAbc", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("XYZ", "value".parse().unwrap());
        request.headers_mut().insert(
            HTTP_HEADER_CONTENT_TYPE,
            "application/octet-stream".parse().unwrap(),
        );

        let time = UNIX_EPOCH + Duration::from_secs(1702782276);
        let sign_time = UNIX_EPOCH + Duration::from_secs(1702781677);
        let mut sign_ctx = SigningContext {
            bucket: Some("bucket".into()),
            key: Some("1234+-/123/1.txt".into()),
            request: request.into(),
            credentials: cred.into(),
            product: Some("oss".into()),
            region: Some("cn-hangzhou".into()),
            auth_method_query: true,
            time: time.into(),
            sign_time: sign_time.into(),
            ..Default::default()
        };

        let mut query = get_query(sign_ctx.request.as_ref().unwrap());
        query.insert("param1".into(), "value1".into());
        query.insert("+param1".into(), "value3".into());
        query.insert("|param1".into(), "value4".into());
        query.insert("+param2".into(), "".into());
        query.insert("|param2".into(), "".into());
        query.insert("param2".into(), "".into());
        set_query(sign_ctx.request.as_mut().unwrap(), &query);

        let _ = SignerV4 {}.sign(&mut sign_ctx);

        let sign_url = sign_ctx.request.as_ref().unwrap().url();
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_SIGNATURE_VERSION.to_lowercase().as_str())
                .unwrap()
                .1,
            ALGORITHM_V4
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_EXPIRES.to_lowercase().as_str())
                .unwrap()
                .1,
            "599"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_CREDENTIAL.to_lowercase().as_str())
                .unwrap()
                .1,
            "ak/20231217/cn-hangzhou/oss/aliyun_v4_request"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_SIGNATURE.to_lowercase().as_str())
                .unwrap()
                .1,
            "a39966c61718be0d5b14e668088b3fa07601033f6518ac7b523100014269c0fe"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_ADDITIONAL_HEADERS.to_lowercase().as_str())
                .unwrap_or_default()
                .1,
            ""
        );
    }

    #[tokio::test]
    async fn test_v4_auth_query_with_additional_headers_case_1() {
        let cred = get_cred("ak", "sk", &[]).await;

        let mut request = Request::new(
            Method::PUT,
            "http://bucket.oss-cn-hangzhou.aliyuncs.com"
                .parse()
                .unwrap(),
        );
        request
            .headers_mut()
            .insert("x-oss-head1", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("abc", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("ZAbc", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("XYZ", "value".parse().unwrap());
        request.headers_mut().insert(
            HTTP_HEADER_CONTENT_TYPE,
            "application/octet-stream".parse().unwrap(),
        );

        let time = UNIX_EPOCH + Duration::from_secs(1702784408);
        let sign_time = UNIX_EPOCH + Duration::from_secs(1702783809);
        let mut sign_ctx = SigningContext {
            bucket: Some("bucket".into()),
            key: Some("1234+-/123/1.txt".into()),
            request: request.into(),
            credentials: cred.into(),
            product: Some("oss".into()),
            region: Some("cn-hangzhou".into()),
            auth_method_query: true,
            time: time.into(),
            sign_time: sign_time.into(),
            additional_headers: vec!["zAbc".into(), "abc".into()],
            ..Default::default()
        };

        let mut query = get_query(sign_ctx.request.as_ref().unwrap());
        query.insert("param1".into(), "value1".into());
        query.insert("+param1".into(), "value3".into());
        query.insert("|param1".into(), "value4".into());
        query.insert("+param2".into(), "".into());
        query.insert("|param2".into(), "".into());
        query.insert("param2".into(), "".into());
        set_query(sign_ctx.request.as_mut().unwrap(), &query);

        let _ = SignerV4 {}.sign(&mut sign_ctx);

        let sign_url = sign_ctx.request.as_ref().unwrap().url();

        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_SIGNATURE_VERSION.to_lowercase().as_str())
                .unwrap()
                .1,
            ALGORITHM_V4
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_DATE.to_lowercase().as_str())
                .unwrap()
                .1,
            "20231217T033009Z"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_EXPIRES.to_lowercase().as_str())
                .unwrap()
                .1,
            "599"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_CREDENTIAL.to_lowercase().as_str())
                .unwrap()
                .1,
            "ak/20231217/cn-hangzhou/oss/aliyun_v4_request"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_SIGNATURE.to_lowercase().as_str())
                .unwrap()
                .1,
            "6bd984bfe531afb6db1f7550983a741b103a8c58e5e14f83ea474c2322dfa2b7"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_ADDITIONAL_HEADERS.to_lowercase().as_str())
                .unwrap_or_default()
                .1,
            "abc;zabc"
        );
    }

    #[tokio::test]
    async fn test_v4_auth_query_with_additional_headers_case_2() {
        let cred = get_cred("ak", "sk", &[]).await;

        let mut request = Request::new(
            Method::PUT,
            "http://bucket.oss-cn-hangzhou.aliyuncs.com"
                .parse()
                .unwrap(),
        );
        request
            .headers_mut()
            .insert("x-oss-head1", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("abc", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("ZAbc", "value".parse().unwrap());
        request
            .headers_mut()
            .insert("XYZ", "value".parse().unwrap());
        request.headers_mut().insert(
            HTTP_HEADER_CONTENT_TYPE,
            "application/octet-stream".parse().unwrap(),
        );

        let time = UNIX_EPOCH + Duration::from_secs(1702784408);
        let sign_time = UNIX_EPOCH + Duration::from_secs(1702783809);
        let mut sign_ctx = SigningContext {
            bucket: Some("bucket".into()),
            key: Some("1234+-/123/1.txt".into()),
            request: request.into(),
            credentials: cred.into(),
            product: Some("oss".into()),
            region: Some("cn-hangzhou".into()),
            auth_method_query: true,
            time: time.into(),
            sign_time: sign_time.into(),
            additional_headers: vec![
                "x-oss-no-exist".into(),
                "abc".into(),
                "x-oss-head1".into(),
                "zAbc".into(),
            ],
            ..Default::default()
        };

        let mut query = get_query(sign_ctx.request.as_ref().unwrap());
        query.insert("param1".into(), "value1".into());
        query.insert("+param1".into(), "value3".into());
        query.insert("|param1".into(), "value4".into());
        query.insert("+param2".into(), "".into());
        query.insert("|param2".into(), "".into());
        query.insert("param2".into(), "".into());
        set_query(sign_ctx.request.as_mut().unwrap(), &query);

        let _ = SignerV4 {}.sign(&mut sign_ctx);

        let sign_url = sign_ctx.request.as_ref().unwrap().url();
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_SIGNATURE_VERSION.to_lowercase().as_str())
                .unwrap()
                .1,
            ALGORITHM_V4
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_DATE.to_lowercase().as_str())
                .unwrap()
                .1,
            "20231217T033009Z"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_EXPIRES.to_lowercase().as_str())
                .unwrap()
                .1,
            "599"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_CREDENTIAL.to_lowercase().as_str())
                .unwrap()
                .1,
            "ak/20231217/cn-hangzhou/oss/aliyun_v4_request"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_SIGNATURE.to_lowercase().as_str())
                .unwrap()
                .1,
            "6bd984bfe531afb6db1f7550983a741b103a8c58e5e14f83ea474c2322dfa2b7"
        );
        assert_eq!(
            sign_url
                .query_pairs()
                .find(|(k, _)| k == HEADER_OSS_ADDITIONAL_HEADERS.to_lowercase().as_str())
                .unwrap_or_default()
                .1,
            "abc;zabc"
        );
    }
}
