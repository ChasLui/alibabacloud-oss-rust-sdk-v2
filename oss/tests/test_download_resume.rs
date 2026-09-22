//! `get_object_to_file_v2` reconnects where a download stopped instead of
//! restarting it. These tests drive a server that drops the connection
//! part-way through, which is the only way to prove the reconnect actually
//! re-requests the missing range rather than the whole object.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use alibabacloud_oss_sdk_rust_v2::api::object::GetObjectRequest;
use alibabacloud_oss_sdk_rust_v2::client::Client;
use alibabacloud_oss_sdk_rust_v2::config::Config;
use alibabacloud_oss_sdk_rust_v2::credential::providers::StaticCredentialsProvider;
use alibabacloud_oss_sdk_rust_v2::SignatureVersionType;

fn client_for(addr: SocketAddr) -> Client {
    let config = Config::default()
        .with_region("cn-hangzhou")
        .with_endpoint(&format!("http://{}", addr))
        .with_use_path_style(true)
        .with_credentials_provider(Rc::new(StaticCredentialsProvider::new("ak", "sk", &[])))
        .with_signature_version(SignatureVersionType::V4)
        .with_read_write_timeout(Duration::from_secs(30));
    Client::new(&config)
}

/// Reads a request's head and returns it as text.
fn read_head(stream: &mut TcpStream) -> String {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout");
    let mut acc = Vec::new();
    let mut buf = [0u8; 4096];
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
    String::from_utf8_lossy(&acc).to_string()
}

/// Accepts one connection, giving up after `timeout`.
fn accept_within(listener: &TcpListener, timeout: Duration) -> Option<TcpStream> {
    listener.set_nonblocking(true).expect("nonblocking");
    let deadline = Instant::now() + timeout;
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(false).expect("blocking");
                return Some(stream);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return None;
                }
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => return None,
        }
    }
}

/// The `bytes=<start>-` a request asks for, if any.
fn requested_start(head: &str) -> Option<usize> {
    head.lines()
        .find(|line| line.to_lowercase().starts_with("range:"))
        .and_then(|line| line.split_once('='))
        .and_then(|(_, value)| value.split_once('-'))
        .and_then(|(start, _)| start.trim().parse().ok())
}

/// Serves `body` over two connections: the first sends `split` bytes and then
/// drops, the second answers the range request that follows. The returned
/// vector is the head of every request the server saw.
fn serve_with_interruption(
    body: Vec<u8>,
    split: usize,
    etags: Vec<String>,
) -> (SocketAddr, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let handle = thread::spawn(move || {
        let mut heads = Vec::new();
        for (attempt, etag) in etags.iter().enumerate() {
            // A client that never reconnects would otherwise park this thread
            // on `accept` forever, turning a failing assertion into a hung
            // test run.
            let Some(mut stream) = accept_within(&listener, Duration::from_secs(20)) else {
                break;
            };
            let head = read_head(&mut stream);
            let start = requested_start(&head).unwrap_or(0);
            heads.push(head);

            let remaining = &body[start.min(body.len())..];
            let header = if start == 0 {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nETag: {}\r\n\r\n",
                    remaining.len(),
                    etag
                )
            } else {
                format!(
                    "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes \
                     {}-{}/{}\r\nETag: {}\r\n\r\n",
                    remaining.len(),
                    start,
                    body.len() - 1,
                    body.len(),
                    etag
                )
            };
            let _ = stream.write_all(header.as_bytes());
            if attempt == 0 {
                // Declare the full length, then send less and hang up: the
                // client must see a truncated body, not a complete one.
                let _ = stream.write_all(&remaining[..split.min(remaining.len())]);
            } else {
                let _ = stream.write_all(remaining);
            }
            let _ = stream.flush();
            drop(stream);
        }
        heads
    });
    (addr, handle)
}

fn temp_path(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("oss-v2-download-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir.join(name)
}

#[test]
fn a_dropped_connection_resumes_from_the_byte_it_stopped_at() {
    const SIZE: usize = 48 * 1024;
    let body: Vec<u8> = (0..SIZE).map(|i| (i % 251) as u8).collect();
    let split = SIZE / 3;
    let (addr, server) = serve_with_interruption(
        body.clone(),
        split,
        vec!["\"v1\"".to_string(), "\"v1\"".to_string()],
    );

    let path = temp_path("resumed.bin");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    rt.block_on(async {
        client_for(addr)
            .get_object_to_file_v2(
                GetObjectRequest {
                    bucket: "probe-bucket".to_string(),
                    key: "probe-object".to_string(),
                    ..Default::default()
                },
                &path,
                None,
            )
            .await
            .expect("download");
    });

    let heads = server.join().expect("server");
    assert_eq!(heads.len(), 2, "the download must reconnect exactly once");

    // The whole-object request carries no Range, so the server's 200 is what
    // the first attempt sees.
    assert!(
        requested_start(&heads[0]).is_none(),
        "the first request must not be ranged:\n{}",
        heads[0]
    );
    // The reconnect asks only for what is missing.
    assert_eq!(
        requested_start(&heads[1]),
        Some(split),
        "the reconnect must resume at the byte it stopped at:\n{}",
        heads[1]
    );

    let written = std::fs::read(&path).expect("read back");
    assert_eq!(
        written.len(),
        SIZE,
        "the resumed file must be the whole object"
    );
    assert_eq!(written, body, "the resumed file must be byte-identical");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_write_buffer_does_not_change_the_bytes_written() {
    const SIZE: usize = 32 * 1024;
    let body: Vec<u8> = (0..SIZE).map(|i| (i % 97) as u8).collect();
    let (addr, server) = serve_with_interruption(body.clone(), SIZE, vec!["\"v1\"".to_string()]);

    let path = temp_path("buffered.bin");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    rt.block_on(async {
        client_for(addr)
            .get_object_to_file_v2(
                GetObjectRequest {
                    bucket: "probe-bucket".to_string(),
                    key: "probe-object".to_string(),
                    ..Default::default()
                },
                &path,
                Some(8 * 1024),
            )
            .await
            .expect("download");
    });

    let _ = server.join().expect("server");
    let written = std::fs::read(&path).expect("read back");
    assert_eq!(written, body, "buffering must not alter the file");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_changed_object_is_not_spliced_into_the_file() {
    const SIZE: usize = 24 * 1024;
    let body: Vec<u8> = (0..SIZE).map(|i| (i % 131) as u8).collect();
    let split = SIZE / 2;
    // The object is replaced between the two attempts.
    let (addr, server) = serve_with_interruption(
        body,
        split,
        vec!["\"v1\"".to_string(), "\"v2\"".to_string()],
    );

    let path = temp_path("changed.bin");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let err = rt
        .block_on(async {
            client_for(addr)
                .get_object_to_file_v2(
                    GetObjectRequest {
                        bucket: "probe-bucket".to_string(),
                        key: "probe-object".to_string(),
                        ..Default::default()
                    },
                    &path,
                    None,
                )
                .await
        })
        .expect_err("a changed object must fail the download");

    let _ = server.join().expect("server");
    assert!(
        err.to_string().contains("changed during the download"),
        "unexpected error: {}",
        err
    );
    let _ = std::fs::remove_file(&path);
}
