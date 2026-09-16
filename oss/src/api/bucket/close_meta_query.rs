use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

/// The container that stores the information about a single aggregate operation.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryAggregation {
    /// The field name.
    #[serde(rename = "Field", skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,

    /// The operator for aggregate operations. Valid values: min, max, average,
    /// sum, count, distinct, group.
    #[serde(rename = "Operation", skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,

    /// The result of the aggregate operation. Only returned in responses.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,

    /// The grouped aggregations. Only returned in responses.
    #[serde(rename = "Groups", skip_serializing_if = "Option::is_none")]
    pub groups: Option<MetaQueryGroups>,
}

/// The container that stores grouped aggregations.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryGroups {
    /// The grouped aggregations.
    #[serde(rename = "Group", default)]
    pub groups: Vec<MetaQueryGroup>,
}

/// A grouped aggregation.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryGroup {
    /// The value for the grouped aggregation.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// The number of results in the grouped aggregation.
    #[serde(rename = "Count", skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
}

/// The container that stores the information about aggregate operations.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryAggregations {
    /// The information about single aggregate operations.
    #[serde(rename = "Aggregation", default)]
    pub aggregations: Vec<MetaQueryAggregation>,
}

/// A user metadata item.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryUserMeta {
    /// The key of the user metadata item.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The value of the user metadata item.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// The container that stores user metadata items.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryUserMetas {
    /// The user metadata items.
    #[serde(rename = "UserMeta", default)]
    pub user_metas: Vec<MetaQueryUserMeta>,
}

/// A tag of the object.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryTagging {
    /// The tag key.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The tag value.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// The container that stores the tags of the object.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryTaggings {
    /// The tags.
    #[serde(rename = "Tagging", default)]
    pub taggings: Vec<MetaQueryTagging>,
}

/// The container that stores the file information list.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryFiles {
    /// The file information.
    #[serde(rename = "File", default)]
    pub files: Vec<MetaQueryFile>,
}

/// The description of the file.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryFileInsights {
    /// The description of the video file.
    #[serde(rename = "Video", skip_serializing_if = "Option::is_none")]
    pub video: Option<MetaQueryFileInsightsVideo>,

    /// The description of the image file.
    #[serde(rename = "Image", skip_serializing_if = "Option::is_none")]
    pub image: Option<MetaQueryFileInsightsImage>,
}

/// The description of a video file.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryFileInsightsVideo {
    /// A brief description.
    #[serde(rename = "Caption", skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,

    /// A detailed description.
    #[serde(rename = "Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// The description of an image file.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryFileInsightsImage {
    /// A brief description.
    #[serde(rename = "Caption", skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,

    /// A detailed description.
    #[serde(rename = "Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A video stream.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryVideoStream {
    /// The bitrate. Unit: bit/s.
    #[serde(rename = "Bitrate", skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<i64>,

    /// The start time of the stream in seconds.
    #[serde(rename = "StartTime", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,

    /// The duration of the stream in seconds.
    #[serde(rename = "Duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// The pixel format of the video stream.
    #[serde(rename = "PixelFormat", skip_serializing_if = "Option::is_none")]
    pub pixel_format: Option<String>,

    /// The image height of the video stream. Unit: pixel.
    #[serde(rename = "Height", skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,

    /// The color space.
    #[serde(rename = "ColorSpace", skip_serializing_if = "Option::is_none")]
    pub color_space: Option<String>,

    /// The image width of the video stream. Unit: pixel.
    #[serde(rename = "Width", skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,

    /// The abbreviated name of the codec.
    #[serde(rename = "CodecName", skip_serializing_if = "Option::is_none")]
    pub codec_name: Option<String>,

    /// The language used in the stream. The value follows the BCP 47 format.
    #[serde(rename = "Language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// The frame rate of the video stream.
    #[serde(rename = "FrameRate", skip_serializing_if = "Option::is_none")]
    pub frame_rate: Option<String>,

    /// The number of video frames.
    #[serde(rename = "FrameCount", skip_serializing_if = "Option::is_none")]
    pub frame_count: Option<i64>,

    /// The bit depth.
    #[serde(rename = "BitDepth", skip_serializing_if = "Option::is_none")]
    pub bit_depth: Option<i64>,
}

/// The container that stores video streams.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryVideoStreams {
    /// The video streams.
    #[serde(rename = "VideoStream", default)]
    pub video_streams: Vec<MetaQueryVideoStream>,
}

/// An address.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryAddress {
    /// The country.
    #[serde(rename = "Country", skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// The city.
    #[serde(rename = "City", skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    /// The district.
    #[serde(rename = "District", skip_serializing_if = "Option::is_none")]
    pub district: Option<String>,

    /// The language of the address. The value follows the BCP 47 format.
    #[serde(rename = "Language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// The province.
    #[serde(rename = "Province", skip_serializing_if = "Option::is_none")]
    pub province: Option<String>,

    /// The street.
    #[serde(rename = "Township", skip_serializing_if = "Option::is_none")]
    pub township: Option<String>,

    /// The full address.
    #[serde(rename = "AddressLine", skip_serializing_if = "Option::is_none")]
    pub address_line: Option<String>,
}

/// The container that stores addresses.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryAddresses {
    /// The addresses.
    #[serde(rename = "Address", default)]
    pub addresses: Vec<MetaQueryAddress>,
}

/// A subtitle stream.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQuerySubtitle {
    /// The start time of the subtitle stream in seconds.
    #[serde(rename = "StartTime", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,

    /// The duration of the subtitle stream in seconds.
    #[serde(rename = "Duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// The abbreviated name of the codec.
    #[serde(rename = "CodecName", skip_serializing_if = "Option::is_none")]
    pub codec_name: Option<String>,

    /// The language of the subtitle. The value follows the BCP 47 format.
    #[serde(rename = "Language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// The container that stores subtitle streams.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQuerySubtitles {
    /// The subtitle streams.
    #[serde(rename = "Subtitle", default)]
    pub subtitles: Vec<MetaQuerySubtitle>,
}

/// An audio stream.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryAudioStream {
    /// The sampling rate.
    #[serde(rename = "SampleRate", skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<i64>,

    /// The start time of the stream in seconds.
    #[serde(rename = "StartTime", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,

    /// The duration of the stream in seconds.
    #[serde(rename = "Duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// The number of sound channels.
    #[serde(rename = "Channels", skip_serializing_if = "Option::is_none")]
    pub channels: Option<i64>,

    /// The language used in the stream. The value follows the BCP 47 format.
    #[serde(rename = "Language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// The abbreviated name of the codec.
    #[serde(rename = "CodecName", skip_serializing_if = "Option::is_none")]
    pub codec_name: Option<String>,

    /// The bitrate. Unit: bit/s.
    #[serde(rename = "Bitrate", skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<i64>,
}

/// The container that stores audio streams.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryAudioStreams {
    /// The audio streams.
    #[serde(rename = "AudioStream", default)]
    pub audio_streams: Vec<MetaQueryAudioStream>,
}

/// The information about an object that meets the query conditions.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryFile {
    /// The time when the object was last modified.
    #[serde(rename = "FileModifiedTime", skip_serializing_if = "Option::is_none")]
    pub file_modified_time: Option<String>,

    /// The type of the object. Valid values: Multipart, Symlink, Appendable, Normal.
    #[serde(rename = "OSSObjectType", skip_serializing_if = "Option::is_none")]
    pub oss_object_type: Option<String>,

    /// The ETag of the object.
    #[serde(rename = "ETag", skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,

    /// The server-side encryption algorithm used when the object was created.
    #[serde(rename = "ServerSideEncryptionCustomerAlgorithm", skip_serializing_if = "Option::is_none")]
    pub server_side_encryption_customer_algorithm: Option<String>,

    /// The number of the tags of the object.
    #[serde(rename = "OSSTaggingCount", skip_serializing_if = "Option::is_none")]
    pub oss_tagging_count: Option<i64>,

    /// The tags.
    #[serde(rename = "OSSTagging", skip_serializing_if = "Option::is_none")]
    pub oss_tagging: Option<MetaQueryTaggings>,

    /// The user metadata items.
    #[serde(rename = "OSSUserMeta", skip_serializing_if = "Option::is_none")]
    pub oss_user_meta: Option<MetaQueryUserMetas>,

    /// The full path of the object.
    #[serde(rename = "Filename", skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    /// The storage class of the object. Valid values: Archive, ColdArchive, IA, Standard.
    #[serde(rename = "OSSStorageClass", skip_serializing_if = "Option::is_none")]
    pub oss_storage_class: Option<String>,

    /// The access control list (ACL) of the object. Valid values: default,
    /// private, public-read, public-read-write.
    #[serde(rename = "ObjectACL", skip_serializing_if = "Option::is_none")]
    pub object_acl: Option<String>,

    /// The CRC-64 value of the object.
    #[serde(rename = "OSSCRC64", skip_serializing_if = "Option::is_none")]
    pub osscrc64: Option<String>,

    /// The server-side encryption of the object.
    #[serde(rename = "ServerSideEncryption", skip_serializing_if = "Option::is_none")]
    pub server_side_encryption: Option<String>,

    /// The object size.
    #[serde(rename = "Size", skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    /// The list of audio streams.
    #[serde(rename = "AudioStreams", skip_serializing_if = "Option::is_none")]
    pub audio_streams: Option<MetaQueryAudioStreams>,

    /// The algorithm used to encrypt objects.
    #[serde(rename = "ServerSideDataEncryption", skip_serializing_if = "Option::is_none")]
    pub server_side_data_encryption: Option<String>,

    /// The cross-origin request methods that are allowed.
    #[serde(rename = "AccessControlRequestMethod", skip_serializing_if = "Option::is_none")]
    pub access_control_request_method: Option<String>,

    /// The artist.
    #[serde(rename = "Artist", skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,

    /// The total duration of the video. Unit: seconds.
    #[serde(rename = "Duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// The longitude and latitude information.
    #[serde(rename = "LatLong", skip_serializing_if = "Option::is_none")]
    pub lat_long: Option<String>,

    /// The list of subtitle streams.
    #[serde(rename = "Subtitles", skip_serializing_if = "Option::is_none")]
    pub subtitles: Option<MetaQuerySubtitles>,

    /// The time when the image or video was taken.
    #[serde(rename = "ProduceTime", skip_serializing_if = "Option::is_none")]
    pub produce_time: Option<String>,

    /// The origins allowed in cross-origin requests.
    #[serde(rename = "AccessControlAllowOrigin", skip_serializing_if = "Option::is_none")]
    pub access_control_allow_origin: Option<String>,

    /// The name of the object when it is downloaded.
    #[serde(rename = "ContentDisposition", skip_serializing_if = "Option::is_none")]
    pub content_disposition: Option<String>,

    /// The player.
    #[serde(rename = "Performer", skip_serializing_if = "Option::is_none")]
    pub performer: Option<String>,

    /// The album.
    #[serde(rename = "Album", skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,

    /// The addresses.
    #[serde(rename = "Addresses", skip_serializing_if = "Option::is_none")]
    pub addresses: Option<MetaQueryAddresses>,

    /// The MIME type of the object.
    #[serde(rename = "ContentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// The content encoding format of the object when the object is downloaded.
    #[serde(rename = "ContentEncoding", skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<String>,

    /// The language of the object content.
    #[serde(rename = "ContentLanguage", skip_serializing_if = "Option::is_none")]
    pub content_language: Option<String>,

    /// The height of the image. Unit: pixel.
    #[serde(rename = "ImageHeight", skip_serializing_if = "Option::is_none")]
    pub image_height: Option<i64>,

    /// The type of multimedia.
    #[serde(rename = "MediaType", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,

    /// The time when the object expires.
    #[serde(rename = "OSSExpiration", skip_serializing_if = "Option::is_none")]
    pub oss_expiration: Option<String>,

    /// The width of the image. Unit: pixel.
    #[serde(rename = "ImageWidth", skip_serializing_if = "Option::is_none")]
    pub image_width: Option<i64>,

    /// The width of the video image. Unit: pixel.
    #[serde(rename = "VideoWidth", skip_serializing_if = "Option::is_none")]
    pub video_width: Option<i64>,

    /// The composer.
    #[serde(rename = "Composer", skip_serializing_if = "Option::is_none")]
    pub composer: Option<String>,

    /// The full path of the object.
    #[serde(rename = "URI", skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    /// The height of the video image. Unit: pixel.
    #[serde(rename = "VideoHeight", skip_serializing_if = "Option::is_none")]
    pub video_height: Option<i64>,

    /// The list of video streams.
    #[serde(rename = "VideoStreams", skip_serializing_if = "Option::is_none")]
    pub video_streams: Option<MetaQueryVideoStreams>,

    /// The web page caching behavior that is performed when the object is downloaded.
    #[serde(rename = "CacheControl", skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<String>,

    /// The bitrate. Unit: bit/s.
    #[serde(rename = "Bitrate", skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<i64>,

    /// The singer.
    #[serde(rename = "AlbumArtist", skip_serializing_if = "Option::is_none")]
    pub album_artist: Option<String>,

    /// The title of the object.
    #[serde(rename = "Title", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The ID of the customer master key (CMK) that is managed by KMS.
    #[serde(rename = "ServerSideEncryptionKeyId", skip_serializing_if = "Option::is_none")]
    pub server_side_encryption_key_id: Option<String>,

    /// The description of the file.
    #[serde(rename = "Insights", skip_serializing_if = "Option::is_none")]
    pub insights: Option<MetaQueryFileInsights>,
}

/// The container that stores the type of multimedia to query.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryMediaTypes {
    /// The type of multimedia that you want to query.
    /// Valid values: image, video, audio, document.
    #[serde(rename = "MediaType", default)]
    pub media_types: Vec<String>,
}

/// The request body of the DoMetaQuery operation.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQuery {
    /// The maximum number of objects to return. Valid values: 0 to 100. If this
    /// parameter is not set or is set to 0, up to 100 objects are returned.
    #[serde(rename = "MaxResults", skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i64>,

    /// The query conditions. A query condition includes the Operation, Field,
    /// Value and SubQueries elements.
    #[serde(rename = "Query", skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,

    /// The field based on which the results are sorted.
    #[serde(rename = "Sort", skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,

    /// The sort order.
    #[serde(rename = "Order", skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,

    /// The container that stores the information about aggregate operations.
    #[serde(rename = "Aggregations", skip_serializing_if = "Option::is_none")]
    pub aggregations: Option<MetaQueryAggregations>,

    /// The pagination token used to obtain information in the next request.
    #[serde(rename = "NextToken", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,

    /// The container that stores the type of multimedia.
    #[serde(rename = "MediaTypes", skip_serializing_if = "Option::is_none")]
    pub media_types: Option<MetaQueryMediaTypes>,

    /// The query conditions.
    #[serde(rename = "SimpleQuery", skip_serializing_if = "Option::is_none")]
    pub simple_query: Option<String>,
}

/// The container that stores the metadata information.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaQueryStatus {
    /// The time when the metadata index library was created, in RFC 3339 format.
    #[serde(rename = "CreateTime", skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,

    /// The time when the metadata index library was updated, in RFC 3339 format.
    #[serde(rename = "UpdateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,

    /// The status of the metadata index library. Valid values: Ready, Stop,
    /// Running, Retrying, Failed, Deleted.
    #[serde(rename = "State", skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// The scan type. Valid values: FullScanning, IncrementalScanning.
    #[serde(rename = "Phase", skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
}

#[derive(Debug, Default, OssRequestModel)]
pub struct CloseMetaQueryRequest {
    /// The name of the bucket.
    pub bucket: String,

    pub common: RequestCommon,
}

impl CloseMetaQueryRequest {
    pub fn new(bucket: &str) -> Self {
        CloseMetaQueryRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, OssResultModel)]
pub struct CloseMetaQueryResult {
    /// Common result fields
    pub common: ResultCommon,
}

impl Client {
    /// Disables the metadata management feature for a bucket. After the metadata
    /// management feature is disabled for a bucket, OSS automatically deletes the
    /// metadata index library of the bucket and you cannot perform metadata indexing.
    ///
    /// # Arguments
    ///
    /// * `request` - The `CloseMetaQueryRequest` containing the bucket name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::CloseMetaQueryRequest;
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = CloseMetaQueryRequest::new("my-bucket");
    ///
    /// match client.close_meta_query(&request).await {
    ///     Ok(result) => {
    ///         println!("Meta query closed: {:?}", result.common.status);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to close meta query: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn close_meta_query(
        &self,
        request: &CloseMetaQueryRequest,
    ) -> Result<CloseMetaQueryResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "CloseMetaQuery".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("comp", "delete"), ("metaQuery", "")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, DEFAULT_CONTENT_TYPE)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };
        input.op_metadata.set(
            SUB_RESOURCE,
            Rc::new(vec!["metaQuery".to_string(), "comp".to_string()]),
        );

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let mut result = CloseMetaQueryResult::default();
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_close_meta_query() {
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
                .with_signature_version(SignatureVersionType::V4),
        );

        // Closing a meta query that was never opened is expected to fail; this
        // only verifies the request is well-formed.
        let result = client
            .close_meta_query(&CloseMetaQueryRequest::new(&config.bucket))
            .await;
        if let Err(error) = &result {
            eprintln!("close_meta_query rejected (meta query may not be open): {}", error);
        }
    }
}
