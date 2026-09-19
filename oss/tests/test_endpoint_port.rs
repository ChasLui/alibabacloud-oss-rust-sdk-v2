
use std::io::{Read, Write};
use std::net::TcpListener;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use alibabacloud_oss_sdk_rust_v2::api::object::GetObjectRequest;
use alibabacloud_oss_sdk_rust_v2::client::Client;
use alibabacloud_oss_sdk_rust_v2::config::Config;
use alibabacloud_oss_sdk_rust_v2::credential::providers::StaticCredentialsProvider;
use alibabacloud_oss_sdk_rust_v2::SignatureVersionType;

/// Regression: a non-default endpoint port must survive build_url (local
/// mocks / MinIO); Go keeps Endpoint.Host with its port.
#[test]
fn endpoint_port_is_preserved_in_host() {
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
                    Ok(n) => { acc.extend_from_slice(&buf[..n]); if acc.windows(4).any(|w| w == b"\r\n\r\n") { break; } }
                    Err(_) => break,
                }
            }
            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nhi");
            tx.send(String::from_utf8_lossy(&acc).to_string()).ok();
        }
    });

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let (err, observed) = rt.block_on(async move {
        // Path style keeps the port in the Host header (virtual-hosted would
        // prefix the bucket, which cannot carry a port).
        let config = Config::default()
            .with_region("cn-hangzhou")
            .with_endpoint(&format!("http://{}", addr))
            .with_use_path_style(true)
            .with_credentials_provider(Rc::new(StaticCredentialsProvider::new("ak", "sk", &[])))
            .with_signature_version(SignatureVersionType::V1);
        let client = Client::new(&config);
        let res = client.get_object(GetObjectRequest {
            bucket: "probe-bucket".to_string(),
            key: "probe-key".to_string(),
            ..Default::default()
        }).await;
        let err = match res {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("ERR: {}", e),
        };
        let observed = rx.recv_timeout(Duration::from_secs(8)).unwrap_or_default();
        (err, observed)
    });

    // Before the fix `host_str()` dropped the port and the request went to
    // port 80, so nothing ever reached the listener.
    assert_eq!(err, "OK", "request must reach the endpoint: {}", observed);
    assert!(
        observed.contains(&format!("host: {}", addr)),
        "Host header must keep the endpoint port; got:\n{}",
        observed
    );
}
