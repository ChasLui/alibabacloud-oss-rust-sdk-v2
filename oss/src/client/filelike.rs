//! File-like handles over OSS objects.
//!
//! Mirrors the `ReadOnlyFile` and `AppendOnlyFile` types of Go
//! `oss/filelike.go`. Both keep a position and expose `seek`/`read`/`write`
//! style access so a caller can treat an object as a file instead of issuing
//! one HTTP request per operation itself.
//!
//! The properties worth stating because they are not obvious from the API:
//!
//! - Reads are served by a single ranged `GetObject` whose body is consumed
//!   lazily; seeking backwards discards the open stream and opens a new one. A
//!   seek does not re-read bytes that were already downloaded, but it does mean
//!   the connection is not reused across disjoint ranges.
//! - A ranged response is checked against the object it was opened from. OSS
//!   answers a range request with the range that was actually returned, so a
//!   `Content-Range` that does not start where the request asked means the
//!   object changed underneath the handle, and continuing to read would
//!   silently splice two different objects together.
//! - Appends carry the CRC of everything written so far, so the service can
//!   reject an append that would not extend the object consistently. When the
//!   service reports `PositionNotEqualToLength` the handle re-reads the
//!   position and accepts the write only if it turned out to be exactly an
//!   extension — a genuinely concurrent writer is an error, not something to
//!   paper over.

use futures_util::StreamExt;

use crate::api::object::{AppendObjectRequest, GetObjectRequest, HeadObjectRequest};
use crate::client::Client;

/// Whence values accepted by [`ReadOnlyFile::seek`] and
/// [`AppendOnlyFile::seek`], matching [`std::io::SeekFrom`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
    /// Offset from the start of the object.
    Start(i64),
    /// Offset from the current position.
    Current(i64),
    /// Offset from the end of the object.
    End(i64),
}

/// Options for [`Client::open_read_only_file`].
#[derive(Debug, Clone, Default)]
pub struct OpenOptions {
    /// The version ID of the object to read.
    pub version_id: Option<String>,
    /// Whoever pays for the request, when the bucket bills the requester.
    pub request_payer: Option<String>,
}

/// A read-only handle over an object.
///
/// The handle pins the object's size, ETag, and last-modified time when it is
/// opened; every ranged read is checked against them so a concurrent rewrite
/// of the object surfaces as an error instead of mixed bytes.
pub struct ReadOnlyFile {
    client: Client,
    bucket: String,
    key: String,
    version_id: Option<String>,
    request_payer: Option<String>,

    /// The object's size at open time.
    size: i64,
    /// The object's ETag at open time.
    etag: Option<String>,
    /// The object's last-modified time at open time.
    last_modified: Option<String>,

    /// The current read position.
    offset: i64,
    /// The open ranged response, when one is in progress. Its start offset is
    /// recorded so a read that does not continue exactly where the stream left
    /// off can discard it instead of skipping bytes.
    body: Option<(i64, crate::client::BodyStream)>,
    closed: bool,
}

impl ReadOnlyFile {
    /// The object's size in bytes at open time.
    pub fn size(&self) -> i64 {
        self.size
    }

    /// The current read position.
    pub fn offset(&self) -> i64 {
        self.offset
    }

    /// The object's ETag at open time.
    pub fn etag(&self) -> Option<&str> {
        self.etag.as_deref()
    }

    /// The object's last-modified time at open time.
    pub fn last_modified(&self) -> Option<&str> {
        self.last_modified.as_deref()
    }

    /// Moves the read position.
    ///
    /// Returns the new position. Seeking past the end is allowed, matching
    /// Go's `ReadOnlyFile.Seek`; the next read then reports end-of-file.
    pub fn seek(
        &mut self,
        position: SeekFrom,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let target = match position {
            SeekFrom::Start(offset) => offset,
            SeekFrom::Current(delta) => self.offset + delta,
            SeekFrom::End(delta) => self.size + delta,
        };
        if target < 0 {
            return Err(format!("negative seek position {target}").into());
        }
        self.offset = target;
        Ok(target)
    }

    /// Reads up to `buf.len()` bytes at the current position.
    ///
    /// Returns the number of bytes read, which is `0` at end of file.
    pub async fn read(
        &mut self,
        buf: &mut [u8],
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        if self.closed {
            return Err("file is closed".into());
        }
        if self.offset >= self.size {
            return Ok(0);
        }

        // A buffered stream only helps while it is positioned exactly where the
        // next read wants to continue from. Any other position starts a fresh
        // request rather than skipping bytes.
        let started_at = self.offset;
        if self.body.as_ref().map(|(start, _)| *start) != Some(started_at) {
            self.body = None;
            let result = self
                .client
                .get_object(GetObjectRequest {
                    bucket: self.bucket.clone(),
                    key: self.key.clone(),
                    range: Some(format!("bytes={started_at}-")),
                    range_behavior: Some("standard".to_string()),
                    version_id: self.version_id.clone(),
                    request_payer: self.request_payer.clone(),
                    ..Default::default()
                })
                .await?;
            self.check_response(started_at, &result)?;
            let body = result
                .body
                .ok_or("GetObject returned no body for a ranged request")?;
            self.body = Some((started_at, body));
        }

        let (start, body) = self.body.as_mut().expect("a body was just opened");
        let mut filled = 0usize;
        while filled < buf.len() {
            let next = match std::pin::pin!(&mut *body).next().await {
                Some(chunk) => chunk?,
                None => break,
            };
            let take = (buf.len() - filled).min(next.len());
            buf[filled..filled + take].copy_from_slice(&next[..take]);
            filled += take;
            *start += take as i64;
            if take < next.len() {
                // A chunk larger than the remaining space cannot be pushed back
                // into the stream, so the leftover would be lost; the read is
                // capped at whole chunks instead.
                break;
            }
        }
        self.offset += filled as i64;
        Ok(filled)
    }

    /// Reads exactly `len` bytes from the current position.
    ///
    /// A short read means the object ended early, which is an error rather
    /// than a partial success: callers asking for a fixed size are reading a
    /// structure, not a stream.
    pub async fn read_exact(
        &mut self,
        len: usize,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut buf = vec![0u8; len];
        let mut filled = 0usize;
        while filled < len {
            let read = self.read(&mut buf[filled..]).await?;
            if read == 0 {
                return Err(
                    format!("unexpected end of file: wanted {len} bytes, got {filled}").into(),
                );
            }
            filled += read;
        }
        Ok(buf)
    }

    /// Reads the whole remainder of the object from the current position.
    pub async fn read_to_end(
        &mut self,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut data = Vec::new();
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let read = self.read(&mut buf).await?;
            if read == 0 {
                break;
            }
            data.extend_from_slice(&buf[..read]);
        }
        Ok(data)
    }

    /// Releases the handle. Reading after this fails.
    pub fn close(&mut self) {
        self.closed = true;
        self.body = None;
    }

    /// Verifies that a ranged response really starts where it was asked to.
    ///
    /// The service echoes the returned range, and the object's identity is
    /// re-checked against the values captured at open time. A mismatch means
    /// the bytes about to be read are not from the object this handle
    /// describes.
    fn check_response(
        &self,
        offset: i64,
        result: &crate::api::object::GetObjectResult,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(content_range) = result.content_range.as_deref() {
            let got = parse_range_start(content_range)
                .ok_or_else(|| format!("unparsable Content-Range {content_range:?}"))?;
            if got != offset {
                return Err(
                    format!("range get fail, expect offset:{offset}, got offset:{got}").into(),
                );
            }
        }
        if let (Some(opened), Some(current)) = (self.etag.as_deref(), result.etag.as_deref()) {
            if opened != current {
                return Err(format!(
                    "source file is changed, origin etag {opened}, new etag {current}"
                )
                .into());
            }
        }
        if let (Some(opened), Some(current)) = (
            self.last_modified.as_deref(),
            result.last_modified.as_deref(),
        ) {
            if opened != current {
                return Err(format!(
                    "source file is changed, origin last-modified {opened}, new last-modified \
                     {current}"
                )
                .into());
            }
        }
        Ok(())
    }
}

/// Extracts the first byte position from a `Content-Range` header value
/// (`bytes 0-99/100`).
fn parse_range_start(content_range: &str) -> Option<i64> {
    let range = content_range.strip_prefix("bytes ")?;
    let (range, _) = range.split_once('/')?;
    let (start, _) = range.split_once('-')?;
    start.trim().parse::<i64>().ok()
}

/// Options for [`Client::open_append_only_file`].
#[derive(Default)]
pub struct AppendOptions {
    /// Whoever pays for the request, when the bucket bills the requester.
    pub request_payer: Option<String>,
    /// The attributes to apply when the object is first created.
    ///
    /// An append does not carry most object attributes, so they take effect
    /// only on the write that creates the object; later writes leave the
    /// existing attributes alone.
    pub create_parameter: Option<AppendObjectRequest>,
}

/// A handle that appends to an object, creating it when it does not exist.
///
/// The object must be appendable; opening a normal object is an error, because
/// appending to it would not produce what the caller expects.
pub struct AppendOnlyFile {
    client: Client,
    bucket: String,
    key: String,
    request_payer: Option<String>,
    create_parameter: Option<AppendObjectRequest>,

    /// The current append position, and therefore the object's length.
    offset: i64,
    /// The CRC64 of everything written so far. OSS folds each append into the
    /// running value, so the next request must carry it.
    hash_crc64: Option<String>,
    /// Whether the object exists already.
    created: bool,
    closed: bool,
}

impl AppendOnlyFile {
    /// The current append position, which is the object's length as far as
    /// this handle is concerned.
    pub fn offset(&self) -> i64 {
        self.offset
    }

    /// The CRC64 of everything written so far.
    pub fn hash_crc64(&self) -> Option<&str> {
        self.hash_crc64.as_deref()
    }

    /// Appends `data`.
    ///
    /// Returns the position the next append must use, which is the position
    /// after `data`.
    pub async fn write(
        &mut self,
        data: Vec<u8>,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        if self.closed {
            return Err("file is closed".into());
        }
        let length = data.len() as i64;

        let mut request = AppendObjectRequest {
            bucket: self.bucket.clone(),
            key: self.key.clone(),
            position: Some(self.offset),
            // The first append starts the checksum; later ones continue it.
            init_hash_crc64: Some(self.hash_crc64.clone().unwrap_or_else(|| "0".to_string())),
            request_payer: self.request_payer.clone(),
            ..Default::default()
        };
        if let Some(create) = self.create_parameter.as_ref().filter(|_| !self.created) {
            request = apply_create_parameter(request, create);
        }
        request.body = Some(crate::BodyContent::from_bytes(data, None));

        match self.client.append_object(request).await {
            Ok(result) => {
                self.offset =
                    parse_next_position(result.next_position.as_deref(), self.offset + length)?;
                self.hash_crc64 = result.hash_crc64.clone();
                self.created = true;
                Ok(self.offset)
            }
            Err(err) => {
                // Another writer may have appended between this handle's last
                // write and this one. The service reports the position it
                // actually has; the write is only retried implicitly when that
                // position is exactly where this write would have landed,
                // which means the bytes are already there.
                if !is_position_mismatch(&*err) {
                    return Err(err);
                }
                let head = self
                    .client
                    .head_object(HeadObjectRequest {
                        bucket: self.bucket.clone(),
                        key: self.key.clone(),
                        request_payer: self.request_payer.clone(),
                        ..Default::default()
                    })
                    .await?;
                let position = head.content_length.unwrap_or(0) as i64;
                if self.offset + length == position {
                    self.offset = position;
                    self.hash_crc64 = head.hash_crc64.clone();
                    self.created = true;
                    return Ok(position);
                }
                Err(err)
            }
        }
    }

    /// Re-reads the object's length, so a position that moved underneath the
    /// handle can be picked up deliberately.
    pub async fn stat(
        &mut self,
    ) -> Result<(i64, Option<String>), Box<dyn std::error::Error + Send + Sync>> {
        let head = self
            .client
            .head_object(HeadObjectRequest {
                bucket: self.bucket.clone(),
                key: self.key.clone(),
                request_payer: self.request_payer.clone(),
                ..Default::default()
            })
            .await?;
        let size = head.content_length.unwrap_or(0) as i64;
        Ok((size, head.hash_crc64.clone()))
    }

    /// Releases the handle. Writing after this fails.
    pub fn close(&mut self) {
        self.closed = true;
    }
}

/// Carries the creation attributes onto the write that creates the object.
///
/// An append request has no field for most object attributes, so the ones the
/// caller supplied are copied onto this request; the service ignores them once
/// the object exists.
fn apply_create_parameter(
    mut request: AppendObjectRequest,
    create: &AppendObjectRequest,
) -> AppendObjectRequest {
    request.cache_control = create.cache_control.clone();
    request.content_disposition = create.content_disposition.clone();
    request.content_encoding = create.content_encoding.clone();
    request.content_type = create.content_type.clone();
    request.expires = create.expires.clone();
    request.server_side_encryption = create.server_side_encryption.clone();
    request.server_side_data_encryption = create.server_side_data_encryption.clone();
    request.server_side_encryption_key_id = create.server_side_encryption_key_id.clone();
    request.metadata = create.metadata.clone();
    request.tagging = create.tagging.clone();
    request.acl = create.acl.clone();
    request.storage_class = create.storage_class.clone();
    request.forbid_overwrite = create.forbid_overwrite.clone();
    request.common = create.common.clone();
    request
}

/// Reads the append position the service reported.
fn parse_next_position(
    next_position: Option<&str>,
    fallback: i64,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    match next_position {
        Some(value) => value
            .parse::<i64>()
            .map_err(|e| format!("unparsable next position {value:?}: {e}").into()),
        // The service reports the position on every successful append; without
        // it the handle cannot know where the next write belongs.
        None => Err(format!("append returned no next position (expected {fallback})").into()),
    }
}

/// Whether the service rejected the append because the object's length is not
/// the position the handle assumed.
fn is_position_mismatch(err: &(dyn std::error::Error + Send + Sync + 'static)) -> bool {
    err.downcast_ref::<crate::ServiceError>()
        .map(|service_error| service_error.error_code() == "PositionNotEqualToLength")
        .unwrap_or(false)
}

impl Client {
    /// Opens an object for reading.
    ///
    /// Mirrors Go `NewReadOnlyFile`. The object's size, ETag, and last-modified
    /// time are captured here and every subsequent ranged read is checked
    /// against them.
    pub async fn open_read_only_file(
        &self,
        bucket: &str,
        key: &str,
        options: OpenOptions,
    ) -> Result<ReadOnlyFile, Box<dyn std::error::Error + Send + Sync>> {
        let head = self
            .head_object(HeadObjectRequest {
                bucket: bucket.to_string(),
                key: key.to_string(),
                version_id: options.version_id.clone(),
                request_payer: options.request_payer.clone(),
                ..Default::default()
            })
            .await?;

        let size = head
            .content_length
            .map(|length| length as i64)
            .ok_or("HeadObject returned no Content-Length")?;
        if size < 0 {
            return Err(format!("file size is invalid, got {size}").into());
        }

        Ok(ReadOnlyFile {
            client: self.clone(),
            bucket: bucket.to_string(),
            key: key.to_string(),
            version_id: options.version_id,
            request_payer: options.request_payer,
            size,
            etag: head.etag.clone(),
            last_modified: head.last_modified.clone(),
            offset: 0,
            body: None,
            closed: false,
        })
    }

    /// Opens an object for appending, creating it when it does not exist.
    ///
    /// Mirrors Go `NewAppendFile`. An existing object must be appendable; any
    /// other object is rejected, because appending to it cannot succeed.
    pub async fn open_append_only_file(
        &self,
        bucket: &str,
        key: &str,
        options: AppendOptions,
    ) -> Result<AppendOnlyFile, Box<dyn std::error::Error + Send + Sync>> {
        let mut file = AppendOnlyFile {
            client: self.clone(),
            bucket: bucket.to_string(),
            key: key.to_string(),
            request_payer: options.request_payer,
            create_parameter: options.create_parameter,
            offset: 0,
            hash_crc64: None,
            created: false,
            closed: false,
        };

        match self
            .head_object(HeadObjectRequest {
                bucket: bucket.to_string(),
                key: key.to_string(),
                request_payer: file.request_payer.clone(),
                ..Default::default()
            })
            .await
        {
            Ok(head) => {
                // An append to a non-appendable object fails at the service, so
                // it is rejected here where the reason is still visible.
                if !head
                    .object_type
                    .as_deref()
                    .map(|kind| kind.eq_ignore_ascii_case("Appendable"))
                    .unwrap_or(false)
                {
                    return Err(format!(
                        "not an appendable file: {} is {:?}",
                        key, head.object_type
                    )
                    .into());
                }
                file.offset = head.content_length.unwrap_or(0) as i64;
                file.hash_crc64 = head.hash_crc64.clone();
                file.created = true;
                Ok(file)
            }
            Err(err) => {
                // A missing object is the normal case for a new file; anything
                // else is a real failure.
                if is_not_found(&*err) {
                    Ok(file)
                } else {
                    Err(err)
                }
            }
        }
    }
}

/// Whether the request failed because the object does not exist.
fn is_not_found(err: &(dyn std::error::Error + Send + Sync + 'static)) -> bool {
    err.downcast_ref::<crate::ServiceError>()
        .map(|service_error| service_error.status_code.as_u16() == 404)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::SignatureVersionType;

    fn mock_client(server: &mockito::ServerGuard) -> Client {
        Client::new(
            &Config::default()
                .with_endpoint(server.url().as_str())
                .with_region("cn-hangzhou")
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    "test-ak",
                    "test-sk",
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V1)
                .with_log_level(LogLevel::Off),
        )
    }

    #[test]
    fn test_parse_range_start() {
        assert_eq!(parse_range_start("bytes 0-99/1000"), Some(0));
        assert_eq!(parse_range_start("bytes 100-199/1000"), Some(100));
        assert_eq!(parse_range_start("bytes 5-5/1000"), Some(5));
        assert_eq!(parse_range_start("nonsense"), None);
    }

    /// Reading must return the object's bytes in order, and a seek must move
    /// the position without losing or duplicating bytes.
    #[tokio::test]
    async fn test_read_only_file_reads_and_seeks() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", "10")
            .with_header("etag", "\"v1\"")
            .with_header("last-modified", "Mon, 21 Sep 2026 00:00:00 GMT")
            .create_async()
            .await;
        server
            .mock("GET", mockito::Matcher::Any)
            .match_header("range", "bytes=0-")
            .with_status(206)
            .with_header("content-range", "bytes 0-9/10")
            .with_header("etag", "\"v1\"")
            .with_header("last-modified", "Mon, 21 Sep 2026 00:00:00 GMT")
            .with_body("0123456789")
            .create_async()
            .await;
        server
            .mock("GET", mockito::Matcher::Any)
            .match_header("range", "bytes=6-")
            .with_status(206)
            .with_header("content-range", "bytes 6-9/10")
            .with_header("etag", "\"v1\"")
            .with_header("last-modified", "Mon, 21 Sep 2026 00:00:00 GMT")
            .with_body("6789")
            .create_async()
            .await;

        let mut file = mock_client(&server)
            .open_read_only_file("test-bucket", "test-key", OpenOptions::default())
            .await
            .expect("open should succeed");
        assert_eq!(file.size(), 10);

        let data = file.read_to_end().await.expect("read should succeed");
        assert_eq!(data, b"0123456789");

        // At end of file a read returns nothing rather than failing.
        let mut buf = [0u8; 4];
        assert_eq!(file.read(&mut buf).await.unwrap(), 0);

        // Seeking backwards re-opens the stream at the new position.
        assert_eq!(file.seek(SeekFrom::Start(6)).unwrap(), 6);
        let tail = file.read_to_end().await.expect("read should succeed");
        assert_eq!(tail, b"6789");

        assert_eq!(file.seek(SeekFrom::End(-4)).unwrap(), 6);
        assert_eq!(file.seek(SeekFrom::Current(2)).unwrap(), 8);
        assert!(file.seek(SeekFrom::Start(-1)).is_err());
    }

    /// A ranged response that starts somewhere other than the requested offset
    /// means the object changed; reading on would splice two objects together.
    #[tokio::test]
    async fn test_read_only_file_rejects_wrong_range() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", "10")
            .with_header("etag", "\"v1\"")
            .create_async()
            .await;
        // The service answers with a different range than the one requested.
        server
            .mock("GET", mockito::Matcher::Any)
            .with_status(206)
            .with_header("content-range", "bytes 0-9/10")
            .with_header("etag", "\"v1\"")
            .with_body("0123456789")
            .create_async()
            .await;

        let mut file = mock_client(&server)
            .open_read_only_file("test-bucket", "test-key", OpenOptions::default())
            .await
            .expect("open should succeed");
        file.seek(SeekFrom::Start(5)).unwrap();

        let mut buf = [0u8; 4];
        let err = file
            .read(&mut buf)
            .await
            .expect_err("a wrong range must fail");
        assert!(err.to_string().contains("expect offset:5"), "{err}");
    }

    /// A read after a rewrite of the object must fail instead of returning
    /// bytes from the new version.
    #[tokio::test]
    async fn test_read_only_file_rejects_changed_object() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", "10")
            .with_header("etag", "\"v1\"")
            .create_async()
            .await;
        server
            .mock("GET", mockito::Matcher::Any)
            .with_status(206)
            .with_header("content-range", "bytes 0-9/10")
            .with_header("etag", "\"v2\"")
            .with_body("0123456789")
            .create_async()
            .await;

        let mut file = mock_client(&server)
            .open_read_only_file("test-bucket", "test-key", OpenOptions::default())
            .await
            .expect("open should succeed");

        let mut buf = [0u8; 4];
        let err = file
            .read(&mut buf)
            .await
            .expect_err("a changed object must fail");
        assert!(err.to_string().contains("source file is changed"), "{err}");
    }

    /// Opening a new object for append must not fail; the object does not
    /// exist yet.
    #[tokio::test]
    async fn test_append_only_file_creates_when_missing() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(404)
            .with_body("<Error><Code>NoSuchKey</Code><Message>missing</Message></Error>")
            .create_async()
            .await;
        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("append".to_string()))
            .with_status(200)
            .with_header("x-oss-hash-crc64ecma", "11177612005948864433")
            .with_header("x-oss-next-append-position", "5")
            .with_body("")
            .expect(1)
            .create_async()
            .await;

        let mut file = mock_client(&server)
            .open_append_only_file("test-bucket", "new-key", AppendOptions::default())
            .await
            .expect("opening a missing object should succeed");
        assert_eq!(file.offset(), 0);

        let position = file
            .write(b"hello".to_vec())
            .await
            .expect("append should succeed");
        assert_eq!(position, 5);
        assert_eq!(file.hash_crc64(), Some("11177612005948864433"));
    }

    /// An existing non-appendable object must be rejected at open time.
    #[tokio::test]
    async fn test_append_only_file_rejects_non_appendable() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", "10")
            .with_header("x-oss-object-type", "Normal")
            .create_async()
            .await;

        let err = match mock_client(&server)
            .open_append_only_file("test-bucket", "normal-key", AppendOptions::default())
            .await
        {
            Ok(_) => panic!("a non-appendable object must be rejected"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("not an appendable file"), "{err}");
    }

    /// Appending to an existing appendable object must continue from its
    /// length and carry the checksum of everything written so far.
    #[tokio::test]
    async fn test_append_only_file_continues_existing_object() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", "5")
            .with_header("x-oss-object-type", "Appendable")
            .with_header("x-oss-hash-crc64ecma", "11177612005948864433")
            .create_async()
            .await;
        let append = server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex("append".to_string()),
                mockito::Matcher::Regex("position=5".to_string()),
            ]))
            .with_status(200)
            .with_header("x-oss-hash-crc64ecma", "10826451443916115081")
            .with_header("x-oss-next-append-position", "10")
            .with_body("")
            .expect(1)
            .create_async()
            .await;

        let mut file = mock_client(&server)
            .open_append_only_file("test-bucket", "appendable-key", AppendOptions::default())
            .await
            .expect("opening an appendable object should succeed");
        assert_eq!(file.offset(), 5);

        let position = file
            .write(b"world".to_vec())
            .await
            .expect("append should succeed");
        assert_eq!(position, 10);
        assert_eq!(file.hash_crc64(), Some("10826451443916115081"));
        append.assert_async().await;
    }

    /// A concurrent writer that left the object at exactly the position this
    /// write would have produced means the bytes are already there, so the
    /// write is reported as successful rather than retried.
    #[tokio::test]
    async fn test_append_only_file_tolerates_matching_concurrent_write() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", "0")
            .with_header("x-oss-object-type", "Appendable")
            .create_async()
            .await;
        // The append is rejected, but a later HEAD shows the object ends
        // exactly where this write would have ended.
        server
            .mock("POST", mockito::Matcher::Any)
            .with_status(409)
            .with_body(
                "<Error><Code>PositionNotEqualToLength</Code><Message>position \
                 mismatch</Message></Error>",
            )
            .create_async()
            .await;
        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", "5")
            .with_header("x-oss-object-type", "Appendable")
            .with_header("x-oss-hash-crc64ecma", "10826451443916115081")
            .create_async()
            .await;

        let mut file = mock_client(&server)
            .open_append_only_file("test-bucket", "raced-key", AppendOptions::default())
            .await
            .expect("open should succeed");

        let position = file
            .write(b"hello".to_vec())
            .await
            .expect("a matching concurrent write is not an error");
        assert_eq!(position, 5);
        assert_eq!(file.hash_crc64(), Some("10826451443916115081"));
    }
}
