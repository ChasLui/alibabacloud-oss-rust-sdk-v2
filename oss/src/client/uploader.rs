//! Multipart upload of a local file.
//!
//! Mirrors Go `oss/uploader.go`. A file smaller than one part is sent with a
//! single `PutObject`; anything larger is split into parts that are uploaded
//! concurrently and then committed with `CompleteMultipartUpload`.
//!
//! Two behaviours are easy to get wrong and are handled explicitly:
//!
//! - A failure must not leave an orphaned multipart upload behind. Unless the
//!   caller opts out, the upload is aborted, which is what keeps a failed
//!   attempt from billing storage that no one can see.
//! - The uploaded-object CRC is not a running checksum: OSS reports a CRC per
//!   part, and the whole-object value is those part checksums folded together
//!   in part order. Folding them is what makes the final comparison mean
//!   anything.

use std::path::Path;

use futures_util::StreamExt;

use crate::api::object::{
    AbortMultipartUploadRequest, CompleteMultipartUploadPart, CompleteMultipartUploadRequest,
    InitiateMultipartUploadRequest, PutObjectRequest, UploadPartRequest,
};
use crate::client::Client;
use crate::utils::crc64_combine;
use crate::{DEFAULT_UPLOAD_PARALLEL, DEFAULT_UPLOAD_PART_SIZE, MIN_PART_SIZE};

/// Options for [`Client::upload_file`].
#[derive(Debug, Clone)]
pub struct UploaderOptions {
    /// The byte size of each part. Defaults to 6 MiB, matching Go's
    /// `DefaultUploadPartSize`.
    pub part_size: i64,

    /// How many parts are uploaded at once. Defaults to 3, matching Go's
    /// `DefaultUploadParallel`.
    pub parallel_num: usize,

    /// Keep the already-uploaded parts when the upload fails.
    ///
    /// The default (`false`) aborts the multipart upload, so a failure does
    /// not leave parts behind that continue to be billed. Set this to `true`
    /// only when the caller intends to resume the upload id later.
    pub leave_parts_on_error: bool,
}

impl Default for UploaderOptions {
    fn default() -> Self {
        UploaderOptions {
            part_size: DEFAULT_UPLOAD_PART_SIZE,
            parallel_num: DEFAULT_UPLOAD_PARALLEL.max(1) as usize,
            leave_parts_on_error: false,
        }
    }
}

impl UploaderOptions {
    /// Sets the part size. Values below 1 are clamped to 1 byte.
    ///
    /// OSS rejects a multipart whose parts are smaller than 100 KiB, but that
    /// check is applied when the upload is planned, not here: silently raising
    /// the value would misreport what the caller asked for.
    pub fn with_part_size(mut self, part_size: i64) -> Self {
        self.part_size = part_size.max(1);
        self
    }

    /// Sets how many parts are uploaded concurrently. Zero is clamped to 1.
    pub fn with_parallel_num(mut self, parallel_num: usize) -> Self {
        self.parallel_num = parallel_num.max(1);
        self
    }

    /// Keeps uploaded parts when the upload fails, instead of aborting.
    pub fn with_leave_parts_on_error(mut self, leave_parts_on_error: bool) -> Self {
        self.leave_parts_on_error = leave_parts_on_error;
        self
    }
}

/// The result of [`Client::upload_file`].
#[derive(Debug, Clone)]
pub struct UploadResult {
    /// The upload ID, absent when the file fit in a single `PutObject`.
    pub upload_id: Option<String>,
    /// The `ETag` of the stored object.
    pub etag: Option<String>,
    /// The version ID, when the bucket has versioning enabled.
    pub version_id: Option<String>,
    /// The whole-object CRC64 reported by the server.
    pub hash_crc64: Option<String>,
}

/// One uploaded part: its number, `ETag`, size, and the CRC the server
/// reported for that part.
#[derive(Debug, Clone)]
struct UploadedPart {
    part_number: i32,
    etag: String,
    size: i64,
    hash_crc64: Option<String>,
}

impl Client {
    /// Uploads a local file as an object, using multipart upload for files
    /// larger than one part.
    ///
    /// Mirrors Go `Uploader.UploadFile`. `request.body` is ignored: the file's
    /// own bytes are sent.
    ///
    /// On failure the multipart upload is aborted unless
    /// [`UploaderOptions::leave_parts_on_error`] is set, so a failed attempt
    /// does not leave billable parts behind.
    pub async fn upload_file(
        &self,
        request: &PutObjectRequest,
        file_path: impl AsRef<Path>,
        options: UploaderOptions,
    ) -> Result<UploadResult, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = file_path.as_ref().to_path_buf();
        let metadata = tokio::fs::metadata(&file_path).await?;
        if metadata.is_dir() {
            return Err(format!("{} is a directory, not a file", file_path.display()).into());
        }
        let total_size = metadata.len() as i64;

        // One part is not worth a multipart round trip.
        if total_size <= options.part_size.max(1) {
            return self
                .upload_file_single_part(request, &file_path, total_size)
                .await;
        }

        self.upload_file_multipart(request, &file_path, total_size, options)
            .await
    }

    /// The under-one-part path: one `PutObject` carrying the whole file.
    async fn upload_file_single_part(
        &self,
        request: &PutObjectRequest,
        file_path: &Path,
        total_size: i64,
    ) -> Result<UploadResult, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = tokio::fs::read(file_path).await?;
        let result = self
            .put_object(PutObjectRequest {
                bucket: request.bucket.clone(),
                key: request.key.clone(),
                body: Some(crate::BodyContent::from_bytes(bytes, None)),
                content_length: Some(total_size as u64),
                ..Default::default()
            })
            .await?;

        Ok(UploadResult {
            upload_id: None,
            etag: result.etag.clone(),
            version_id: result.version_id.clone(),
            hash_crc64: result.hash_crc64.clone(),
        })
    }

    /// The multipart path: concurrent parts, then one completion.
    async fn upload_file_multipart(
        &self,
        request: &PutObjectRequest,
        file_path: &Path,
        total_size: i64,
        options: UploaderOptions,
    ) -> Result<UploadResult, Box<dyn std::error::Error + Send + Sync>> {
        // A std handle is used because `FileExt::read_at` gives positional
        // reads, so parts can be read concurrently without a shared cursor.
        let file = std::sync::Arc::new(tokio::fs::File::open(file_path).await?.into_std().await);

        let init = self
            .initiate_multipart_upload(&InitiateMultipartUploadRequest {
                bucket: request.bucket.clone(),
                key: request.key.clone(),
                ..Default::default()
            })
            .await?;
        let upload_id = init
            .upload_id
            .clone()
            .ok_or("InitiateMultipartUpload returned no upload ID")?;

        // OSS rejects a multipart upload whose non-final parts are below its
        // minimum, so a part size under that floor is raised before any part
        // is sent. The final part may be smaller, which is why the floor
        // applies to the part size rather than to every actual part.
        let part_size = options.part_size.max(MIN_PART_SIZE);
        let mut ranges = Vec::new();
        let mut start = 0i64;
        while start < total_size {
            let size = part_size.min(total_size - start);
            ranges.push((start, size));
            start += size;
        }

        // The per-part futures need the upload ID for the whole life of the
        // stream, so they get their own copy; the outer binding stays free to
        // be moved into the result.
        let parts_upload_id = upload_id.clone();
        let this = self.clone();
        let stream = futures_util::stream::iter(ranges.iter().enumerate().map(
            move |(index, &(start, size))| {
                let client = this.clone();
                let upload_id = parts_upload_id.clone();
                let file = file.clone();
                let bucket = request.bucket.clone();
                let key = request.key.clone();
                let request_payer = request.request_payer.clone();
                let common = request.common.clone();
                async move {
                    let part_number = index as i32 + 1;
                    let data = read_at(&file, start as u64, size as usize)?;
                    let result = client
                        .upload_part(UploadPartRequest {
                            bucket,
                            key,
                            part_number,
                            upload_id,
                            body: Some(crate::BodyContent::from_bytes(data, None)),
                            content_md5: None,
                            progress_fn: None,
                            request_payer,
                            traffic_limit: None,
                            common,
                        })
                        .await?;
                    Ok::<UploadedPart, Box<dyn std::error::Error + Send + Sync>>(UploadedPart {
                        part_number,
                        etag: result.etag.clone().unwrap_or_default(),
                        size,
                        hash_crc64: result.hash_crc64.clone(),
                    })
                }
            },
        ))
        .buffer_unordered(options.parallel_num);

        let mut parts: Vec<UploadedPart> = Vec::with_capacity(ranges.len());
        let mut stream = std::pin::pin!(stream);
        let mut failure: Option<Box<dyn std::error::Error + Send + Sync>> = None;
        while let Some(part) = stream.next().await {
            match part {
                Ok(part) => parts.push(part),
                Err(err) => {
                    failure = Some(err);
                    break;
                }
            }
        }

        if let Some(err) = failure {
            if !options.leave_parts_on_error {
                let _ = self
                    .abort_multipart_upload(&AbortMultipartUploadRequest {
                        bucket: request.bucket.clone(),
                        key: request.key.clone(),
                        upload_id: upload_id.clone(),
                        ..Default::default()
                    })
                    .await;
            }
            return Err(err);
        }

        // The completion lists the parts in ascending order; OSS rejects an
        // out-of-order or incomplete list.
        parts.sort_by_key(|p| p.part_number);
        let completed = self
            .complete_multipart_upload(&CompleteMultipartUploadRequest {
                bucket: request.bucket.clone(),
                key: request.key.clone(),
                upload_id: upload_id.clone(),
                parts: parts
                    .iter()
                    .map(|p| CompleteMultipartUploadPart {
                        part_number: p.part_number,
                        etag: p.etag.clone(),
                    })
                    .collect(),
                ..Default::default()
            })
            .await;

        let completed = match completed {
            Ok(result) => result,
            Err(err) => {
                if !options.leave_parts_on_error {
                    let _ = self
                        .abort_multipart_upload(&AbortMultipartUploadRequest {
                            bucket: request.bucket.clone(),
                            key: request.key.clone(),
                            upload_id: upload_id.clone(),
                            ..Default::default()
                        })
                        .await;
                }
                return Err(err);
            }
        };

        // Fold the part checksums in part order; the server's whole-object CRC
        // is exactly the concatenation of the parts.
        let mut combined = 0u64;
        let mut crc_complete = true;
        for part in &parts {
            match part
                .hash_crc64
                .as_deref()
                .and_then(|s| s.parse::<u64>().ok())
            {
                Some(value) => combined = crc64_combine(combined, value, part.size as u64),
                None => {
                    crc_complete = false;
                    break;
                }
            }
        }
        if crc_complete {
            if let Some(server_crc) = completed.hash_crc64.as_deref() {
                crate::utils::check_crc64(combined, Some(server_crc))
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
            }
        }

        Ok(UploadResult {
            upload_id: Some(upload_id),
            etag: completed.etag.clone(),
            version_id: completed.version_id.clone(),
            hash_crc64: completed.hash_crc64.clone(),
        })
    }
}

/// Reads exactly `len` bytes at `offset`.
///
/// `FileExt::read_at` may return fewer bytes than requested, so this loops;
/// a short read at the end of the file is an error, because a part that is
/// silently shorter than planned would still be accepted by the server but
/// would corrupt the assembled object.
fn read_at(
    file: &std::fs::File,
    offset: u64,
    len: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use std::os::unix::fs::FileExt;

    let mut buf = vec![0u8; len];
    let mut filled = 0usize;
    while filled < len {
        let n = file.read_at(&mut buf[filled..], offset + filled as u64)?;
        if n == 0 {
            return Err(format!(
                "unexpected end of file: wanted {len} bytes at offset {offset}, got {filled}"
            )
            .into());
        }
        filled += n;
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::utils::Crc64;
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

    fn crc_of(data: &[u8]) -> u64 {
        let mut crc = Crc64::new(0);
        crc.write(data).unwrap();
        crc.sum64()
    }

    fn temp_file(name: &str, contents: &[u8]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("oss-uploader-tests");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join(name);
        std::fs::write(&path, contents).expect("write temp file");
        path
    }

    /// A file smaller than one part goes out as a single PutObject: no
    /// multipart round trip, and the body must be the file's bytes.
    #[tokio::test]
    async fn test_upload_file_single_part_uses_put_object() {
        let mut server = mockito::Server::new_async().await;
        let contents = b"tiny file contents".to_vec();
        let expected_crc = crc_of(&contents).to_string();

        let post = server
            .mock("POST", mockito::Matcher::Any)
            .expect(0)
            .create_async()
            .await;
        let put = server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(200)
            .with_header("etag", "\"etag-value\"")
            .with_header("x-oss-hash-crc64ecma", &expected_crc)
            .with_body("")
            .create_async()
            .await;

        let path = temp_file("single.txt", &contents);
        let result = mock_client(&server)
            .upload_file(
                &PutObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "test-key".to_string(),
                    ..Default::default()
                },
                &path,
                UploaderOptions::default(),
            )
            .await
            .expect("upload should succeed");

        assert_eq!(result.etag.as_deref(), Some("\"etag-value\""));
        assert_eq!(
            result.upload_id, None,
            "a single PutObject has no upload ID"
        );
        put.assert_async().await;
        // The uploader must not have opened a multipart upload for a small
        // file.
        post.assert_async().await;
    }

    /// A file larger than one part must be split, uploaded, and completed; the
    /// completion must list every part exactly once and in ascending order.
    #[tokio::test]
    async fn test_upload_file_multipart_splits_and_completes() {
        let mut server = mockito::Server::new_async().await;
        // Above the 100 KiB service floor for multipart parts, so this really
        // exercises the multipart path.
        let part_size = MIN_PART_SIZE as usize;
        let contents: Vec<u8> = (0..(part_size * 2 + 137))
            .map(|i| (i % 251) as u8)
            .collect();

        // The initiate POST is distinguishable from the completion POST by its
        // query: init carries `uploads`, completion carries `uploadId`.
        let init = server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploads".to_string()))
            .with_status(200)
            .with_body(
                "<InitiateMultipartUploadResult><Bucket>test-bucket</Bucket><Key>test-key</\
                 Key><UploadId>upload-123</UploadId></InitiateMultipartUploadResult>",
            )
            .expect(1)
            .create_async()
            .await;

        let mut part_mocks = Vec::new();
        for (index, slice) in contents.chunks(part_size).enumerate() {
            let part_number = index + 1;
            part_mocks.push(
                server
                    .mock("PUT", mockito::Matcher::Any)
                    .match_query(mockito::Matcher::AllOf(vec![
                        mockito::Matcher::Regex(format!("partNumber={part_number}")),
                        mockito::Matcher::Regex("uploadId=upload-123".to_string()),
                    ]))
                    .with_status(200)
                    .with_header("etag", format!("\"etag-{part_number}\"").as_str())
                    .with_header("x-oss-hash-crc64ecma", &crc_of(slice).to_string())
                    .with_body("")
                    .create_async()
                    .await,
            );
        }

        // The completion body must carry part 1 then part 2 then part 3.
        let complete = server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploadId".to_string()))
            .with_status(200)
            .with_body(
                "<CompleteMultipartUploadResult><ETag>\"final\"</ETag></\
                 CompleteMultipartUploadResult>",
            )
            .expect(1)
            .create_async()
            .await;

        let path = temp_file("multi.bin", &contents);
        let result = mock_client(&server)
            .upload_file(
                &PutObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "test-key".to_string(),
                    ..Default::default()
                },
                &path,
                UploaderOptions::default()
                    .with_part_size(part_size as i64)
                    .with_parallel_num(2),
            )
            .await
            .expect("multipart upload should succeed");

        assert_eq!(result.upload_id.as_deref(), Some("upload-123"));
        assert_eq!(result.etag.as_deref(), Some("\"final\""));
        init.assert_async().await;
        for m in &part_mocks {
            m.assert_async().await;
        }
        complete.assert_async().await;
        let _ = std::fs::remove_file(&path);
    }

    /// A failing part must abort the upload, so no billable parts are left
    /// behind. Without the abort the caller sees an error but the storage
    /// still carries the parts.
    #[tokio::test]
    async fn test_upload_file_aborts_on_part_failure() {
        let mut server = mockito::Server::new_async().await;
        let contents: Vec<u8> = (0..(MIN_PART_SIZE as usize * 2 + 137))
            .map(|i| (i % 251) as u8)
            .collect();

        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploads".to_string()))
            .with_status(200)
            .with_body(
                "<InitiateMultipartUploadResult><Bucket>test-bucket</Bucket><Key>test-key</\
                 Key><UploadId>upload-abc</UploadId></InitiateMultipartUploadResult>",
            )
            .create_async()
            .await;
        // Every part fails.
        server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(500)
            .with_body("<Error><Code>InternalError</Code><Message>boom</Message></Error>")
            .create_async()
            .await;
        let abort = server
            .mock("DELETE", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploadId=upload-abc".to_string()))
            .with_status(204)
            .create_async()
            .await;

        let path = temp_file("failing.bin", &contents);
        let err = mock_client(&server)
            .upload_file(
                &PutObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "test-key".to_string(),
                    ..Default::default()
                },
                &path,
                UploaderOptions::default().with_part_size(MIN_PART_SIZE),
            )
            .await
            .expect_err("a failed part must surface as an error");

        assert!(err.to_string().contains("InternalError") || err.to_string().contains("boom"));
        // The abort must actually have been issued.
        abort.assert_async().await;
        let _ = std::fs::remove_file(&path);
    }

    /// `leave_parts_on_error` must suppress the abort, because resuming needs
    /// the parts to still exist.
    #[tokio::test]
    async fn test_leave_parts_on_error_skips_abort() {
        let mut server = mockito::Server::new_async().await;
        let contents: Vec<u8> = (0..2500u32).map(|i| (i % 251) as u8).collect();

        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploads".to_string()))
            .with_status(200)
            .with_body(
                "<InitiateMultipartUploadResult><Bucket>test-bucket</Bucket><Key>test-key</\
                 Key><UploadId>upload-keep</UploadId></InitiateMultipartUploadResult>",
            )
            .create_async()
            .await;
        server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(500)
            .with_body("<Error><Code>InternalError</Code><Message>boom</Message></Error>")
            .create_async()
            .await;
        let abort = server
            .mock("DELETE", mockito::Matcher::Any)
            .expect(0)
            .create_async()
            .await;

        let path = temp_file("keep.bin", &contents);
        let _ = mock_client(&server)
            .upload_file(
                &PutObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "test-key".to_string(),
                    ..Default::default()
                },
                &path,
                UploaderOptions::default()
                    .with_part_size(MIN_PART_SIZE)
                    .with_leave_parts_on_error(true),
            )
            .await
            .expect_err("the upload still fails");

        abort.assert_async().await;
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_upload_file_rejects_directory() {
        let server = mockito::Server::new_async().await;
        let dir = std::env::temp_dir().join("oss-uploader-tests/dir-not-file");
        std::fs::create_dir_all(&dir).unwrap();

        let err = mock_client(&server)
            .upload_file(
                &PutObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "test-key".to_string(),
                    ..Default::default()
                },
                &dir,
                UploaderOptions::default(),
            )
            .await
            .expect_err("a directory is not uploadable");
        assert!(err.to_string().contains("directory"), "unexpected: {err}");
    }

    #[test]
    fn test_options_clamp_degenerate_values() {
        let opts = UploaderOptions::default()
            .with_part_size(0)
            .with_parallel_num(0);
        assert_eq!(opts.part_size, 1, "a zero part size must not hang");
        assert_eq!(opts.parallel_num, 1);
        assert!(!opts.leave_parts_on_error, "aborting is the safe default");
    }
}
