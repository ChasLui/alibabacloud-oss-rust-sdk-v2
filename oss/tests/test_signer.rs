// Integration tests for signing functionality

use alibabacloud_oss_sdk_rust_v2::signer::{Signer, SigningContext, SignerV4};
use alibabacloud_oss_sdk_rust_v2::credential::{Credentials, CredentialsProvider, StaticCredentialsProvider};
use reqwest::Request;
use http::{Method, StatusCode};
use std::time::{Duration, UNIX_EPOCH};

#[tokio::test]
async fn test_signer_v4_basic_auth_header() {
    let credentials_provider = StaticCredentialsProvider::new("ak", "sk", &[]);
    let credentials = credentials_provider.get_credentials().await.unwrap();
    
    let request = Request::new(
        Method::GET,
        "https://bucket.oss-cn-hangzhou.aliyuncs.com/object.txt"
            .parse()
            .unwrap(),
    );
    
    let mut sign_ctx = SigningContext {
        bucket: Some("bucket".to_string()),
        key: Some("object.txt".to_string()),
        request: Some(request),
        credentials: Some(credentials),
        region: Some("cn-hangzhou".to_string()),
        product: Some("oss".to_string()),
        time: Some(UNIX_EPOCH + Duration::from_secs(1702784856)),
        ..Default::default()
    };
    
    let signer = SignerV4;
    let result = signer.sign(&mut sign_ctx);
    
    assert!(result.is_ok());
    
    // Check that authorization header was added
    let auth_header = sign_ctx.request.as_ref().unwrap()
        .headers()
        .get("Authorization");
    assert!(auth_header.is_some());
    
    let auth_value = auth_header.unwrap().to_str().unwrap();
    assert!(auth_value.starts_with("OSS4-HMAC-SHA256"));
}

#[tokio::test]
async fn test_signer_v4_with_security_token() {
    let credentials_provider = StaticCredentialsProvider::new("ak", "sk", &["token"]);
    let credentials = credentials_provider.get_credentials().await.unwrap();
    
    let request = Request::new(
        Method::PUT,
        "https://bucket.oss-cn-beijing.aliyuncs.com/test.txt"
            .parse()
            .unwrap(),
    );
    
    let mut sign_ctx = SigningContext {
        bucket: Some("bucket".to_string()),
        key: Some("test.txt".to_string()),
        request: Some(request),
        credentials: Some(credentials),
        region: Some("cn-beijing".to_string()),
        product: Some("oss".to_string()),
        time: Some(UNIX_EPOCH + Duration::from_secs(1702784856)),
        ..Default::default()
    };
    
    let signer = SignerV4;
    let result = signer.sign(&mut sign_ctx);
    
    assert!(result.is_ok());
    
    // Check security token header
    let token_header = sign_ctx.request.as_ref().unwrap()
        .headers()
        .get("x-oss-security-token");
    assert!(token_header.is_some());
    assert_eq!(token_header.unwrap().to_str().unwrap(), "token");
}

#[tokio::test]
async fn test_signer_v4_missing_credentials() {
    let request = Request::new(
        Method::GET,
        "https://bucket.oss-cn-hangzhou.aliyuncs.com/"
            .parse()
            .unwrap(),
    );
    
    let mut sign_ctx = SigningContext {
        bucket: Some("bucket".to_string()),
        request: Some(request),
        credentials: None,
        ..Default::default()
    };
    
    let signer = SignerV4;
    let result = signer.sign(&mut sign_ctx);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Credentials"));
}

#[tokio::test]
async fn test_signer_v4_missing_request() {
    let credentials_provider = StaticCredentialsProvider::new("ak", "sk", &[]);
    let credentials = credentials_provider.get_credentials().await.unwrap();
    
    let mut sign_ctx = SigningContext {
        bucket: Some("bucket".to_string()),
        request: None,
        credentials: Some(credentials),
        ..Default::default()
    };
    
    let signer = SignerV4;
    let result = signer.sign(&mut sign_ctx);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Request"));
}

#[tokio::test]
async fn test_signer_v4_query_string_authentication() {
    let credentials_provider = StaticCredentialsProvider::new("ak", "sk", &[]);
    let credentials = credentials_provider.get_credentials().await.unwrap();
    
    let request = Request::new(
        Method::GET,
        "https://bucket.oss-cn-hangzhou.aliyuncs.com/object.txt?param1=value1"
            .parse()
            .unwrap(),
    );
    
    let sign_time = UNIX_EPOCH + Duration::from_secs(1702781677);
    let expire_time = UNIX_EPOCH + Duration::from_secs(1702782276);
    
    let mut sign_ctx = SigningContext {
        bucket: Some("bucket".to_string()),
        key: Some("object.txt".to_string()),
        request: Some(request),
        credentials: Some(credentials),
        region: Some("cn-hangzhou".to_string()),
        product: Some("oss".to_string()),
        auth_method_query: true,
        time: Some(expire_time),
        sign_time: Some(sign_time),
        ..Default::default()
    };
    
    let signer = SignerV4;
    let result = signer.sign(&mut sign_ctx);
    
    assert!(result.is_ok());
    
    // Check that signature is in query string
    let url = sign_ctx.request.as_ref().unwrap().url();
    let query_pairs: Vec<_> = url.query_pairs().collect();
    
    let has_signature = query_pairs.iter().any(|(k, _)| k == "x-oss-signature");
    let has_credential = query_pairs.iter().any(|(k, _)| k == "x-oss-credential");
    let has_expires = query_pairs.iter().any(|(k, _)| k == "x-oss-expires");
    
    assert!(has_signature);
    assert!(has_credential);
    assert!(has_expires);
}

#[tokio::test]
async fn test_signer_v4_with_additional_headers() {
    let credentials_provider = StaticCredentialsProvider::new("ak", "sk", &[]);
    let credentials = credentials_provider.get_credentials().await.unwrap();
    
    let mut request = Request::new(
        Method::PUT,
        "https://bucket.oss-cn-hangzhou.aliyuncs.com/test.txt"
            .parse()
            .unwrap(),
    );
    request
        .headers_mut()
        .insert("x-custom-header", "custom-value".parse().unwrap());
    
    let mut sign_ctx = SigningContext {
        bucket: Some("bucket".to_string()),
        key: Some("test.txt".to_string()),
        request: Some(request),
        credentials: Some(credentials),
        region: Some("cn-hangzhou".to_string()),
        product: Some("oss".to_string()),
        time: Some(UNIX_EPOCH + Duration::from_secs(1702784856)),
        additional_headers: vec!["x-custom-header".to_string()],
        ..Default::default()
    };
    
    let signer = SignerV4;
    let result = signer.sign(&mut sign_ctx);
    
    assert!(result.is_ok());
    
    // Authorization should include AdditionalHeaders
    let auth_header = sign_ctx.request.as_ref().unwrap()
        .headers()
        .get("Authorization")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(auth_header.contains("AdditionalHeaders=x-custom-header"));
}

#[tokio::test]
async fn test_signer_v4_empty_credentials() {
    let credentials = Credentials::default();
    
    let request = Request::new(
        Method::GET,
        "https://bucket.oss-cn-hangzhou.aliyuncs.com/"
            .parse()
            .unwrap(),
    );
    
    let mut sign_ctx = SigningContext {
        bucket: Some("bucket".to_string()),
        request: Some(request),
        credentials: Some(credentials),
        ..Default::default()
    };
    
    let signer = SignerV4;
    let result = signer.sign(&mut sign_ctx);
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_signing_context_default() {
    let ctx = SigningContext::default();
    
    assert!(ctx.bucket.is_none());
    assert!(ctx.key.is_none());
    assert!(ctx.request.is_none());
    assert!(ctx.credentials.is_none());
    assert!(ctx.region.is_none());
    assert!(ctx.product.is_none());
    assert!(ctx.time.is_none());
    assert!(ctx.sign_time.is_none());
    assert!(!ctx.auth_method_query);
    assert!(ctx.additional_headers.is_empty());
    assert_eq!(ctx.string_to_sign, "");
}

#[tokio::test]
async fn test_signer_v4_different_regions() {
    let regions = vec![
        "cn-hangzhou",
        "cn-beijing",
        "cn-shanghai",
        "cn-shenzhen",
        "us-west-1",
    ];
    
    for region in regions {
        let credentials_provider = StaticCredentialsProvider::new("ak", "sk", &[]);
        let credentials = credentials_provider.get_credentials().await.unwrap();
        
        let request = Request::new(
            Method::GET,
            format!("https://bucket.oss-{}.aliyuncs.com/", region)
                .parse()
                .unwrap(),
        );
        
        let mut sign_ctx = SigningContext {
            bucket: Some("bucket".to_string()),
            request: Some(request),
            credentials: Some(credentials),
            region: Some(region.to_string()),
            product: Some("oss".to_string()),
            time: Some(UNIX_EPOCH + Duration::from_secs(1702784856)),
            ..Default::default()
        };
        
        let signer = SignerV4;
        let result = signer.sign(&mut sign_ctx);
        
        assert!(result.is_ok(), "Failed to sign for region: {}", region);
    }
}

#[tokio::test]
async fn test_signer_v4_special_characters_in_key() {
    let credentials_provider = StaticCredentialsProvider::new("ak", "sk", &[]);
    let credentials = credentials_provider.get_credentials().await.unwrap();
    
    let special_keys = vec![
        "file with spaces.txt",
        "file+with+plus.txt",
        "file/with/slashes.txt",
        "file%with%percent.txt",
    ];
    
    for key in special_keys {
        let request = Request::new(
            Method::GET,
            format!("https://bucket.oss-cn-hangzhou.aliyuncs.com/{}", key)
                .parse()
                .unwrap(),
        );
        
        let mut sign_ctx = SigningContext {
            bucket: Some("bucket".to_string()),
            key: Some(key.to_string()),
            request: Some(request),
            credentials: Some(credentials.clone()),
            region: Some("cn-hangzhou".to_string()),
            product: Some("oss".to_string()),
            time: Some(UNIX_EPOCH + Duration::from_secs(1702784856)),
            ..Default::default()
        };
        
        let signer = SignerV4;
        let result = signer.sign(&mut sign_ctx);
        
        assert!(result.is_ok(), "Failed to sign for key: {}", key);
    }
}
