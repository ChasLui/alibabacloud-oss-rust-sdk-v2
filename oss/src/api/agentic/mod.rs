//! Agentic bucket APIs.
//!
//! Operations live in this module as `XxxRequest` / `XxxResult` pairs plus
//! `impl Client` blocks, following the pattern of [`crate::api::bucket`].
//!
//! # Addressing
//!
//! Every request's `bucket` field is a *prefix*, not a physical bucket name.
//! Build the client with [`Client::new_agentic`], which installs the resolver
//! that expands the prefix to `{prefix}-{accountId}-{region}-ab-apsr` for both
//! signing and the request host; `Config::with_account_id` and
//! `Config::with_region` are therefore required. To address a bucket space
//! instead, expand the prefix with [`BucketSpaceHelper::to_bucket_name`] and
//! pass the result to a plain [`Client`].
//!
//! Mirrors Go's `agentic` package and Java's `oss2.agentic` package.

mod create_agentic_bucket;
mod delete_agentic_bucket;
mod delete_agentic_bucket_encryption;
mod delete_agentic_bucket_policy;
mod delete_agentic_bucket_public_access_block;
mod get_agentic_bucket;
mod get_agentic_bucket_acl;
mod get_agentic_bucket_encryption;
mod get_agentic_bucket_policy;
mod get_agentic_bucket_public_access_block;
mod get_agentic_bucket_versioning;
mod list_agentic_buckets;
mod list_bucket_spaces;
mod paginators;
mod provider;
mod put_agentic_bucket_acl;
mod put_agentic_bucket_encryption;
mod put_agentic_bucket_policy;
mod put_agentic_bucket_public_access_block;
mod put_agentic_bucket_status;
mod put_agentic_bucket_versioning;

#[cfg(test)]
mod test_support;

pub use self::create_agentic_bucket::*;
pub use self::delete_agentic_bucket::*;
pub use self::delete_agentic_bucket_encryption::*;
pub use self::delete_agentic_bucket_policy::*;
pub use self::delete_agentic_bucket_public_access_block::*;
pub use self::get_agentic_bucket::*;
pub use self::get_agentic_bucket_acl::*;
pub use self::get_agentic_bucket_encryption::*;
pub use self::get_agentic_bucket_policy::*;
pub use self::get_agentic_bucket_public_access_block::*;
pub use self::get_agentic_bucket_versioning::*;
pub use self::list_agentic_buckets::*;
pub use self::list_bucket_spaces::*;
pub use self::paginators::*;
pub use self::provider::*;
pub use self::put_agentic_bucket_acl::*;
pub use self::put_agentic_bucket_encryption::*;
pub use self::put_agentic_bucket_policy::*;
pub use self::put_agentic_bucket_public_access_block::*;
pub use self::put_agentic_bucket_status::*;
pub use self::put_agentic_bucket_versioning::*;
