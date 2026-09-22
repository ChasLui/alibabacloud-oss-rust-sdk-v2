use std::collections::HashMap;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use futures_util::StreamExt;

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_md5};
use crate::{BodyStream, OperationInput, OperationOutput};

#[derive(Default, OssRequestModel)]
pub struct GetObjectRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// If the ETag specified in the request matches the ETag value of the
    /// object, the object and 200 OK are returned. Otherwise, 412
    /// Precondition Failed is returned.
    #[field(type = "header", rename = "If-Match")]
    pub if_match: Option<String>,

    /// If the ETag specified in the request does not match the ETag value of
    /// the object, the object and 200 OK are returned. Otherwise, 304 Not
    /// Modified is returned.
    #[field(type = "header", rename = "If-None-Match")]
    pub if_none_match: Option<String>,

    /// If the time specified in this header is earlier than the object modified
    /// time or is invalid, the object and 200 OK are returned. Otherwise,
    /// 304 Not Modified is returned. The time must be in GMT. Example: Fri,
    /// 13 Nov 2015 14:47:53 GMT.
    #[field(type = "header", rename = "If-Modified-Since")]
    pub if_modified_since: Option<String>,

    /// If the time specified in this header is the same as or later than the
    /// object modified time, the object and 200 OK are returned. Otherwise,
    /// 412 Precondition Failed is returned. The time must be in GMT.
    /// Example: Fri, 13 Nov 2015 14:47:53 GMT.
    #[field(type = "header", rename = "If-Unmodified-Since")]
    pub if_unmodified_since: Option<String>,

    /// The content range of the object to be returned.
    /// If the value of Range is valid, the total size of the object and the
    /// content range are returned. For example, `Range: bytes=0-9`
    /// indicates that the range of data returned is the first 10 bytes.
    /// However, if the value of Range is invalid, the entire object is
    /// returned, and the response does not include the Content-Range
    /// parameter.
    #[field(type = "header", rename = "Range")]
    pub range: Option<String>,

    /// Specify standard behaviors to download data by range
    /// If the value is "standard", the download behavior is modified when the
    /// specified range is not within the valid range. For an object whose
    /// size is 1,000 bytes:
    /// 1) If you set Range: bytes to 500-2000, the value at the end of the
    ///    range is invalid. In this case, OSS returns HTTP status code 206 and
    ///    the data that is within the range of byte 500 to byte 999.
    /// 2) If you set Range: bytes to 1000-2000, the value at the start of the
    ///    range is invalid. In this case, OSS returns HTTP status code 416 and
    ///    the InvalidRange error code.
    #[field(type = "header", rename = "x-oss-range-behavior")]
    pub range_behavior: Option<String>,

    /// The cache-control header to be returned in the response.
    #[field(type = "query", rename = "response-cache-control")]
    pub response_cache_control: Option<String>,

    /// The content-disposition header to be returned in the response.
    #[field(type = "query", rename = "response-content-disposition")]
    pub response_content_disposition: Option<String>,

    /// The content-encoding header to be returned in the response.
    #[field(type = "query", rename = "response-content-encoding")]
    pub response_content_encoding: Option<String>,

    /// The content-language header to be returned in the response.
    #[field(type = "query", rename = "response-content-language")]
    pub response_content_language: Option<String>,

    /// The content-type header to be returned in the response.
    #[field(type = "query", rename = "response-content-type")]
    pub response_content_type: Option<String>,

    /// The expires header to be returned in the response.
    #[field(type = "query", rename = "response-expires")]
    pub response_expires: Option<String>,

    /// VersionId used to reference a specific version of the object.
    #[field(type = "query", rename = "versionId")]
    pub version_id: Option<String>,

    /// Specify the speed limit value. The speed limit value ranges from 245760
    /// to 838860800, with a unit of bit/s.
    #[field(type = "header", rename = "x-oss-traffic-limit")]
    pub traffic_limit: Option<u64>,

    /// Progress callback function
    pub progress_fn: Option<Box<dyn Fn(i64, i64) + Send + Sync>>,

    /// Image processing parameters
    #[field(type = "query", rename = "x-oss-process")]
    pub process: Option<String>,

    /// To indicate that the requester is aware that the request and data
    /// download will incur costs
    #[field(type = "header", rename = "x-oss-request-payer")]
    pub request_payer: Option<String>,

    pub common: RequestCommon,
}

#[derive(Default, OssResultModel)]
pub struct GetObjectResult {
    /// Size of the body in bytes. -1 indicates that the Content-Length does not
    /// exist.
    #[field(type = "header", rename = "Content-Length")]
    pub content_length: Option<u64>,

    /// The portion of the object returned in the response.
    #[field(type = "header", rename = "Content-Range")]
    pub content_range: Option<String>,

    /// A standard MIME type describing the format of the object data.
    #[field(type = "header", rename = "Content-Type")]
    pub content_type: Option<String>,

    /// The entity tag (ETag). An ETag is created when an object is created to
    /// identify the content of the object.
    #[field(type = "header", rename = "ETag")]
    pub etag: Option<String>,

    /// The time when the returned objects were last modified.
    #[field(type = "header", rename = "Last-Modified")]
    pub last_modified: Option<String>,

    /// The storage class of the object.
    #[field(type = "header", rename = "x-oss-storage-class")]
    pub storage_class: Option<String>,

    /// Content-Md5 for the uploaded object.
    #[field(type = "header", rename = "Content-MD5")]
    pub content_md5: Option<String>,

    /// A map of metadata to store with the object.
    #[field(type = "header", rename = "x-oss-meta-", usermeta)]
    pub metadata: HashMap<String, String>,

    /// If the requested object is encrypted by using a server-side encryption
    /// algorithm based on entropy encoding, OSS automatically decrypts the
    /// object and returns the decrypted object after OSS receives the GetObject
    /// request. The x-oss-server-side-encryption header is included in the
    /// response to indicate the encryption algorithm used to encrypt the
    /// object on the server.
    #[field(type = "header", rename = "x-oss-server-side-encryption")]
    pub server_side_encryption: Option<String>,

    /// The ID of the customer master key (CMK) that is managed by Key
    /// Management Service (KMS).
    #[field(type = "header", rename = "x-oss-server-side-data-encryption")]
    pub server_side_data_encryption: Option<String>,

    /// The ID of the customer master key (CMK) that is managed by Key
    /// Management Service (KMS).
    #[field(type = "header", rename = "x-oss-server-side-encryption-key-id")]
    pub sse_kms_key_id: Option<String>,

    /// The type of the object.
    #[field(type = "header", rename = "x-oss-object-type")]
    pub object_type: Option<String>,

    /// The position for the next append operation.
    /// If the type of the object is Appendable, this header is included in the
    /// response.
    #[field(type = "header", rename = "x-oss-next-append-position")]
    pub next_append_position: Option<String>,

    /// The 64-bit CRC value of the object.
    /// This value is calculated based on the ECMA-182 standard.
    #[field(type = "header", rename = "x-oss-hash-crc64ecma")]
    pub hash_crc64: Option<String>,

    /// The lifecycle information about the object.
    /// If lifecycle rules are configured for the object, this header is
    /// included in the response. This header contains the following
    /// parameters: expiry-date that indicates the expiration time of the
    /// object, and rule-id that indicates the ID of the matched lifecycle
    /// rule.
    #[field(type = "header", rename = "x-oss-expiration")]
    pub expiration: Option<String>,

    /// The status of the object when you restore an object.
    /// If the storage class of the bucket is Archive and a RestoreObject
    /// request is submitted,
    #[field(type = "header", rename = "x-oss-restore")]
    pub restore: Option<String>,

    /// The result of an event notification that is triggered for the object.
    #[field(type = "header", rename = "x-oss-process-status")]
    pub process_status: Option<String>,

    /// The number of tags added to the object.
    /// This header is included in the response only when you have read
    /// permissions on tags.
    #[field(type = "header", rename = "x-oss-tagging-count")]
    pub tagging_count: Option<u32>,

    /// Specifies whether the object retrieved was (true) or was not (false) a
    /// Delete Marker.
    #[field(type = "header", rename = "x-oss-delete-marker")]
    pub delete_marker: Option<bool>,

    /// Version of the object.
    #[field(type = "header", rename = "x-oss-version-id")]
    pub version_id: Option<String>,

    /// Object data.
    pub body: Option<BodyStream>,

    pub common: ResultCommon,
}

impl std::fmt::Debug for GetObjectResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GetObjectResult")
            .field("content_length", &self.content_length)
            .field("content_range", &self.content_range)
            .field("content_type", &self.content_type)
            .field("etag", &self.etag)
            .field("last_modified", &self.last_modified)
            .field("storage_class", &self.storage_class)
            .field("content_md5", &self.content_md5)
            .field("server_side_encryption", &self.server_side_encryption)
            .field(
                "server_side_data_encryption",
                &self.server_side_data_encryption,
            )
            .field("sse_kms_key_id", &self.sse_kms_key_id)
            .field("object_type", &self.object_type)
            .field("next_append_position", &self.next_append_position)
            .field("hash_crc64", &self.hash_crc64)
            .field("expiration", &self.expiration)
            .field("restore", &self.restore)
            .field("process_status", &self.process_status)
            .field("tagging_count", &self.tagging_count)
            .field("delete_marker", &self.delete_marker)
            .field("version_id", &self.version_id)
            .field("metadata", &self.metadata)
            .field("body_stream", &"<stream>") // Don't print the stream itself
            .field("common", &self.common)
            .finish()
    }
}

// 为 GetObjectResult 实现 BodyDataReader trait
impl crate::client::BodyDataReader for GetObjectResult {
    fn take_body(&mut self) -> Option<crate::client::BodyStream> {
        self.body.take()
    }

    fn set_body(&mut self, body: Option<crate::client::BodyStream>) {
        self.body = body;
    }
}

impl Client {
    /// Retrieves an object from the OSS bucket.
    ///
    /// This method sends a GET request to the OSS API to retrieve the specified
    /// object. It takes an `GetObjectRequest` as input and returns a
    /// `Result` containing the `GetObjectResult` or an error implementing
    /// the `std::error::Error` trait.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetObjectRequest` specifying the bucket and key of
    ///   the object to retrieve.
    /// # Returns
    ///
    /// A `Result` containing the `GetObjectResult` if the object was retrieved
    /// successfully, or an error implementing the `std::error::Error` trait
    /// if an error occurred.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::{GetObjectRequest, GetObjectResult};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = GetObjectRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// match client.get_object(request).await {
    ///     Ok(_result) => {
    ///         // Object retrieved successfully
    ///         println!("Object retrieved successfully");
    ///     }
    ///     Err(err) => {
    ///         // Error occurred while retrieving the object
    ///         eprintln!("Error: {}", err);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn get_object(
        &self,
        mut request: GetObjectRequest,
    ) -> Result<GetObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetObject".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5],
        )?;

        if let Some(progress_fn) = request.progress_fn.take() {
            crate::utils::add_response_progress_tracker(&mut input, progress_fn);
        }

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = GetObjectResult::default();

        result.update_result(&output);

        result.body = output.body;

        Ok(result)
    }

    /// Downloads an object into a local file.
    ///
    /// When `ENABLE_CRC64_CHECK_DOWNLOAD` is set on the client, the bytes
    /// written to disk are accumulated into a CRC64 that is compared against
    /// the object's `x-oss-hash-crc64ecma` once the download completes. A
    /// mismatch is reported with the same `crc is inconsistent` message as the
    /// upload check, which `ConnectionErrorRetryable` treats as retryable — so
    /// a corrupted download is retried rather than silently accepted.
    ///
    /// A ranged download is **not** checksum-verified: the server reports the
    /// CRC64 of the whole object, so a partial body can never match it.
    /// Mirrors Go `Client.GetObjectToFile`.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetObjectRequest` describing the object to download.
    /// * `file_path` - Where the object is written. An existing file is
    ///   truncated; the parent directory must exist.
    pub async fn get_object_to_file(
        &self,
        request: GetObjectRequest,
        file_path: impl AsRef<std::path::Path>,
    ) -> Result<GetObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = file_path.as_ref();
        let check_crc = self
            .options
            .feature_flags
            .contains(crate::FeatureFlagsType::ENABLE_CRC64_CHECK_DOWNLOAD)
            && request.range.is_none();

        // A failed checksum makes the whole download retryable: the bytes that
        // reached disk are known to be wrong, so the only useful recovery is
        // to fetch the object again. Each attempt truncates the file and
        // restarts the checksum, so a retry pays off exactly when the
        // corruption was transient. Mirrors Go `Client.GetObjectToFile`, which
        // loops until the checksum validates or the attempt budget runs out.
        let max_attempts = self.retry_max_attempts(None).max(1);
        let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;

        for attempt in 1..=max_attempts {
            match self
                .download_object_to_file(&request, file_path, check_crc)
                .await
            {
                Ok(result) => return Ok(result),
                Err(err) => {
                    if attempt == max_attempts {
                        return Err(err);
                    }
                    last_error = Some(err);
                }
            }
        }

        Err(last_error.expect("a failed download always records the last error"))
    }

    /// One attempt of [`Client::get_object_to_file`]: issues the request,
    /// streams the body to disk, and verifies the checksum when requested.
    async fn download_object_to_file(
        &self,
        request: &GetObjectRequest,
        file_path: &std::path::Path,
        check_crc: bool,
    ) -> Result<GetObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "GetObject".to_string(),
            method: http::Method::GET,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let mut result = GetObjectResult::default();
        result.update_result(&output);

        let body = output
            .body
            .ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> {
                "GetObject returned no body".into()
            })?;

        // The body is streamed straight to disk so a large object never has to
        // fit in memory; the checksum rides the same path.
        let mut file = tokio::fs::File::create(file_path).await?;
        let mut crc = crate::utils::Crc64::new(0);
        let mut body = std::pin::pin!(body);

        while let Some(chunk) = body.next().await {
            let chunk = chunk?;
            if check_crc {
                crc.write(&chunk)
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                        e.to_string().into()
                    })?;
            }
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
        }
        tokio::io::AsyncWriteExt::flush(&mut file).await?;

        if check_crc {
            crate::utils::check_crc64(crc.sum64(), result.hash_crc64.as_deref())
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
        }

        Ok(result)
    }

    /// Downloads the object into a local file, reconnecting where it stopped.
    ///
    /// Compared to [`Client::get_object_to_file`], this:
    ///
    /// 1. Buffers writes when `write_buffer_size` is set, so a large object
    ///    costs fewer write syscalls. `None` writes each chunk straight
    ///    through.
    /// 2. Reconnects from the byte it stopped at when the connection drops, so
    ///    an unstable link does not restart the download. Each reconnect
    ///    re-requests only the bytes that are still missing.
    ///
    /// A reconnect refuses to continue if the object's ETag changed: the file
    /// would otherwise be a splice of two different objects, which is worse
    /// than a failed download. Mirrors Go `Client.GetObjectToFileV2`.
    ///
    /// # Arguments
    ///
    /// * `request` - The `GetObjectRequest` describing the object to download.
    /// * `file_path` - Where the object is written. An existing file is
    ///   truncated; the parent directory must exist.
    /// * `write_buffer_size` - Bytes to buffer before writing through, or
    ///   `None` to write each chunk as it arrives.
    pub async fn get_object_to_file_v2(
        &self,
        mut request: GetObjectRequest,
        file_path: impl AsRef<std::path::Path>,
        write_buffer_size: Option<usize>,
    ) -> Result<GetObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = file_path.as_ref();

        // Where the caller wants the download to start, and how much of it. A
        // count of zero means "to the end of the object".
        let requested = match request.range.as_deref() {
            Some(range) => Some(crate::utils::parse_range(range).map_err(
                |e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() },
            )?),
            None => None,
        };
        let start = requested.as_ref().map(|range| range.offset).unwrap_or(0);
        let requested_count = requested.as_ref().map(|range| range.count).unwrap_or(0);

        // A ranged download sees a slice of the object, so the whole-object
        // CRC cannot be checked against it.
        let check_crc = self
            .options
            .feature_flags
            .contains(crate::FeatureFlagsType::ENABLE_CRC64_CHECK_DOWNLOAD)
            && request.range.is_none();

        // The callback reports the whole download rather than one attempt, so
        // it is taken out of the request before the loop starts rebuilding it.
        let progress_fn = request.progress_fn.take();

        let file = tokio::fs::File::create(file_path).await?;
        let mut writer: Box<dyn tokio::io::AsyncWrite + Unpin + Send> = match write_buffer_size {
            Some(size) if size > 0 => Box::new(tokio::io::BufWriter::with_capacity(size, file)),
            _ => Box::new(file),
        };

        let mut offset = start;
        let mut transferred: i64 = 0;
        let mut crc = crate::utils::Crc64::new(0);
        let mut first: Option<GetObjectResult> = None;
        let mut etag: Option<String> = None;
        let mut total: i64 = -1;

        loop {
            // What is left of the caller's range, counted from where this
            // attempt picks up.
            let remaining = if requested_count > 0 {
                requested_count - (offset - start)
            } else {
                0
            };
            if requested_count > 0 && remaining <= 0 {
                break;
            }

            // A whole-object download carries no Range header at all, which is
            // what keeps its response a 200 rather than a 206.
            let range_header = if offset == 0 && remaining == 0 {
                None
            } else if remaining > 0 {
                Some(format!("bytes={}-{}", offset, offset + remaining - 1))
            } else {
                Some(format!("bytes={}-", offset))
            };

            let mut input = OperationInput {
                op_name: "GetObject".to_string(),
                method: http::Method::GET,
                bucket: Some(request.bucket.clone()),
                key: Some(request.key.clone()),
                ..Default::default()
            };
            let mut headers = request.header_map();
            match &range_header {
                Some(range) => {
                    headers.insert("Range".to_string(), range.clone());
                    // `standard` stops the server answering an out-of-range
                    // request with the whole object, which would append bytes
                    // the caller never asked for.
                    headers.insert("x-oss-range-behavior".to_string(), "standard".to_string());
                }
                None => {
                    headers.remove("Range");
                    headers.remove("x-oss-range-behavior");
                }
            }
            modify_request(
                &mut input,
                headers,
                request.query_map(),
                vec![update_content_md5],
            )?;

            let output = self.invoke_operation(input, vec![]).await?;
            let mut result = GetObjectResult::default();
            result.update_result(&output);

            // Where the response actually starts, and how large the object is.
            let (response_start, response_total) = match result.content_range.as_deref() {
                Some(content_range) => {
                    let (from, _, total) = crate::utils::parse_content_range(content_range)
                        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                            e.to_string().into()
                        })?;
                    (from, total)
                }
                None => (0, result.content_length.map(|len| len as i64).unwrap_or(-1)),
            };
            if response_start != offset {
                return Err(format!(
                    "range get returned the wrong offset: asked for {}, got {}",
                    offset, response_start
                )
                .into());
            }

            match first {
                None => {
                    // The first response fixes the object's identity and size.
                    etag = result.etag.clone();
                    total = response_total;
                    first = Some(result);
                }
                Some(_) => {
                    // A reconnect that lands on a different object would write
                    // a file made of two of them.
                    if result.etag != etag {
                        return Err(format!(
                            "source object changed during the download: expected etag {:?}, got \
                             {:?}",
                            etag, result.etag
                        )
                        .into());
                    }
                }
            }

            let body = output
                .body
                .ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> {
                    "GetObject returned no body".into()
                })?;
            let mut body = std::pin::pin!(body);
            let before = offset;
            let mut interrupted = false;

            while let Some(chunk) = body.next().await {
                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(_) => {
                        // The connection dropped mid-body. Whatever arrived is
                        // already on its way to disk, so the next attempt only
                        // asks for what is missing.
                        interrupted = true;
                        break;
                    }
                };
                if check_crc {
                    crc.write(&chunk)
                        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                            e.to_string().into()
                        })?;
                }
                tokio::io::AsyncWriteExt::write_all(&mut writer, &chunk).await?;
                offset += chunk.len() as i64;
                transferred += chunk.len() as i64;
                if let Some(progress_fn) = &progress_fn {
                    progress_fn(transferred, total);
                }
            }

            if !interrupted {
                break;
            }
            if offset == before {
                // A reconnect that moves nothing would spin forever.
                return Err(format!("download stalled at byte {}", offset).into());
            }
        }

        tokio::io::AsyncWriteExt::flush(&mut writer).await?;

        // A body that ends early would leave a truncated file behind, so the
        // byte count is checked rather than assumed.
        let expected = if requested_count > 0 {
            requested_count
        } else if total > 0 {
            total - start
        } else {
            -1
        };
        if expected >= 0 && offset - start != expected {
            return Err(format!(
                "download ended after {} of {} bytes",
                offset - start,
                expected
            )
            .into());
        }

        let first = first.ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> {
            "download made no request".into()
        })?;

        if check_crc {
            crate::utils::check_crc64(crc.sum64(), first.hash_crc64.as_deref())
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
        }

        Ok(first)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::api::object::tests::{
        delete_multiple, put, put_with_size, TEST_OBJECT_CONTENT, TEST_OBJECT_NAME,
    };
    use crate::client::BodyDataReader;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::log::LogLevel;
    use crate::test_utils::load_test_config;
    use crate::{SignatureVersionType, HTTP_HEADER_CONTENT_RANGE};

    pub(super) async fn get_by_range(
        client: &Client,
        bucket: &str,
        range: &str,
    ) -> Result<GetObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        client
            .get_object(GetObjectRequest {
                bucket: bucket.to_string(),
                key: TEST_OBJECT_NAME.to_string(),
                range: Some(range.to_string()),
                ..Default::default()
            })
            .await
    }

    // Configuration structure to hold test credentials
    // Using shared TestConfig from test_utils

    // Function to load test configuration from file
    // Using shared load_test_config from test_utils

    /// A download whose body does not match the server's
    /// `x-oss-hash-crc64ecma` must fail and must leave the wrong bytes off
    /// disk only if the check runs — the error, not a silent success, is the
    /// contract.
    #[tokio::test]
    async fn test_get_object_to_file_crc_mismatch_fails() {
        let mut server = mockito::Server::new_async().await;

        // A permanently wrong checksum is retried until the attempt budget
        // runs out, so the mock has to answer every attempt.
        let attempts = crate::retry::DEFAULT_MAX_ATTEMPTS;
        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("x-oss-hash-crc64ecma", "1") // never the CRC of "payload"
            .with_body("payload")
            .expect(attempts as usize)
            .create_async()
            .await;

        let client = Client::new(
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
        );

        let dir = std::env::temp_dir().join("oss-crc-download-test");
        std::fs::create_dir_all(&dir).expect("Failed to create temp dir");
        let path = dir.join("mismatch.bin");

        let err = client
            .get_object_to_file(
                GetObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "crc-object".to_string(),
                    ..Default::default()
                },
                &path,
            )
            .await
            .expect_err("a CRC mismatch must fail the download");

        assert!(
            err.to_string().contains("crc is inconsistent"),
            "unexpected error: {}",
            err
        );
        mock.assert_async().await;

        let _ = std::fs::remove_file(&path);
    }

    /// The same download with the correct checksum succeeds and the file
    /// contents match what the server sent.
    #[tokio::test]
    async fn test_get_object_to_file_crc_match_succeeds() {
        let mut server = mockito::Server::new_async().await;

        let expected = {
            let mut crc = crate::utils::Crc64::new(0);
            crc.write(b"payload").unwrap();
            crc.sum64().to_string()
        };

        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("x-oss-hash-crc64ecma", expected.as_str())
            .with_body("payload")
            .create_async()
            .await;

        let client = Client::new(
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
        );

        let dir = std::env::temp_dir().join("oss-crc-download-test");
        std::fs::create_dir_all(&dir).expect("Failed to create temp dir");
        let path = dir.join("match.bin");

        client
            .get_object_to_file(
                GetObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "crc-object".to_string(),
                    ..Default::default()
                },
                &path,
            )
            .await
            .expect("a matching CRC must succeed");
        mock.assert_async().await;

        let written = std::fs::read(&path).expect("downloaded file must exist");
        assert_eq!(written, b"payload");

        let _ = std::fs::remove_file(&path);
    }

    /// A ranged download is not checksum-verified: the server reports the CRC
    /// of the whole object, so a partial body could never match. A wrong
    /// header must therefore NOT fail the download.
    #[tokio::test]
    async fn test_get_object_to_file_skips_crc_for_ranged_download() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(206)
            .with_header("x-oss-hash-crc64ecma", "1") // whole-object CRC
            .with_body("part")
            .create_async()
            .await;

        let client = Client::new(
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
        );

        let dir = std::env::temp_dir().join("oss-crc-download-test");
        std::fs::create_dir_all(&dir).expect("Failed to create temp dir");
        let path = dir.join("ranged.bin");

        client
            .get_object_to_file(
                GetObjectRequest {
                    bucket: "test-bucket".to_string(),
                    key: "crc-object".to_string(),
                    range: Some("bytes=0-3".to_string()),
                    ..Default::default()
                },
                &path,
            )
            .await
            .expect("a ranged download must not be checksum-verified");
        mock.assert_async().await;

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_object_by_valid_range() {
        let config = match load_test_config() {
            Some(cfg) => cfg,
            None => {
                eprintln!("Test configuration not found. Skipping test.");
                return;
            }
        };

        let client = Client::new(
            &Config::default()
                .with_region(&config.region)
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    &config.access_key_id,
                    &config.access_key_secret,
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V4)
                .with_log_level(LogLevel::Debug),
        );

        // put object
        match put(&client, &config.bucket).await {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

        // test get object by valid range
        let length = TEST_OBJECT_CONTENT.len();
        let begin = rand::random::<usize>() % length;
        let end = begin + rand::random::<usize>() % (length - begin);
        let range = format!("bytes={}-{}", begin, end);

        match get_by_range(&client, &config.bucket, &range).await {
            Ok(mut result) => {
                println!("{:?}", result);

                // check status
                assert_eq!(result.common.status, http::StatusCode::PARTIAL_CONTENT);
                // return partial object content
                let content_bytes = result.get_all_data().await.unwrap_or_default();
                let content = String::from_utf8_lossy(&content_bytes).into_owned();
                assert_eq!(content, TEST_OBJECT_CONTENT[begin..=end]);
                // header Content-Range should be correctly set
                assert!(result.common.headers.iter().any(|(key, val)| key
                    .eq_ignore_ascii_case(HTTP_HEADER_CONTENT_RANGE)
                    && val.eq(&format!("bytes {}-{}/{}", begin, end, length))));
            }
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

        // delete object
        match delete_multiple(&client, &config.bucket).await {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_object_by_invalid_range() {
        let config = match load_test_config() {
            Some(cfg) => cfg,
            None => {
                eprintln!("Test configuration not found. Skipping test.");
                return;
            }
        };

        let client = Client::new(
            &Config::default()
                .with_region(&config.region)
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    &config.access_key_id,
                    &config.access_key_secret,
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V4)
                .with_log_level(LogLevel::Debug),
        );

        // put object
        match put(&client, &config.bucket).await {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

        // test get object by invalid range (maximum valid range:
        // "bytes=0-{len-1}")
        let range = format!("bytes={}-{}", 0, TEST_OBJECT_CONTENT.len());

        match get_by_range(&client, &config.bucket, &range).await {
            Ok(mut result) => {
                println!("{:?}", result);

                // check status (not 206)
                assert_eq!(result.common.status, http::StatusCode::OK);
                // return full object content
                let content_bytes = result.get_all_data().await.unwrap_or_default();
                let content = String::from_utf8_lossy(&content_bytes).into_owned();
                assert_eq!(content, TEST_OBJECT_CONTENT);
                // header Content-Range should not be set
                assert!(!result
                    .common
                    .headers
                    .keys()
                    .any(|key| key.eq_ignore_ascii_case(HTTP_HEADER_CONTENT_RANGE)));
            }
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

        // delete object
        match delete_multiple(&client, &config.bucket).await {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_object_with_nonexistent_version() {
        let config = match load_test_config() {
            Some(cfg) => cfg,
            None => {
                eprintln!("Test configuration not found. Skipping test.");
                return;
            }
        };

        let client = Client::new(
            &Config::default()
                .with_region(&config.region)
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    &config.access_key_id,
                    &config.access_key_secret,
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V4)
                .with_log_level(LogLevel::Debug),
        );

        // Attempt to get an object with a fake/nonexistent version ID
        // This should return a 404 error since the specific version doesn't
        // exist
        let request = GetObjectRequest {
            bucket: config.bucket.to_string(),
            key: "nonexistent-object-key".to_string(), // Use a key that doesn't exist
            version_id: Some(
                "CAEQUhiBgMDwk5fQ3hkiIDQ1NmExZTM4YzcyYTRlZmU5NjViNjE2YmQwZDU0MTc0".to_string(),
            ), /* Specify a version that definitely
                                                        * doesn't exist */
            ..Default::default()
        };

        match client.get_object(request).await {
            Ok(result) => {
                // We expect a 404 error code
                assert_eq!(result.common.status, http::StatusCode::NOT_FOUND);
                println!(
                    "Got expected 404 error when requesting nonexistent object version: {:?}",
                    result
                );
            }
            Err(err) => {
                // The error might contain the 404 status as well
                println!(
                    "Got error as expected when requesting nonexistent object version: {:?}",
                    err
                );
                // We could potentially check the error details to confirm it's
                // a 404
            }
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_try_next_download() {
        let config = match load_test_config() {
            Some(cfg) => cfg,
            None => {
                eprintln!("Test configuration not found. Skipping test.");
                return;
            }
        };

        let client = Client::new(
            &Config::default()
                .with_region(&config.region)
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    &config.access_key_id,
                    &config.access_key_secret,
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V4)
                .with_log_level(LogLevel::Debug),
        );

        // put object 10MB
        match put_with_size(&client, &config.bucket, 10 * 1024 * 1024).await {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }

        // test try_next download
        let mut result = client
            .get_object(GetObjectRequest {
                bucket: config.bucket.to_string(),
                key: TEST_OBJECT_NAME.to_string(),
                ..Default::default()
            })
            .await
            .unwrap();

        // 使用 try_next 方法逐步读取数据
        let mut full_content = Vec::new();
        let mut total_bytes = 0;

        while let Ok(Some(bytes)) = result.try_next().await {
            full_content.extend_from_slice(&bytes);
            total_bytes += bytes.len();
            println!(
                "Received chunk of {} bytes, total: {}",
                bytes.len(),
                total_bytes
            );
        }

        // let content = String::from_utf8(full_content).unwrap();
        // assert_eq!(content, TEST_OBJECT_CONTENT);
        println!(
            "Successfully downloaded '{}' bytes using try_next",
            total_bytes
        );

        // delete object
        match delete_multiple(&client, &config.bucket).await {
            Ok(output) => println!("{:?}", output),
            Err(err) => panic!("Invoke operation failed: {:?}", err),
        }
    }
}
