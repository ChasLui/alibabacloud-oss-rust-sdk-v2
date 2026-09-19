use std::io::{Read, Write};
use std::net::TcpListener;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use alibabacloud_oss_sdk_rust_v2::api::object::GetObjectMetaRequest;
use alibabacloud_oss_sdk_rust_v2::client::Client;
use alibabacloud_oss_sdk_rust_v2::config::Config;
use alibabacloud_oss_sdk_rust_v2::credential::providers::StaticCredentialsProvider;
use alibabacloud_oss_sdk_rust_v2::SignatureVersionType;

/// Regression: GetObjectMeta must be sent as HEAD (the authoritative Go SDK and
/// the OSS API use HEAD with ?objectMeta); it used to be sent as GET.
#[test]
fn get_object_meta_uses_head() {
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
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nETag: \"abc\"\r\n\r\n",
            );
            tx.send(String::from_utf8_lossy(&acc).to_string()).ok();
        }
    });

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let (err, observed) = rt.block_on(async move {
        let config = Config::default()
            .with_region("cn-hangzhou")
            .with_endpoint(&format!("http://{}", addr))
            .with_use_path_style(true)
            .with_credentials_provider(Rc::new(StaticCredentialsProvider::new("ak", "sk", &[])))
            .with_signature_version(SignatureVersionType::V1);
        let client = Client::new(&config);
        let res = client
            .get_object_meta(&GetObjectMetaRequest {
                bucket: "probe-bucket".to_string(),
                key: "probe-key".to_string(),
                ..Default::default()
            })
            .await;
        let err = match res {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("ERR: {}", e),
        };
        let observed = rx.recv_timeout(Duration::from_secs(8)).unwrap_or_default();
        (err, observed)
    });

    assert_eq!(err, "OK", "request must succeed: {}", observed);
    assert!(
        observed.starts_with("HEAD /probe-bucket/probe-key?objectMeta HTTP/1.1"),
        "must be HEAD with ?objectMeta; got:\n{}",
        observed
    );
}