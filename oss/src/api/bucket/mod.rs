mod abort_bucket_worm;
mod close_meta_query;
mod complete_bucket_worm;
mod create_access_point_for_object_process;
mod create_bucket;
mod create_bucket_data_redundancy_transition;
mod create_cname_token;
mod delete_access_point_for_object_process;
mod delete_access_point_policy_for_object_process;
mod delete_bucket;
mod delete_bucket_cors;
mod delete_bucket_data_redundancy_transition;
mod delete_bucket_encryption;
mod delete_bucket_inventory;
mod delete_bucket_lifecycle;
mod delete_bucket_logging;
mod delete_bucket_overwrite_config;
mod delete_bucket_policy;
mod delete_bucket_public_access_block;
mod delete_bucket_replication;
mod delete_bucket_tags;
mod delete_bucket_website;
mod delete_cname;
mod delete_style;
mod delete_user_defined_log_fields_config;
mod do_meta_query;
mod extend_bucket_worm;
mod get_access_point_config_for_object_process;
mod get_access_point_for_object_process;
mod get_access_point_policy_for_object_process;
mod get_bucket_access_monitor;
mod get_bucket_acl;
mod get_bucket_archive_direct_read;
mod get_bucket_cors;
mod get_bucket_data_redundancy_transition;
mod get_bucket_encryption;
mod get_bucket_https_config;
mod get_bucket_info;
mod get_bucket_inventory;
mod get_bucket_lifecycle;
mod get_bucket_location;
mod get_bucket_logging;
mod get_bucket_object_worm_configuration;
mod get_bucket_overwrite_config;
mod get_bucket_policy;
mod get_bucket_policy_status;
mod get_bucket_public_access_block;
mod get_bucket_referer;
mod get_bucket_replication;
mod get_bucket_replication_location;
mod get_bucket_replication_progress;
mod get_bucket_request_payment;
mod get_bucket_resource_group;
mod get_bucket_stat;
mod get_bucket_tags;
mod get_bucket_transfer_acceleration;
mod get_bucket_versioning;
mod get_bucket_website;
mod get_bucket_worm;
mod get_cname_token;
mod get_meta_query_status;
mod get_style;
mod get_user_defined_log_fields_config;
mod initiate_bucket_worm;
mod list_access_point_for_object_process;
mod list_bucket_data_redundancy_transition;
mod list_bucket_inventory;
mod list_cname;
mod list_object_versions;
mod list_objects;
mod list_objects_v2;
mod list_style;
mod list_user_data_redundancy_transition;
mod open_meta_query;
mod option_object;
mod put_access_point_config_for_object_process;
mod put_access_point_policy_for_object_process;
mod put_bucket_access_monitor;
mod put_bucket_acl;
mod put_bucket_archive_direct_read;
mod put_bucket_cors;
mod put_bucket_encryption;
mod put_bucket_https_config;
mod put_bucket_inventory;
mod put_bucket_lifecycle;
mod put_bucket_logging;
mod put_bucket_object_worm_configuration;
mod put_bucket_overwrite_config;
mod put_bucket_policy;
mod put_bucket_public_access_block;
mod put_bucket_referer;
mod put_bucket_replication;
mod put_bucket_request_payment;
mod put_bucket_resource_group;
mod put_bucket_rtc;
mod put_bucket_tags;
mod put_bucket_transfer_acceleration;
mod put_bucket_versioning;
mod put_bucket_website;
mod put_cname;
mod put_style;
mod put_user_defined_log_fields_config;
mod write_get_object_response;

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

pub use self::abort_bucket_worm::*;
pub use self::close_meta_query::*;
pub use self::complete_bucket_worm::*;
pub use self::create_access_point_for_object_process::*;
pub use self::create_bucket::*;
pub use self::create_bucket_data_redundancy_transition::*;
pub use self::create_cname_token::*;
pub use self::delete_access_point_for_object_process::*;
pub use self::delete_access_point_policy_for_object_process::*;
pub use self::delete_bucket::*;
pub use self::delete_bucket_cors::*;
pub use self::delete_bucket_data_redundancy_transition::*;
pub use self::delete_bucket_encryption::*;
pub use self::delete_bucket_inventory::*;
pub use self::delete_bucket_lifecycle::*;
pub use self::delete_bucket_logging::*;
pub use self::delete_bucket_overwrite_config::*;
pub use self::delete_bucket_policy::*;
pub use self::delete_bucket_public_access_block::*;
pub use self::delete_bucket_replication::*;
pub use self::delete_bucket_tags::*;
pub use self::delete_bucket_website::*;
pub use self::delete_cname::*;
pub use self::delete_style::*;
pub use self::delete_user_defined_log_fields_config::*;
pub use self::do_meta_query::*;
pub use self::extend_bucket_worm::*;
pub use self::get_access_point_config_for_object_process::*;
pub use self::get_access_point_for_object_process::*;
pub use self::get_access_point_policy_for_object_process::*;
pub use self::get_bucket_access_monitor::*;
pub use self::get_bucket_acl::*;
pub use self::get_bucket_archive_direct_read::*;
pub use self::get_bucket_cors::*;
pub use self::get_bucket_data_redundancy_transition::*;
pub use self::get_bucket_encryption::*;
pub use self::get_bucket_https_config::*;
pub use self::get_bucket_info::*;
pub use self::get_bucket_inventory::*;
pub use self::get_bucket_lifecycle::*;
pub use self::get_bucket_location::*;
pub use self::get_bucket_logging::*;
pub use self::get_bucket_object_worm_configuration::*;
pub use self::get_bucket_overwrite_config::*;
pub use self::get_bucket_policy::*;
pub use self::get_bucket_policy_status::*;
pub use self::get_bucket_public_access_block::*;
pub use self::get_bucket_referer::*;
pub use self::get_bucket_replication::*;
pub use self::get_bucket_replication_location::*;
pub use self::get_bucket_replication_progress::*;
pub use self::get_bucket_request_payment::*;
pub use self::get_bucket_resource_group::*;
pub use self::get_bucket_stat::*;
pub use self::get_bucket_tags::*;
pub use self::get_bucket_transfer_acceleration::*;
pub use self::get_bucket_versioning::*;
pub use self::get_bucket_website::*;
pub use self::get_bucket_worm::*;
pub use self::get_cname_token::*;
pub use self::get_meta_query_status::*;
pub use self::get_style::*;
pub use self::get_user_defined_log_fields_config::*;
pub use self::initiate_bucket_worm::*;
pub use self::list_access_point_for_object_process::*;
pub use self::list_bucket_data_redundancy_transition::*;
pub use self::list_bucket_inventory::*;
pub use self::list_cname::*;
pub use self::list_object_versions::*;
pub use self::list_objects::*;
pub use self::list_objects_v2::*;
pub use self::list_style::*;
pub use self::list_user_data_redundancy_transition::*;
pub use self::open_meta_query::*;
pub use self::option_object::*;
pub use self::put_access_point_config_for_object_process::*;
pub use self::put_access_point_policy_for_object_process::*;
pub use self::put_bucket_access_monitor::*;
pub use self::put_bucket_acl::*;
pub use self::put_bucket_archive_direct_read::*;
pub use self::put_bucket_cors::*;
pub use self::put_bucket_encryption::*;
pub use self::put_bucket_https_config::*;
pub use self::put_bucket_inventory::*;
pub use self::put_bucket_lifecycle::*;
pub use self::put_bucket_logging::*;
pub use self::put_bucket_object_worm_configuration::*;
pub use self::put_bucket_overwrite_config::*;
pub use self::put_bucket_policy::*;
pub use self::put_bucket_public_access_block::*;
pub use self::put_bucket_referer::*;
pub use self::put_bucket_replication::*;
pub use self::put_bucket_request_payment::*;
pub use self::put_bucket_resource_group::*;
pub use self::put_bucket_rtc::*;
pub use self::put_bucket_tags::*;
pub use self::put_bucket_transfer_acceleration::*;
pub use self::put_bucket_versioning::*;
pub use self::put_bucket_website::*;
pub use self::put_cname::*;
pub use self::put_style::*;
pub use self::put_user_defined_log_fields_config::*;
pub use self::write_get_object_response::*;
use crate::utils::{acl_grant_de, option_time_rfc3339_serde};

#[derive(Debug, Default, Deserialize)]
pub struct BucketProperties {
    /// The name of the bucket.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The data center in which the bucket is located.
    #[serde(rename = "Location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// The time when the bucket was created. Format:
    /// yyyy-mm-ddThh:mm:ss.timezone.
    #[serde(rename = "CreationDate", with = "option_time_rfc3339_serde")]
    pub creation_date: Option<SystemTime>,

    /// The storage class of the bucket. Valid values:
    /// Standard, IA, Archive, ColdArchive, and DeepColdArchive.
    #[serde(rename = "StorageClass", skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,

    /// The public endpoint used to access the bucket over the Internet.
    #[serde(rename = "ExtranetEndpoint", skip_serializing_if = "Option::is_none")]
    pub extranet_endpoint: Option<String>,

    /// The internal endpoint that is used to access the bucket from ECS
    /// instances that reside in the same region as the bucket.
    #[serde(rename = "IntranetEndpoint", skip_serializing_if = "Option::is_none")]
    pub intranet_endpoint: Option<String>,

    /// The region in which the bucket is located.
    #[serde(rename = "Region", skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// The ID of the resource group to which the bucket belongs.
    #[serde(rename = "ResourceGroupId", skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct BucketInfo {
    /// Indicates whether access tracking is enabled for the bucket.
    #[serde(rename = "AccessMonitor", skip_serializing_if = "Option::is_none")]
    pub access_monitor: Option<String>,

    /// The time when the bucket is created. The time is in UTC.
    #[serde(rename = "CreationDate", with = "option_time_rfc3339_serde", default)]
    pub creation_date: Option<SystemTime>,

    /// Indicates whether cross-region replication (CRR) is enabled for the
    /// bucket.
    #[serde(
        rename = "CrossRegionReplication",
        skip_serializing_if = "Option::is_none"
    )]
    pub cross_region_replication: Option<String>,

    /// The disaster recovery type of the bucket.
    #[serde(rename = "DataRedundancyType", skip_serializing_if = "Option::is_none")]
    pub data_redundancy_type: Option<String>,

    /// The public endpoint that is used to access the bucket over the Internet.
    #[serde(rename = "ExtranetEndpoint", skip_serializing_if = "Option::is_none")]
    pub extranet_endpoint: Option<String>,

    /// The internal endpoint that is used to access the bucket from Elastic
    #[serde(rename = "IntranetEndpoint", skip_serializing_if = "Option::is_none")]
    pub intranet_endpoint: Option<String>,

    /// The region in which the bucket is located.
    #[serde(rename = "Location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// The name of the bucket.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The ID of the resource group to which the bucket belongs.
    #[serde(rename = "ResourceGroupId", skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<String>,

    /// The storage class of the bucket.
    #[serde(rename = "StorageClass", skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,

    /// Indicates whether transfer acceleration is enabled for the bucket.
    #[serde(
        rename = "TransferAcceleration",
        skip_serializing_if = "Option::is_none"
    )]
    pub transfer_acceleration: Option<String>,

    /// The container that stores the information about the bucket owner.
    #[serde(rename = "Owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<Owner>,

    /// The container that stores the access control list (ACL) information
    /// about the bucket.
    #[serde(rename = "AccessControlList", with = "acl_grant_de")]
    pub acl: Option<String>,

    /// The container that stores the server-side encryption method.
    #[serde(rename = "ServerSideEncryptionRule")]
    pub sse_rule: SSERule,

    /// The container that stores the logs.
    #[serde(rename = "BucketPolicy")]
    pub bucket_policy: BucketPolicy,

    /// Indicates whether versioning is enabled for the bucket.
    #[serde(rename = "Versioning", skip_serializing_if = "Option::is_none")]
    pub versioning: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Owner {
    /// The ID of the bucket owner.
    #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The name of the bucket owner.
    #[serde(rename = "DisplayName", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SSERule {
    /// The customer master key (CMK) ID in use. A valid value is returned only
    /// if you set SSEAlgorithm to KMS and specify the CMK ID. In other
    /// cases, an empty value is returned.
    #[serde(rename = "KMSMasterKeyID", skip_serializing_if = "Option::is_none")]
    pub kms_master_key_id: Option<String>,

    /// The server-side encryption method that is used by default.
    #[serde(rename = "SSEAlgorithm", skip_serializing_if = "Option::is_none")]
    pub sse_algorithm: Option<String>,

    /// Object's encryption algorithm. If this element is not included in the
    /// response, it indicates that the object is using the AES256
    /// encryption algorithm. This option is only valid if the SSEAlgorithm
    /// value is KMS.
    #[serde(rename = "KMSDataEncryption", skip_serializing_if = "Option::is_none")]
    pub kms_data_encryption: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BucketPolicy {
    /// The name of the bucket that stores the logs.
    #[serde(rename = "LogBucket", skip_serializing_if = "Option::is_none")]
    pub log_bucket: Option<String>,

    /// The directory in which logs are stored.
    #[serde(rename = "LogPrefix", skip_serializing_if = "Option::is_none")]
    pub log_prefix: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CommonPrefix {
    #[serde(rename = "Prefix")]
    pub prefix: String,
}
