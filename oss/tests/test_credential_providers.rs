// Integration tests for credential providers

use alibabacloud_oss_sdk_rust_v2::credential::{
    Credentials, CredentialsProvider, StaticCredentialsProvider, AnonymousCredentialsProvider,
};
use std::time::{SystemTime, Duration};

#[tokio::test]
async fn test_static_credentials_provider_with_token() {
    let access_key_id = "test_access_key_id";
    let access_key_secret = "test_access_key_secret";
    let security_token = "test_security_token";
    
    let provider = StaticCredentialsProvider::new(
        access_key_id,
        access_key_secret,
        &[security_token],
    );
    
    let credentials = provider.get_credentials().await.unwrap();
    
    assert_eq!(credentials.access_key_id, access_key_id);
    assert_eq!(credentials.access_key_secret, access_key_secret);
    assert_eq!(credentials.security_token, security_token);
    assert!(credentials.expires.is_none());
}

#[tokio::test]
async fn test_static_credentials_provider_without_token() {
    let access_key_id = "test_access_key_id";
    let access_key_secret = "test_access_key_secret";
    
    let provider = StaticCredentialsProvider::new(
        access_key_id,
        access_key_secret,
        &[],
    );
    
    let credentials = provider.get_credentials().await.unwrap();
    
    assert_eq!(credentials.access_key_id, access_key_id);
    assert_eq!(credentials.access_key_secret, access_key_secret);
    assert_eq!(credentials.security_token, "");
    assert!(credentials.expires.is_none());
}

#[tokio::test]
async fn test_static_credentials_provider_multiple_tokens() {
    let access_key_id = "test_access_key_id";
    let access_key_secret = "test_access_key_secret";
    let tokens = vec!["token1", "token2", "token3"];
    
    let provider = StaticCredentialsProvider::new(
        access_key_id,
        access_key_secret,
        &tokens,
    );
    
    let credentials = provider.get_credentials().await.unwrap();
    
    // Should use the first token
    assert_eq!(credentials.security_token, "token1");
}

#[tokio::test]
async fn test_anonymous_credentials_provider() {
    let provider = AnonymousCredentialsProvider::new();
    
    let credentials = provider.get_credentials().await.unwrap();
    
    assert_eq!(credentials.access_key_id, "");
    assert_eq!(credentials.access_key_secret, "");
    assert_eq!(credentials.security_token, "");
    assert!(credentials.expires.is_none());
}

#[tokio::test]
async fn test_credentials_clone() {
    let original = Credentials {
        access_key_id: "ak".to_string(),
        access_key_secret: "sk".to_string(),
        security_token: "token".to_string(),
        expires: Some(SystemTime::now() + Duration::from_secs(3600)),
    };
    
    let cloned = original.clone();
    
    assert_eq!(original.access_key_id, cloned.access_key_id);
    assert_eq!(original.access_key_secret, cloned.access_key_secret);
    assert_eq!(original.security_token, cloned.security_token);
    assert_eq!(original.expires, cloned.expires);
}

#[tokio::test]
async fn test_credentials_default() {
    let credentials = Credentials::default();
    
    assert_eq!(credentials.access_key_id, "");
    assert_eq!(credentials.access_key_secret, "");
    assert_eq!(credentials.security_token, "");
    assert!(credentials.expires.is_none());
    assert!(!credentials.has_keys());
    assert!(!credentials.expired());
}

#[tokio::test]
async fn test_credentials_has_keys_variations() {
    // Both keys present
    let creds1 = Credentials {
        access_key_id: "ak".to_string(),
        access_key_secret: "sk".to_string(),
        ..Default::default()
    };
    assert!(creds1.has_keys());
    
    // Only access key id
    let creds2 = Credentials {
        access_key_id: "ak".to_string(),
        access_key_secret: "".to_string(),
        ..Default::default()
    };
    assert!(!creds2.has_keys());
    
    // Only access key secret
    let creds3 = Credentials {
        access_key_id: "".to_string(),
        access_key_secret: "sk".to_string(),
        ..Default::default()
    };
    assert!(!creds3.has_keys());
    
    // Neither key
    let creds4 = Credentials::default();
    assert!(!creds4.has_keys());
}

#[tokio::test]
async fn test_credentials_expired_edge_cases() {
    // Exactly now - should be expired
    let creds1 = Credentials {
        expires: Some(SystemTime::now()),
        ..Default::default()
    };
    assert!(creds1.expired());
    
    // 1 second in the future - not expired
    let creds2 = Credentials {
        expires: Some(SystemTime::now() + Duration::from_secs(1)),
        ..Default::default()
    };
    assert!(!creds2.expired());
    
    // 1 second in the past - expired
    let creds3 = Credentials {
        expires: Some(SystemTime::now() - Duration::from_secs(1)),
        ..Default::default()
    };
    assert!(creds3.expired());
}

#[tokio::test]
async fn test_credentials_serialization_deserialization() {
    let original = Credentials {
        access_key_id: "test_ak".to_string(),
        access_key_secret: "test_sk".to_string(),
        security_token: "test_token".to_string(),
        expires: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1702784856)),
    };
    
    // Serialize to JSON
    let json = serde_json::to_string(&original).unwrap();
    
    // Deserialize from JSON
    let deserialized: Credentials = serde_json::from_str(&json).unwrap();
    
    assert_eq!(original.access_key_id, deserialized.access_key_id);
    assert_eq!(original.access_key_secret, deserialized.access_key_secret);
    assert_eq!(original.security_token, deserialized.security_token);
    assert_eq!(original.expires, deserialized.expires);
}

#[tokio::test]
async fn test_credentials_partial_eq() {
    let creds1 = Credentials {
        access_key_id: "ak".to_string(),
        access_key_secret: "sk".to_string(),
        security_token: "token".to_string(),
        expires: None,
    };
    
    let creds2 = Credentials {
        access_key_id: "ak".to_string(),
        access_key_secret: "sk".to_string(),
        security_token: "token".to_string(),
        expires: None,
    };
    
    let creds3 = Credentials {
        access_key_id: "different_ak".to_string(),
        access_key_secret: "sk".to_string(),
        security_token: "token".to_string(),
        expires: None,
    };
    
    assert_eq!(creds1, creds2);
    assert_ne!(creds1, creds3);
}
