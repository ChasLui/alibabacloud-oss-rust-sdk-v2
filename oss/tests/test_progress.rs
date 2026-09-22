//! The progress callback declared on `PutObjectRequest`, `UploadPartRequest`
//! and `GetObjectRequest` was never invoked: a caller could set one and never
//! hear from it. These tests drive real requests through a local server to
//! prove the callbacks fire, and that the numbers they report are the bytes
//! that actually moved.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use alibabacloud_oss_sdk_rust_v2::api::object::{GetObjectRequest, PutObjectRequest};
use alibabacloud_oss_sdk_rust_v2::client::{BodyDataReader, Client};
use alibabacloud_oss_sdk_rust_v2::config::Config;
use alibabacloud_oss_sdk_rust_v2::credential::providers::StaticCredentialsProvider;
use alibabacloud_oss_sdk_rust_v2::{BodyContent, SignatureVersionType};

/// The `(transferred, total)` pairs the callback was given, in order.
type Progress = Arc<Mutex<Vec<(i64, i64)>>>;

fn recorder() -> (Box<dyn Fn(i64, i64) + Send + Sync>, Progress) {
    let seen: Progress = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    let callback = Box::new(move |transferred: i64, total: i64| {
        sink.lock().expect("lock").push((transferred, total));
    });
    (callback, seen)
}

fn client_for(addr: std::net::SocketAddr) -> Client {
    let config = Config::default()
        .with_region("cn-hangzhou")
        .with_endpoint(&format!("http://{}", addr))
        .with_use_path_style(true)
        .with_credentials_provider(Rc::new(StaticCredentialsProvider::new("ak", "sk", &[])))
        .with_signature_version(SignatureVersionType::V4)
        // A paced or slow transfer must not hit the default request timeout.
        .with_read_write_timeout(Duration::from_secs(60));
    Client::new(&config)
}

/// Reads one request off `stream`, including its body.
fn read_request(stream: &mut std::net::TcpStream) -> Vec<u8> {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout");
    let mut acc = Vec::new();
    let mut buf = [0u8; 8192];
    // Headers first.
    loop {
        match stream.read(&mut buf) {
            Ok(0) => return acc,
            Ok(n) => {
                acc.extend_from_slice(&buf[..n]);
                if acc.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            Err(_) => return acc,
        }
    }
    let header_end = acc
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("header terminator")
        + 4;
    let headers = String::from_utf8_lossy(&acc[..header_end]).to_lowercase();
    let content_length: usize = headers
        .lines()
        .find_map(|line| line.strip_prefix("content-length:"))
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(0);
    // Then the body.
    while acc.len() < header_end + content_length {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => acc.extend_from_slice(&buf[..n]),
            Err(_) => break,
        }
    }
    acc
}

#[test]
fn put_object_reports_upload_progress() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let received = read_request(&mut stream);
        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
        received
    });

    let (callback, seen) = recorder();
    const SIZE: usize = 64 * 1024;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    rt.block_on(async {
        let client = client_for(addr);
        client
            .put_object(PutObjectRequest {
                bucket: "probe-bucket".to_string(),
                key: "probe-object".to_string(),
                body: Some(BodyContent::from_bytes(vec![7u8; SIZE], None)),
                progress_fn: Some(callback),
                ..Default::default()
            })
            .await
            .expect("put_object");
    });

    let received = server.join().expect("server");
    assert!(
        received.len() > SIZE,
        "the server must have received the body, got {} bytes",
        received.len()
    );

    let seen = seen.lock().expect("lock");
    assert!(
        !seen.is_empty(),
        "the progress callback must be invoked at least once"
    );
    assert_eq!(
        seen.last().expect("a last report").0,
        SIZE as i64,
        "the final report must account for every byte sent"
    );
    assert!(
        seen.iter().all(|(_, total)| *total == SIZE as i64),
        "every report must carry the body's size as its total: {:?}",
        seen
    );
    assert!(
        seen.windows(2).all(|pair| pair[1].0 >= pair[0].0),
        "transferred bytes must never go backwards: {:?}",
        seen
    );
}

#[test]
fn get_object_reports_download_progress() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    const SIZE: usize = 32 * 1024;
    let body = vec![9u8; SIZE];
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let _ = read_request(&mut stream);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nETag: \"probe\"\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.write_all(&body);
    });

    let (callback, seen) = recorder();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    rt.block_on(async {
        let client = client_for(addr);
        let mut result = client
            .get_object(GetObjectRequest {
                bucket: "probe-bucket".to_string(),
                key: "probe-object".to_string(),
                progress_fn: Some(callback),
                ..Default::default()
            })
            .await
            .expect("get_object");
        // The callback rides the body stream, so the stream has to be drained.
        let data = result.get_all_data().await.expect("body");
        assert_eq!(data.len(), SIZE, "the whole body must arrive");
    });

    server.join().expect("server");

    let seen = seen.lock().expect("lock");
    assert!(
        !seen.is_empty(),
        "the progress callback must be invoked at least once"
    );
    assert_eq!(
        seen.last().expect("a last report").0,
        SIZE as i64,
        "the final report must account for every byte received"
    );
    assert!(
        seen.iter().all(|(_, total)| *total == SIZE as i64),
        "the response's Content-Length must be reported as the total: {:?}",
        seen
    );
}
