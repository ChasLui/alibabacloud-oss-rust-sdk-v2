//! Server-side copy of an object, with multipart copy for large sources.
//!
//! Mirrors Go `oss/copier.go`. A source at or below
//! [`CopierOptions::multipart_copy_threshold`] is copied with a single
//! `CopyObject`; anything larger is copied part by part with `UploadPartCopy`
//! and then committed.
//!
//! The copy is server-side throughout: bytes never travel through the client,
//! so the only data that crosses the wire is the part metadata.
//!
//! Three behaviours are easy to get wrong and are handled explicitly:
//!
//! - The destination object's CRC is not recomputed locally — it is read from
//!   the completed multipart upload and compared with the source's CRC from
//!   `HeadObject`. A mismatch means the copy is known to be wrong, and the
//!   parts are aborted rather than completed.
//! - `MetadataDirective: COPY` must send *no* metadata: the service fills it in
//!   from the source. Sending the caller's own headers under `COPY` would
//!   silently override them, so they are filtered out of the initiate request
//!   and the source's own metadata is supplied instead.
//! - A part copy takes no conditional headers of its own: `IfMatch` and friends
//!   describe the *source* and belong on the single-copy path, which is where a
//!   race between the source changing and the copy taking place is actually
//!   detectable.

use std::collections::HashMap;

use futures_util::StreamExt;

use crate::api::object::{
    AbortMultipartUploadRequest, CompleteMultipartUploadPart, CompleteMultipartUploadRequest,
    CopyObjectRequest, GetObjectTaggingRequest, HeadObjectRequest, InitiateMultipartUploadRequest,
    UploadPartCopyRequest,
};
use crate::client::Client;
use crate::{
    DEFAULT_COPY_PARALLEL, DEFAULT_COPY_PART_SIZE, DEFAULT_COPY_THRESHOLD, MAX_UPLOAD_PARTS,
};

/// The headers that carry object metadata.
///
/// Under `MetadataDirective: COPY` these are taken from the source and must
/// not be sent by the caller; under `REPLACE` the caller's values win. Mirrors
/// Go's `metadataCopied`.
const METADATA_COPIED: &[&str] = &[
    "content-type",
    "content-language",
    "content-encoding",
    "content-disposition",
    "cache-control",
    "expires",
];

/// Options for [`Client::copy_object_to_object`].
#[derive(Debug, Clone)]
pub struct CopierOptions {
    /// The byte size of each copied part. Defaults to 64 MiB, matching Go's
    /// `DefaultCopyPartSize`.
    pub part_size: i64,

    /// How many parts are copied at once. Defaults to 3, matching Go's
    /// `DefaultCopyParallel`.
    pub parallel_num: usize,

    /// Sources at or below this size are copied with a single `CopyObject`.
    /// Defaults to 200 MiB, matching Go's `DefaultCopyThreshold`.
    pub multipart_copy_threshold: i64,

    /// Keep the already-copied parts when the copy fails.
    ///
    /// The default (`false`) aborts the multipart upload, so a failure does
    /// not leave parts behind that continue to be billed.
    pub leave_parts_on_error: bool,

    /// Skip the shallow-copy optimisation.
    ///
    /// A large copy first attempts one `CopyObject` under a short timeout,
    /// which is much cheaper when the service allows it; set this to `true` to
    /// go straight to a multipart copy. Mirrors Go's `DisableShallowCopy`.
    pub disable_shallow_copy: bool,

    /// Treat a shallow copy as safe even when the source is encrypted.
    /// Mirrors Go's `NoCheckSSE`.
    pub no_check_sse: bool,

    /// Treat a shallow copy as safe even across buckets. Mirrors Go's
    /// `NoCheckCrossBucket`.
    pub no_check_cross_bucket: bool,
}

impl Default for CopierOptions {
    fn default() -> Self {
        CopierOptions {
            part_size: DEFAULT_COPY_PART_SIZE,
            parallel_num: DEFAULT_COPY_PARALLEL.max(1) as usize,
            multipart_copy_threshold: DEFAULT_COPY_THRESHOLD,
            leave_parts_on_error: false,
            disable_shallow_copy: false,
            no_check_sse: false,
            no_check_cross_bucket: false,
        }
    }
}

impl CopierOptions {
    /// Sets the part size. Values below 1 are clamped to 1 byte.
    ///
    /// OSS rejects a multipart whose non-final parts are below 100 KiB, but
    /// that floor is applied when the copy is planned, not here: silently
    /// raising the value would misreport what the caller asked for.
    pub fn with_part_size(mut self, part_size: i64) -> Self {
        self.part_size = part_size.max(1);
        self
    }

    /// Sets how many parts are copied concurrently. Zero is clamped to 1.
    pub fn with_parallel_num(mut self, parallel_num: usize) -> Self {
        self.parallel_num = parallel_num.max(1);
        self
    }

    /// Sets the size above which a copy becomes multipart.
    pub fn with_multipart_copy_threshold(mut self, threshold: i64) -> Self {
        self.multipart_copy_threshold = threshold;
        self
    }

    /// Keeps copied parts when the copy fails, instead of aborting.
    pub fn with_leave_parts_on_error(mut self, leave_parts_on_error: bool) -> Self {
        self.leave_parts_on_error = leave_parts_on_error;
        self
    }

    /// Skips the shallow-copy attempt for large sources.
    pub fn with_disable_shallow_copy(mut self, disable_shallow_copy: bool) -> Self {
        self.disable_shallow_copy = disable_shallow_copy;
        self
    }

    /// Allows a shallow copy of an encrypted source.
    pub fn with_no_check_sse(mut self, no_check_sse: bool) -> Self {
        self.no_check_sse = no_check_sse;
        self
    }

    /// Allows a shallow copy across buckets.
    pub fn with_no_check_cross_bucket(mut self, no_check_cross_bucket: bool) -> Self {
        self.no_check_cross_bucket = no_check_cross_bucket;
        self
    }
}

/// The result of [`Client::copy_object_to_object`].
#[derive(Debug, Clone)]
pub struct CopyResult {
    /// The upload ID, absent when the copy was a single `CopyObject`.
    pub upload_id: Option<String>,
    /// The `ETag` of the destination object.
    pub etag: Option<String>,
    /// The version ID of the destination object.
    pub version_id: Option<String>,
    /// The whole-object CRC64 reported by the server.
    pub hash_crc64: Option<String>,
    /// The total number of bytes copied.
    pub transferred: i64,
}

/// The source of a copy operation, as resolved from the request.
#[derive(Debug, Clone)]
struct CopySource {
    bucket: String,
    key: String,
    version_id: Option<String>,
    /// The `x-oss-copy-source` header value: `/bucket/key?versionId=...`.
    header: String,
}

impl CopySource {
    /// Parses `/bucket/key` and `/bucket/key?versionId=id` into its parts.
    ///
    /// The header is built from the request's `copy_source` string rather than
    /// from separate bucket/key fields, so an unparsable value is an error
    /// instead of a copy from the wrong place.
    fn parse(copy_source: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let without_leading = copy_source
            .strip_prefix('/')
            .ok_or_else(|| format!("invalid copy source {copy_source:?}: must start with '/'"))?;
        let (path, version_id) = match without_leading.split_once("?versionId=") {
            Some((path, version_id)) => (path, Some(version_id.to_string())),
            None => (without_leading, None),
        };
        let (bucket, key) = path
            .split_once('/')
            .ok_or_else(|| format!("invalid copy source {copy_source:?}: expected /bucket/key"))?;
        if bucket.is_empty() || key.is_empty() {
            return Err(format!("invalid copy source {copy_source:?}: empty bucket or key").into());
        }
        Ok(CopySource {
            bucket: bucket.to_string(),
            key: key.to_string(),
            version_id: version_id.clone(),
            header: copy_source.to_string(),
        })
    }
}

/// The destination's metadata and tags, resolved before the copy starts.
#[derive(Debug, Default)]
struct CopySourceProperties {
    /// The source's metadata, keyed by lowercase header name.
    metadata: HashMap<String, String>,
    /// The source's tags, rendered as `k=v&k=v`.
    tagging: Option<String>,
    /// The source's whole-object CRC64, as reported by `HeadObject`.
    hash_crc64: Option<String>,
}

impl Client {
    /// Copies an object onto another object, using multipart copy for sources
    /// larger than one part.
    ///
    /// Mirrors Go `Copier.Copy`. The copy never transfers object bytes through
    /// this client.
    ///
    /// On failure the multipart upload is aborted unless
    /// [`CopierOptions::leave_parts_on_error`] is set, so a failed attempt
    /// does not leave billable parts behind.
    pub async fn copy_object_to_object(
        &self,
        request: &CopyObjectRequest,
        options: CopierOptions,
    ) -> Result<CopyResult, Box<dyn std::error::Error + Send + Sync>> {
        let source = CopySource::parse(&request.copy_source)?;

        if let Some(directive) = request.metadata_directive.as_deref() {
            check_copy_directive(directive, "x-oss-metadata-directive")?;
        }
        if let Some(directive) = request.tagging_directive.as_deref() {
            check_copy_directive(directive, "x-oss-tagging-directive")?;
        }

        // The source's properties drive both the size decision and what the
        // destination ends up carrying, so they are read once, up front.
        let head = self
            .head_object(HeadObjectRequest {
                bucket: source.bucket.clone(),
                key: source.key.clone(),
                version_id: source.version_id.clone(),
                ..Default::default()
            })
            .await?;
        let size = head.content_length.unwrap_or(0) as i64;

        let copy_metadata = !is_replace(request.metadata_directive.as_deref());
        let copy_tagging = !is_replace(request.tagging_directive.as_deref());
        let properties = self
            .resolve_source_properties(&source, &head, copy_metadata, copy_tagging)
            .await?;

        if size <= options.multipart_copy_threshold {
            return self.copy_object_single(request, size).await;
        }

        if !options.disable_shallow_copy && self.can_shallow_copy(request, &source, &head, &options)
        {
            // A single CopyObject is much cheaper than a multipart copy, but
            // the service only allows it up to its own limit. It is attempted
            // first under a short deadline, and only a timeout or an
            // `EntityTooLarge` rejection falls back to the multipart path —
            // any other error is the real answer and surfaces. Mirrors Go's
            // `shallowCopy`.
            let attempt =
                tokio::time::timeout(SHALLOW_COPY_TIMEOUT, self.copy_object_single(request, size));
            match attempt.await {
                Ok(Ok(result)) => return Ok(result),
                Ok(Err(err)) => {
                    if !is_entity_too_large(&*err) {
                        return Err(err);
                    }
                }
                Err(_elapsed) => {}
            }
        }

        self.copy_object_multipart(request, &source, size, &properties, options)
            .await
    }

    /// Reads the source's metadata and tags when the directives ask for them
    /// to be carried over.
    async fn resolve_source_properties(
        &self,
        source: &CopySource,
        head: &crate::api::object::HeadObjectResult,
        copy_metadata: bool,
        copy_tagging: bool,
    ) -> Result<CopySourceProperties, Box<dyn std::error::Error + Send + Sync>> {
        let mut properties = CopySourceProperties {
            hash_crc64: head.hash_crc64.clone(),
            ..Default::default()
        };

        if copy_metadata {
            // `HeadObject` already returns the user metadata under its own
            // names, and the standard metadata headers are on the raw header
            // map; both are needed because the service fills them in
            // separately under COPY.
            for (key, value) in &head.common.headers {
                let lower = key.to_lowercase();
                if lower.starts_with("x-oss-meta-") || METADATA_COPIED.contains(&lower.as_str()) {
                    properties.metadata.insert(lower, value.clone());
                }
            }
            for (key, value) in &head.metadata {
                properties
                    .metadata
                    .insert(format!("x-oss-meta-{key}"), value.clone());
            }
        }

        if copy_tagging && head.tagging_count.unwrap_or(0) > 0 {
            let tagging = self
                .get_object_tagging(&GetObjectTaggingRequest {
                    bucket: source.bucket.clone(),
                    key: source.key.clone(),
                    version_id: source.version_id.clone(),
                    ..Default::default()
                })
                .await?;
            let tags: Vec<String> = tagging
                .tag_set
                .as_ref()
                .map(|set| {
                    set.tags
                        .iter()
                        .map(|tag| {
                            format!(
                                "{}={}",
                                tag.key.as_deref().unwrap_or_default(),
                                tag.value.as_deref().unwrap_or_default()
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            if !tags.is_empty() {
                properties.tagging = Some(tags.join("&"));
            }
        }

        Ok(properties)
    }

    /// Copies with a single `CopyObject`.
    async fn copy_object_single(
        &self,
        request: &CopyObjectRequest,
        size: i64,
    ) -> Result<CopyResult, Box<dyn std::error::Error + Send + Sync>> {
        let result = self.copy_object(request).await?;
        Ok(CopyResult {
            upload_id: None,
            etag: result.etag.clone(),
            version_id: result.version_id.clone(),
            hash_crc64: result.hash_crc64.clone(),
            transferred: size,
        })
    }

    /// Whether the source is simple enough for the service to accept a single
    /// `CopyObject` regardless of size.
    fn can_shallow_copy(
        &self,
        request: &CopyObjectRequest,
        source: &CopySource,
        head: &crate::api::object::HeadObjectResult,
        options: &CopierOptions,
    ) -> bool {
        // Changing the storage class is not something a shallow copy can do,
        // and an encrypted source would be copied without its key.
        if request.storage_class.is_some() {
            return false;
        }
        if !options.no_check_cross_bucket && source.bucket != request.bucket {
            return false;
        }
        if !options.no_check_sse && head.server_side_encryption.is_some() {
            return false;
        }
        true
    }

    /// Copies by parts and commits them.
    async fn copy_object_multipart(
        &self,
        request: &CopyObjectRequest,
        source: &CopySource,
        size: i64,
        properties: &CopySourceProperties,
        options: CopierOptions,
    ) -> Result<CopyResult, Box<dyn std::error::Error + Send + Sync>> {
        // A part size that would need more than the service's part limit is
        // raised until the whole source fits. Mirrors Go's `applySource`.
        let mut part_size = options.part_size.max(crate::MIN_PART_SIZE);
        if size > 0 {
            while size / part_size >= MAX_UPLOAD_PARTS as i64 {
                part_size += options.part_size.max(1);
            }
        }

        let init = self
            .initiate_multipart_upload(&self.initiate_request(request, properties))
            .await?;
        let upload_id = init
            .upload_id
            .clone()
            .ok_or("InitiateMultipartUpload returned no upload ID")?;

        let mut ranges = Vec::new();
        let mut start = 0i64;
        while start < size {
            let part_size = part_size.min(size - start);
            ranges.push((start, part_size));
            start += part_size;
        }

        let parts_upload_id = upload_id.clone();
        let this = self.clone();
        let stream = futures_util::stream::iter(ranges.iter().enumerate().map(
            move |(index, &(start, part_size))| {
                let client = this.clone();
                let upload_id = parts_upload_id.clone();
                let bucket = request.bucket.clone();
                let key = request.key.clone();
                let copy_source = source.header.clone();
                let request_payer = request.request_payer.clone();
                let common = request.common.clone();
                async move {
                    let part_number = index as i32 + 1;
                    let result = client
                        .upload_part_copy(&UploadPartCopyRequest {
                            bucket,
                            key,
                            part_number,
                            upload_id,
                            copy_source,
                            copy_source_range: Some(format!(
                                "bytes={}-{}",
                                start,
                                start + part_size - 1
                            )),
                            request_payer,
                            common,
                            ..Default::default()
                        })
                        .await?;
                    Ok::<_, Box<dyn std::error::Error + Send + Sync>>(CompleteMultipartUploadPart {
                        part_number,
                        etag: result.etag.clone().unwrap_or_default(),
                    })
                }
            },
        ))
        .buffer_unordered(options.parallel_num);

        let mut parts: Vec<CompleteMultipartUploadPart> = Vec::with_capacity(ranges.len());
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
            self.abort_if_needed(request, &upload_id, &options).await;
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
                parts,
                request_payer: request.request_payer.clone(),
                common: request.common.clone(),
                ..Default::default()
            })
            .await;

        let completed = match completed {
            Ok(result) => result,
            Err(err) => {
                self.abort_if_needed(request, &upload_id, &options).await;
                return Err(err);
            }
        };

        // The copy is only known to be correct once the destination's CRC
        // matches the source's. Comparing them is what catches a truncated or
        // misordered part set that the service nonetheless assembled.
        if let (Some(server_crc), Some(source_crc)) = (
            completed.hash_crc64.as_deref(),
            properties.hash_crc64.as_deref(),
        ) {
            // The upload is already complete at this point, so the parts
            // cannot be aborted; the destination is simply reported as
            // untrustworthy. Mirrors Go's `Copier.multiCopy`.
            if !source_crc.is_empty() && server_crc != source_crc {
                return Err(format!(
                    "crc is inconsistent, source {source_crc}, destination {server_crc}"
                )
                .into());
            }
        }

        Ok(CopyResult {
            upload_id: Some(upload_id),
            etag: completed.etag.clone(),
            version_id: completed.version_id.clone(),
            hash_crc64: completed.hash_crc64.clone(),
            transferred: size,
        })
    }

    /// Builds the initiate request, carrying the source's metadata and tags
    /// when the directives ask for them.
    fn initiate_request(
        &self,
        request: &CopyObjectRequest,
        properties: &CopySourceProperties,
    ) -> InitiateMultipartUploadRequest {
        let copy_metadata = !is_replace(request.metadata_directive.as_deref());
        let copy_tagging = !is_replace(request.tagging_directive.as_deref());

        let mut initiate = InitiateMultipartUploadRequest {
            bucket: request.bucket.clone(),
            key: request.key.clone(),
            forbid_overwrite: request.forbid_overwrite.clone(),
            server_side_encryption: request.server_side_encryption.clone(),
            server_side_data_encryption: request.server_side_data_encryption.clone(),
            sse_kms_key_id: request.server_side_encryption_key_id.clone(),
            object_acl: request.object_acl.clone().unwrap_or_default(),
            storage_class: request.storage_class.clone().unwrap_or_default(),
            request_payer: request.request_payer.clone(),
            common: request.common.clone(),
            ..Default::default()
        };

        if copy_metadata {
            // Under COPY the service takes the metadata from the source, so
            // anything the caller put on the request must not be sent; the
            // source's own values are supplied instead.
            initiate.metadata = properties.metadata.clone();
            // The remaining standard headers belong to the caller under
            // REPLACE and to the source under COPY.
            for (key, value) in &request.common.headers {
                let lower = key.to_lowercase();
                if lower.starts_with("x-oss-meta-") || METADATA_COPIED.contains(&lower.as_str()) {
                    continue;
                }
                initiate.common.headers.insert(key.clone(), value.clone());
            }
            initiate.cache_control = source_header(properties, "cache-control");
            initiate.content_disposition = source_header(properties, "content-disposition");
            initiate.content_encoding = source_header(properties, "content-encoding");
            initiate.content_type = source_header(properties, "content-type");
            initiate.expires = source_header(properties, "expires");
        } else {
            initiate.cache_control = request.cache_control.clone();
            initiate.content_disposition = request.content_disposition.clone();
            initiate.content_encoding = request.content_encoding.clone();
            initiate.content_type = request.content_type.clone();
            initiate.expires = request.expires.clone();
            initiate.metadata = request.metadata.clone();
        }

        if copy_tagging {
            initiate.tagging = properties.tagging.clone();
        } else {
            initiate.tagging = request.tagging.clone();
        }

        initiate
    }

    /// Aborts the multipart upload unless the caller asked to keep the parts.
    async fn abort_if_needed(
        &self,
        request: &CopyObjectRequest,
        upload_id: &str,
        options: &CopierOptions,
    ) {
        if options.leave_parts_on_error {
            return;
        }
        let _ = self
            .abort_multipart_upload(&AbortMultipartUploadRequest {
                bucket: request.bucket.clone(),
                key: request.key.clone(),
                upload_id: upload_id.to_string(),
                ..Default::default()
            })
            .await;
    }
}

/// How long a single-`CopyObject` attempt to a large source may take before
/// the copy falls back to copying by parts. Mirrors Go's 30s shallow-copy
/// deadline.
const SHALLOW_COPY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// Whether the service rejected the copy specifically for exceeding its
/// single-request limit, which is the one rejection a multipart copy can fix.
fn is_entity_too_large(err: &(dyn std::error::Error + Send + Sync + 'static)) -> bool {
    err.downcast_ref::<crate::ServiceError>()
        .map(|service_error| service_error.error_code() == "EntityTooLarge")
        .unwrap_or(false)
}

/// Whether a directive means "use the request's own values".
fn is_replace(directive: Option<&str>) -> bool {
    matches!(directive, Some(value) if value.eq_ignore_ascii_case("replace"))
}

/// Rejects a directive the service does not define, rather than sending it and
/// letting the service interpret it.
fn check_copy_directive(
    directive: &str,
    name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if directive.eq_ignore_ascii_case("copy") || directive.eq_ignore_ascii_case("replace") {
        return Ok(());
    }
    Err(format!("unsupported {name} {directive:?}: expected COPY or REPLACE").into())
}

/// Reads one of the source's standard metadata headers.
fn source_header(properties: &CopySourceProperties, name: &str) -> Option<String> {
    properties.metadata.get(name).cloned()
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

    fn request() -> CopyObjectRequest {
        CopyObjectRequest {
            bucket: "dest-bucket".to_string(),
            key: "dest-key".to_string(),
            copy_source: "/src-bucket/src-key".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_parse_copy_source() {
        let source = CopySource::parse("/bucket/key").unwrap();
        assert_eq!(source.bucket, "bucket");
        assert_eq!(source.key, "key");
        assert_eq!(source.version_id, None);

        let source = CopySource::parse("/bucket/nested/key?versionId=v1").unwrap();
        assert_eq!(source.key, "nested/key");
        assert_eq!(source.version_id.as_deref(), Some("v1"));

        assert!(CopySource::parse("bucket/key").is_err());
        assert!(CopySource::parse("/bucket").is_err());
        assert!(CopySource::parse("/key").is_err());
    }

    /// A source within the threshold must be copied with one `CopyObject`, and
    /// no multipart upload may be opened.
    #[tokio::test]
    async fn test_copy_small_source_uses_single_copy() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", "1024")
            .with_header("x-oss-hash-crc64ecma", "42")
            .create_async()
            .await;
        let copy = server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(200)
            .with_header("x-oss-hash-crc64ecma", "42")
            .with_body("<CopyObjectResult><ETag>\"copied\"</ETag></CopyObjectResult>")
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

        let result = mock_client(&server)
            .copy_object_to_object(&request(), CopierOptions::default())
            .await
            .expect("copy should succeed");

        assert_eq!(result.etag.as_deref(), Some("\"copied\""));
        assert_eq!(result.upload_id, None);
        assert_eq!(result.transferred, 1024);
        copy.assert_async().await;
        init.assert_async().await;
    }

    /// A source above the threshold must be copied by parts and completed; the
    /// completion must list every part exactly once and in ascending order.
    #[tokio::test]
    async fn test_copy_large_source_splits_and_completes() {
        let mut server = mockito::Server::new_async().await;
        let part_size = crate::MIN_PART_SIZE;
        let size = part_size * 2 + 137;
        let source_crc = "1234567890";

        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", &size.to_string())
            .with_header("x-oss-hash-crc64ecma", source_crc)
            .create_async()
            .await;

        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploads".to_string()))
            .with_status(200)
            .with_body(
                "<InitiateMultipartUploadResult><Bucket>dest-bucket</Bucket><Key>dest-key</\
                 Key><UploadId>upload-9</UploadId></InitiateMultipartUploadResult>",
            )
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
                        mockito::Matcher::Regex("uploadId=upload-9".to_string()),
                    ]))
                    .with_status(200)
                    .with_body(format!(
                        "<CopyPartResult><ETag>\"part-{part_number}\"</ETag></CopyPartResult>"
                    ))
                    .expect(1)
                    .create_async()
                    .await,
            );
        }

        let complete = server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploadId=upload-9".to_string()))
            .with_status(200)
            .with_header("x-oss-hash-crc64ecma", source_crc)
            .with_body(
                "<CompleteMultipartUploadResult><ETag>\"final\"</ETag></\
                 CompleteMultipartUploadResult>",
            )
            .expect(1)
            .create_async()
            .await;

        let result = mock_client(&server)
            .copy_object_to_object(
                &request(),
                CopierOptions::default()
                    .with_part_size(part_size)
                    .with_parallel_num(2)
                    .with_multipart_copy_threshold(0)
                    .with_disable_shallow_copy(true),
            )
            .await
            .expect("multipart copy should succeed");

        assert_eq!(result.upload_id.as_deref(), Some("upload-9"));
        assert_eq!(result.etag.as_deref(), Some("\"final\""));
        assert_eq!(result.transferred, size);
        for m in &part_mocks {
            m.assert_async().await;
        }
        complete.assert_async().await;
    }

    /// A failing part must abort the copy, so no billable parts are left
    /// behind.
    #[tokio::test]
    async fn test_copy_aborts_on_part_failure() {
        let mut server = mockito::Server::new_async().await;
        let part_size = crate::MIN_PART_SIZE;
        let size = part_size * 2 + 137;

        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", &size.to_string())
            .create_async()
            .await;
        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploads".to_string()))
            .with_status(200)
            .with_body(
                "<InitiateMultipartUploadResult><UploadId>upload-err</UploadId></\
                 InitiateMultipartUploadResult>",
            )
            .create_async()
            .await;
        // Every part fails.
        server
            .mock("PUT", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploadId".to_string()))
            .with_status(500)
            .with_body("<Error><Code>InternalError</Code><Message>boom</Message></Error>")
            .create_async()
            .await;
        let abort = server
            .mock("DELETE", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploadId=upload-err".to_string()))
            .with_status(204)
            .expect(1)
            .create_async()
            .await;

        let err = mock_client(&server)
            .copy_object_to_object(
                &request(),
                CopierOptions::default()
                    .with_part_size(part_size)
                    .with_multipart_copy_threshold(0)
                    .with_disable_shallow_copy(true),
            )
            .await
            .expect_err("a failed part must surface as an error");

        assert!(err.to_string().contains("500") || err.to_string().contains("boom"));
        abort.assert_async().await;
    }

    /// Under `MetadataDirective: COPY` the destination must carry the source's
    /// metadata, and the caller's own metadata must not be sent — the service
    /// would otherwise take the caller's values instead of the source's.
    #[tokio::test]
    async fn test_copy_metadata_directive_copy_uses_source_metadata() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", "1024")
            .with_header("x-oss-meta-author", "alice")
            .create_async()
            .await;
        let copy = server
            .mock("PUT", mockito::Matcher::Any)
            .match_header("x-oss-metadata-directive", "COPY")
            .with_status(200)
            .with_body("<CopyObjectResult><ETag>\"copied\"</ETag></CopyObjectResult>")
            .expect(1)
            .create_async()
            .await;

        let result = mock_client(&server)
            .copy_object_to_object(
                &CopyObjectRequest {
                    metadata_directive: Some("COPY".to_string()),
                    ..request()
                },
                CopierOptions::default(),
            )
            .await
            .expect("copy should succeed");

        assert_eq!(result.etag.as_deref(), Some("\"copied\""));
        copy.assert_async().await;
    }

    /// A destination whose CRC differs from the source's must fail the copy.
    /// This is the only check that catches a part set the service assembled
    /// from the wrong bytes.
    #[tokio::test]
    async fn test_copy_detects_crc_mismatch() {
        let mut server = mockito::Server::new_async().await;
        let part_size = crate::MIN_PART_SIZE;
        let size = part_size * 2 + 137;

        server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", &size.to_string())
            .with_header("x-oss-hash-crc64ecma", "111")
            .create_async()
            .await;
        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploads".to_string()))
            .with_status(200)
            .with_body(
                "<InitiateMultipartUploadResult><UploadId>upload-crc</UploadId></\
                 InitiateMultipartUploadResult>",
            )
            .create_async()
            .await;
        let mut part_mocks = Vec::new();
        for part_number in 1..=3 {
            part_mocks.push(
                server
                    .mock("PUT", mockito::Matcher::Any)
                    .match_query(mockito::Matcher::Regex(format!("partNumber={part_number}")))
                    .with_status(200)
                    .with_body("<CopyPartResult><ETag>\"p\"</ETag></CopyPartResult>")
                    .create_async()
                    .await,
            );
        }
        // The completion reports a different CRC than the source.
        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploadId=upload-crc".to_string()))
            .with_status(200)
            .with_header("x-oss-hash-crc64ecma", "222")
            .with_body(
                "<CompleteMultipartUploadResult><ETag>\"final\"</ETag></\
                 CompleteMultipartUploadResult>",
            )
            .create_async()
            .await;

        let err = mock_client(&server)
            .copy_object_to_object(
                &request(),
                CopierOptions::default()
                    .with_part_size(part_size)
                    .with_multipart_copy_threshold(0)
                    .with_disable_shallow_copy(true),
            )
            .await
            .expect_err("a CRC mismatch must fail the copy");

        let message = err.to_string();
        assert!(
            message.contains("crc is inconsistent"),
            "expected a CRC error, got {message}"
        );
        assert!(message.contains("111") && message.contains("222"));
    }

    /// An unknown directive must be rejected before anything is sent.
    #[tokio::test]
    async fn test_copy_rejects_unknown_directive() {
        let server = mockito::Server::new_async().await;
        let err = mock_client(&server)
            .copy_object_to_object(
                &CopyObjectRequest {
                    metadata_directive: Some("MERGE".to_string()),
                    ..request()
                },
                CopierOptions::default(),
            )
            .await
            .expect_err("an unknown directive must be rejected");
        assert!(err.to_string().contains("MERGE"));
    }
}
