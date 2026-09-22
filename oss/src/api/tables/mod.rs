//! Table bucket APIs.
//!
//! Operations live in this module as `XxxRequest` / `XxxResult` pairs plus
//! `impl Client` blocks, following the pattern of [`crate::api::bucket`].
//!
//! # Addressing
//!
//! A table bucket is addressed by its ARN
//! (`acs:osstables:{region}:{account_id}:bucket/{name}`), so build the client
//! with [`Client::new_tables`]. `Config::with_region` is required; the endpoint
//! defaults to `{region}.oss-tables.aliyuncs.com` when none is configured, and
//! requests are signed through [`crate::signer::tables_v4::SignerTablesV4`].
//!
//! The three list operations also come with paginators, mirroring Go's
//! `client_paginators.go`.
//!
//! Mirrors Go's `tables` package.

mod create_namespace;
mod create_table;
mod create_table_bucket;
mod delete_namespace;
mod delete_table;
mod delete_table_bucket;
mod delete_table_bucket_encryption;
mod delete_table_bucket_policy;
mod delete_table_policy;
mod get_namespace;
mod get_table;
mod get_table_bucket;
mod get_table_bucket_encryption;
mod get_table_bucket_maintenance_configuration;
mod get_table_bucket_policy;
mod get_table_encryption;
mod get_table_maintenance_configuration;
mod get_table_maintenance_job_status;
mod get_table_metadata_location;
mod get_table_policy;
mod list_namespaces;
mod list_table_buckets;
mod list_tables;
mod paginators;
mod provider;
mod put_table_bucket_encryption;
mod put_table_bucket_maintenance_configuration;
mod put_table_bucket_policy;
mod put_table_maintenance_configuration;
mod put_table_policy;
mod rename_table;
mod types;
mod update_table_metadata_location;

pub use self::create_namespace::*;
pub use self::create_table::*;
pub use self::create_table_bucket::*;
pub use self::delete_namespace::*;
pub use self::delete_table::*;
pub use self::delete_table_bucket::*;
pub use self::delete_table_bucket_encryption::*;
pub use self::delete_table_bucket_policy::*;
pub use self::delete_table_policy::*;
pub use self::get_namespace::*;
pub use self::get_table::*;
pub use self::get_table_bucket::*;
pub use self::get_table_bucket_encryption::*;
pub use self::get_table_bucket_maintenance_configuration::*;
pub use self::get_table_bucket_policy::*;
pub use self::get_table_encryption::*;
pub use self::get_table_maintenance_configuration::*;
pub use self::get_table_maintenance_job_status::*;
pub use self::get_table_metadata_location::*;
pub use self::get_table_policy::*;
pub use self::list_namespaces::*;
pub use self::list_table_buckets::*;
pub use self::list_tables::*;
pub use self::paginators::*;
pub use self::provider::*;
pub use self::put_table_bucket_encryption::*;
pub use self::put_table_bucket_maintenance_configuration::*;
pub use self::put_table_bucket_policy::*;
pub use self::put_table_maintenance_configuration::*;
pub use self::put_table_policy::*;
pub use self::rename_table::*;
pub use self::types::*;
pub use self::update_table_metadata_location::*;
