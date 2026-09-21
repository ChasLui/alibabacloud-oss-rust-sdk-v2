//! A streaming writer that turns sequential writes into a multipart upload.
//!
//! Mirrors Go `oss/filelike_writeonly.go`. Bytes are buffered until a part is
//! full, at which point the part is uploaded in the background while the
//! caller keeps writing. Closing the handle commits the upload; a small object
//! that never filled a part is written with a single `PutObject` instead.
//!
//! Three properties are worth stating because they are not visible from the
//! signatures:
//!
//! - Only whole parts are dispatched. A part that is still being buffered is
//!   never sent, because the next write belongs in it; that also means the
//!   last part is short, and it is dispatched by `close`.
//! - Background failures are sticky. A worker that fails leaves the handle in
//!   a failed state that every later call reports, so a caller cannot keep
//!   writing into an upload that is already doomed and get a silent success.
//! - The durable position ([`WriteOnlyFile::stat_checkpoint`]) only advances
//!   over a contiguous run of completed parts. A part that finished out of
//!   order is uploaded but not yet durable, so a resume would have to
//!   re-upload from the first gap.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use futures_util::stream::{FuturesUnordered, StreamExt};

use crate::api::object::{
    AbortMultipartUploadRequest, CompleteMultipartUploadPart, CompleteMultipartUploadRequest,
    InitiateMultipartUploadRequest, PutObjectRequest, UploadPartRequest,
};
use crate::client::Client;
use crate::utils::crc64_combine;
use crate::utils::Crc64;
use crate::{DEFAULT_UPLOAD_PARALLEL, DEFAULT_UPLOAD_PART_SIZE, MIN_PART_SIZE};

/// Options for [`Client::open_write_only_file`].
pub struct WriteOnlyOptions {
    /// The multipart part size. Defaults to 6 MiB, matching Go's
    /// `DefaultUploadPartSize`.
    pub part_size: i64,

    /// How many parts are uploaded at once. Defaults to 3, matching Go's
    /// `DefaultUploadParallel`.
    pub parallel_num: usize,

    /// The attributes applied when the object is created.
    ///
    /// Attributes without a multipart equivalent (such as the ACL) apply on
    /// both paths; the small-object path passes the whole request through.
    pub create_parameter: Option<PutObjectRequest>,
}

impl Default for WriteOnlyOptions {
    fn default() -> Self {
        WriteOnlyOptions {
            part_size: DEFAULT_UPLOAD_PART_SIZE,
            parallel_num: DEFAULT_UPLOAD_PARALLEL.max(1) as usize,
            create_parameter: None,
        }
    }
}

impl WriteOnlyOptions {
    /// Sets the part size. Values below 1 are clamped to 1 byte; the service's
    /// 100 KiB floor is applied when the part is dispatched, not here.
    pub fn with_part_size(mut self, part_size: i64) -> Self {
        self.part_size = part_size.max(1);
        self
    }

    /// Sets how many parts are uploaded concurrently. Zero is clamped to 1.
    pub fn with_parallel_num(mut self, parallel_num: usize) -> Self {
        self.parallel_num = parallel_num.max(1);
        self
    }

    /// Sets the attributes applied when the object is created.
    pub fn with_create_parameter(mut self, create_parameter: PutObjectRequest) -> Self {
        self.create_parameter = Some(create_parameter);
        self
    }
}

/// A snapshot of how far the upload has durably progressed.
///
/// Mirrors Go's `WriteCheckpoint`. The values describe a contiguous prefix of
/// the object; anything past [`WriteOnlyFile::stat_checkpoint`]'s `offset` is
/// either still buffered or dispatched but not yet part of that prefix.
#[derive(Debug, Clone, Default)]
pub struct WriteCheckpoint {
    /// The upload ID, empty before the multipart upload is initiated.
    pub upload_id: String,
    /// A contiguous durable prefix `[0, offset)`; always a multiple of the
    /// part size.
    pub offset: i64,
    /// The CRC64 of `[0, offset)`.
    pub crc64: u64,
    /// The part size the upload ID is bound to.
    pub part_size: i64,
    /// The most recent unrecoverable error, when the handle is failing.
    pub last_error: Option<String>,
}

/// One part that has finished uploading.
#[derive(Debug, Clone)]
struct DonePart {
    etag: String,
    crc64: u64,
    size: i64,
}

/// The outcome of one dispatched part.
type PartOutcome = (
    i32,
    i64,
    Result<(String, u64), Box<dyn std::error::Error + Send + Sync>>,
);

/// A streaming writer over an object.
///
/// Writes are buffered into parts and uploaded as they fill, so memory stays
/// bounded by the part size times the number of in-flight parts regardless of
/// how much is written.
pub struct WriteOnlyFile {
    client: Client,
    bucket: String,
    key: String,
    request_payer: Option<String>,
    create_parameter: Option<PutObjectRequest>,
    part_size: i64,
    parallel_num: usize,

    /// The part being filled.
    buffer: Vec<u8>,
    /// The number of bytes already written into the object, including the
    /// bytes still buffered.
    write_cursor: i64,
    /// The number of parts dispatched so far, so the next part number is this
    /// plus one.
    assigned_parts: i32,

    /// Parts that finished uploading, by part number.
    done_parts: HashMap<i32, DonePart>,
    /// The highest contiguous part number whose parts are all complete. The
    /// durable prefix ends after this part.
    next_contiguous_part: i32,
    /// In-flight part uploads. Holding them in the handle rather than spawning
    /// tasks keeps the client's `Rc`-based state usable — the futures are
    /// polled by this handle's own calls.
    in_flight: FuturesUnordered<Pin<Box<dyn Future<Output = PartOutcome>>>>,

    upload_id: String,
    initiated: bool,
    /// The first unrecoverable background error.
    sticky_error: Option<String>,
    closed: bool,
}

impl WriteOnlyFile {
    /// The current write position, which is the number of bytes written so far.
    pub fn offset(&self) -> i64 {
        self.write_cursor
    }

    /// The upload ID, empty until the multipart upload is initiated.
    pub fn upload_id(&self) -> &str {
        &self.upload_id
    }

    /// How far the upload has durably progressed.
    ///
    /// The offset stops at the first gap, so a resume re-uploads from there
    /// rather than trusting parts that are not contiguous with the prefix.
    pub fn stat_checkpoint(&self) -> WriteCheckpoint {
        let mut offset = 0i64;
        let mut crc64 = 0u64;
        let mut part = 1i32;
        while let Some(done) = self.done_parts.get(&part) {
            crc64 = crc64_combine(crc64, done.crc64, done.size as u64);
            offset += done.size;
            part += 1;
        }
        WriteCheckpoint {
            upload_id: self.upload_id.clone(),
            offset,
            crc64,
            part_size: self.part_size,
            last_error: self.sticky_error.clone(),
        }
    }

    /// Buffers `data`, dispatching parts as they fill.
    ///
    /// Returns the new write position.
    pub async fn write(
        &mut self,
        data: Vec<u8>,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        self.check_valid()?;

        if let Some(error) = &self.sticky_error {
            return Err(error.clone().into());
        }

        let mut remaining = data.as_slice();
        while !remaining.is_empty() {
            let space = self.part_size as usize - self.buffer.len();
            let take = space.min(remaining.len());
            self.buffer.extend_from_slice(&remaining[..take]);
            remaining = &remaining[take..];
            self.write_cursor += take as i64;

            if self.buffer.len() as i64 == self.part_size {
                self.seal_full_part().await?;
            }
        }
        Ok(self.write_cursor)
    }

    /// Commits the object.
    ///
    /// The buffered bytes become the final part. An object that never filled a
    /// part is written with a single `PutObject`, so small writes do not open
    /// a multipart upload at all.
    pub async fn close(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;

        // Let every dispatched part report before deciding, so a failure that
        // already happened is not lost behind a successful completion.
        self.drain_in_flight().await?;
        if let Some(error) = self.sticky_error.clone() {
            return Err(error.into());
        }

        if !self.initiated {
            return self.put_single_object().await;
        }

        if !self.buffer.is_empty() {
            let part_number = self.assigned_parts + 1;
            let buffer = std::mem::take(&mut self.buffer);
            let size = buffer.len() as i64;
            self.dispatch(part_number, buffer, size);
            self.assigned_parts = part_number;
            self.drain_in_flight().await?;
            if let Some(error) = self.sticky_error.clone() {
                return Err(error.into());
            }
        }

        self.complete().await
    }

    /// Aborts the multipart upload, discarding everything written.
    pub async fn abort_close(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        self.drain_in_flight().await.ok();

        if !self.initiated {
            return Ok(());
        }
        self.client
            .abort_multipart_upload(&AbortMultipartUploadRequest {
                bucket: self.bucket.clone(),
                key: self.key.clone(),
                upload_id: self.upload_id.clone(),
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    /// Rejects use after close and after a background failure.
    fn check_valid(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.closed {
            return Err("file is closed".into());
        }
        if let Some(error) = &self.sticky_error {
            return Err(error.clone().into());
        }
        Ok(())
    }

    /// Uploads the buffered part without waiting for it.
    async fn seal_full_part(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.ensure_initiated().await?;
        let part_number = self.assigned_parts + 1;
        let buffer = std::mem::take(&mut self.buffer);
        let size = buffer.len() as i64;
        self.dispatch(part_number, buffer, size);
        self.assigned_parts = part_number;
        // Bound the number of in-flight parts, which is what keeps memory
        // usage proportional to the parallelism rather than to the object.
        while self.in_flight.len() >= self.parallel_num {
            self.collect_one().await?;
        }
        if let Some(error) = &self.sticky_error {
            return Err(error.clone().into());
        }
        Ok(())
    }

    /// Queues one part upload.
    fn dispatch(&mut self, part_number: i32, buffer: Vec<u8>, size: i64) {
        let client = self.client.clone();
        let bucket = self.bucket.clone();
        let key = self.key.clone();
        let upload_id = self.upload_id.clone();
        let request_payer = self.request_payer.clone();
        self.in_flight.push(Box::pin(async move {
            let mut crc = Crc64::new(0);
            if let Err(err) = crc.write(&buffer) {
                return (part_number, size, Err(err.to_string().into()));
            }
            let local_crc = crc.sum64();

            let result = client
                .upload_part(UploadPartRequest {
                    bucket,
                    key,
                    part_number,
                    upload_id,
                    body: Some(crate::BodyContent::from_bytes(buffer, None)),
                    content_md5: None,
                    progress_fn: None,
                    request_payer,
                    traffic_limit: None,
                    common: Default::default(),
                })
                .await;

            match result {
                Ok(result) => (
                    part_number,
                    size,
                    Ok((result.etag.clone().unwrap_or_default(), local_crc)),
                ),
                Err(err) => (part_number, size, Err(err)),
            }
        }));
    }

    /// Waits for one dispatched part to finish.
    async fn collect_one(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self.in_flight.next().await {
            Some((part_number, size, Ok((etag, crc64)))) => {
                self.done_parts
                    .insert(part_number, DonePart { etag, crc64, size });
                self.advance_contiguous();
                Ok(())
            }
            Some((part_number, _size, Err(err))) => {
                // The failure is sticky: the upload cannot be completed, and
                // every later call must say so instead of writing on. The raw
                // service error is folded into the message so the cause is not
                // lost.
                self.sticky_error = Some(format!("upload part {part_number} failed: {err}"));
                Ok(())
            }
            None => Ok(()),
        }
    }

    /// Waits for every dispatched part to finish.
    async fn drain_in_flight(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        while !self.in_flight.is_empty() {
            self.collect_one().await?;
        }
        Ok(())
    }

    /// Takes one outcome if one is already available.
    ///
    /// Polling once and dropping the future leaves the uploads untouched; it
    /// only reports outcomes that have already landed.
    fn try_next_part(&mut self) -> Option<PartOutcome> {
        use futures_util::FutureExt;
        self.in_flight.next().now_or_never().flatten()
    }

    /// Consumes whatever part outcomes are already available.
    ///
    /// Polling stops as soon as nothing is ready, so a part still uploading
    /// does not block the next write; only an outcome that has already landed
    /// is acted on.
    fn poll_completed_parts(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        while let Some(outcome) = self.try_next_part() {
            match outcome {
                (part_number, size, Ok((etag, crc64))) => {
                    self.done_parts.insert(part_number, DonePart { etag, crc64, size });
                    self.advance_contiguous();
                }
                (part_number, _size, Err(err)) => {
                    let message = format!("upload part {part_number} failed: {err}");
                    self.sticky_error = Some(message.clone());
                    return Err(message.into());
                }
            }
        }
        Ok(())
    }

    /// Advances the durable prefix over any run of completed parts.
    fn advance_contiguous(&mut self) {
        while self.done_parts.contains_key(&self.next_contiguous_part) {
            self.next_contiguous_part += 1;
        }
    }

    /// Starts the multipart upload on first use.
    async fn ensure_initiated(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.initiated {
            return Ok(());
        }
        let request = self.initiate_request();
        let result = self.client.initiate_multipart_upload(&request).await?;
        self.upload_id = result
            .upload_id
            .clone()
            .ok_or("InitiateMultipartUpload returned no upload ID")?;
        self.initiated = true;
        Ok(())
    }

    /// Builds the initiate request from the create parameter.
    fn initiate_request(&self) -> InitiateMultipartUploadRequest {
        let mut request = InitiateMultipartUploadRequest {
            bucket: self.bucket.clone(),
            key: self.key.clone(),
            request_payer: self.request_payer.clone(),
            ..Default::default()
        };
        if let Some(create) = &self.create_parameter {
            request.forbid_overwrite = create.forbid_overwrite.clone();
            request.server_side_encryption = create.server_side_encryption.clone();
            request.server_side_data_encryption = create.server_side_data_encryption.clone();
            request.sse_kms_key_id = create.sse_kms_key_id.clone();
            request.object_acl = create.object_acl.clone();
            request.storage_class = create.storage_class.clone();
            request.cache_control = create.cache_control.clone();
            request.content_disposition = create.content_disposition.clone();
            request.content_encoding = create.content_encoding.clone();
            request.content_type = create.content_type.clone();
            request.expires = create.expires.clone();
            request.metadata = create.metadata.clone();
            request.tagging = create.tagging.clone();
            request.common = create.common.clone();
        }
        request
    }

    /// Writes the buffered bytes as a single object, for a write that never
    /// filled a part.
    async fn put_single_object(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let body = std::mem::take(&mut self.buffer);
        let mut request = match self.create_parameter.as_ref() {
            Some(create) => PutObjectRequest {
                bucket: self.bucket.clone(),
                key: self.key.clone(),
                cache_control: create.cache_control.clone(),
                content_disposition: create.content_disposition.clone(),
                content_encoding: create.content_encoding.clone(),
                content_type: create.content_type.clone(),
                expires: create.expires.clone(),
                forbid_overwrite: create.forbid_overwrite.clone(),
                server_side_encryption: create.server_side_encryption.clone(),
                server_side_data_encryption: create.server_side_data_encryption.clone(),
                sse_kms_key_id: create.sse_kms_key_id.clone(),
                object_acl: create.object_acl.clone(),
                storage_class: create.storage_class.clone(),
                tagging: create.tagging.clone(),
                metadata: create.metadata.clone(),
                request_payer: self.request_payer.clone(),
                common: create.common.clone(),
                ..Default::default()
            },
            None => PutObjectRequest {
                bucket: self.bucket.clone(),
                key: self.key.clone(),
                request_payer: self.request_payer.clone(),
                ..Default::default()
            },
        };
        // The body is owned here; a copied length or callback from the caller
        // would describe a different upload.
        request.content_length = None;
        request.progress_fn = None;
        request.body = Some(crate::BodyContent::from_bytes(body, None));
        self.client.put_object(request).await?;
        Ok(())
    }

    /// Commits the multipart upload.
    async fn complete(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut parts: Vec<CompleteMultipartUploadPart> = self
            .done_parts
            .iter()
            .map(|(part_number, done)| CompleteMultipartUploadPart {
                part_number: *part_number,
                etag: done.etag.clone(),
            })
            .collect();
        // OSS rejects an out-of-order or incomplete list.
        parts.sort_by_key(|part| part.part_number);

        let mut request = CompleteMultipartUploadRequest {
            bucket: self.bucket.clone(),
            key: self.key.clone(),
            upload_id: self.upload_id.clone(),
            parts,
            request_payer: self.request_payer.clone(),
            ..Default::default()
        };
        // The ACL has no multipart equivalent, so it is applied here.
        if let Some(create) = &self.create_parameter {
            request.object_acl = Some(create.object_acl.clone());
        }
        self.client.complete_multipart_upload(&request).await?;
        Ok(())
    }
}

impl Client {
    /// Opens a streaming writer over an object.
    ///
    /// Mirrors Go `NewWriteOnlyFile`. The handle makes no request until the
    /// first part is dispatched, so opening is cheap.
    pub fn open_write_only_file(
        &self,
        bucket: &str,
        key: &str,
        options: WriteOnlyOptions,
    ) -> WriteOnlyFile {
        let request_payer = options
            .create_parameter
            .as_ref()
            .and_then(|create| create.request_payer.clone());
        WriteOnlyFile {
            client: self.clone(),
            bucket: bucket.to_string(),
            key: key.to_string(),
            request_payer,
            create_parameter: options.create_parameter,
            // The service rejects multipart parts below its floor, so a part
            // size under it is raised before any part is dispatched.
            part_size: options.part_size.max(MIN_PART_SIZE),
            parallel_num: options.parallel_num.max(1),
            buffer: Vec::new(),
            write_cursor: 0,
            assigned_parts: 0,
            done_parts: HashMap::new(),
            next_contiguous_part: 1,
            in_flight: FuturesUnordered::new(),
            upload_id: String::new(),
            initiated: false,
            sticky_error: None,
            closed: false,
        }
    }
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

    /// A small write must not open a multipart upload at all; it is one
    /// `PutObject` on close.
    #[tokio::test]
    async fn test_write_only_file_small_write_uses_put_object() {
        let mut server = mockito::Server::new_async().await;
        let put = server
            .mock("PUT", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Missing)
            .with_status(200)
            .with_header("etag", "\"small\"")
            .expect(1)
            .create_async()
            .await;
        let init = server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploads".to_string()))
            .with_status(200)
            .expect(0)
            .create_async()
            .await;

        let mut file = mock_client(&server).open_write_only_file(
            "test-bucket",
            "small-key",
            WriteOnlyOptions::default(),
        );
        file.write(b"hello".to_vec())
            .await
            .expect("write should succeed");
        assert_eq!(file.offset(), 5);
        file.close().await.expect("close should succeed");

        put.assert_async().await;
        init.assert_async().await;
    }

    /// Writing more than one part must dispatch each full part and commit the
    /// tail as the last part.
    #[tokio::test]
    async fn test_write_only_file_streams_parts_and_completes() {
        let mut server = mockito::Server::new_async().await;
        let part_size = MIN_PART_SIZE;

        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploads".to_string()))
            .with_status(200)
            .with_body("<InitiateMultipartUploadResult><UploadId>up-1</UploadId></InitiateMultipartUploadResult>")
            .expect(1)
            .create_async()
            .await;

        let mut part_mocks = Vec::new();
        for part_number in 1..=3 {
            part_mocks.push(
                server
                    .mock("PUT", mockito::Matcher::Any)
                    .match_query(mockito::Matcher::AllOf(vec![
                        mockito::Matcher::Regex(format!("partNumber={part_number}")),
                        mockito::Matcher::Regex("uploadId=up-1".to_string()),
                    ]))
                    .with_status(200)
                    .with_header("etag", format!("\"p{part_number}\"").as_str())
                    .expect(1)
                    .create_async()
                    .await,
            );
        }
        let complete = server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploadId=up-1".to_string()))
            .with_status(200)
            .with_body("<CompleteMultipartUploadResult><ETag>\"final\"</ETag></CompleteMultipartUploadResult>")
            .expect(1)
            .create_async()
            .await;

        let mut file = mock_client(&server).open_write_only_file(
            "test-bucket",
            "stream-key",
            WriteOnlyOptions::default()
                .with_part_size(part_size)
                .with_parallel_num(2),
        );

        // Two full parts plus a short tail.
        let mut data = vec![0u8; part_size as usize * 2];
        data.extend_from_slice(b"tail");
        file.write(data).await.expect("write should succeed");
        assert_eq!(file.offset(), part_size * 2 + 4);
        file.close().await.expect("close should succeed");

        assert_eq!(file.upload_id(), "up-1");
        for m in &part_mocks {
            m.assert_async().await;
        }
        complete.assert_async().await;
    }

    /// A failing part must make the handle report the failure, and the failure
    /// must stay reported: a caller cannot write past it and then see a
    /// successful close.
    #[tokio::test]
    async fn test_write_only_file_sticky_error_after_part_failure() {
        let mut server = mockito::Server::new_async().await;
        let part_size = MIN_PART_SIZE;

        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploads".to_string()))
            .with_status(200)
            .with_body("<InitiateMultipartUploadResult><UploadId>up-err</UploadId></InitiateMultipartUploadResult>")
            .create_async()
            .await;
        server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(500)
            .with_body("<Error><Code>InternalError</Code><Message>boom</Message></Error>")
            .create_async()
            .await;

        let mut file = mock_client(&server).open_write_only_file(
            "test-bucket",
            "error-key",
            WriteOnlyOptions::default()
                .with_part_size(part_size)
                .with_parallel_num(1),
        );

        // With one in-flight slot the second full part forces the handle to
        // wait for the first, which is where the failure lands.
        let outcome = file.write(vec![0u8; part_size as usize * 2]).await;
        if let Err(err) = &outcome {
            assert!(err.to_string().contains("upload part 1 failed"), "{err}");
        }

        // Whatever the write observed, the handle must not commit afterwards.
        let err = file
            .close()
            .await
            .expect_err("close must report the failed upload");
        assert!(err.to_string().contains("upload part 1 failed"), "{err}");
    }

    /// Closing after a failure must not report success.
    #[tokio::test]
    async fn test_write_only_file_close_reports_failure() {
        let mut server = mockito::Server::new_async().await;
        let part_size = MIN_PART_SIZE;

        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploads".to_string()))
            .with_status(200)
            .with_body("<InitiateMultipartUploadResult><UploadId>up-fail</UploadId></InitiateMultipartUploadResult>")
            .create_async()
            .await;
        server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(500)
            .with_body("<Error><Code>InternalError</Code><Message>boom</Message></Error>")
            .create_async()
            .await;
        // No completion may be attempted once a part failed.
        let complete = server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploadId=up-fail".to_string()))
            .with_status(200)
            .expect(0)
            .create_async()
            .await;

        let mut file = mock_client(&server).open_write_only_file(
            "test-bucket",
            "close-fail-key",
            WriteOnlyOptions::default().with_part_size(part_size),
        );
        file.write(vec![0u8; part_size as usize])
            .await
            .expect("the first write only dispatches");

        let err = file
            .close()
            .await
            .expect_err("close must report the failure");
        assert!(err.to_string().contains("upload part 1 failed"), "{err}");
        complete.assert_async().await;
    }

    /// Aborting must discard the upload rather than committing it.
    #[tokio::test]
    async fn test_write_only_file_abort_close() {
        let mut server = mockito::Server::new_async().await;
        let part_size = MIN_PART_SIZE;

        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploads".to_string()))
            .with_status(200)
            .with_body("<InitiateMultipartUploadResult><UploadId>up-abort</UploadId></InitiateMultipartUploadResult>")
            .create_async()
            .await;
        server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(200)
            .with_header("etag", "\"p1\"")
            .create_async()
            .await;
        let abort = server
            .mock("DELETE", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploadId=up-abort".to_string()))
            .with_status(204)
            .expect(1)
            .create_async()
            .await;
        let complete = server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploadId=up-abort".to_string()))
            .with_status(200)
            .expect(0)
            .create_async()
            .await;

        let mut file = mock_client(&server).open_write_only_file(
            "test-bucket",
            "abort-key",
            WriteOnlyOptions::default().with_part_size(part_size),
        );
        file.write(vec![0u8; part_size as usize])
            .await
            .expect("write should succeed");

        file.abort_close().await.expect("abort should succeed");
        abort.assert_async().await;
        complete.assert_async().await;
    }

    /// The durable prefix must stop at the first gap, so a resume does not
    /// trust a part that is not contiguous with it.
    #[tokio::test]
    async fn test_write_only_file_checkpoint_stops_at_gap() {
        let mut server = mockito::Server::new_async().await;
        let part_size = MIN_PART_SIZE;

        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploads".to_string()))
            .with_status(200)
            .with_body("<InitiateMultipartUploadResult><UploadId>up-gap</UploadId></InitiateMultipartUploadResult>")
            .create_async()
            .await;
        // Part 1 fails, part 2 succeeds: the prefix must stay empty.
        server
            .mock("PUT", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("partNumber=1".to_string()))
            .with_status(500)
            .with_body("<Error><Code>InternalError</Code><Message>boom</Message></Error>")
            .create_async()
            .await;
        server
            .mock("PUT", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("partNumber=2".to_string()))
            .with_status(200)
            .with_header("etag", "\"p2\"")
            .create_async()
            .await;

        let mut file = mock_client(&server).open_write_only_file(
            "test-bucket",
            "gap-key",
            WriteOnlyOptions::default()
                .with_part_size(part_size)
                .with_parallel_num(2),
        );
        let _ = file.write(vec![0u8; part_size as usize * 2]).await;

        // Whatever finished, the durable prefix cannot include part 2 while
        // part 1 is missing.
        let checkpoint = file.stat_checkpoint();
        assert_eq!(checkpoint.offset, 0, "the prefix must stop at the gap");
        assert_eq!(checkpoint.part_size, part_size);
        assert_eq!(checkpoint.upload_id, "up-gap");
    }
}
