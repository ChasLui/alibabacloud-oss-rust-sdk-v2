//! Regression tests pinning wire-level behavior aligned with the Go SDK:
//! 1. `CreateBucket` sends its configuration as an `application/xml` body
//!    (`<CreateBucketConfiguration>`) — it used to send a header-only
//!    `x-oss-storage-class` with a hardcoded empty body.
//! 2. Default (zero-valued) non-`Option` fields never leak empty headers,
//!    mirroring Go's `isEmptyValue` skip in `marshalInput`.
//!
//! The first test fails on the pre-fix tree; the second pins the macro guard.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use alibabacloud_oss_sdk_rust_v2::api::bucket::{
    CreateBucketConfiguration, CreateBucketRequest,
};
use alibabacloud_oss_sdk_rust_v2::client::Client;
use alibabacloud_oss_sdk_rust_v2::config::Config;
use alibabacloud_oss_sdk_rust_v2::credential::providers::StaticCredentialsProvider;
use alibabacloud_oss_sdk_rust_v2::SignatureVersionType;

/// Serve one request and return the raw bytes (headers + body).
fn capture_wire(request: CreateBucketRequest) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
            let mut acc = Vec::new();
            let mut buf = [0u8; 8192];
            loop {
                match stream.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        acc.extend_from_slice(&buf[..n]);
                        if acc.windows(4).any(|w| w == b"\r\n\r\n") {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
            tx.send(String::from_utf8_lossy(&acc).to_string()).ok();
        }
    });

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        let config = Config::default()
            .with_region("cn-hangzhou")
            .with_endpoint(&format!("http://{}", addr))
            .with_use_path_style(true)
            .with_credentials_provider(Rc::new(StaticCredentialsProvider::new("ak", "sk", &[])))
            .with_signature_version(SignatureVersionType::V4);
        let client = Client::new(&config);
        let _ = client.create_bucket(&request).await;
        rx.recv_timeout(Duration::from_secs(10))
            .unwrap_or_default()
    })
}

#[test]
fn create_bucket_sends_config_as_xml_body_not_header() {
    let observed = capture_wire(CreateBucketRequest {
        bucket: "probe-bucket".to_string(),
        create_bucket_configuration: Some(CreateBucketConfiguration {
            storage_class: Some("Archive".to_string()),
            data_redundancy_type: Some("LRS".to_string()),
        }),
        ..Default::default()
    });

    assert!(
        observed.contains("application/xml"),
        "content-type must be application/xml:\n{}",
        observed
    );
    assert!(
        !observed.to_lowercase().contains("x-oss-storage-class:"),
        "storage class must not be sent as a header:\n{}",
        observed
    );
    assert!(
        observed.contains("<StorageClass>Archive</StorageClass>"),
        "storage class must be in the XML body:\n{}",
        observed
    );
    assert!(
        observed.contains("<DataRedundancyType>LRS</DataRedundancyType>"),
        "data redundancy type must be in the XML body:\n{}",
        observed
    );
}

#[test]
fn create_bucket_without_config_sends_no_body() {
    let observed = capture_wire(CreateBucketRequest {
        bucket: "probe-bucket".to_string(),
        ..Default::default()
    });
    let body = observed
        .split_once("\r\n\r\n")
        .map(|(_, b)| b.trim())
        .unwrap_or("");
    assert!(body.is_empty(), "expected empty body, got: [{}]", body);
    assert!(
        !observed.to_lowercase().contains("content-length:"),
        "no body configured means no content-length header:\n{}",
        observed
    );
}