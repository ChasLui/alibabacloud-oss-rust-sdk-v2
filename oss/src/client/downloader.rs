//! Concurrent ranged download of an object to a local file.
//!
//! Mirrors Go `oss/downloader.go`. An object larger than one part is split
//! into byte ranges that are fetched in parallel and written straight to their
//! final offsets, so a large object never has to fit in memory and the parts
//! do not have to complete in order.
//!
//! The integrity check is the interesting part: OSS reports one CRC64 for the
//! whole object, but the parts are checksummed independently and may finish in
//! any order. [`crc64_combine`] folds the per-part checksums back into the
//! whole-object value, which is then compared against the response header.

use std::path::{Path, PathBuf};

use futures_util::StreamExt;

use crate::api::object::{GetObjectRequest, HeadObjectRequest};
use crate::client::Client;
use crate::utils::crc64_combine;
use crate::utils::Crc64;
use crate::{DEFAULT_DOWNLOAD_PARALLEL, DEFAULT_DOWNLOAD_PART_SIZE};

/// Options for [`Client::download_file`].
#[derive(Debug, Clone)]
pub struct DownloaderOptions {
    /// The byte size of each ranged request. Defaults to 6 MiB, matching Go's
    /// `DefaultDownloadPartSize`.
    pub part_size: i64,

    /// How many ranged requests run at once. Defaults to 3, matching Go's
    /// `DefaultDownloadParallel`.
    pub parallel_num: usize,
}

impl Default for DownloaderOptions {
    fn default() -> Self {
        DownloaderOptions {
            part_size: DEFAULT_DOWNLOAD_PART_SIZE,
            parallel_num: DEFAULT_DOWNLOAD_PARALLEL.max(1) as usize,
        }
    }
}

impl DownloaderOptions {
    /// Sets the part size. Values below 1 are clamped to 1 byte so a
    /// misconfigured part size degrades into a slow download rather than an
    /// infinite loop.
    pub fn with_part_size(mut self, part_size: i64) -> Self {
        self.part_size = part_size.max(1);
        self
    }

    /// Sets how many parts are fetched concurrently. Zero is clamped to 1.
    pub fn with_parallel_num(mut self, parallel_num: usize) -> Self {
        self.parallel_num = parallel_num.max(1);
        self
    }
}

/// One downloaded byte range and the CRC64 of exactly those bytes.
#[derive(Debug, Clone, Copy)]
struct DownloadedPart {
    start: i64,
    size: i64,
    crc64: u64,
}

/// The result of [`Client::download_file`].
#[derive(Debug, Clone)]
pub struct DownloadResult {
    /// Total number of bytes written to the file.
    pub written: i64,
    /// The `ETag` of the downloaded object, when the server reported one.
    pub etag: Option<String>,
    /// The whole-object CRC64, when the download verified it.
    pub crc64: Option<u64>,
}

impl Client {
    /// Downloads an object to `file_path`, fetching large objects in parallel
    /// byte ranges.
    ///
    /// Mirrors Go `Downloader.DownloadFile`. The object's size is resolved
    /// with a `HeadObject` so a large object is never fetched twice.
    ///
    /// `request.range`, when present, must be the closed `bytes=first-last`
    /// form: an open-ended range has no upper bound to plan parts against, so
    /// it is rejected rather than silently guessed.
    ///
    /// The file is created (or truncated) before the first part is written, so
    /// a failure part-way through leaves whatever had already been written.
    /// Callers needing all-or-nothing semantics should download to a
    /// temporary path and rename on success, as Go's `UseTempFile` does.
    pub async fn download_file(
        &self,
        request: &GetObjectRequest,
        file_path: impl AsRef<Path>,
        options: DownloaderOptions,
    ) -> Result<DownloadResult, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = file_path.as_ref().to_path_buf();

        let requested_range = parse_range(request.range.as_deref())?;

        // The probe carries the caller's conditionals so a guarded download
        // fails here — before any bytes are written — rather than after the
        // first part returns.
        let head = self
            .head_object(HeadObjectRequest {
                bucket: request.bucket.clone(),
                key: request.key.clone(),
                if_match: request.if_match.clone(),
                if_none_match: request.if_none_match.clone(),
                if_modified_since: request.if_modified_since.clone(),
                if_unmodified_since: request.if_unmodified_since.clone(),
                version_id: request.version_id.clone(),
                request_payer: request.request_payer.clone(),
                common: request.common.clone(),
            })
            .await?;
        let total_size = head
            .content_length
            .ok_or("HeadObject returned no Content-Length; cannot plan parts")?
            as i64;
        let etag = head.etag.clone();
        // The whole-object CRC is only meaningful when the caller asked for the
        // whole object; a ranged download sees a slice and cannot check it.
        let verify_crc = requested_range.is_none();
        let server_crc = head.hash_crc64.clone();

        let (first, last) = match requested_range {
            Some((f, l)) => {
                if f >= total_size {
                    return Err(format!(
                        "invalid range, object size: {total_size}, range: bytes={f}-{l}"
                    )
                    .into());
                }
                (f, l.min(total_size - 1).max(f))
            }
            None => (0, total_size - 1),
        };
        let span = last - first + 1;

        let file = tokio::fs::File::create(&file_path).await?.into_std().await;
        if span <= 0 {
            // An empty object still produces an (empty) file, so callers can
            // rely on the path existing after a successful call.
            return Ok(DownloadResult {
                written: 0,
                etag,
                crc64: None,
            });
        }

        let part_size = options.part_size.max(1);
        let mut ranges = Vec::new();
        let mut start = first;
        while start <= last {
            let size = part_size.min(last - start + 1);
            ranges.push((start, size));
            start += size;
        }

        let file = std::sync::Arc::new(file);
        let written = std::sync::Arc::new(std::sync::atomic::AtomicI64::new(0));

        let stream = futures_util::stream::iter(ranges.iter().map(|&(start, size)| {
            let client = self.clone();
            // `GetObjectRequest` is not `Clone` (its response body is a
            // stream), so each part rebuilds the request. Every conditional and
            // behavioural field is carried over — dropping, say, `if_match`
            // would silently downgrade a guarded download into an unguarded
            // one — and only the range differs between parts.
            let mut part_request = GetObjectRequest {
                bucket: request.bucket.clone(),
                key: request.key.clone(),
                if_match: request.if_match.clone(),
                if_none_match: request.if_none_match.clone(),
                if_modified_since: request.if_modified_since.clone(),
                if_unmodified_since: request.if_unmodified_since.clone(),
                range: None,
                range_behavior: Some("standard".to_string()),
                response_cache_control: request.response_cache_control.clone(),
                response_content_disposition: request.response_content_disposition.clone(),
                response_content_encoding: request.response_content_encoding.clone(),
                response_content_language: request.response_content_language.clone(),
                response_content_type: request.response_content_type.clone(),
                response_expires: request.response_expires.clone(),
                version_id: request.version_id.clone(),
                traffic_limit: request.traffic_limit,
                progress_fn: None,
                process: request.process.clone(),
                request_payer: request.request_payer.clone(),
                common: request.common.clone(),
            };
            part_request.range = Some(format!("bytes={}-{}", start, start + size - 1));
            let file = file.clone();
            let written = written.clone();
            async move {
                let result = client.get_object(part_request).await?;
                let body = result
                    .body
                    .ok_or("GetObject returned no body for a ranged request")?;
                let mut crc = Crc64::new(0);
                let mut offset = start;
                let mut body = std::pin::pin!(body);
                while let Some(chunk) = body.next().await {
                    let chunk = chunk?;
                    crc.write(&chunk)
                        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                            e.to_string().into()
                        })?;
                    // Writing at an absolute offset lets parts land in any
                    // order without coordinating through a lock.
                    write_all_at(&file, &chunk, offset as u64)?;
                    offset += chunk.len() as i64;
                }
                let size = offset - start;
                written.fetch_add(size, std::sync::atomic::Ordering::Relaxed);
                Ok::<DownloadedPart, Box<dyn std::error::Error + Send + Sync>>(DownloadedPart {
                    start,
                    size,
                    crc64: crc.sum64(),
                })
            }
        }))
        .buffer_unordered(options.parallel_num);

        let mut parts: Vec<DownloadedPart> = Vec::with_capacity(ranges.len());
        let mut stream = std::pin::pin!(stream);
        while let Some(part) = stream.next().await {
            parts.push(part?);
        }

        // Fold the parts into the whole-object CRC. Order matters: the folding
        // reproduces a checksum over the concatenated bytes, so the parts must
        // be visited by ascending offset.
        parts.sort_by_key(|p| p.start);
        let mut combined = 0u64;
        for part in &parts {
            combined = crc64_combine(combined, part.crc64, part.size as u64);
        }

        let crc64 = if verify_crc {
            crate::utils::check_crc64(combined, server_crc.as_deref())
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
            Some(combined)
        } else {
            None
        };

        Ok(DownloadResult {
            written: written.load(std::sync::atomic::Ordering::Relaxed),
            etag,
            crc64,
        })
    }
}

/// Writes `data` at `offset` without moving the file cursor.
///
/// `FileExt::write_at` is a positional write, so parts never contend for a
/// cursor and can land in any order. It can write fewer bytes than requested,
/// so the remainder is retried; a zero-byte write is an error rather than a
/// spin.
fn write_all_at(
    file: &std::fs::File,
    data: &[u8],
    offset: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::os::unix::fs::FileExt;

    let mut written = 0usize;
    while written < data.len() {
        let n = file.write_at(&data[written..], offset + written as u64)?;
        if n == 0 {
            return Err("write_at made no progress".into());
        }
        written += n;
    }
    Ok(())
}

/// Parses an OSS/HTTP `bytes=first-last` range into a closed interval.
fn parse_range(
    range: Option<&str>,
) -> Result<Option<(i64, i64)>, Box<dyn std::error::Error + Send + Sync>> {
    let Some(range) = range else { return Ok(None) };
    let spec = range.strip_prefix("bytes=").ok_or_else(
        || -> Box<dyn std::error::Error + Send + Sync> {
            format!("unsupported Range header: {range}").into()
        },
    )?;
    let (first, last) =
        spec.split_once('-')
            .ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> {
                format!("unsupported Range header: {range}").into()
            })?;
    if first.is_empty() || last.is_empty() {
        return Err(format!("unsupported Range header: {range}").into());
    }
    let first: i64 = first.parse()?;
    let last: i64 = last.parse()?;
    Ok(Some((first, last)))
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

    /// CRC64 of `data`, the way the downloader must compute it.
    fn crc_of(data: &[u8]) -> u64 {
        let mut crc = Crc64::new(0);
        crc.write(data).unwrap();
        crc.sum64()
    }

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("oss-downloader-tests");
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir.join(name)
    }

    /// Serves the HEAD probe plus one ranged response per part, each with the
    /// CRC of exactly the slice it returns.
    async fn serve_ranged(
        server: &mut mockito::ServerGuard,
        body: &[u8],
        part_size: usize,
    ) -> Vec<mockito::Mock> {
        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", &body.len().to_string())
            .with_header("etag", "\"etag-value\"")
            .with_header("x-oss-hash-crc64ecma", &crc_of(body).to_string())
            .create_async()
            .await;

        let mut mocks = Vec::new();
        let mut offset = 0usize;
        while offset < body.len() {
            let size = part_size.min(body.len() - offset);
            let slice = body[offset..offset + size].to_vec();
            mocks.push(
                server
                    .mock("GET", mockito::Matcher::Any)
                    .match_header(
                        "range",
                        format!("bytes={}-{}", offset, offset + size - 1).as_str(),
                    )
                    .with_status(206)
                    .with_header("content-length", &size.to_string())
                    .with_header("x-oss-hash-crc64ecma", &crc_of(&slice).to_string())
                    .with_body(slice)
                    .create_async()
                    .await,
            );
            offset += size;
        }
        mocks
    }

    /// Many parts must reassemble into the exact object. A downloader that
    /// wrote every part at offset 0, or that mis-sized the trailing part,
    /// would produce a file of the right *length* but wrong *contents*.
    #[tokio::test]
    async fn test_download_file_splits_and_reassembles() {
        let mut server = mockito::Server::new_async().await;
        // Not a multiple of the part size, so the trailing part is short.
        let body: Vec<u8> = (0..4096u32).map(|i| (i % 251) as u8).collect();
        let mocks = serve_ranged(&mut server, &body, 1000).await;

        let path = temp_path("reassembled.bin");
        let result = mock_client(&server)
            .download_file(
                &GetObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "test-key".to_string(),
                    ..Default::default()
                },
                &path,
                DownloaderOptions::default()
                    .with_part_size(1000)
                    .with_parallel_num(3),
            )
            .await
            .expect("download should succeed");

        assert_eq!(result.written, body.len() as i64);
        assert_eq!(result.etag.as_deref(), Some("\"etag-value\""));
        assert_eq!(result.crc64, Some(crc_of(&body)));
        assert_eq!(
            std::fs::read(&path).expect("downloaded file"),
            body,
            "the reassembled file must equal the object byte for byte"
        );
        for m in &mocks {
            m.assert_async().await;
        }
        let _ = std::fs::remove_file(&path);
    }

    /// The part count must follow the part size, and concurrency must not
    /// reorder the resulting bytes.
    #[tokio::test]
    async fn test_download_file_parallel_parts_stay_ordered() {
        let mut server = mockito::Server::new_async().await;
        let body: Vec<u8> = (0..2500u32).map(|i| (i * 3 % 256) as u8).collect();
        let mocks = serve_ranged(&mut server, &body, 100).await;

        let path = temp_path("ordered.bin");
        mock_client(&server)
            .download_file(
                &GetObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "test-key".to_string(),
                    ..Default::default()
                },
                &path,
                DownloaderOptions::default()
                    .with_part_size(100)
                    .with_parallel_num(8),
            )
            .await
            .expect("download should succeed");

        assert_eq!(std::fs::read(&path).unwrap(), body);
        for m in &mocks {
            m.assert_async().await;
        }
        let _ = std::fs::remove_file(&path);
    }

    /// An object smaller than one part is fetched in a single request and
    /// still verified.
    #[tokio::test]
    async fn test_download_file_single_part() {
        let mut server = mockito::Server::new_async().await;
        let body = b"small object".to_vec();
        serve_ranged(&mut server, &body, 1024).await;

        let path = temp_path("single.bin");
        let result = mock_client(&server)
            .download_file(
                &GetObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "test-key".to_string(),
                    ..Default::default()
                },
                &path,
                DownloaderOptions::default(),
            )
            .await
            .expect("download should succeed");

        assert_eq!(result.crc64, Some(crc_of(&body)));
        assert_eq!(std::fs::read(&path).unwrap(), body);
        let _ = std::fs::remove_file(&path);
    }

    /// A server checksum that disagrees with the delivered bytes must fail the
    /// download; keeping the bytes silently is the failure mode that matters
    /// (corrupt data that looks downloaded).
    #[tokio::test]
    async fn test_download_file_rejects_mismatched_crc() {
        let mut server = mockito::Server::new_async().await;
        let body = b"payload".to_vec();

        // The object-level checksum is what the downloader validates against,
        // so the probe must advertise the wrong one.
        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", &body.len().to_string())
            .with_header("x-oss-hash-crc64ecma", "1")
            .create_async()
            .await;
        server
            .mock("GET", mockito::Matcher::Any)
            .with_status(206)
            .with_header("content-length", &body.len().to_string())
            .with_body(&body)
            .create_async()
            .await;

        let path = temp_path("bad-crc.bin");
        let err = mock_client(&server)
            .download_file(
                &GetObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "test-key".to_string(),
                    ..Default::default()
                },
                &path,
                DownloaderOptions::default(),
            )
            .await
            .expect_err("a checksum mismatch must surface as an error");

        assert!(
            err.to_string().contains("crc is inconsistent"),
            "unexpected error: {err}"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// A ranged download cannot check a whole-object checksum, so it must not
    /// claim one — reporting the slice's CRC would be a false reassurance.
    #[tokio::test]
    async fn test_ranged_download_skips_whole_object_crc() {
        let mut server = mockito::Server::new_async().await;
        let body: Vec<u8> = (0..300u32).map(|i| (i % 256) as u8).collect();

        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", &body.len().to_string())
            .create_async()
            .await;
        let slice = body[..100].to_vec();
        server
            .mock("GET", mockito::Matcher::Any)
            .match_header("range", "bytes=0-99")
            .with_status(206)
            .with_header("content-length", &slice.len().to_string())
            .with_body(slice)
            .create_async()
            .await;

        let path = temp_path("ranged.bin");
        let result = mock_client(&server)
            .download_file(
                &GetObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "test-key".to_string(),
                    range: Some("bytes=0-99".to_string()),
                    ..Default::default()
                },
                &path,
                DownloaderOptions::default(),
            )
            .await
            .expect("ranged download should succeed");

        assert_eq!(result.written, 100);
        assert_eq!(result.crc64, None, "a slice has no whole-object checksum");
        assert_eq!(std::fs::read(&path).unwrap(), body[..100]);
        let _ = std::fs::remove_file(&path);
    }


    /// Conditional headers must reach the server on every part. A downloader
    /// that dropped `if_match` when splitting would silently fetch an object
    /// the caller asked to guard against.
    #[tokio::test]
    async fn test_conditional_headers_reach_every_part() {
        let mut server = mockito::Server::new_async().await;
        let body: Vec<u8> = (0..200u32).map(|i| (i % 256) as u8).collect();

        server
            .mock("HEAD", mockito::Matcher::Any)
            .match_header("if-match", "\"guard\"")
            .with_status(200)
            .with_header("content-length", &body.len().to_string())
            .with_header("x-oss-hash-crc64ecma", &crc_of(&body).to_string())
            .create_async()
            .await;

        let mut mocks = Vec::new();
        for (offset, size) in [(0usize, 100usize), (100, 100)] {
            let slice = body[offset..offset + size].to_vec();
            mocks.push(
                server
                    .mock("GET", mockito::Matcher::Any)
                    .match_header("range", format!("bytes={}-{}", offset, offset + size - 1).as_str())
                    .match_header("if-match", "\"guard\"")
                    .with_status(206)
                    .with_header("content-length", &size.to_string())
                    .with_body(slice)
                    .create_async()
                    .await,
            );
        }

        let path = temp_path("conditional.bin");
        mock_client(&server)
            .download_file(
                &GetObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "test-key".to_string(),
                    if_match: Some("\"guard\"".to_string()),
                    ..Default::default()
                },
                &path,
                DownloaderOptions::default().with_part_size(100),
            )
            .await
            .expect("download should succeed");

        assert_eq!(std::fs::read(&path).unwrap(), body);
        // `assert_async` panics if the header never arrived.
        for m in &mocks {
            m.assert_async().await;
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_options_clamp_degenerate_values() {
        let opts = DownloaderOptions::default()
            .with_part_size(0)
            .with_parallel_num(0);
        assert_eq!(opts.part_size, 1, "a zero part size must not hang");
        assert_eq!(
            opts.parallel_num, 1,
            "zero parallelism must still make progress"
        );
    }

    #[test]
    fn test_parse_range() {
        assert_eq!(parse_range(None).unwrap(), None);
        assert_eq!(parse_range(Some("bytes=0-99")).unwrap(), Some((0, 99)));
        assert_eq!(
            parse_range(Some("bytes=100-199")).unwrap(),
            Some((100, 199))
        );
        // Open-ended and suffix ranges have no upper bound to plan against.
        assert!(parse_range(Some("bytes=100-")).is_err());
        assert!(parse_range(Some("bytes=-100")).is_err());
        assert!(parse_range(Some("0-99")).is_err());
    }
}
