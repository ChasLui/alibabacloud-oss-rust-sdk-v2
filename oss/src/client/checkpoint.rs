//! Checkpointed transfers: resuming a download or an upload after a failure.
//!
//! Mirrors the checkpoint behaviour of Go `oss/downloader.go` and
//! `oss/uploader.go` (whose files live in `oss/checkpoint.go`).
//!
//! A checkpoint is a small JSON file next to the transfer's temporary file. It
//! records what the transfer knows about its own progress; the service remains
//! the authority on what actually arrived.
//!
//! Three properties make the difference between a resume that is safe and one
//! that silently produces a corrupt object:
//!
//! - A checkpoint is only trusted if it describes *this* transfer. The file
//!   name is derived from the source, the destination, and the version, and the
//!   payload carries a magic value plus an MD5 over its own fields, so a
//!   checkpoint left by a different object, a different destination, or a
//!   truncated write is discarded rather than resumed from.
//! - A download's progress stops at the first gap. Bytes are downloaded
//!   concurrently and land out of order, so the commit point is the end of the
//!   longest contiguous run from the start — never the total number of bytes
//!   written, which would skip the hole in between.
//! - An upload is *not* tracked from the checkpoint's own part list. The
//!   checkpoint records only the upload ID; the parts that actually exist are
//!   read back with `ListParts`, because a part the service never accepted must
//!   not be counted as done.
//!
//! A resumed upload also re-reads each existing part's CRC and folds them, so
//! the client's running checksum matches the bytes that are actually stored.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::api::object::{HeadObjectRequest, ListPartsRequest, PutObjectRequest};
use crate::client::Client;
use crate::utils::{crc64_combine, Crc64};
use crate::{
    CHECKPOINT_FILE_SUFFIX_DOWNLOADER, CHECKPOINT_FILE_SUFFIX_UPLOADER, CHECKPOINT_MAGIC,
    DEFAULT_UPLOAD_PART_SIZE, MIN_PART_SIZE, TEMP_FILE_SUFFIX,
};

/// The checkpoint payload, shared by both directions.
///
/// Mirrors the JSON Go writes: a magic value, an MD5 over `data`, and the data
/// itself.
#[derive(Debug, Serialize, Deserialize)]
struct CheckpointFile<T> {
    #[serde(rename = "Magic")]
    magic: String,
    #[serde(rename = "MD5")]
    md5: String,
    #[serde(rename = "Data")]
    data: T,
}

/// What a download checkpoint records.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DownloadData {
    #[serde(rename = "ObjectInfo")]
    object_info: DownloadObjectInfo,
    #[serde(rename = "ObjectMeta")]
    object_meta: DownloadObjectMeta,
    #[serde(rename = "FilePath")]
    file_path: String,
    #[serde(rename = "PartSize")]
    part_size: i64,
    #[serde(rename = "DownloadInfo")]
    download_info: DownloadInfo,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DownloadObjectInfo {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "VersionId")]
    version_id: String,
    #[serde(rename = "Range")]
    range: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DownloadObjectMeta {
    #[serde(rename = "Size")]
    size: i64,
    #[serde(rename = "LastModified")]
    last_modified: String,
    #[serde(rename = "ETag")]
    etag: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DownloadInfo {
    #[serde(rename = "Offset")]
    offset: i64,
    #[serde(rename = "CRC64")]
    crc64: u64,
}

/// What an upload checkpoint records.
///
/// Only the upload ID: which parts exist is answered by the service, because a
/// part that was never accepted must not be treated as done.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct UploadData {
    #[serde(rename = "FilePath")]
    file_path: String,
    #[serde(rename = "FileMeta")]
    file_meta: UploadFileMeta,
    #[serde(rename = "ObjectInfo")]
    object_info: UploadObjectInfo,
    #[serde(rename = "PartSize")]
    part_size: i64,
    #[serde(rename = "UploadInfo")]
    upload_info: UploadInfo,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct UploadFileMeta {
    #[serde(rename = "Size")]
    size: i64,
    #[serde(rename = "LastModified")]
    last_modified: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct UploadObjectInfo {
    #[serde(rename = "Name")]
    name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct UploadInfo {
    #[serde(rename = "UploadId")]
    upload_id: String,
}

/// The path a transfer's checkpoint file lives at.
///
/// The name is derived from the source identity and the destination path, so a
/// checkpoint can only be picked up by the transfer that wrote it.
fn checkpoint_path(
    directory: &str,
    source_identity: &str,
    destination: &str,
    suffix: &str,
) -> PathBuf {
    let source_hash = md5_hex(source_identity.as_bytes());
    let destination_hash = md5_hex(
        std::fs::canonicalize(destination)
            .unwrap_or_else(|_| PathBuf::from(destination))
            .to_string_lossy()
            .as_bytes(),
    );
    let directory = if directory.is_empty() {
        std::env::temp_dir()
    } else {
        PathBuf::from(directory)
    };
    directory.join(format!("{source_hash}-{destination_hash}{suffix}"))
}

/// The hexadecimal MD5 of `data`.
fn md5_hex(data: &[u8]) -> String {
    let digest = md5_compute(data);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Computes MD5 with the crate's existing dependency.
fn md5_compute(data: &[u8]) -> [u8; 16] {
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Writes a checkpoint, stamping it with the magic value and its own MD5.
///
/// The digest covers the `Data` member exactly as it is written. That matters
/// because the read side verifies the same bytes: routing the payload through
/// `serde_json::Value` reorders object keys (a struct keeps declaration order,
/// a `Value` sorts), so re-serializing would compute a different digest for an
/// untouched file and reject every checkpoint the writer ever produced.
fn write_checkpoint<T: Serialize>(
    path: &Path,
    data: &T,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let data_json = serde_json::to_string(data)?;
    let payload = format!(
        "{{\"Magic\":{},\"MD5\":{},\"Data\":{data_json}}}",
        serde_json::to_string(CHECKPOINT_MAGIC)?,
        serde_json::to_string(&md5_hex(data_json.as_bytes()))?,
    );
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Written through a temporary file so a crash mid-write cannot leave a
    // half-serialized checkpoint that the next attempt would have to detect.
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, payload)?;
    std::fs::rename(&temporary, path)?;
    Ok(())
}

/// Reads a checkpoint, returning `None` when it is absent or untrustworthy.
///
/// A checkpoint is rejected when its magic or MD5 does not match, which is how
/// a truncated or tampered file is kept from being resumed from.
fn read_checkpoint<T: for<'de> Deserialize<'de>>(path: &Path) -> Option<T> {
    let contents = std::fs::read_to_string(path).ok()?;
    let envelope: CheckpointFile<serde_json::Value> = serde_json::from_str(&contents).ok()?;
    if envelope.magic != CHECKPOINT_MAGIC {
        let _ = std::fs::remove_file(path);
        return None;
    }
    // The digest covers the Data exactly as it appears in the file. Passing it
    // through `Value` reorders object keys, so re-serializing would compute a
    // different digest for an untouched file and reject every checkpoint.
    let raw_data = extract_raw_data(&contents)?;
    if md5_hex(raw_data.as_bytes()) != envelope.md5 {
        let _ = std::fs::remove_file(path);
        return None;
    }
    let data: T = serde_json::from_value(envelope.data).ok()?;
    Some(data)
}

/// Returns the `Data` member of the checkpoint JSON exactly as it was written.
///
/// The member is sliced out of the original text rather than re-serialized:
/// passing it through `serde_json::Value` reorders its keys, which would
/// change the bytes the digest was taken over.
fn extract_raw_data(contents: &str) -> Option<String> {
    let marker = "\"Data\":";
    let start = contents.find(marker)? + marker.len();
    let rest = contents[start..].trim_start();
    // The writer emits Data last, so the remainder is the member plus the
    // closing brace of the envelope.
    let end = rest.rfind('}')?;
    Some(rest[..end].to_string())
}

/// Options for a checkpointed download.
#[derive(Debug, Clone)]
pub struct DownloaderCheckpointOptions {
    /// Where the checkpoint file is stored. Defaults to the system temp
    /// directory.
    pub checkpoint_dir: String,
    /// Downloads the object into a temporary file and renames it on success,
    /// so a partial download never appears at the destination path.
    pub use_temp_file: bool,
    /// Re-reads the already-downloaded prefix and compares its CRC before
    /// resuming, which catches a partially written file that the checkpoint's
    /// offset would otherwise trust.
    pub verify_data: bool,
}

impl Default for DownloaderCheckpointOptions {
    fn default() -> Self {
        DownloaderCheckpointOptions {
            checkpoint_dir: String::new(),
            use_temp_file: true,
            verify_data: false,
        }
    }
}

impl DownloaderCheckpointOptions {
    /// Sets the directory the checkpoint file is written to.
    pub fn with_checkpoint_dir(mut self, checkpoint_dir: &str) -> Self {
        self.checkpoint_dir = checkpoint_dir.to_string();
        self
    }

    /// Chooses whether the download goes through a temporary file.
    pub fn with_use_temp_file(mut self, use_temp_file: bool) -> Self {
        self.use_temp_file = use_temp_file;
        self
    }

    /// Chooses whether the downloaded prefix is verified before resuming.
    pub fn with_verify_data(mut self, verify_data: bool) -> Self {
        self.verify_data = verify_data;
        self
    }
}

/// The outcome of a checkpointed download.
#[derive(Debug, Clone)]
pub struct CheckpointDownloadResult {
    /// How many bytes this attempt downloaded.
    pub written: i64,
    /// How many bytes were already present when the attempt started.
    pub resumed_from: i64,
    /// The total size of the downloaded span.
    pub total: i64,
    /// The object's `ETag`.
    pub etag: Option<String>,
    /// Whether the download completed and the checkpoint was removed.
    pub completed: bool,
}

/// The outcome of a checkpointed upload.
#[derive(Debug, Clone)]
pub struct CheckpointUploadResult {
    /// The upload ID the transfer ran under.
    pub upload_id: Option<String>,
    /// How many bytes this attempt uploaded.
    pub written: i64,
    /// How many bytes were already uploaded when the attempt started.
    pub resumed_from: i64,
    /// The total size of the file.
    pub total: i64,
    /// The `ETag` of the stored object.
    pub etag: Option<String>,
    /// Whether the upload completed and the checkpoint was removed.
    pub completed: bool,
}

/// Options for a checkpointed upload.
#[derive(Debug, Clone)]
pub struct UploaderCheckpointOptions {
    /// Where the checkpoint file is stored. Defaults to the system temp
    /// directory.
    pub checkpoint_dir: String,
    /// The multipart part size. Defaults to 6 MiB.
    pub part_size: i64,
    /// How many parts are uploaded at once. Defaults to 3.
    pub parallel_num: usize,
}

impl Default for UploaderCheckpointOptions {
    fn default() -> Self {
        UploaderCheckpointOptions {
            checkpoint_dir: String::new(),
            part_size: DEFAULT_UPLOAD_PART_SIZE,
            parallel_num: 3,
        }
    }
}

impl UploaderCheckpointOptions {
    /// Sets the directory the checkpoint file is written to.
    pub fn with_checkpoint_dir(mut self, checkpoint_dir: &str) -> Self {
        self.checkpoint_dir = checkpoint_dir.to_string();
        self
    }

    /// Sets the part size.
    pub fn with_part_size(mut self, part_size: i64) -> Self {
        self.part_size = part_size.max(1);
        self
    }

    /// Sets how many parts are uploaded at once.
    pub fn with_parallel_num(mut self, parallel_num: usize) -> Self {
        self.parallel_num = parallel_num.max(1);
        self
    }
}

/// A downloaded byte range, half-open: `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DownloadedRange {
    start: i64,
    end: i64,
}

/// The contiguous prefix of a set of downloaded ranges.
///
/// Ranges arrive out of order, so the resume point is the end of the longest
/// run that starts where the transfer had already got to. `already_have` is
/// that starting point: bytes before it were downloaded by an earlier attempt
/// and are not in `ranges`. Using the sum of all ranges instead would skip any
/// hole between them.
fn contiguous_prefix(
    mut ranges: Vec<DownloadedRange>,
    already_have: i64,
) -> (i64, Vec<DownloadedRange>) {
    ranges.sort_by_key(|range| range.start);
    let mut prefix_end = already_have;
    let mut remaining = Vec::new();
    for range in ranges {
        if range.start <= prefix_end {
            // Overlapping or adjacent: the range extends the prefix.
            if range.end > prefix_end {
                prefix_end = range.end;
            }
        } else {
            remaining.push(range);
        }
    }
    (prefix_end, remaining)
}

/// One downloaded part, as used while assembling a checkpointed download.
#[derive(Debug, Clone, Copy)]
struct DownloadedPart {
    start: i64,
    size: i64,
    crc64: u64,
}

impl Client {
    /// Downloads an object to `file_path`, resuming from a checkpoint when one
    /// exists.
    ///
    /// Mirrors Go `Downloader.DownloadFile` with `EnableCheckpoint`. The
    /// checkpoint records the contiguous prefix that has been written; a
    /// resume re-requests only the bytes after it.
    pub async fn download_file_with_checkpoint(
        &self,
        request: &crate::api::object::GetObjectRequest,
        file_path: impl AsRef<Path>,
        part_size: i64,
        checkpoint: DownloaderCheckpointOptions,
    ) -> Result<CheckpointDownloadResult, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = file_path.as_ref().to_path_buf();
        let file_path_string = file_path.to_string_lossy().to_string();

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

        let source_identity = format!(
            "oss://{}/{}\n{}\n{}",
            request.bucket,
            request.key,
            request.version_id.as_deref().unwrap_or(""),
            request.range.as_deref().unwrap_or("")
        );
        let checkpoint_path = checkpoint_path(
            &checkpoint.checkpoint_dir,
            &source_identity,
            &file_path_string,
            CHECKPOINT_FILE_SUFFIX_DOWNLOADER,
        );
        let part_size = part_size.max(1);

        // A checkpoint is only used when it describes this same object at this
        // same destination and part size; anything else is discarded so the
        // download restarts cleanly.
        let mut resume_from = 0i64;
        let mut resume_crc = 0u64;
        if let Some(data) = read_checkpoint::<DownloadData>(&checkpoint_path) {
            let matches = data.object_info.name
                == format!("oss://{}/{}", request.bucket, request.key)
                && data.object_info.version_id == request.version_id.clone().unwrap_or_default()
                && data.object_info.range == request.range.clone().unwrap_or_default()
                && data.object_meta.size == total_size
                && data.object_meta.etag == etag.clone().unwrap_or_default()
                && data.file_path == file_path_string
                && data.part_size == part_size;
            let plausible = data.download_info.offset >= 0
                && data.download_info.offset <= total_size
                && data.download_info.offset % part_size == 0;
            if matches && plausible {
                resume_from = data.download_info.offset;
                resume_crc = data.download_info.crc64;
            } else {
                let _ = std::fs::remove_file(&checkpoint_path);
            }
        }

        // A prefix whose recorded checksum does not match what is on disk
        // cannot be trusted; the download restarts rather than extending it.
        if checkpoint.verify_data && resume_from > 0 && resume_crc != 0 {
            let mut crc = Crc64::new(0);
            let data = tokio::fs::read(&file_path).await.unwrap_or_default();
            let limit = (resume_from as usize).min(data.len());
            crc.write(&data[..limit]).map_err(
                |err| -> Box<dyn std::error::Error + Send + Sync> { err.to_string().into() },
            )?;
            if crc.sum64() != resume_crc {
                resume_from = 0;
                resume_crc = 0;
                let _ = std::fs::remove_file(&checkpoint_path);
            }
        }

        let temporary = if checkpoint.use_temp_file {
            PathBuf::from(format!("{file_path_string}{TEMP_FILE_SUFFIX}"))
        } else {
            file_path.clone()
        };
        if resume_from > 0 {
            // The bytes an earlier attempt downloaded live at the destination
            // (or, when a temporary file is in use, possibly still in the
            // temporary one). Whichever holds them has to be the file this
            // attempt extends, so a temporary download starts from the
            // destination's contents instead of an empty file.
            if checkpoint.use_temp_file && !temporary.exists() && file_path.exists() {
                std::fs::rename(&file_path, &temporary)?;
            }
            let existing = std::fs::metadata(&temporary)
                .map(|meta| meta.len())
                .unwrap_or(0);
            if existing < resume_from as u64 {
                return Err(format!(
                    "checkpoint records {resume_from} bytes but only {existing} are present"
                )
                .into());
            }
        } else {
            // A fresh start truncates, so stale bytes past the end of this
            // download cannot be mistaken for content.
            std::fs::write(&temporary, vec![0u8; total_size as usize])?;
        }

        let mut ranges = Vec::new();
        let mut start = resume_from;
        while start < total_size {
            let size = part_size.min(total_size - start);
            ranges.push((start, size));
            start += size;
        }

        let written = std::sync::Arc::new(std::sync::atomic::AtomicI64::new(0));
        let file = std::sync::Arc::new(std::fs::OpenOptions::new().write(true).open(&temporary)?);

        let parts: Vec<DownloadedPart> = {
            use futures_util::StreamExt;
            let stream = futures_util::stream::iter(ranges.iter().map(|&(start, size)| {
                let client = self.clone();
                let request = crate::api::object::GetObjectRequest {
                    bucket: request.bucket.clone(),
                    key: request.key.clone(),
                    if_match: request.if_match.clone(),
                    if_none_match: request.if_none_match.clone(),
                    if_modified_since: request.if_modified_since.clone(),
                    if_unmodified_since: request.if_unmodified_since.clone(),
                    range: Some(format!("bytes={start}-{}", start + size - 1)),
                    range_behavior: Some("standard".to_string()),
                    version_id: request.version_id.clone(),
                    request_payer: request.request_payer.clone(),
                    common: request.common.clone(),
                    ..Default::default()
                };
                let file = file.clone();
                let written = written.clone();
                async move {
                    let result = client.get_object(request).await?;
                    let body = result
                        .body
                        .ok_or("GetObject returned no body for a ranged request")?;
                    let mut crc = Crc64::new(0);
                    let mut offset = start;
                    let mut body = std::pin::pin!(body);
                    while let Some(chunk) = body.next().await {
                        let chunk = chunk?;
                        crc.write(&chunk).map_err(
                            |err| -> Box<dyn std::error::Error + Send + Sync> {
                                err.to_string().into()
                            },
                        )?;
                        write_at(&file, &chunk, offset as u64)?;
                        offset += chunk.len() as i64;
                    }
                    written.fetch_add(offset - start, std::sync::atomic::Ordering::Relaxed);
                    Ok::<DownloadedPart, Box<dyn std::error::Error + Send + Sync>>(DownloadedPart {
                        start,
                        size: offset - start,
                        crc64: crc.sum64(),
                    })
                }
            }))
            .buffer_unordered(4);
            let mut stream = std::pin::pin!(stream);
            let mut collected = Vec::with_capacity(ranges.len());
            while let Some(part) = stream.next().await {
                collected.push(part?);
            }
            collected
        };

        let mut all = parts;
        all.sort_by_key(|part| part.start);
        let mut combined = resume_crc;
        for part in &all {
            combined = crc64_combine(combined, part.crc64, part.size as u64);
        }

        // Persist the progress that is known to be contiguous, which is what
        // the next attempt will resume from.
        let downloaded: Vec<DownloadedRange> = all
            .iter()
            .map(|part| DownloadedRange {
                start: part.start,
                end: part.start + part.size,
            })
            .collect();
        let (prefix_end, _) = contiguous_prefix(downloaded, resume_from);
        write_checkpoint(
            &checkpoint_path,
            &DownloadData {
                object_info: DownloadObjectInfo {
                    name: format!("oss://{}/{}", request.bucket, request.key),
                    version_id: request.version_id.clone().unwrap_or_default(),
                    range: request.range.clone().unwrap_or_default(),
                },
                object_meta: DownloadObjectMeta {
                    size: total_size,
                    last_modified: head.last_modified.clone().unwrap_or_default(),
                    etag: etag.clone().unwrap_or_default(),
                },
                file_path: file_path_string.clone(),
                part_size,
                download_info: DownloadInfo {
                    offset: prefix_end,
                    crc64: combined,
                },
            },
        )?;

        if prefix_end >= total_size && checkpoint.use_temp_file {
            std::fs::rename(&temporary, &file_path)?;
            let _ = std::fs::remove_file(&checkpoint_path);
        }

        Ok(CheckpointDownloadResult {
            written: written.load(std::sync::atomic::Ordering::Relaxed),
            resumed_from: resume_from,
            total: total_size,
            etag,
            completed: prefix_end >= total_size,
        })
    }

    /// Uploads a local file, resuming a previous multipart upload when a
    /// checkpoint says one is still open.
    ///
    /// Mirrors Go `Uploader.UploadFile` with `EnableCheckpoint`. The upload ID
    /// comes from the checkpoint; the parts that already exist come from
    /// `ListParts`, so a part the service never accepted is uploaded again.
    pub async fn upload_file_with_checkpoint(
        &self,
        request: &PutObjectRequest,
        file_path: impl AsRef<Path>,
        checkpoint: UploaderCheckpointOptions,
    ) -> Result<CheckpointUploadResult, Box<dyn std::error::Error + Send + Sync>> {
        use futures_util::StreamExt;

        let file_path = file_path.as_ref().to_path_buf();
        let file_path_string = file_path.to_string_lossy().to_string();
        let metadata = tokio::fs::metadata(&file_path).await?;
        if metadata.is_dir() {
            return Err(format!("{} is a directory, not a file", file_path.display()).into());
        }
        let total_size = metadata.len() as i64;
        let file_modified = metadata
            .modified()
            .map(|time| {
                time.duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs().to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        let checkpoint_path = checkpoint_path(
            &checkpoint.checkpoint_dir,
            &format!("oss://{}/{}", request.bucket, request.key),
            &file_path_string,
            CHECKPOINT_FILE_SUFFIX_UPLOADER,
        );
        let part_size = checkpoint.part_size.max(MIN_PART_SIZE);

        // Resume only when the checkpoint describes this file at this size and
        // part size; otherwise the previous upload is abandoned.
        let mut upload_id: Option<String> = None;
        if let Some(data) = read_checkpoint::<UploadData>(&checkpoint_path) {
            let matches = data.file_path == file_path_string
                && data.file_meta.size == total_size
                && data.file_meta.last_modified == file_modified
                && data.object_info.name == format!("oss://{}/{}", request.bucket, request.key)
                && data.part_size == part_size
                && !data.upload_info.upload_id.is_empty();
            if matches {
                upload_id = Some(data.upload_info.upload_id);
            } else {
                eprintln!(
                    "UL REJECT file_ok={} size_ok={} mtime_ok={} name_ok={} ps_ok={} id_ok={}",
                    data.file_path == file_path_string,
                    data.file_meta.size == total_size,
                    data.file_meta.last_modified == file_modified,
                    data.object_info.name == format!("oss://{}/{}", request.bucket, request.key),
                    data.part_size == part_size,
                    !data.upload_info.upload_id.is_empty()
                );
                eprintln!(
                    "UL mtime cp={:?} actual={:?} size cp={} actual={}",
                    data.file_meta.last_modified, file_modified, data.file_meta.size, total_size
                );
                let _ = std::fs::remove_file(&checkpoint_path);
            }
        }

        // Which parts exist is answered by the service, never by the
        // checkpoint: a part that was accepted but not recorded, or recorded
        // but rejected, would otherwise be skipped or duplicated.
        let mut uploaded_parts: Vec<(i32, String)> = Vec::new();
        let mut resume_from = 0i64;
        let mut resume_crc = 0u64;
        if let Some(id) = upload_id.clone() {
            let mut marker: Option<i32> = None;
            loop {
                let page = self
                    .list_parts(&ListPartsRequest {
                        bucket: request.bucket.clone(),
                        key: request.key.clone(),
                        upload_id: id.clone(),
                        max_parts: Some(1000),
                        part_number_marker: marker,
                        ..Default::default()
                    })
                    .await;
                let page = match page {
                    Ok(page) => page,
                    Err(_) => {
                        // The previous upload is gone; the transfer restarts
                        // rather than failing outright.
                        upload_id = None;
                        uploaded_parts.clear();
                        resume_from = 0;
                        resume_crc = 0;
                        let _ = std::fs::remove_file(&checkpoint_path);
                        break;
                    }
                };
                let mut parts: Vec<crate::api::object::Part> = Vec::new();
                parts.extend(page.parts.iter().map(|part| crate::api::object::Part {
                    part_number: part.part_number,
                    last_modified: part.last_modified.clone(),
                    etag: part.etag.clone(),
                    size: part.size,
                    hash_crc64: part.hash_crc64.clone(),
                }));
                parts.sort_by_key(|part| part.part_number);
                let mut complete = true;
                for part in &parts {
                    let expected = (uploaded_parts.len() + 1) as i32;
                    // Only a run of full parts from part 1 is usable: a short
                    // part in the middle would leave a hole.
                    if part.part_number != expected || part.size != part_size {
                        complete = false;
                        break;
                    }
                    uploaded_parts.push((part.part_number, part.etag.clone()));
                    resume_from += part.size;
                    if let Some(crc) = part
                        .hash_crc64
                        .as_deref()
                        .and_then(|value| value.parse::<u64>().ok())
                    {
                        resume_crc = crc64_combine(resume_crc, crc, part.size as u64);
                    }
                }
                if !complete {
                    break;
                }
                match page.next_part_number_marker {
                    Some(next) if page.is_truncated.unwrap_or(false) => marker = Some(next),
                    _ => break,
                }
            }
            if uploaded_parts.is_empty() {
                upload_id = None;
            }
        }

        let file = std::sync::Arc::new(std::fs::File::open(&file_path)?);
        let mut ranges = Vec::new();
        let mut start = resume_from;
        while start < total_size {
            let size = part_size.min(total_size - start);
            ranges.push((start, size));
            start += size;
        }

        // A brand-new transfer opens its upload here; a resumed one continues
        // the upload the checkpoint named.
        let upload_id = match upload_id {
            Some(id) => id,
            None => {
                let init = self
                    .initiate_multipart_upload(
                        &crate::api::object::InitiateMultipartUploadRequest {
                            bucket: request.bucket.clone(),
                            key: request.key.clone(),
                            request_payer: request.request_payer.clone(),
                            ..Default::default()
                        },
                    )
                    .await?;
                let id = init
                    .upload_id
                    .clone()
                    .ok_or("InitiateMultipartUpload returned no upload ID")?;
                write_checkpoint(
                    &checkpoint_path,
                    &UploadData {
                        file_path: file_path_string.clone(),
                        file_meta: UploadFileMeta {
                            size: total_size,
                            last_modified: file_modified.clone(),
                        },
                        object_info: UploadObjectInfo {
                            name: format!("oss://{}/{}", request.bucket, request.key),
                        },
                        part_size,
                        upload_info: UploadInfo {
                            upload_id: id.clone(),
                        },
                    },
                )?;
                id
            }
        };

        let mut new_parts: Vec<(i32, String, i64, u64)> = Vec::new();
        {
            let stream = futures_util::stream::iter((0..ranges.len()).map(|index| {
                let client = self.clone();
                let file = file.clone();
                let bucket = request.bucket.clone();
                let key = request.key.clone();
                let upload_id = upload_id.clone();
                let request_payer = request.request_payer.clone();
                let (start, size) = ranges[index];
                let part_number = uploaded_parts.len() as i32 + index as i32 + 1;
                async move {
                    let data = read_at(&file, start as u64, size as usize)?;
                    let mut crc = Crc64::new(0);
                    crc.write(&data).map_err(
                        |err| -> Box<dyn std::error::Error + Send + Sync> {
                            err.to_string().into()
                        },
                    )?;
                    let result = client
                        .upload_part(crate::api::object::UploadPartRequest {
                            bucket,
                            key,
                            part_number,
                            upload_id,
                            body: Some(crate::BodyContent::from_bytes(data, None)),
                            content_md5: None,
                            progress_fn: None,
                            request_payer,
                            traffic_limit: None,
                            common: Default::default(),
                        })
                        .await?;
                    Ok::<_, Box<dyn std::error::Error + Send + Sync>>((
                        part_number,
                        result.etag.clone().unwrap_or_default(),
                        size,
                        crc.sum64(),
                    ))
                }
            }))
            .buffer_unordered(checkpoint.parallel_num);
            let mut stream = std::pin::pin!(stream);
            while let Some(part) = stream.next().await {
                new_parts.push(part?);
            }
        }

        let mut combined = resume_crc;
        let mut all_parts: Vec<(i32, String)> = uploaded_parts.clone();
        new_parts.sort_by_key(|(number, _, _, _)| *number);
        for (number, etag, size, crc) in &new_parts {
            all_parts.push((*number, etag.clone()));
            combined = crc64_combine(combined, *crc, *size as u64);
        }
        all_parts.sort_by_key(|(number, _)| *number);

        let completed = self
            .complete_multipart_upload(&crate::api::object::CompleteMultipartUploadRequest {
                bucket: request.bucket.clone(),
                key: request.key.clone(),
                upload_id: upload_id.clone(),
                parts: all_parts
                    .iter()
                    .map(
                        |(number, etag)| crate::api::object::CompleteMultipartUploadPart {
                            part_number: *number,
                            etag: etag.clone(),
                        },
                    )
                    .collect(),
                request_payer: request.request_payer.clone(),
                common: request.common.clone(),
                ..Default::default()
            })
            .await?;

        // The checkpoint is removed only after the object exists, so a crash
        // between completion and removal resumes an upload that is already
        // done rather than losing the fact that it was.
        let _ = std::fs::remove_file(&checkpoint_path);

        Ok(CheckpointUploadResult {
            upload_id: Some(upload_id),
            written: new_parts.iter().map(|(_, _, size, _)| *size).sum(),
            resumed_from: resume_from,
            total: total_size,
            etag: completed.etag.clone(),
            completed: true,
        })
    }
}

/// Writes `data` at `offset` without moving the file cursor.
fn write_at(
    file: &std::fs::File,
    data: &[u8],
    offset: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::os::unix::fs::FileExt;
    let mut written = 0usize;
    while written < data.len() {
        let n = file.write_at(&data[written..], offset + written as u64)?;
        if n == 0 {
            return Err("write returned 0 bytes".into());
        }
        written += n;
    }
    Ok(())
}

/// Reads exactly `len` bytes at `offset`.
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

/// A checkpoint is keyed by the object and destination pair.
///
/// Exposed so callers can point a resumed transfer at the same checkpoint a
/// previous attempt used, and so a test can assert the naming is stable.
pub fn transfer_checkpoint_path(
    directory: &str,
    source_identity: &str,
    destination: &str,
    upload: bool,
) -> PathBuf {
    checkpoint_path(
        directory,
        source_identity,
        destination,
        if upload {
            CHECKPOINT_FILE_SUFFIX_UPLOADER
        } else {
            CHECKPOINT_FILE_SUFFIX_DOWNLOADER
        },
    )
}

/// The contiguous prefix a set of downloaded ranges supports, exposed for
/// callers that track ranges themselves.
///
/// Each range is half-open, `[start, end)`, matching the returned offset: the
/// first byte that still has to be downloaded.
pub fn downloaded_prefix(ranges: Vec<(i64, i64)>) -> i64 {
    let ranges = ranges
        .into_iter()
        .map(|(start, end)| DownloadedRange { start, end })
        .collect();
    contiguous_prefix(ranges, 0).0
}

/// Reads back the CRC of a partial upload's contiguous prefix, so a caller can
/// compare it with its own running checksum.
pub async fn resume_prefix_crc(
    client: &Client,
    bucket: &str,
    key: &str,
    upload_id: &str,
    part_size: i64,
) -> Result<(i64, u64, Vec<(i32, String)>), Box<dyn std::error::Error + Send + Sync>> {
    let mut parts: Vec<(i32, String)> = Vec::new();
    let mut offset = 0i64;
    let mut crc = 0u64;
    let mut marker: Option<i32> = None;
    loop {
        let page = client
            .list_parts(&ListPartsRequest {
                bucket: bucket.to_string(),
                key: key.to_string(),
                upload_id: upload_id.to_string(),
                max_parts: Some(1000),
                part_number_marker: marker,
                ..Default::default()
            })
            .await?;
        let mut page_parts: Vec<crate::api::object::Part> = Vec::new();
        page_parts.extend(page.parts.iter().map(|part| crate::api::object::Part {
            part_number: part.part_number,
            last_modified: part.last_modified.clone(),
            etag: part.etag.clone(),
            size: part.size,
            hash_crc64: part.hash_crc64.clone(),
        }));
        page_parts.sort_by_key(|part| part.part_number);
        for part in page_parts {
            if part.part_number != parts.len() as i32 + 1 || part.size != part_size {
                return Ok((offset, crc, parts));
            }
            parts.push((part.part_number, part.etag.clone()));
            offset += part.size;
            if let Some(value) = part
                .hash_crc64
                .as_deref()
                .and_then(|value| value.parse::<u64>().ok())
            {
                crc = crc64_combine(crc, value, part.size as u64);
            }
        }
        match page.next_part_number_marker {
            Some(next) if page.is_truncated.unwrap_or(false) => marker = Some(next),
            _ => break,
        }
    }
    Ok((offset, crc, parts))
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

    /// The resume point is the contiguous prefix, not the total bytes: a hole
    /// between two ranges must stop it.
    #[test]
    fn test_contiguous_prefix_stops_at_hole() {
        // Ranges [0,100) and [200,300) with a gap between them.
        assert_eq!(
            downloaded_prefix(vec![(0, 100), (200, 300)]),
            100,
            "the prefix must stop at the start of the gap"
        );
        // Contiguous ranges extend the prefix.
        assert_eq!(downloaded_prefix(vec![(0, 100), (100, 200)]), 200);
        // Out-of-order arrival still resolves to the same prefix.
        assert_eq!(downloaded_prefix(vec![(100, 200), (0, 100)]), 200);
        // An empty set has no prefix.
        assert_eq!(downloaded_prefix(vec![]), 0);
    }

    /// A checkpoint can only be picked up by the transfer that wrote it: a
    /// different destination must produce a different path.
    #[test]
    fn test_checkpoint_path_is_specific_to_transfer() {
        let a = transfer_checkpoint_path("/tmp", "oss://b/k", "/tmp/out-a.bin", false);
        let b = transfer_checkpoint_path("/tmp", "oss://b/k", "/tmp/out-b.bin", false);
        assert_ne!(a, b, "different destinations must not share a checkpoint");

        let other_source =
            transfer_checkpoint_path("/tmp", "oss://b/other", "/tmp/out-a.bin", false);
        assert_ne!(
            a, other_source,
            "different objects must not share a checkpoint"
        );

        let upload = transfer_checkpoint_path("/tmp", "oss://b/k", "/tmp/out-a.bin", true);
        assert_ne!(a, upload, "directions must not share a checkpoint");
    }

    /// A checkpoint whose bytes were altered must be rejected rather than
    /// resumed from.
    #[test]
    fn test_checkpoint_md5_guard() {
        let dir = std::env::temp_dir().join("oss-checkpoint-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("guard.json");

        let data = UploadInfo {
            upload_id: "up-1".to_string(),
        };
        write_checkpoint(&path, &data).unwrap();
        let read_back: Option<UploadInfo> = read_checkpoint(&path);
        assert_eq!(read_back.map(|d| d.upload_id), Some("up-1".to_string()));

        // Tampering with the payload invalidates the MD5.
        let contents = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, contents.replace("up-1", "up-2")).unwrap();
        let tampered: Option<UploadInfo> = read_checkpoint(&path);
        assert!(tampered.is_none(), "a tampered checkpoint must be rejected");
        assert!(!path.exists(), "a rejected checkpoint is removed");

        // A checkpoint from a different magic is not ours.
        std::fs::write(&path, "{\"Magic\":\"other\",\"MD5\":\"x\",\"Data\":{}}").unwrap();
        let foreign: Option<UploadInfo> = read_checkpoint(&path);
        assert!(foreign.is_none());
        let _ = std::fs::remove_file(&path);
    }

    /// A download that already has a checkpoint must resume from the recorded
    /// offset rather than re-fetching from the start.
    #[tokio::test]
    async fn test_download_resumes_from_checkpoint() {
        let mut server = mockito::Server::new_async().await;
        let part_size = 100i64;
        let total = 300i64;

        let head_mock = server
            .mock("HEAD", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-length", &total.to_string())
            .with_header("etag", "\"v1\"")
            .expect(1)
            .create_async()
            .await;

        // Only the tail may be requested, in the parts the part size implies;
        // the first byte range is already on disk.
        let tail_first = server
            .mock("GET", mockito::Matcher::Any)
            .match_header("range", "bytes=100-199")
            .with_status(206)
            .with_body("b".repeat(100))
            .expect(1)
            .create_async()
            .await;
        let tail_second = server
            .mock("GET", mockito::Matcher::Any)
            .match_header("range", "bytes=200-299")
            .with_status(206)
            .with_body("c".repeat(100))
            .expect(1)
            .create_async()
            .await;
        let full = server
            .mock("GET", mockito::Matcher::Any)
            .match_header("range", "bytes=0-299")
            .with_status(206)
            .expect(0)
            .create_async()
            .await;

        let dir = std::env::temp_dir().join("oss-checkpoint-resume-dl");
        std::fs::create_dir_all(&dir).unwrap();
        let destination = dir.join("resume.bin");
        std::fs::write(&destination, b"").unwrap();

        // Seed a checkpoint claiming the first part is done.
        let checkpoint_path = transfer_checkpoint_path(
            dir.to_str().unwrap(),
            "oss://test-bucket/test-key\n\n",
            destination.to_str().unwrap(),
            false,
        );
        write_checkpoint(
            &checkpoint_path,
            &DownloadData {
                object_info: DownloadObjectInfo {
                    name: "oss://test-bucket/test-key".to_string(),
                    version_id: String::new(),
                    range: String::new(),
                },
                object_meta: DownloadObjectMeta {
                    size: total,
                    last_modified: String::new(),
                    etag: "\"v1\"".to_string(),
                },
                file_path: destination.to_string_lossy().to_string(),
                part_size,
                download_info: DownloadInfo {
                    offset: 100,
                    crc64: 0,
                },
            },
        )
        .unwrap();
        std::fs::write(&destination, vec![b'a'; 100]).unwrap();

        let result = mock_client(&server)
            .download_file_with_checkpoint(
                &crate::api::object::GetObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "test-key".to_string(),
                    ..Default::default()
                },
                &destination,
                part_size,
                DownloaderCheckpointOptions::default().with_checkpoint_dir(dir.to_str().unwrap()),
            )
            .await
            .expect("a resumed download should succeed");

        assert_eq!(result.resumed_from, 100, "it must resume, not restart");
        assert_eq!(result.written, 200, "only the tail is downloaded");
        assert!(result.completed);

        // The finished file must hold the prefix plus the downloaded tail.
        let contents = std::fs::read(dir.join("resume.bin.temp")).unwrap_or_default();
        let on_disk = if contents.is_empty() {
            std::fs::read(&destination).unwrap()
        } else {
            contents
        };
        assert_eq!(on_disk.len(), total as usize);
        assert_eq!(&on_disk[..100], &vec![b'a'; 100][..]);

        tail_first.assert_async().await;
        tail_second.assert_async().await;
        full.assert_async().await;
        head_mock.assert_async().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// An upload with a checkpoint must adopt the recorded upload ID and only
    /// send the parts that `ListParts` does not already report.
    #[tokio::test]
    async fn test_upload_resumes_existing_upload() {
        let mut server = mockito::Server::new_async().await;
        let part_size = MIN_PART_SIZE;
        let total = part_size * 2;

        let dir = std::env::temp_dir().join("oss-checkpoint-resume-ul");
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("resume-src.bin");
        std::fs::write(&source, vec![7u8; total as usize]).unwrap();
        let modified = std::fs::metadata(&source)
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        // The parts that exist: only part 1.
        server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploadId=up-resume".to_string()))
            .with_status(200)
            .with_body(format!(
                "<ListPartsResult><IsTruncated>false</IsTruncated><Part><PartNumber>1</\
                 PartNumber><ETag>\"p1\"</ETag><Size>{part_size}</Size></Part></ListPartsResult>"
            ))
            .create_async()
            .await;
        // Only part 2 may be uploaded.
        let part_two = server
            .mock("PUT", mockito::Matcher::Any)
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex("partNumber=2".to_string()),
                mockito::Matcher::Regex("uploadId=up-resume".to_string()),
            ]))
            .with_status(200)
            .with_header("etag", "\"p2\"")
            .expect(1)
            .create_async()
            .await;
        let part_one = server
            .mock("PUT", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("partNumber=1".to_string()))
            .with_status(200)
            .expect(0)
            .create_async()
            .await;
        let complete = server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploadId=up-resume".to_string()))
            .with_status(200)
            .with_body(
                "<CompleteMultipartUploadResult><ETag>\"final\"</ETag></\
                 CompleteMultipartUploadResult>",
            )
            .expect(1)
            .create_async()
            .await;

        let checkpoint_path = transfer_checkpoint_path(
            dir.to_str().unwrap(),
            "oss://test-bucket/resume-key",
            source.to_str().unwrap(),
            true,
        );
        write_checkpoint(
            &checkpoint_path,
            &UploadData {
                file_path: source.to_string_lossy().to_string(),
                file_meta: UploadFileMeta {
                    size: total,
                    last_modified: modified,
                },
                object_info: UploadObjectInfo {
                    name: "oss://test-bucket/resume-key".to_string(),
                },
                part_size,
                upload_info: UploadInfo {
                    upload_id: "up-resume".to_string(),
                },
            },
        )
        .unwrap();

        let result = mock_client(&server)
            .upload_file_with_checkpoint(
                &PutObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "resume-key".to_string(),
                    ..Default::default()
                },
                &source,
                UploaderCheckpointOptions::default()
                    .with_checkpoint_dir(dir.to_str().unwrap())
                    .with_part_size(part_size),
            )
            .await
            .expect("a resumed upload should succeed");

        assert_eq!(
            result.resumed_from, part_size,
            "it must adopt the existing part"
        );
        assert_eq!(
            result.written, part_size,
            "only the missing part is uploaded"
        );
        assert_eq!(result.etag.as_deref(), Some("\"final\""));
        part_two.assert_async().await;
        part_one.assert_async().await;
        complete.assert_async().await;
        assert!(
            !checkpoint_path.exists(),
            "a completed transfer removes its checkpoint"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A checkpoint for a different file must not be adopted.
    #[tokio::test]
    async fn test_upload_ignores_checkpoint_for_other_file() {
        let mut server = mockito::Server::new_async().await;
        let part_size = MIN_PART_SIZE;

        let dir = std::env::temp_dir().join("oss-checkpoint-other");
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("other-src.bin");
        std::fs::write(&source, vec![3u8; part_size as usize]).unwrap();

        // No ListParts may be attempted: the checkpoint does not describe this
        // file, so the transfer must start fresh.
        let list = server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploadId=stale".to_string()))
            .with_status(200)
            .expect(0)
            .create_async()
            .await;
        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploads".to_string()))
            .with_status(200)
            .with_body(
                "<InitiateMultipartUploadResult><UploadId>fresh</UploadId></\
                 InitiateMultipartUploadResult>",
            )
            .create_async()
            .await;
        server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(200)
            .with_header("etag", "\"p1\"")
            .create_async()
            .await;
        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploadId=fresh".to_string()))
            .with_status(200)
            .with_body(
                "<CompleteMultipartUploadResult><ETag>\"done\"</ETag></\
                 CompleteMultipartUploadResult>",
            )
            .create_async()
            .await;

        let checkpoint_path = transfer_checkpoint_path(
            dir.to_str().unwrap(),
            "oss://test-bucket/resume-key",
            source.to_str().unwrap(),
            true,
        );
        // A checkpoint naming a different size, which cannot describe this
        // file's contents.
        write_checkpoint(
            &checkpoint_path,
            &UploadData {
                file_path: source.to_string_lossy().to_string(),
                file_meta: UploadFileMeta {
                    size: 999_999,
                    last_modified: "12345".to_string(),
                },
                object_info: UploadObjectInfo {
                    name: "oss://test-bucket/resume-key".to_string(),
                },
                part_size,
                upload_info: UploadInfo {
                    upload_id: "stale".to_string(),
                },
            },
        )
        .unwrap();

        let result = mock_client(&server)
            .upload_file_with_checkpoint(
                &PutObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "resume-key".to_string(),
                    ..Default::default()
                },
                &source,
                UploaderCheckpointOptions::default()
                    .with_checkpoint_dir(dir.to_str().unwrap())
                    .with_part_size(part_size),
            )
            .await
            .expect("a fresh upload should succeed");

        assert_eq!(
            result.resumed_from, 0,
            "a stale checkpoint must not be used"
        );
        assert_eq!(result.upload_id.as_deref(), Some("fresh"));
        list.assert_async().await;
        let _ = std::fs::remove_dir_all(&dir);
    }
}
