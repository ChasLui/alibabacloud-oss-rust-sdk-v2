//! Vector bucket APIs.
//!
//! Operations live in this module as `XxxRequest` / `XxxResult` pairs plus
//! `impl Client` blocks, following the pattern of [`crate::api::bucket`].
//!
//! # Addressing
//!
//! A vector bucket is addressed as `{bucket}-{account_id}.{endpoint}` and is
//! signed through an ARN path, so build the client with
//! [`Client::new_vectors`]. `Config::with_account_id` and
//! `Config::with_region` are required; the endpoint defaults to
//! `{region}.oss-vectors.aliyuncs.com` when none is configured.
//!
//! # Bucket policy and logging
//!
//! Those operations are the plain bucket ones — the product reuses them — so
//! their types are re-exported here rather than duplicated. Call them on a
//! client built with [`Client::new_vectors`].
//!
//! Mirrors Go's `vectors` package and Java's `oss2.vectors` package.

mod delete_vector_bucket;
mod delete_vector_index;
mod delete_vectors;
mod get_vector_bucket;
mod get_vector_index;
mod get_vectors;
mod list_vector_buckets;
mod list_vector_indexes;
mod list_vectors;
mod provider;
mod put_vector_bucket;
mod put_vector_index;
mod put_vectors;
mod query_vectors;

pub use self::delete_vector_bucket::*;
pub use self::delete_vector_index::*;
pub use self::delete_vectors::*;
pub use self::get_vector_bucket::*;
pub use self::get_vector_index::*;
pub use self::get_vectors::*;
pub use self::list_vector_buckets::*;
pub use self::list_vector_indexes::*;
pub use self::list_vectors::*;
pub use self::provider::*;
pub use self::put_vector_bucket::*;
pub use self::put_vector_index::*;
pub use self::put_vectors::*;
pub use self::query_vectors::*;
// Bucket policy and logging are the plain bucket operations, delegated to the
// same `Client` methods. Mirrors Go's type aliases in the vectors package.
pub use crate::api::bucket::{
    BucketLoggingStatus, DeleteBucketLoggingRequest, DeleteBucketLoggingResult,
    DeleteBucketPolicyRequest, DeleteBucketPolicyResult, GetBucketLoggingRequest,
    GetBucketLoggingResult, GetBucketPolicyRequest, GetBucketPolicyResult, LoggingEnabled,
    PutBucketLoggingRequest, PutBucketLoggingResult, PutBucketPolicyRequest, PutBucketPolicyResult,
};
