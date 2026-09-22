//! Data process APIs.
//!
//! Operations live in this module as `XxxRequest` / `XxxResult` pairs plus
//! `impl Client` blocks, following the pattern of [`crate::api::bucket`].

mod create_dataset;
mod create_smart_cluster;
mod delete_data_pipeline_configuration;
mod delete_dataset;
mod delete_file_meta;
mod delete_smart_cluster;
mod get_data_pipeline_configuration;
mod get_dataset;
mod get_smart_cluster;
mod list_data_pipeline_configurations;
mod list_datasets;
mod list_smart_clusters;
mod paginator;
mod pause_data_pipeline;
mod put_data_pipeline_configuration;
mod restart_data_pipeline;
mod semantic_query;
mod simple_query;
mod update_dataset;
mod update_smart_cluster;

pub use self::create_dataset::*;
pub use self::create_smart_cluster::*;
pub use self::delete_data_pipeline_configuration::*;
pub use self::delete_dataset::*;
pub use self::delete_file_meta::*;
pub use self::delete_smart_cluster::*;
pub use self::get_data_pipeline_configuration::*;
pub use self::get_dataset::*;
pub use self::get_smart_cluster::*;
pub use self::list_data_pipeline_configurations::*;
pub use self::list_datasets::*;
pub use self::list_smart_clusters::*;
pub use self::paginator::*;
pub use self::pause_data_pipeline::*;
pub use self::put_data_pipeline_configuration::*;
pub use self::restart_data_pipeline::*;
pub use self::semantic_query::*;
pub use self::simple_query::*;
pub use self::update_dataset::*;
pub use self::update_smart_cluster::*;
// The four metadata-index operations are implemented alongside the other
// bucket operations but belong to the data process surface. Re-export their
// types so that `api::dataprocess` mirrors the Go SDK's `dataprocess` package.
pub use crate::api::bucket::{
    CloseMetaQueryRequest, CloseMetaQueryResult, DoMetaQueryRequest, DoMetaQueryResult,
    GetMetaQueryStatusRequest, GetMetaQueryStatusResult, MetaQuery, MetaQueryAggregation,
    MetaQueryAggregations, MetaQueryFile, MetaQueryFiles, MetaQueryMediaTypes, MetaQueryStatus,
    OpenMetaQueryRequest, OpenMetaQueryResult,
};
