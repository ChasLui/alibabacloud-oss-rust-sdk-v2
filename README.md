# Alibaba Cloud OSS SDK for Rust V2

Rust SDK for [Alibaba Cloud Object Storage Service (OSS)](https://www.alibabacloud.com/product/oss) V2.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Configuration](#configuration)
- [Examples](#examples)
- [API Reference](#api-reference)
  - [Concurrent Transfers](#concurrent-transfers)
  - [Resumable Transfers](#resumable-transfers)
  - [Server-Side Copy](#server-side-copy)
  - [File-Like Handles](#file-like-handles)
  - [Client-Side Encryption](#client-side-encryption)
  - [Bandwidth Limits](#bandwidth-limits)
  - [Presigning](#presigning)
  - [Paginators](#paginators)

- [Architecture](#architecture)
- [Testing](#testing)
- [Development](#development)
- [Contributing](#contributing)
- [Troubleshooting](#troubleshooting)
- [Additional Resources](#additional-resources)

## Overview

This SDK provides a comprehensive set of APIs for interacting with Alibaba Cloud Object Storage Service (OSS) in Rust. The APIs are organized into five main categories:

- **Service APIs**: Operations related to service-level management (list buckets, public access block, meta query pipeline actions)
- **Bucket APIs**: Full bucket configuration coverage — ACL, CORS, lifecycle, logging, encryption, replication, WORM, inventory, policies, and more (100+ operations)
- **Object APIs**: Object data, metadata, tagging, retention, symlinks, append uploads, and SelectObject (frame-parsed streaming responses)
- **Region APIs**: `describe_regions` for listing OSS endpoints by region
- **Access Point APIs**: Access point lifecycle and policy management

### Key Features

- **Type-Safe API Design**: Strongly-typed request and response structures
- **Automatic Serialization**: Header/query parameter handling via macros
- **Full API Coverage**: 154 operations aligned with the Go SDK v2 — objects, buckets, service, regions, and access points (the Go SDK's operation set exactly)
- **Concurrent Transfers**: `Client::download_file` and `Client::upload_file` — ranged parallel download with per-part checksum folding, and multipart upload that aborts on failure so no billable parts are orphaned
- **Server-Side Copy**: `Client::copy_object_to_object` — one `CopyObject` for small sources, `UploadPartCopy` for large ones, with the destination's checksum compared against the source's
- **File-Like Handles**: `open_read_only_file` / `open_append_only_file` / `open_write_only_file` — seekable reads, appends that carry the running checksum, and a streaming writer that uploads parts as they fill
- **Resumable Transfers**: `download_file_with_checkpoint` / `upload_file_with_checkpoint` — progress survives a failure; an upload resumes from the parts the service actually holds
- **Client-Side Encryption**: `EncryptionClient` — AES-CTR envelope encryption with an RSA master key, including ranged reads and multipart uploads
- **Bandwidth Limits**: `with_upload_bandwidth_limit` / `with_download_bandwidth_limit` — token-bucket pacing on the body streams
- **Pre-signed URLs**: `Client::presign` with V4/V1 signing for upload, download, and multipart workflows
- **Paginators**: Ergonomic page-by-page iteration for all six list operations, with URL-decoded keys
- **Comprehensive Testing**: Extensive integration tests with automatic resource cleanup
- **Flexible Configuration**: Support for various authentication methods
- **Robust Error Handling**: Custom error types and retry mechanisms
- **Logging Support**: Configurable logging levels and outputs

## Quick Start

### 1. Setup Environment Variables

```bash
export ACCESS_KEY_ID="your-access-key-id"
export ACCESS_KEY_SECRET="your-access-key-secret"
export OSS_REGION="cn-hangzhou"
export OSS_BUCKET="your-bucket-name"
```

### 2. Basic Example

```rust
use alibabacloud_oss_sdk_rust_v2::{
    api::object::{GetObjectRequest, PutObjectRequest},
    client::Client,
    config::Config,
    credential::providers::StaticCredentialsProvider,
    BodyContent,
};
use std::rc::Rc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the client
    let config = Config::default()
        .with_region(&std::env::var("OSS_REGION")?)
        .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
            &std::env::var("ACCESS_KEY_ID")?,
            &std::env::var("ACCESS_KEY_SECRET")?,
            &[],
        )))
        .with_signature_version(alibabacloud_oss_sdk_rust_v2::SignatureVersionType::V4);

    let client = Client::new(&config);

    // Upload an object
    let put_request = PutObjectRequest {
        bucket: std::env::var("OSS_BUCKET")?,
        key: "hello.txt".to_string(),
        body: Some(BodyContent::from_text("Hello OSS!".to_string(), None)),
        ..Default::default()
    };

    match client.put_object(put_request).await {
        Ok(result) => println!("Upload successful!"),
        Err(e) => eprintln!("Upload failed: {}", e),
    }

    // Download the object
    let get_request = GetObjectRequest {
        bucket: std::env::var("OSS_BUCKET")?,
        key: "hello.txt".to_string(),
        ..Default::default()
    };

    match client.get_object(get_request).await {
        Ok(mut result) => {
            use alibabacloud_oss_sdk_rust_v2::client::BodyDataReader;
            let data = result.get_all_data().await?;
            println!("Content: {}", String::from_utf8_lossy(&data));
        }
        Err(e) => eprintln!("Download failed: {}", e),
    }

    Ok(())
}
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
alibabacloud-oss-sdk-rust-v2 = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

Then run:

```bash
cargo build
```

## Configuration

### Authentication Methods

#### Static Credentials

```rust
let config = Config::default()
    .with_region("cn-hangzhou")
    .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
        "access_key_id",
        "access_key_secret",
        &[],
    )))
    .with_signature_version(SignatureVersionType::V4);
```

#### Environment Variables

```rust
let access_key_id = std::env::var("ACCESS_KEY_ID")?;
let access_key_secret = std::env::var("ACCESS_KEY_SECRET")?;
```

#### ECS Role (for applications running on ECS)

```rust
use alibabacloud_oss_sdk_rust_v2::credential::providers::EcsRoleCredentialsProvider;

let config = Config::default()
    .with_region("cn-hangzhou")
    .with_credentials_provider(Rc::new(EcsRoleCredentialsProvider::new()));
```

### Advanced Configuration

```rust
use std::rc::Rc;
use alibabacloud_oss_sdk_rust_v2::retry::Standard;

let config = Config::default()
    .with_region("cn-hangzhou")
    .with_credentials_provider(credentials_provider)
    .with_signature_version(SignatureVersionType::V4)
    // Requests are retried by the `Standard` retryer (3 attempts) unless a
    // retryer is supplied explicitly.
    .with_retryer(Rc::new(Standard::new().with_max_attempts(5)))
    .with_log_level(LogLevel::Debug);
```

## Examples

We provide comprehensive examples in the `oss/examples` directory:

### Available Examples

1. **Quickstart** (`01_quickstart.rs`)
   - Basic client setup
   - Upload and download objects
   - Get bucket information
   - Run: `cargo run --example 01_quickstart`

2. **File Upload** (`02_file_upload.rs`)
   - Upload text content
   - Upload binary data
   - Set custom metadata
   - Multipart upload for large files
   - Run: `cargo run --example 02_file_upload`

3. **Bucket Management** (`03_bucket_management.rs`)
   - Get bucket information
   - Manage bucket ACL
   - List objects in bucket
   - Create/delete buckets
   - Run: `cargo run --example 03_bucket_management`

4. **Error Handling** (`04_error_handling.rs`)
   - Handle common errors (404, permission denied, etc.)
   - Custom error types
   - Retry strategies
   - Graceful error recovery
   - Run: `cargo run --example 04_error_handling`

### Running Examples

```bash
# Navigate to project root
cd /path/to/aliyun-oss-sdk-rust-v2

# Set environment variables
export ACCESS_KEY_ID="your-access-key-id"
export ACCESS_KEY_SECRET="your-access-key-secret"
export OSS_REGION="cn-hangzhou"
export OSS_BUCKET="your-bucket-name"

# Run specific example
cargo run --example 01_quickstart
cargo run --example 02_file_upload
cargo run --example 03_bucket_management
cargo run --example 04_error_handling
```

## API Reference

All operations take a request struct and return a result struct. Most configuration-style operations borrow the request (`&Request`), while data-plane operations take it by value — check the signature in rustdoc. Every request type implements `Default`, so `..Default::default()` works everywhere.

### Bucket APIs (103 operations)

#### Basic

- `create_bucket` / `delete_bucket` - Create and delete buckets
- `get_bucket_info` / `get_bucket_stat` / `get_bucket_location` - Bucket metadata
- `get_bucket_acl` / `put_bucket_acl` - Bucket ACL
- `list_objects` / `list_objects_v2` / `list_object_versions` - Object listing

#### Bucket Configuration

- CORS: `get_bucket_cors` / `put_bucket_cors` / `delete_bucket_cors`
- Lifecycle: `get_bucket_lifecycle` / `put_bucket_lifecycle` / `delete_bucket_lifecycle`
- Logging: `get_bucket_logging` / `put_bucket_logging` / `delete_bucket_logging`
- Encryption: `get_bucket_encryption` / `put_bucket_encryption` / `delete_bucket_encryption`
- Policy: `get_bucket_policy` / `put_bucket_policy` / `delete_bucket_policy` / `get_bucket_policy_status`
- Tags: `get_bucket_tags` / `put_bucket_tags` / `delete_bucket_tags`
- Website: `get_bucket_website` / `put_bucket_website` / `delete_bucket_website`
- Referer: `get_bucket_referer` / `put_bucket_referer`
- Replication: `get_bucket_replication` / `put_bucket_replication` / `delete_bucket_replication` / `get_bucket_replication_location` / `get_bucket_replication_progress`
- Versioning: `get_bucket_versioning` / `put_bucket_versioning`
- Request payment: `get_bucket_request_payment` / `put_bucket_request_payment`
- Transfer acceleration: `get_bucket_transfer_acceleration` / `put_bucket_transfer_acceleration`
- Resource group: `get_bucket_resource_group` / `put_bucket_resource_group`
- Access monitor: `get_bucket_access_monitor` / `put_bucket_access_monitor`
- HTTPS config: `get_bucket_https_config` / `put_bucket_https_config`
- Archive direct read: `get_bucket_archive_direct_read` / `put_bucket_archive_direct_read`
- Overwrite config: `get_bucket_overwrite_config` / `put_bucket_overwrite_config` / `delete_bucket_overwrite_config`
- Public access block: `get_bucket_public_access_block` / `put_bucket_public_access_block` / `delete_bucket_public_access_block`
- RTC: `put_bucket_rtc`

#### WORM (retention)

- `initiate_bucket_worm` / `abort_bucket_worm` / `complete_bucket_worm` / `extend_bucket_worm` / `get_bucket_worm`
- `get_bucket_object_worm_configuration` / `put_bucket_object_worm_configuration`

#### Inventory & redundancy

- `list_bucket_inventory` / `get_bucket_inventory` / `put_bucket_inventory` / `delete_bucket_inventory`
- `create_bucket_data_redundancy_transition` / `get_bucket_data_redundancy_transition` / `delete_bucket_data_redundancy_transition` / `list_bucket_data_redundancy_transition` / `list_user_data_redundancy_transition`

#### Meta query

- `open_meta_query` / `get_meta_query_status` / `do_meta_query` / `close_meta_query`
- `option_object` - Object-level meta query options

#### Custom domains (CNAME) & styles

- `create_cname_token` / `get_cname_token` / `put_cname` / `list_cname` / `delete_cname`
- `put_style` / `get_style` / `list_style` / `delete_style`

#### User-defined log fields

- `get_user_defined_log_fields_config` / `put_user_defined_log_fields_config` / `delete_user_defined_log_fields_config`

#### Object process access points

- `create_access_point_for_object_process` / `get_access_point_for_object_process` / `delete_access_point_for_object_process` / `list_access_point_for_object_process`
- `get_access_point_config_for_object_process` / `put_access_point_config_for_object_process`
- `get_access_point_policy_for_object_process` / `put_access_point_policy_for_object_process` / `delete_access_point_policy_for_object_process`

#### SelectObject write-back

- `write_get_object_response` - Upload the result of a SelectObject request (sent to the `oss-cn-*.oss-object-process.aliyuncs.com` host)

### Object APIs (33 operations)

#### Basic Operations

- `get_object` / `put_object` / `delete_object` / `copy_object` / `head_object` - Core object operations
- `append_object` / `seal_append_object` - Appendable object upload and sealing
- `delete_multiple_objects` - Batch delete

#### Access Control & Metadata

- `get_object_acl` / `put_object_acl` - Object ACL
- `get_object_meta` - Object metadata
- `get_object_tagging` / `put_object_tagging` / `delete_object_tagging` - Object tagging
- `get_object_retention` / `put_object_retention` - Object retention
- `get_object_legal_hold` / `put_object_legal_hold` - Object legal hold

#### Symlinks

- `put_symlink` / `get_symlink` - Create and resolve symbolic links

#### Multipart Upload

- `initiate_multipart_upload` / `upload_part` / `complete_multipart_upload` / `abort_multipart_upload`
- `upload_part_copy` - Copy a part from another object
- `list_multipart_uploads` / `list_parts`

#### Restore & tiering

- `restore_object` / `clean_restored_object` - Archive restore and cleanup

#### Data processing

- `async_process_object` / `process_object` - Asynchronous and synchronous object processing
- `select_object` - SelectObject with full frame parsing: `SelectObjectResult::body` is a `SelectObjectBodyReader` (an `AsyncRead` body reader) that decodes the framing protocol — Data (8388609), Continuous (8388612), End (8388613), MetaEndCSV (8388614), MetaEndJSON (8388615) — including per-frame CRC validation; an `End`/`MetaEnd` frame carrying an error surfaces as a read error
- `create_select_object_meta` - SelectObject metadata indexing

### Service APIs (7 operations)

- `list_buckets` - List all buckets owned by the account
- `get_public_access_block` / `put_public_access_block` / `delete_public_access_block` - Account-level public access block
- `do_meta_query_action` - Meta query pipeline actions
- `do_data_pipe_line_action` - Data pipeline actions
- `list_cloud_boxes` - List CloudBoxes

### Region APIs (1 operation)

- `describe_regions` - List regions and their endpoints (`pub regions: Option<String>` field per the Go SDK)

### Access Point APIs (10 operations)

- `create_access_point` / `get_access_point` / `delete_access_point` / `list_access_points`
- `get_access_point_policy` / `put_access_point_policy` / `delete_access_point_policy`
- `get_access_point_public_access_block` / `put_access_point_public_access_block` / `delete_access_point_public_access_block`

### Concurrent Transfers

Large objects are moved in parallel parts. Both helpers resolve what they need
with a `HeadObject` (download) or the file's own metadata (upload), so nothing
is fetched twice.

```rust
use alibabacloud_oss_sdk_rust_v2::api::object::{GetObjectRequest, PutObjectRequest};
use alibabacloud_oss_sdk_rust_v2::client::{DownloaderOptions, UploaderOptions};

// Download: parts are fetched concurrently and written at their final offsets.
let result = client
    .download_file(
        &GetObjectRequest {
            bucket: "my-bucket".to_string(),
            key: "large.bin".to_string(),
            ..Default::default()
        },
        "/local/large.bin",
        DownloaderOptions::default()
            .with_part_size(6 * 1024 * 1024)
            .with_parallel_num(4),
    )
    .await?;
println!("{} bytes, etag {:?}", result.written, result.etag);

// Upload: below one part it is a single PutObject; above, a multipart upload.
let result = client
    .upload_file(
        &PutObjectRequest {
            bucket: "my-bucket".to_string(),
            key: "large.bin".to_string(),
            ..Default::default()
        },
        "/local/large.bin",
        UploaderOptions::default().with_parallel_num(4),
    )
    .await?;
println!("upload id {:?}, etag {:?}", result.upload_id, result.etag);
```

Notes:

- **Checksums.** OSS reports one CRC64 for the whole object but checksums each
  part separately, and parts finish in arbitrary order. Both helpers fold the
  per-part checksums with `crc64_combine` and compare the result against the
  server's value; a mismatch fails the transfer rather than keeping bytes that
  are known to be wrong. A ranged download sees only a slice, so it reports no
  whole-object checksum instead of one that would be meaningless.
- **Partial writes.** The destination file is created before the first part
  lands, so a failure leaves what was already written. Download to a temporary
  path and rename on success if you need all-or-nothing semantics.
- **Upload failures.** A failed upload aborts the multipart upload, so no
  billable parts are left behind. Set `UploaderOptions::leave_parts_on_error`
  only when you intend to resume the upload id.
- **Part size floor.** OSS rejects multipart parts below 100 KiB; the uploader
  applies that floor before sending any part, so a smaller `part_size` is
  raised rather than failing at completion.

### Resumable Transfers

A transfer that fails can continue where it stopped instead of starting over. Progress lives in a small JSON checkpoint next to the destination; it is keyed to the source object and destination path, so a checkpoint can only be picked up by the transfer that wrote it.

```rust
use alibabacloud_oss_sdk_rust_v2::client::{
    DownloaderCheckpointOptions, UploaderCheckpointOptions,
};

let result = client
    .download_file_with_checkpoint(
        &GetObjectRequest {
            bucket: "my-bucket".to_string(),
            key: "large.bin".to_string(),
            ..Default::default()
        },
        "/local/large.bin",
        6 * 1024 * 1024, // part size
        DownloaderCheckpointOptions::default()
            .with_checkpoint_dir("/local/checkpoints")
            .with_verify_data(true),
    )
    .await?;
println!("resumed at {}, {} bytes this attempt", result.resumed_from, result.written);

let result = client
    .upload_file_with_checkpoint(
        &PutObjectRequest {
            bucket: "my-bucket".to_string(),
            key: "large.bin".to_string(),
            ..Default::default()
        },
        "/local/large.bin",
        UploaderCheckpointOptions::default()
            .with_checkpoint_dir("/local/checkpoints")
            .with_part_size(6 * 1024 * 1024),
    )
    .await?;
```

Notes:

- **Downloads resume at the first gap.** Parts arrive out of order, so the recorded progress is the longest contiguous run from the start — not the number of bytes written, which would skip a hole in the middle. `with_verify_data(true)` re-reads that prefix and checks its checksum before trusting it.
- **Uploads trust the service, not the checkpoint.** The checkpoint records only the upload ID; which parts exist is read back with `ListParts`, so a part the service never accepted is uploaded again. Only a run of full parts from part 1 is adopted — a short part in the middle would leave a hole.
- **A stale checkpoint is discarded, not resumed.** A checkpoint whose object, size, modification time, or part size does not match the transfer at hand is removed and the transfer starts fresh.
- **The checkpoint is removed only after the object exists.** A crash between completion and removal resumes an upload that is already done rather than losing the fact that it was.

### Server-Side Copy

Copies an object without the bytes travelling through the client. Sources above the threshold are copied part by part, and the destination's checksum is compared against the source's when the copy completes.

```rust
use alibabacloud_oss_sdk_rust_v2::api::object::CopyObjectRequest;
use alibabacloud_oss_sdk_rust_v2::client::CopierOptions;

let result = client
    .copy_object_to_object(
        &CopyObjectRequest {
            bucket: "dest-bucket".to_string(),
            key: "dest-key".to_string(),
            copy_source: "/src-bucket/src-key".to_string(),
            ..Default::default()
        },
        CopierOptions::default()
            .with_part_size(64 * 1024 * 1024)
            .with_multipart_copy_threshold(200 * 1024 * 1024),
    )
    .await?;
println!("etag {:?}, {} bytes", result.etag, result.transferred);
```

Notes:

- `MetadataDirective: COPY` (the default) carries the source's metadata to the destination and drops any metadata on the request; `REPLACE` uses the request's own values. An unrecognised directive is rejected before anything is sent.
- A large source first attempts one `CopyObject` under a 30-second deadline, which is much cheaper when the service allows it. Only a timeout or an `EntityTooLarge` rejection falls back to copying by parts; any other error is reported as-is.
- A failed part aborts the upload unless `leave_parts_on_error` is set.

### File-Like Handles

Three handles treat an object as a file. None of them buffer the whole object.

```rust
use alibabacloud_oss_sdk_rust_v2::client::{OpenOptions, AppendOptions, WriteOnlyOptions};

// Read: seekable, and checked against the object it was opened from.
let mut file = client
    .open_read_only_file("my-bucket", "my-object", OpenOptions::default())
    .await?;
let head = file.read_exact(16).await?;

// Append: the running checksum is carried on every write.
let mut file = client
    .open_append_only_file("my-bucket", "log.txt", AppendOptions::default())
    .await?;
file.write(b"a line\n".to_vec()).await?;

// Write: parts are uploaded as they fill.
let mut file = client.open_write_only_file(
    "my-bucket",
    "streamed.bin",
    WriteOnlyOptions::default().with_part_size(6 * 1024 * 1024),
);
file.write(vec![0u8; 1024]).await?;
file.close().await?;
```

Notes:

- **Reads are guarded.** The size, ETag, and last-modified time are captured at open; a ranged response whose `Content-Range` starts somewhere else, or whose object identity changed, fails the read instead of splicing two versions together.
- **Appends verify position.** The service rejects an append whose position does not match the object's length. That is retried once, and only when a re-read shows the object ends exactly where this write would have — a genuine concurrent writer is an error.
- **Small writes never open a multipart upload.** A `WriteOnlyFile` that never filled a part is written with a single `PutObject` on close. A failure is sticky: every later call reports it, and `close` refuses to commit.
- Attributes that have no multipart equivalent (the ACL) are applied on completion. An append's creation attributes apply only on the write that creates the object.

### Client-Side Encryption

`EncryptionClient` encrypts an object's bytes before they are sent and decrypts them on the way back. The service stores ciphertext plus an envelope: a fresh AES key and its IV, each wrapped with your RSA master key.

```rust
use alibabacloud_oss_sdk_rust_v2::client::EncryptionClient;
use alibabacloud_oss_sdk_rust_v2::crypto::{MasterRsaCipher, MasterCipher};
use std::collections::HashMap;

let master = MasterRsaCipher::new(&HashMap::new(), PUBLIC_KEY_PEM, PRIVATE_KEY_PEM);
let client = EncryptionClient::new(client, Box::new(master));

let result = client
    .put_object(PutObjectRequest {
        bucket: "my-bucket".to_string(),
        key: "secret.txt".to_string(),
        body: Some(BodyContent::from_bytes(b"contents".to_vec(), None)),
        ..Default::default()
    })
    .await?;

let mut result = client
    .get_object(GetObjectRequest {
        bucket: "my-bucket".to_string(),
        key: "secret.txt".to_string(),
        ..Default::default()
    })
    .await?;
let plaintext = result.get_all_data().await?;
```

Notes:

- **Ranges are widened, then trimmed.** AES-CTR's keystream is defined per block, so a ranged read of `bytes=15-29` is issued as `bytes=0-29` and the leading 15 bytes are discarded after decryption. Returning them would hand back plaintext the caller did not ask for.
- **Multipart parts are independent.** Each part is encrypted from a counter derived from its part number, so parts may be uploaded concurrently, retried, or replaced. The part size must be a multiple of the block size (16), or a part boundary would fall inside a block.
- **An unusable envelope is an error, not a pass-through.** Bytes that cannot be decrypted are ciphertext, and returning them as the object's contents would be worse than failing.
- **A plain object passes through unchanged.** Only objects carrying the envelope metadata are decrypted.
- Master keys are PEM, in either PKCS#8 or PKCS#1 form. Additional keys can be registered with `with_master_cipher` and are selected by the object's recorded key description.

### Bandwidth Limits

Paces uploads and downloads client-side. The limit is configured in KBps and applies to the bytes that actually cross the wire.

```rust
let config = Config::default()
    .with_upload_bandwidth_limit(1024)      // 1 MiB/s
    .with_download_bandwidth_limit(4 * 1024) // 4 MiB/s
    // A paced transfer outlives the default request timeout, so allow for it.
    .with_read_write_timeout(std::time::Duration::from_secs(300));
```

Notes:

- **The limit is a token bucket, not a per-packet delay.** The bucket admits a burst (a quarter second's worth, with a 4 MiB floor) and then paces the average. A 4 MiB floor is what lets one request still make progress under a low limit on a fast link.
- **Waiting is asynchronous.** The pacing happens on the request and response body streams, which the runtime is driving; a blocking wait there would stall everything else.
- **A limit and a short timeout conflict.** `read_write_timeout` bounds the whole request, so raising a limit's effect on duration means raising the timeout with it.
- Downloads are paced where the response body arrives, uploads where the request body leaves; reqwest owns the socket, so those streams are the only places the SDK can throttle.

### Presigning

Generate pre-signed URLs without sending a request:

```rust
use alibabacloud_oss_sdk_rust_v2::api::object::{GetObjectRequest, PutObjectRequest};
use alibabacloud_oss_sdk_rust_v2::client::PresignOptions;

// Default expiration (900s)
let result = client
    .presign(
        &GetObjectRequest {
            bucket: "my-bucket".to_string(),
            key: "my-object".to_string(),
            ..Default::default()
        },
        None,
    )
    .await?;
println!("url: {}", result.url);          // signed URL
println!("headers: {:?}", result.signed_headers); // send these unchanged

// Custom expiration (must be <= 7 days for the V4 signer)
let options = PresignOptions {
    expires: Some(std::time::Duration::from_secs(3600)),
    ..Default::default()
};
let put_request = PutObjectRequest {
    bucket: "my-bucket".to_string(),
    key: "upload-target.bin".to_string(),
    ..Default::default()
};
let result = client.presign(&put_request, Some(&options)).await?;
```

`PresignRequest` is implemented for `GetObjectRequest`, `PutObjectRequest`, `HeadObjectRequest`, `InitiateMultipartUploadRequest`, `UploadPartRequest`, `CompleteMultipartUploadRequest`, and `AbortMultipartUploadRequest` — the same set as the Go SDK v2. Signed headers (e.g. `Content-Type`, `x-oss-*`) are returned in `PresignResult::signed_headers` and must be sent unchanged with the URL.

### Paginators

All six list operations have paginators that handle markers, continuation tokens, and URL decoding:

```rust
let mut paginator = client.list_objects_v2_paginator(ListObjectsV2Request {
    bucket: "my-bucket".to_string(),
    ..Default::default()
});

while let Some(page) = paginator.next_page().await? {   // None when exhausted
    for object in &page.contents {                       // keys are already URL-decoded
        println!("{} ({} bytes)", object.key.as_deref().unwrap_or(""), object.size);
    }
}
// paginator.limit = Some(100);  // optional page-size override
```

Available: `list_objects_paginator`, `list_objects_v2_paginator`, `list_object_versions_paginator`, `list_buckets_paginator`, `list_parts_paginator`, `list_multipart_uploads_paginator`.

Notes:

- `has_next()` returns `true` until a non-truncated page arrives — the stop signal is the service's `IsTruncated`, not an empty page.
- Listing paginators set `encoding_type=url` on every request; object keys and markers are URL-decoded in the results.
- `limit` (public field) overrides the request's max page size (`max_keys` / `max_parts` / `max_uploads`) on every page when set.

## Architecture

### Project Structure

```text
aliyun-oss-sdk-rust-v2/
├── oss/                          # Main SDK implementation
│   ├── src/
│   │   ├── api/                  # API definitions
│   │   │   ├── accesspoint/     # Access point operations
│   │   │   ├── bucket/          # Bucket operations (103)
│   │   │   ├── object/          # Object operations (33)
│   │   │   ├── region/          # Region operations
│   │   │   ├── service/         # Service operations (7)
│   │   │   └── mod.rs
│   │   ├── client/              # Client implementation
│   │   │   ├── downloader.rs    # Concurrent ranged download
│   │   │   ├── invoker.rs       # Request invoking, signing context assembly
│   │   │   ├── paginators.rs    # Page-by-page list iterators
│   │   │   ├── presign.rs       # Pre-signed URL generation
│   │   │   └── uploader.rs      # Multipart upload
│   │   ├── credential/          # Authentication providers
│   │   ├── retry/               # Retry mechanisms
│   │   ├── signer/              # Request signing (V1/V4)
│   │   ├── transport/           # HTTP transport
│   │   ├── types/               # Type definitions
│   │   ├── utils/               # Utility functions
│   │   └── lib.rs
│   └── examples/                # Code examples
├── api_model/                   # API model macros
└── test_config.json             # Local integration test credentials (gitignored)
```

### Core Components

#### Request/Response Models

The SDK uses procedural macros to generate request and response models:

```rust
// Request structure
#[derive(Debug, OssRequestModel)]
pub struct GetObjectRequest {
    pub bucket: String,
    pub key: String,
    #[field(type = "header", rename = "If-Match")]
    pub if_match: Option<String>,
    #[field(type = "query", rename = "versionId")]
    pub version_id: Option<String>,
    pub common: RequestCommon,
}

// Response structure
#[derive(Debug, OssResultModel)]
pub struct GetObjectResult {
    #[field(type = "header", rename = "Content-Length")]
    pub content_length: Option<u64>,
    #[field(type = "header", rename = "ETag")]
    pub etag: Option<String>,
    pub common: ResultCommon,
}
```

#### Logging System

```rust
use std::rc::Rc;
use alibabacloud_oss_sdk_rust_v2::log::{LogLevel, LogOutput, StandardLogPrinter, StandardLogger};

let logger = StandardLogger::new(
    Rc::new(StandardLogPrinter::new(LogOutput::Stdout)),
    LogLevel::Debug,
);
```

In most cases you configure logging through `Config` instead:

```rust
let config = Config::default().with_log_level(LogLevel::Debug);
```

## Testing

### Run Unit Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test --lib client
```

All unit tests run offline and deterministically — the suite is fully green (493 tests) without network access or credentials. Tests that require a live OSS service skip themselves automatically when no real configuration is present (see below).

### Generate Coverage Report

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate HTML report
cargo llvm-cov --html

# Generate lcov report
cargo llvm-cov --lcov --output-path lcov.info
```

### Integration Tests with OSS Service

1. Obtain your AK/SK (AccessKey ID and AccessKey Secret)
   - **Recommended**: Create a RAM user with `AliyunOSSFullAccess` policy
   - **Alternative**: Use primary account AccessKey (not recommended for production)

2. Create an OSS bucket in the [OSS Console](https://oss.console.aliyun.com/bucket)

3. Create `test_config.json` in the project root:

```json
{
  "region": "cn-hangzhou",
  "bucket": "your-bucket-name",
  "version_bucket": "your-versioned-bucket-name",
  "object": "your-test-object-name",
  "access_key_id": "your-access-key-id",
  "access_key_secret": "your-access-key-secret"
}
```

1. Run tests:

```bash
cargo test
```

Live tests are skipped in two cases, both reported on stderr:

- No `test_config.json` found (any of the standard search paths)
- The config still contains the shipped placeholder values (`xxxxx`, `xxxxxxxxxxxx`, `xxxxxxxx`) — so the committed template never triggers accidental live runs

## Development

### Prerequisites

Install [pre-commit](https://pre-commit.com) for code quality:

```bash
pip install pre-commit
pre-commit install
```

### Build from Source

```bash
git clone https://github.com/aliyun/alibabacloud-oss-rust-sdk-v2.git
cd alibabacloud-oss-rust-sdk-v2
cargo build --release
```

### Adding a New API

1. Create `{api_name}.rs` in `oss/src/api/{accesspoint|bucket|object|region|service}/`
2. Define `ApiNameRequest` and `ApiNameResult` structs
3. Implement `api_name()` method for `Client`
4. Export it from the category's `mod.rs` (`pub use self::api_name::*;`)
5. Add unit tests in the same file (`#[cfg(test)] mod tests`) — offline tests only; live tests must skip when no real `test_config.json` is present

Example:

```rust
// oss/src/api/object/my_api.rs
#[derive(Debug, OssRequestModel)]
pub struct MyApiRequest {
    pub bucket: String,
    pub key: String,
    #[field(type = "query")]
    pub param: Option<String>,
    pub common: RequestCommon,
}

#[derive(Debug, OssResultModel)]
pub struct MyApiResult {
    #[field(type = "header")]
    pub custom_header: Option<String>,
    pub common: ResultCommon,
}

impl Client {
    pub async fn my_api(&self, request: &MyApiRequest) -> Result<MyApiResult, Box<dyn std::error::Error + Send + Sync>> {
        // Implementation
    }
}
```

## Contributing

We welcome contributions! Here's how you can help:

### Ways to Contribute

- Report bugs and issues
- Suggest new features
- Improve documentation
- Submit pull requests
- Add new API implementations
- Enhance test coverage

### Guidelines

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/your-feature`
3. **Make your changes**
4. **Run tests**: Ensure all tests pass
5. **Run pre-commit hooks**: `pre-commit run --all-files`
6. **Commit with clear messages**
7. **Submit a pull request**

### Good First Issues

Look for issues labeled "good first issue" or "help wanted" to get started.

## Troubleshooting

### Common Issues

#### Authentication Failed

```text
Error: InvalidAccessKeyId
```

**Solution**:

- Verify `ACCESS_KEY_ID` and `ACCESS_KEY_SECRET` are correct
- Ensure the AccessKey is active in RAM console
- Check for extra spaces in environment variables

#### Bucket Not Found

```text
Error: NoSuchBucket
```

**Solution**:

- Verify the bucket exists in the specified region
- Check `OSS_BUCKET` environment variable
- Ensure region matches bucket location

#### Permission Denied

```text
Error: AccessDenied
```

**Solution**:

- Verify RAM user has necessary OSS permissions
- Check bucket ACL and policy settings
- Use `AliyunOSSFullAccess` policy for testing

#### Network Errors

```text
Error: Connection timeout
```

**Solution**:

- Check network connection
- Verify region endpoint is accessible
- Transient failures are retried automatically by the default `Standard` retryer; tune it via `.with_retryer(...)` or cap attempts with `.with_retry_max_attempts(n)`
- Adjust timeout settings in configuration

#### "region is not set (required for V4 signature)"

```text
Error: InvalidConfiguration: region is not set (required for V4 signature)
```

**Solution**:

- The V4 signer requires a region even when a custom endpoint is set — add `.with_region("cn-hangzhou")` (or your region) to the config
- With the V1 signer the region is optional

#### Presigned URL Rejected After a While

**Symptom**: A pre-signed URL returns `AccessDenied` or `RequestTimeTooSkewed` after some time.

**Solution**:

- Pre-signed URLs expire; the default is 900 seconds. Set `PresignOptions::expires` for a longer window
- The V4 signer caps expiration at **7 days** — expiry values beyond that are rejected at signing time
- If the client uses a clock different from OSS, the signature can appear expired early; keep the client clock in sync (NTP)

### Getting Help

1. Check the [FAQ](https://www.alibabacloud.com/help/en/oss/faq)
2. Review existing GitHub issues
3. Create a new issue with detailed information
4. Contact Alibaba Cloud support

## Additional Resources

### Documentation

- [Official OSS Documentation](https://www.alibabacloud.com/help/en/oss)
- [API Reference](https://www.alibabacloud.com/help/en/oss/api-reference)
- [SDK Source Code](https://github.com/aliyun/aliyun-oss-rust-sdk)
- [Pricing Calculator](https://www.alibabacloud.com/pricing-calculator)

### Related Projects

- [Alibaba Cloud SDK for Python](https://github.com/aliyun/aliyun-openapi-python-sdk)
- [Alibaba Cloud SDK for Java](https://github.com/aliyun/aliyun-openapi-java-sdk)
- [Alibaba Cloud SDK for Go](https://github.com/aliyun/aliyun-oss-go-sdk)

### Security Best Practices

- Never hardcode credentials in source code
- Use environment variables or secure secret management
- Apply principle of least privilege for RAM policies
- Use HTTPS for all OSS operations
- Rotate AccessKeys regularly
- Enable OSS access logging for audit

---

**License**: Apache License 2.0

**Support**: For questions and issues, please open a GitHub issue or contact Alibaba Cloud support.
