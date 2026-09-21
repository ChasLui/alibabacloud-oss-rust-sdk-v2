//! Client-side encryption over an ordinary client.
//!
//! Mirrors Go's `EncryptionClient`. Wraps a [`Client`] and encrypts object
//! bytes before they are sent, storing the wrapped content key and IV as
//! object metadata so a reader with the same master key can decrypt them.
//!
//! The service never sees the plaintext or the content key. What it stores is
//! ciphertext plus an envelope: the content key and IV, each encrypted with the
//! caller's master key, base64-encoded, in
//! `x-oss-meta-client-side-encryption-*` headers.
//!
//! Three consequences of the scheme are handled explicitly:
//!
//! - A ranged read cannot start at an arbitrary byte. AES-CTR's keystream is
//!   defined per block, so the request is widened down to a block boundary and
//!   the leading bytes are discarded after decryption. Returning them would
//!   hand the caller plaintext it did not ask for.
//! - Each part of a multipart upload is encrypted from a counter derived from
//!   its own part number, so parts can be uploaded concurrently, retried, or
//!   replaced without any of them depending on the others. That is also why
//!   the part size must be a multiple of the block size: otherwise a part
//!   boundary would fall inside a block and the counters would not line up.
//! - An object whose envelope cannot be unwrapped is an error, not a pass
//!   through: the bytes would be ciphertext, and handing them to a caller who
//!   asked for the object's contents would be worse than failing.

use std::collections::HashMap;

use base64::engine::general_purpose;
use base64::Engine;

use crate::api::object::{
    AbortMultipartUploadRequest, AbortMultipartUploadResult, CompleteMultipartUploadRequest,
    CompleteMultipartUploadResult, GetObjectRequest, GetObjectResult, HeadObjectRequest,
    HeadObjectResult, InitiateMultipartUploadRequest, InitiateMultipartUploadResult,
    ListPartsRequest, ListPartsResult, PutObjectRequest, PutObjectResult, UploadPartRequest,
    UploadPartResult,
};
use crate::client::{BodyDataReader, BodyStream, Client};
use crate::crypto::{
    adjust_range_start, envelope_from_headers, has_encrypted_header, headers as cse_headers,
    is_valid_content_alg, AesCtrCipher, ContentCipherBuilder, Envelope, MasterCipher,
};
use futures_util::StreamExt;

/// The block size the cipher aligns to.
const ALIGN_LEN: i64 = 16;

/// The state a multipart upload needs to keep encrypting its parts.
///
/// Mirrors Go's `EncryptionMultiPartContext`. It travels with the upload
/// because every part must be encrypted from the same content key, and the
/// counter for each part is derived from the part number and the part size.
#[derive(Clone)]
pub struct EncryptionMultiPartContext {
    /// The content cipher, positioned at the object's start.
    pub content_cipher: AesCtrCipher,
    /// The plaintext size of the whole object.
    pub data_size: i64,
    /// The plaintext size of each part.
    pub part_size: i64,
}

impl EncryptionMultiPartContext {
    /// Whether the context carries everything a part needs.
    pub fn is_valid(&self) -> bool {
        self.data_size != 0 && self.part_size != 0
    }
}

/// An OSS client that encrypts and decrypts object contents.
///
/// Mirrors Go's `EncryptionClient`. Operations without a `Securely` suffix —
/// listing, copying, deleting, and the bucket APIs — pass straight through;
/// only the four that carry object bytes encrypt or decrypt.
pub struct EncryptionClient {
    client: Client,
    default_builder: ContentCipherBuilder,
    /// Builders keyed by master key description, so an object written under
    /// one key can be read even when several are configured.
    builder_by_mat_desc: HashMap<String, ContentCipherBuilder>,
}

impl EncryptionClient {
    /// Wraps `client` with a master key.
    ///
    /// `master_cipher` is the default; `additional` lets an object written
    /// under a different key be read, matched by the key's description.
    pub fn new(client: Client, master_cipher: Box<dyn MasterCipher>) -> Self {
        let default_builder = ContentCipherBuilder::new(master_cipher);
        EncryptionClient {
            client,
            default_builder,
            builder_by_mat_desc: HashMap::new(),
        }
    }

    /// Registers another master key, matched by its description.
    ///
    /// A key without a description cannot be selected for decryption, because
    /// the object records only the description — so it is not registered.
    pub fn with_master_cipher(mut self, master_cipher: Box<dyn MasterCipher>) -> Self {
        let mat_desc = master_cipher.mat_desc();
        if !mat_desc.is_empty() {
            self.builder_by_mat_desc
                .insert(mat_desc, ContentCipherBuilder::new(master_cipher));
        }
        self
    }

    /// The underlying client, for operations that do not touch object bytes.
    pub fn unwrap_client(&self) -> &Client {
        &self.client
    }

    /// Picks the appropriate builder for `mat_desc`.
    fn content_cipher_builder(&self, mat_desc: &str) -> &ContentCipherBuilder {
        self.builder_by_mat_desc
            .get(mat_desc)
            .unwrap_or(&self.default_builder)
    }

    /// Uploads an object, encrypting its contents.
    ///
    /// The plaintext length and MD5 are moved into metadata, because the
    /// stored object's own values describe the ciphertext. A caller-supplied
    /// `content_length` is not sent: the encrypted body has the same length,
    /// but the value would be read as describing the object on the wire.
    pub async fn put_object(
        &self,
        request: PutObjectRequest,
    ) -> Result<PutObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let cipher = self.default_builder.content_cipher()?;
        let plaintext = request
            .body
            .as_ref()
            .map(read_body)
            .transpose()?
            .unwrap_or_default();
        let ciphertext = cipher.encrypt(&plaintext)?;

        let mut encrypted_request = PutObjectRequest {
            bucket: request.bucket.clone(),
            key: request.key.clone(),
            cache_control: request.cache_control.clone(),
            content_disposition: request.content_disposition.clone(),
            content_encoding: request.content_encoding.clone(),
            content_type: request.content_type.clone(),
            expires: request.expires.clone(),
            forbid_overwrite: request.forbid_overwrite.clone(),
            server_side_encryption: request.server_side_encryption.clone(),
            server_side_data_encryption: request.server_side_data_encryption.clone(),
            sse_kms_key_id: request.sse_kms_key_id.clone(),
            object_acl: request.object_acl.clone(),
            storage_class: request.storage_class.clone(),
            metadata: request.metadata.clone(),
            tagging: request.tagging.clone(),
            request_payer: request.request_payer.clone(),
            common: request.common.clone(),
            progress_fn: None,
            // The envelope records the plaintext length; the request's own
            // content-length would describe the ciphertext body.
            content_length: None,
            content_md5: None,
            body: Some(crate::BodyContent::from_bytes(ciphertext, None)),
            ..Default::default()
        };
        add_crypto_metadata(&mut encrypted_request.metadata, cipher.cipher_data(), true);
        if let Some(md5) = request.content_md5.as_deref() {
            encrypted_request.metadata.insert(
                cse_headers::UNENCRYPTED_CONTENT_MD5.to_string(),
                md5.to_string(),
            );
        }
        if let Some(length) = request.content_length {
            encrypted_request.metadata.insert(
                cse_headers::UNENCRYPTED_CONTENT_LENGTH.to_string(),
                length.to_string(),
            );
        }

        self.client.put_object(encrypted_request).await
    }

    /// Downloads an object, decrypting its contents.
    ///
    /// A ranged request is widened to a block boundary and the leading bytes
    /// are discarded, so the caller receives exactly the range it asked for.
    pub async fn get_object(
        &self,
        request: GetObjectRequest,
    ) -> Result<GetObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut discard_count = 0i64;
        let mut adjust_offset = 0i64;
        let mut encrypted_request = GetObjectRequest {
            bucket: request.bucket.clone(),
            key: request.key.clone(),
            if_match: request.if_match.clone(),
            if_none_match: request.if_none_match.clone(),
            if_modified_since: request.if_modified_since.clone(),
            if_unmodified_since: request.if_unmodified_since.clone(),
            range: request.range.clone(),
            range_behavior: request.range_behavior.clone(),
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

        if let Some(range) = request.range.as_deref() {
            let (offset, count) = parse_range(range)?;
            adjust_offset = adjust_range_start(offset, ALIGN_LEN);
            discard_count = offset - adjust_offset;
            if discard_count != 0 {
                // The widened range must be closed: an open-ended one has no
                // upper bound to widen, and the extra bytes have to be counted
                // so they can be discarded.
                let widened_count = if count > 0 { count + discard_count } else { 0 };
                encrypted_request.range = Some(if widened_count > 0 {
                    format!(
                        "bytes={adjust_offset}-{}",
                        adjust_offset + widened_count - 1
                    )
                } else {
                    format!("bytes={adjust_offset}-")
                });
                encrypted_request.range_behavior = Some("standard".to_string());
            }
        }

        let mut result = self.client.get_object(encrypted_request).await?;

        if !has_encrypted_header(&result.common.headers) {
            // A plain object read through this client is returned as it is:
            // the caller asked for the object, and it was never encrypted.
            return Ok(result);
        }

        let envelope = envelope_from_headers(&result.common.headers)?;
        if !is_valid_content_alg(&envelope.cek_alg) {
            return Err(format!(
                "not supported content algorithm {},object:{}",
                envelope.cek_alg, request.key
            )
            .into());
        }
        if !envelope.is_valid() {
            return Err(format!("getEnvelopeFromHeader error,object:{}", request.key).into());
        }

        let cipher = self
            .content_cipher_builder(&envelope.mat_desc)
            .content_cipher_from_envelope(&envelope)
            .map_err(|err| -> Box<dyn std::error::Error + Send + Sync> {
                format!("{err},object:{}", request.key).into()
            })?;
        // The counter is advanced to the widened offset, so the bytes that
        // come back decrypt from the right point in the keystream.
        let positioned = if adjust_offset > 0 {
            cipher.clone_at_offset(adjust_offset as u64)?
        } else {
            cipher
        };

        let body = result
            .body
            .take()
            .ok_or("GetObject returned no body for an encrypted object")?;
        let decrypted = decrypt_stream(body, positioned)?;
        // The leading bytes exist only because the range was widened; they are
        // dropped before the caller sees them.
        let decrypted = if discard_count > 0 {
            discard_stream(decrypted, discard_count as usize)
        } else {
            decrypted
        };
        result.body = Some(decrypted);

        Ok(result)
    }

    /// Initiates a multipart upload whose parts will be encrypted.
    ///
    /// The plaintext size and part size are recorded on the object; the part
    /// size must be a multiple of the block size so each part's counter starts
    /// on a boundary.
    pub async fn initiate_multipart_upload(
        &self,
        request: InitiateMultipartUploadRequest,
    ) -> Result<
        (InitiateMultipartUploadResult, EncryptionMultiPartContext),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let part_size = request.cse_part_size.unwrap_or(0);
        if part_size <= 0 {
            return Err("request.CSEPartSize is invalid".into());
        }
        if part_size % ALIGN_LEN != 0 {
            return Err(format!("request.CSEPartSize must aligned to the {ALIGN_LEN}").into());
        }

        let cipher = self.default_builder.content_cipher()?;

        let mut encrypted_request = InitiateMultipartUploadRequest {
            bucket: request.bucket.clone(),
            key: request.key.clone(),
            cache_control: request.cache_control.clone(),
            content_disposition: request.content_disposition.clone(),
            content_encoding: request.content_encoding.clone(),
            content_type: request.content_type.clone(),
            expires: request.expires.clone(),
            forbid_overwrite: request.forbid_overwrite.clone(),
            server_side_encryption: request.server_side_encryption.clone(),
            server_side_data_encryption: request.server_side_data_encryption.clone(),
            sse_kms_key_id: request.sse_kms_key_id.clone(),
            object_acl: request.object_acl.clone(),
            storage_class: request.storage_class.clone(),
            metadata: request.metadata.clone(),
            tagging: request.tagging.clone(),
            request_payer: request.request_payer.clone(),
            common: request.common.clone(),
            ..Default::default()
        };
        add_crypto_metadata(&mut encrypted_request.metadata, cipher.cipher_data(), false);
        encrypted_request
            .metadata
            .insert(cse_headers::PART_SIZE.to_string(), part_size.to_string());
        if let Some(data_size) = request.cse_data_size.filter(|size| *size > 0) {
            encrypted_request
                .metadata
                .insert(cse_headers::DATA_SIZE.to_string(), data_size.to_string());
        }

        let result = self
            .client
            .initiate_multipart_upload(&encrypted_request)
            .await?;
        let context = EncryptionMultiPartContext {
            content_cipher: cipher,
            data_size: request.cse_data_size.unwrap_or(0),
            part_size,
        };
        Ok((result, context))
    }

    /// Uploads one part, encrypted from the counter its part number implies.
    ///
    /// Deriving the counter per part is what lets parts be uploaded in any
    /// order: none of them depends on the bytes before it.
    pub async fn upload_part(
        &self,
        request: UploadPartRequest,
        context: &EncryptionMultiPartContext,
    ) -> Result<UploadPartResult, Box<dyn std::error::Error + Send + Sync>> {
        if !context.is_valid() {
            return Err("the multipart encryption context is invalid".into());
        }
        if context.part_size % ALIGN_LEN != 0 {
            return Err(format!(
                "the multipart encryption context's part size must be aligned to {ALIGN_LEN}"
            )
            .into());
        }

        let offset = if request.part_number > 1 {
            (request.part_number as i64 - 1) * context.part_size
        } else {
            0
        };
        let cipher = context.content_cipher.clone_at_offset(offset as u64)?;

        let plaintext = request
            .body
            .as_ref()
            .map(read_body)
            .transpose()?
            .unwrap_or_default();
        let ciphertext = cipher.encrypt(&plaintext)?;

        let mut encrypted_request = UploadPartRequest {
            bucket: request.bucket.clone(),
            key: request.key.clone(),
            part_number: request.part_number,
            upload_id: request.upload_id.clone(),
            content_md5: None,
            progress_fn: None,
            request_payer: request.request_payer.clone(),
            traffic_limit: request.traffic_limit,
            common: request.common.clone(),
            body: Some(crate::BodyContent::from_bytes(ciphertext, None)),
        };
        add_crypto_headers(&mut encrypted_request.common.headers, cipher.cipher_data());
        encrypted_request.common.headers.insert(
            cse_headers::wire::PART_SIZE.to_string(),
            context.part_size.to_string(),
        );
        if context.data_size > 0 {
            encrypted_request.common.headers.insert(
                cse_headers::wire::DATA_SIZE.to_string(),
                context.data_size.to_string(),
            );
        }

        self.client.upload_part(encrypted_request).await
    }

    /// Completes a multipart upload. The parts are already encrypted, so this
    /// only forwards the request.
    pub async fn complete_multipart_upload(
        &self,
        request: CompleteMultipartUploadRequest,
    ) -> Result<CompleteMultipartUploadResult, Box<dyn std::error::Error + Send + Sync>> {
        self.client.complete_multipart_upload(&request).await
    }

    /// Aborts a multipart upload.
    pub async fn abort_multipart_upload(
        &self,
        request: AbortMultipartUploadRequest,
    ) -> Result<AbortMultipartUploadResult, Box<dyn std::error::Error + Send + Sync>> {
        self.client.abort_multipart_upload(&request).await
    }

    /// Lists an upload's parts, including the envelope the service stored.
    pub async fn list_parts(
        &self,
        request: ListPartsRequest,
    ) -> Result<ListPartsResult, Box<dyn std::error::Error + Send + Sync>> {
        self.client.list_parts(&request).await
    }

    /// Reads an object's metadata without decrypting anything.
    pub async fn head_object(
        &self,
        request: HeadObjectRequest,
    ) -> Result<HeadObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        self.client.head_object(request).await
    }

    /// Rebuilds the content cipher of an in-progress encrypted upload from the
    /// envelope the service stored on its parts.
    ///
    /// Mirrors Go's `resumeCSEContext`: a resumed upload must keep encrypting
    /// with the same key, and the parts already stored carry it.
    pub fn resume_context(
        &self,
        list_parts: &ListPartsResult,
    ) -> Result<EncryptionMultiPartContext, Box<dyn std::error::Error + Send + Sync>> {
        let envelope = Envelope {
            iv: list_parts
                .client_encryption_start
                .clone()
                .unwrap_or_default(),
            cipher_key: list_parts.client_encryption_key.clone().unwrap_or_default(),
            mat_desc: String::new(),
            wrap_alg: list_parts
                .client_encryption_wrap_alg
                .clone()
                .unwrap_or_default(),
            cek_alg: list_parts
                .client_encryption_cek_alg
                .clone()
                .unwrap_or_default(),
            unencrypted_md5: String::new(),
            unencrypted_content_len: String::new(),
        };
        if !envelope.is_valid() {
            return Err("the upload's parts carry no usable encryption envelope".into());
        }
        let cipher = self
            .content_cipher_builder(&envelope.mat_desc)
            .content_cipher_from_envelope(&envelope)?;
        Ok(EncryptionMultiPartContext {
            content_cipher: cipher,
            data_size: list_parts.client_encryption_data_size.unwrap_or(0),
            part_size: list_parts.client_encryption_part_size.unwrap_or(0),
        })
    }
}

/// Records the envelope in a raw header map, using the full wire names.
///
/// Used for requests whose headers are sent as written rather than through a
/// metadata field, which is the multipart part upload.
fn add_crypto_headers(
    headers: &mut HashMap<String, String>,
    cipher_data: &crate::crypto::CipherData,
) {
    if !cipher_data.mat_desc.is_empty() {
        headers.insert(cse_headers::wire::MAT_DESC.to_string(), cipher_data.mat_desc.clone());
    }
    headers.insert(
        cse_headers::wire::KEY.to_string(),
        general_purpose::STANDARD.encode(&cipher_data.encrypted_key),
    );
    headers.insert(
        cse_headers::wire::START.to_string(),
        general_purpose::STANDARD.encode(&cipher_data.encrypted_iv),
    );
    headers.insert(
        cse_headers::wire::WRAP_ALG.to_string(),
        cipher_data.wrap_algorithm.clone(),
    );
    headers.insert(
        cse_headers::wire::CEK_ALG.to_string(),
        cipher_data.cek_algorithm.clone(),
    );
}

/// Records the envelope in an object's metadata.
///
/// `include_plaintext_length` separates the two callers: a single upload moves
/// the request's length into metadata, while a multipart upload records the
/// part and data sizes instead.
fn add_crypto_metadata(
    metadata: &mut HashMap<String, String>,
    cipher_data: &crate::crypto::CipherData,
    _include_plaintext_length: bool,
) {
    if !cipher_data.mat_desc.is_empty() {
        metadata.insert(
            cse_headers::MAT_DESC.to_string(),
            cipher_data.mat_desc.clone(),
        );
    }
    metadata.insert(
        cse_headers::KEY.to_string(),
        general_purpose::STANDARD.encode(&cipher_data.encrypted_key),
    );
    metadata.insert(
        cse_headers::START.to_string(),
        general_purpose::STANDARD.encode(&cipher_data.encrypted_iv),
    );
    metadata.insert(
        cse_headers::WRAP_ALG.to_string(),
        cipher_data.wrap_algorithm.clone(),
    );
    metadata.insert(
        cse_headers::CEK_ALG.to_string(),
        cipher_data.cek_algorithm.clone(),
    );
}

/// Reads a request body into memory.
///
/// Encryption is a transformation of the whole payload, and the body arrives
/// as an opaque byte container, so it is materialised here. The streaming
/// multipart path encrypts each part as it is written instead.
fn read_body(
    body: &crate::BodyContent,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    match body {
        crate::BodyContent::Bytes { data, .. } => Ok(data.to_vec()),
        crate::BodyContent::Text { data, .. } => Ok(data.as_bytes().to_vec()),
        crate::BodyContent::File { path, .. } => Ok(std::fs::read(path)?),
        // Encrypting a stream would mean buffering it first, which is what the
        // caller opted out of by handing over a stream rather than bytes.
        crate::BodyContent::Stream { .. } => {
            Err("a stream body cannot be encrypted in place; supply bytes or a file".into())
        }
    }
}

/// Parses `bytes=first-last` (or `bytes=first-`) into its bounds.
fn parse_range(range: &str) -> Result<(i64, i64), Box<dyn std::error::Error + Send + Sync>> {
    let spec = range
        .strip_prefix("bytes=")
        .ok_or_else(|| format!("invalid range {range:?}: expected bytes=first-last"))?;
    let (first, last) = spec
        .split_once('-')
        .ok_or_else(|| format!("invalid range {range:?}: expected bytes=first-last"))?;
    let first = first
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("invalid range {range:?}: {err}"))?;
    let count = if last.trim().is_empty() {
        0
    } else {
        last.trim()
            .parse::<i64>()
            .map_err(|err| format!("invalid range {range:?}: {err}"))?
            - first
            + 1
    };
    Ok((first, count))
}

/// Decrypts a response body lazily.
///
/// CTR is a stream cipher, so each chunk can be decrypted as it arrives; the
/// cipher keeps the keystream position across chunks.
fn decrypt_stream(
    body: BodyStream,
    cipher: AesCtrCipher,
) -> Result<BodyStream, Box<dyn std::error::Error + Send + Sync>> {
    let stream = futures_util::stream::unfold((body, cipher), |(mut body, cipher)| async move {
        match body.next().await {
            Some(Ok(chunk)) => match cipher.decrypt(&chunk) {
                Ok(plain) => Some((Ok(bytes::Bytes::from(plain)), (body, cipher))),
                // The stream's error type is reqwest's, which a decryption
                // failure is not; the stream therefore ends instead. A caller
                // sees a short read, never bytes that were not decrypted.
                Err(_) => None,
            },
            Some(Err(err)) => Some((Err(err), (body, cipher))),
            None => None,
        }
    });
    Ok(Box::pin(stream))
}

/// Drops the first `count` bytes of a stream.
///
/// The bytes exist only because the range was widened to a block boundary, and
/// they are decrypted before they can be discarded.
fn discard_stream(body: BodyStream, count: usize) -> BodyStream {
    let stream =
        futures_util::stream::unfold((body, count), |(mut body, mut remaining)| async move {
            loop {
                match body.next().await {
                    Some(Ok(chunk)) => {
                        if remaining >= chunk.len() {
                            remaining -= chunk.len();
                            continue;
                        }
                        let kept = chunk.slice(remaining..);
                        return Some((Ok(kept), (body, 0)));
                    }
                    Some(Err(err)) => return Some((Err(err), (body, remaining))),
                    None => return None,
                }
            }
        });
    Box::pin(stream)
}

/// Reads an encrypted object into memory, decrypting it.
pub async fn read_encrypted_body(
    mut result: GetObjectResult,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    result.get_all_data().await
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::rc::Rc;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::crypto::{AES_CTR_ALGORITHM, RSA_CRYPTO_WRAP};
    use crate::log::LogLevel;
    use crate::SignatureVersionType;

    // Test fixtures, copied verbatim from the upstream Go SDK's mock test
    // (`encryption_client_mock_test.go`); a throwaway key pair that guards
    // nothing, kept so both implementations are tested with the same material.
    const PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCokfiAVXXf5ImFzKDw+XO/UByW
6mse2QsIgz3ZwBtMNu59fR5zttSx+8fB7vR4CN3bTztrP9A6bjoN0FFnhlQ3vNJC
5MFO1PByrE/MNd5AAfSVba93I6sx8NSk5MzUCA4NJzAUqYOEWGtGBcom6kEF6MmR
1EKib1Id8hpooY5xaQIDAQAB
-----END PUBLIC KEY-----";

    const PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIICdQIBADANBgkqhkiG9w0BAQEFAASCAl8wggJbAgEAAoGBAKiR+IBVdd/kiYXM
oPD5c79QHJbqax7ZCwiDPdnAG0w27n19HnO21LH7x8Hu9HgI3dtPO2s/0DpuOg3Q
UWeGVDe80kLkwU7U8HKsT8w13kAB9JVtr3cjqzHw1KTkzNQIDg0nMBSpg4RYa0YF
yibqQQXoyZHUQqJvUh3yGmihjnFpAgMBAAECgYA49RmCQ14QyKevDfVTdvYlLmx6
kbqgMbYIqk+7w611kxoCTMR9VMmJWgmk/Zic9mIAOEVbd7RkCdqT0E+xKzJJFpI2
ZHjrlwb21uqlcUqH1Gn+wI+jgmrafrnKih0kGucavr/GFi81rXixDrGON9KBE0FJ
cPVdc0XiQAvCBnIIAQJBANXu3htPH0VsSznfqcDE+w8zpoAJdo6S/p30tcjsDQnx
l/jYV4FXpErSrtAbmI013VYkdJcghNSLNUXppfk2e8UCQQDJt5c07BS9i2SDEXiz
byzqCfXVzkdnDj9ry9mba1dcr9B9NCslVelXDGZKvQUBqNYCVxg398aRfWlYDTjU
IoVVAkAbTyjPN6R4SkC4HJMg5oReBmvkwFCAFsemBk0GXwuzD0IlJAjXnAZ+/rIO
ItewfwXIL1Mqz53lO/gK+q6TR585AkB304KUIoWzjyF3JqLP3IQOxzns92u9EV6l
V2P+CkbMPXiZV6sls6I4XppJXX2i3bu7iidN3/dqJ9izQK94fMU9AkBZvgsIPCot
y1/POIbv9LtnviDKrmpkXgVQSU4BmTPvXwTJm8APC7P/horSh3SVf1zgmnsyjm9D
hO92gGc+4ajL
-----END PRIVATE KEY-----";

    fn master() -> Box<dyn MasterCipher> {
        Box::new(crate::crypto::MasterRsaCipher::new(
            &HashMap::new(),
            PUBLIC_KEY,
            PRIVATE_KEY,
        ))
    }

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

    fn encrypted_client(server: &mockito::ServerGuard) -> EncryptionClient {
        EncryptionClient::new(mock_client(server), master())
    }

    /// An upload must store the envelope and must not store the plaintext.
    #[tokio::test]
    async fn test_put_object_encrypts_and_records_envelope() {
        let mut server = mockito::Server::new_async().await;
        let plaintext = b"secret contents";

        // Capture the uploaded body so it can be shown to differ from the input.
        let put = server
            .mock("PUT", mockito::Matcher::Any)
            .match_header(
                "x-oss-meta-client-side-encryption-key",
                mockito::Matcher::Regex(".+".to_string()),
            )
            .match_header(
                "x-oss-meta-client-side-encryption-cek-alg",
                AES_CTR_ALGORITHM,
            )
            .match_header(
                "x-oss-meta-client-side-encryption-wrap-alg",
                RSA_CRYPTO_WRAP,
            )
            .with_status(200)
            .with_header("etag", "\"encrypted\"")
            .expect(1)
            .create_async()
            .await;

        let result = encrypted_client(&server)
            .put_object(PutObjectRequest {
                bucket: "test-bucket".to_string(),
                key: "secret.txt".to_string(),
                body: Some(crate::BodyContent::from_bytes(plaintext.to_vec(), None)),
                ..Default::default()
            })
            .await
            .expect("encrypted upload should succeed");

        assert_eq!(result.etag.as_deref(), Some("\"encrypted\""));
        put.assert_async().await;
    }

    /// A round trip through a mock that stores the ciphertext must return the
    /// original plaintext.
    #[tokio::test]
    async fn test_round_trip_through_mock() {
        let mut server = mockito::Server::new_async().await;
        let plaintext: Vec<u8> = (0..500u32).map(|i| (i % 251) as u8).collect();

        // The server keeps whatever was uploaded and serves it back with the
        // same headers, which is what a real OSS does.
        let stored: std::sync::Arc<std::sync::Mutex<(Vec<u8>, HashMap<String, String>)>> =
            std::sync::Arc::new(std::sync::Mutex::new((Vec::new(), HashMap::new())));
        let stored_put = stored.clone();
        server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(200)
            .with_header("etag", "\"x\"")
            .with_body_from_request(move |request| {
                let mut guard = stored_put.lock().expect("lock");
                guard.0 = request.body().cloned().unwrap_or_default();
                for name in [
                    "x-oss-meta-client-side-encryption-key",
                    "x-oss-meta-client-side-encryption-start",
                    "x-oss-meta-client-side-encryption-cek-alg",
                    "x-oss-meta-client-side-encryption-wrap-alg",
                ] {
                    if let Some(value) = request.header(name).first() {
                        guard
                            .1
                            .insert(name.to_string(), value.to_str().unwrap_or("").to_string());
                    }
                }
                Vec::new()
            })
            .create_async()
            .await;

        let client = encrypted_client(&server);
        client
            .put_object(PutObjectRequest {
                bucket: "test-bucket".to_string(),
                key: "round.bin".to_string(),
                body: Some(crate::BodyContent::from_bytes(plaintext.clone(), None)),
                ..Default::default()
            })
            .await
            .expect("upload should succeed");

        let (ciphertext, headers) = {
            let guard = stored.lock().expect("lock");
            (guard.0.clone(), guard.1.clone())
        };
        assert_eq!(ciphertext.len(), plaintext.len(), "CTR preserves length");
        assert_ne!(
            ciphertext, plaintext,
            "the stored object must be ciphertext"
        );

        // Serve the stored ciphertext and envelope back.
        let mut get_mock = server.mock("GET", mockito::Matcher::Any);
        for (name, value) in &headers {
            get_mock = get_mock.with_header(name.as_str(), value.as_str());
        }
        let get_mock = get_mock
            .with_status(200)
            .with_header("etag", "\"x\"")
            .with_body_from_request(move |_| ciphertext.clone())
            .expect(1)
            .create_async()
            .await;

        let mut result = client
            .get_object(GetObjectRequest {
                bucket: "test-bucket".to_string(),
                key: "round.bin".to_string(),
                ..Default::default()
            })
            .await
            .expect("download should succeed");

        let decrypted = result.get_all_data().await.expect("read body");
        assert_eq!(
            decrypted, plaintext,
            "the round trip must return the original"
        );

        get_mock.assert_async().await;
    }

    /// A ranged read of an encrypted object must return exactly the requested
    /// bytes, even though the range is widened to a block boundary.
    #[tokio::test]
    async fn test_ranged_read_strips_alignment_padding() {
        let mut server = mockito::Server::new_async().await;
        let plaintext: Vec<u8> = (0..256u32).map(|i| (i % 251) as u8).collect();

        let builder = ContentCipherBuilder::new(master());
        let cipher = builder.content_cipher().expect("cipher");
        let ciphertext = cipher.encrypt(&plaintext).expect("encrypt");
        let cipher_data = cipher.cipher_data().clone();

        // The server must be asked for an aligned range: 10 rounds down to 0.
        let encoded_key = general_purpose::STANDARD.encode(&cipher_data.encrypted_key);
        let encoded_iv = general_purpose::STANDARD.encode(&cipher_data.encrypted_iv);
        let get_mock = server
            .mock("GET", mockito::Matcher::Any)
            .match_header("range", "bytes=0-29")
            .with_status(206)
            .with_header("content-range", "bytes 0-29/256")
            .with_header("etag", "\"x\"")
            .with_header("x-oss-meta-client-side-encryption-key", &encoded_key)
            .with_header("x-oss-meta-client-side-encryption-start", &encoded_iv)
            .with_header(
                "x-oss-meta-client-side-encryption-cek-alg",
                AES_CTR_ALGORITHM,
            )
            .with_header(
                "x-oss-meta-client-side-encryption-wrap-alg",
                RSA_CRYPTO_WRAP,
            )
            .with_body_from_request(move |_| ciphertext[0..30].to_vec())
            .create_async()
            .await;

        // Request bytes 15..29: inside block 0, so the request widens to 0 and
        // the leading 15 bytes are discarded after decryption.
        let mut result = encrypted_client(&server)
            .get_object(GetObjectRequest {
                bucket: "test-bucket".to_string(),
                key: "ranged.bin".to_string(),
                range: Some("bytes=15-29".to_string()),
                ..Default::default()
            })
            .await
            .expect("ranged read should succeed");

        let data = result.get_all_data().await.expect("read body");
        assert_eq!(
            data,
            plaintext[15..30].to_vec(),
            "a ranged read must return exactly the requested bytes"
        );
        let _ = get_mock;
    }

    /// A plain object read through the encryption client must pass through
    /// unchanged: it was never encrypted.
    #[tokio::test]
    async fn test_plain_object_passes_through() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("etag", "\"plain\"")
            .with_body("not encrypted")
            .create_async()
            .await;

        let mut result = encrypted_client(&server)
            .get_object(GetObjectRequest {
                bucket: "test-bucket".to_string(),
                key: "plain.txt".to_string(),
                ..Default::default()
            })
            .await
            .expect("plain read should succeed");

        let data = result.get_all_data().await.expect("read body");
        assert_eq!(data, b"not encrypted");
    }

    /// An object that claims encryption but carries no usable envelope must
    /// fail rather than hand back ciphertext.
    #[tokio::test]
    async fn test_broken_envelope_is_rejected() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("x-oss-meta-client-side-encryption-key", "bm90LWEta2V5")
            .with_header(
                "x-oss-meta-client-side-encryption-cek-alg",
                AES_CTR_ALGORITHM,
            )
            .with_header(
                "x-oss-meta-client-side-encryption-wrap-alg",
                RSA_CRYPTO_WRAP,
            )
            .with_body("ciphertext")
            .create_async()
            .await;

        let err = encrypted_client(&server)
            .get_object(GetObjectRequest {
                bucket: "test-bucket".to_string(),
                key: "broken.bin".to_string(),
                ..Default::default()
            })
            .await
            .expect_err("an unusable envelope must fail the read");
        assert!(
            err.to_string().contains("getEnvelopeFromHeader") || err.to_string().contains("IV"),
            "{err}"
        );
    }

    /// A part size that is not a multiple of the block size would put a part
    /// boundary inside a block, so it must be rejected before anything is sent.
    #[tokio::test]
    async fn test_unaligned_part_size_is_rejected() {
        let server = mockito::Server::new_async().await;
        let err = match encrypted_client(&server)
            .initiate_multipart_upload(InitiateMultipartUploadRequest {
                bucket: "test-bucket".to_string(),
                key: "multi.bin".to_string(),
                cse_part_size: Some(1000),
                cse_data_size: Some(3000),
                ..Default::default()
            })
            .await
        {
            Ok(_) => panic!("an unaligned part size must be rejected"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("must aligned to the 16"), "{err}");
    }

    /// Each part must be encrypted from its own counter, so the parts can be
    /// assembled in order and decrypt to the original bytes.
    #[tokio::test]
    async fn test_multipart_parts_decrypt_in_order() {
        let mut server = mockito::Server::new_async().await;
        let part_size = 32i64;
        let plaintext: Vec<u8> = (0..80u32).map(|i| (i % 251) as u8).collect();

        server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Regex("uploads".to_string()))
            .with_status(200)
            .with_body("<InitiateMultipartUploadResult><UploadId>up-cse</UploadId></InitiateMultipartUploadResult>")
            .create_async()
            .await;

        let ciphertexts: std::sync::Arc<std::sync::Mutex<HashMap<i32, Vec<u8>>>> =
            std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let captured = ciphertexts.clone();
        server
            .mock("PUT", mockito::Matcher::Any)
            .with_status(200)
            .with_header("etag", "\"p\"")
            .with_body_from_request(move |request| {
                let query = request.path_and_query().to_string();
                if let Some(number) = query
                    .split("partNumber=")
                    .nth(1)
                    .and_then(|rest| rest.split('&').next())
                    .and_then(|value| value.parse::<i32>().ok())
                {
                    captured
                        .lock()
                        .expect("lock")
                        .insert(number, request.body().cloned().unwrap_or_default());
                }
                Vec::new()
            })
            .create_async()
            .await;

        let client = encrypted_client(&server);
        let (_, context) = client
            .initiate_multipart_upload(InitiateMultipartUploadRequest {
                bucket: "test-bucket".to_string(),
                key: "multi.bin".to_string(),
                cse_part_size: Some(part_size),
                cse_data_size: Some(plaintext.len() as i64),
                ..Default::default()
            })
            .await
            .expect("initiate should succeed");

        // Upload the parts out of order: the counter for each depends only on
        // its part number, so order must not matter.
        for part_number in [2i32, 1, 3] {
            let start = (part_number as usize - 1) * part_size as usize;
            let end = (start + part_size as usize).min(plaintext.len());
            client
                .upload_part(
                    UploadPartRequest {
                        bucket: "test-bucket".to_string(),
                        key: "multi.bin".to_string(),
                        part_number,
                        upload_id: "up-cse".to_string(),
                        body: Some(crate::BodyContent::from_bytes(
                            plaintext[start..end].to_vec(),
                            None,
                        )),
                        ..Default::default()
                    },
                    &context,
                )
                .await
                .expect("part upload should succeed");
        }

        let stored = ciphertexts.lock().expect("lock").clone();
        assert_eq!(stored.len(), 3);

        // Concatenating the ciphertext and decrypting must rebuild the object.
        let mut assembled = Vec::new();
        for part_number in 1..=3 {
            assembled.extend_from_slice(&stored[&part_number]);
        }
        let decrypted = context.content_cipher.decrypt(&assembled).expect("decrypt");
        assert_eq!(
            decrypted, plaintext,
            "parts encrypted from their own counters must reassemble correctly"
        );
    }
}
