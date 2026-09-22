use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::{Deserialize, Serialize};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::{BodyDataReader, Client};
use crate::utils::{modify_request, update_content_length};
use crate::{OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE};

/// A query condition, as embedded in the `query` query parameter.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SimpleQuery {
    /// The name of the field to query.
    #[serde(rename = "Field", skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,

    /// The value of the field to query.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// The operator of the query condition.
    #[serde(rename = "Operation", skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,

    /// The sub-conditions of the query condition.
    #[serde(rename = "SubQueries", default, skip_serializing_if = "Vec::is_empty")]
    pub sub_queries: Vec<SimpleQuery>,
}

impl SimpleQuery {
    /// Serializes the query condition into the JSON object expected by the
    /// `query` query parameter.
    pub fn to_parameter_value(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// The fields to return, as carried by the `withFields` query parameter.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WithFields {
    /// The names of the fields to return.
    #[serde(rename = "WithField", default)]
    pub with_field: Vec<String>,
}

impl WithFields {
    /// Serializes the fields into the JSON array expected by the `withFields`
    /// query parameter.
    pub fn to_parameter_value(&self) -> String {
        serde_json::to_string(&self.with_field).unwrap_or_default()
    }
}

/// The multimedia types to query, as carried by the `mediaTypes` query
/// parameter.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MediaTypes {
    /// The multimedia types to query. Valid values: image, video, audio,
    /// document.
    #[serde(rename = "MediaType", default)]
    pub media_types: Vec<String>,
}

impl MediaTypes {
    /// Serializes the multimedia types into the JSON array expected by the
    /// `mediaTypes` query parameter.
    pub fn to_parameter_value(&self) -> String {
        serde_json::to_string(&self.media_types).unwrap_or_default()
    }
}

/// An address.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Address {
    /// The country.
    #[serde(rename = "Country", skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// The province.
    #[serde(rename = "Province", skip_serializing_if = "Option::is_none")]
    pub province: Option<String>,

    /// The city.
    #[serde(rename = "City", skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    /// The district.
    #[serde(rename = "District", skip_serializing_if = "Option::is_none")]
    pub district: Option<String>,

    /// The town.
    #[serde(rename = "Town", skip_serializing_if = "Option::is_none")]
    pub town: Option<String>,

    /// The street.
    #[serde(rename = "Street", skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,

    /// The street number.
    #[serde(rename = "StreetNumber", skip_serializing_if = "Option::is_none")]
    pub street_number: Option<String>,

    /// The postal code.
    #[serde(rename = "PostalCode", skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
}

/// The container that stores addresses.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Addresses {
    /// The addresses.
    #[serde(rename = "Address", default)]
    pub addresses: Vec<Address>,
}

/// A 2D point with integer coordinates.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct PointInt64 {
    /// The x-coordinate.
    #[serde(rename = "X", skip_serializing_if = "Option::is_none")]
    pub x: Option<i64>,

    /// The y-coordinate.
    #[serde(rename = "Y", skip_serializing_if = "Option::is_none")]
    pub y: Option<i64>,
}

/// The container that stores the polygon points of a boundary.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Polygon {
    /// The points.
    #[serde(rename = "PointInt64", default)]
    pub points: Vec<PointInt64>,
}

/// A rectangular boundary.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Boundary {
    /// The width.
    #[serde(rename = "Width", skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,

    /// The height.
    #[serde(rename = "Height", skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,

    /// The distance from the left edge.
    #[serde(rename = "Left", skip_serializing_if = "Option::is_none")]
    pub left: Option<i64>,

    /// The distance from the top edge.
    #[serde(rename = "Top", skip_serializing_if = "Option::is_none")]
    pub top: Option<i64>,

    /// The polygon of the boundary.
    #[serde(rename = "Polygon", skip_serializing_if = "Option::is_none")]
    pub polygon: Option<Polygon>,
}

/// The image quality score.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ImageScore {
    /// The overall quality score.
    #[serde(
        rename = "OverallQualityScore",
        skip_serializing_if = "Option::is_none"
    )]
    pub overall_quality_score: Option<f64>,
}

/// An OCR recognition result.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct OCRContent {
    /// The language of the recognized text.
    #[serde(rename = "Language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// The recognized text.
    #[serde(rename = "Contents", skip_serializing_if = "Option::is_none")]
    pub contents: Option<String>,

    /// The confidence of the recognition.
    #[serde(rename = "Confidence", skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,

    /// The boundary of the recognized text.
    #[serde(rename = "Boundary", skip_serializing_if = "Option::is_none")]
    pub boundary: Option<Boundary>,
}

/// The container that stores OCR recognition results.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct OCRContents {
    /// The OCR recognition results.
    #[serde(rename = "OCRContent", default)]
    pub ocr_contents: Vec<OCRContent>,
}

/// An image cropping suggestion.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct CroppingSuggestion {
    /// The aspect ratio of the suggested crop.
    #[serde(rename = "AspectRatio", skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,

    /// The confidence of the suggestion.
    #[serde(rename = "Confidence", skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,

    /// The boundary of the suggested crop.
    #[serde(rename = "Boundary", skip_serializing_if = "Option::is_none")]
    pub boundary: Option<Boundary>,
}

/// The container that stores image cropping suggestions.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct CroppingSuggestions {
    /// The cropping suggestions.
    #[serde(rename = "CroppingSuggestion", default)]
    pub cropping_suggestions: Vec<CroppingSuggestion>,
}

/// A time range clip of a label.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Clip {
    /// The time range of the clip, in milliseconds.
    #[serde(rename = "TimeRange", default)]
    pub time_range: Vec<i64>,

    /// The URI of the clip.
    #[serde(rename = "ClipURI", skip_serializing_if = "Option::is_none")]
    pub clip_uri: Option<String>,
}

/// The container that stores clips.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Clips {
    /// The clips.
    #[serde(rename = "Clip", default)]
    pub clips: Vec<Clip>,
}

/// A label with a confidence score.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Label {
    /// The language of the label.
    #[serde(rename = "Language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// The name of the label.
    #[serde(rename = "LabelName", skip_serializing_if = "Option::is_none")]
    pub label_name: Option<String>,

    /// The level of the label.
    #[serde(rename = "LabelLevel", skip_serializing_if = "Option::is_none")]
    pub label_level: Option<i64>,

    /// The confidence of the label.
    #[serde(rename = "LabelConfidence", skip_serializing_if = "Option::is_none")]
    pub label_confidence: Option<f64>,

    /// The name of the parent label.
    #[serde(rename = "ParentLabelName", skip_serializing_if = "Option::is_none")]
    pub parent_label_name: Option<String>,

    /// The centrality score of the label.
    #[serde(rename = "CentricScore", skip_serializing_if = "Option::is_none")]
    pub centric_score: Option<f64>,

    /// The alias of the label.
    #[serde(rename = "LabelAlias", skip_serializing_if = "Option::is_none")]
    pub label_alias: Option<String>,

    /// The clips in which the label appears.
    #[serde(rename = "Clips", skip_serializing_if = "Option::is_none")]
    pub clips: Option<Clips>,
}

/// The container that stores labels.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Labels {
    /// The labels.
    #[serde(rename = "Label", default)]
    pub labels: Vec<Label>,
}

/// The head pose of a figure.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct HeadPose {
    /// The pitch of the head.
    #[serde(rename = "Pitch", skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f64>,

    /// The yaw of the head.
    #[serde(rename = "Yaw", skip_serializing_if = "Option::is_none")]
    pub yaw: Option<f64>,

    /// The roll of the head.
    #[serde(rename = "Roll", skip_serializing_if = "Option::is_none")]
    pub roll: Option<f64>,
}

/// A figure that is detected in an image or video.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Figure {
    /// The ID of the figure.
    #[serde(rename = "FigureId", skip_serializing_if = "Option::is_none")]
    pub figure_id: Option<String>,

    /// The confidence of the figure.
    #[serde(rename = "FigureConfidence", skip_serializing_if = "Option::is_none")]
    pub figure_confidence: Option<f64>,

    /// The ID of the figure cluster that contains the figure.
    #[serde(rename = "FigureClusterId", skip_serializing_if = "Option::is_none")]
    pub figure_cluster_id: Option<String>,

    /// The confidence of the figure cluster.
    #[serde(
        rename = "FigureClusterConfidence",
        skip_serializing_if = "Option::is_none"
    )]
    pub figure_cluster_confidence: Option<f64>,

    /// The type of the figure.
    #[serde(rename = "FigureType", skip_serializing_if = "Option::is_none")]
    pub figure_type: Option<String>,

    /// The estimated age.
    #[serde(rename = "Age", skip_serializing_if = "Option::is_none")]
    pub age: Option<i64>,

    /// The standard deviation of the estimated age.
    #[serde(rename = "AgeSD", skip_serializing_if = "Option::is_none")]
    pub age_sd: Option<f64>,

    /// The gender.
    #[serde(rename = "Gender", skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,

    /// The confidence of the gender.
    #[serde(rename = "GenderConfidence", skip_serializing_if = "Option::is_none")]
    pub gender_confidence: Option<f64>,

    /// The emotion.
    #[serde(rename = "Emotion", skip_serializing_if = "Option::is_none")]
    pub emotion: Option<String>,

    /// The confidence of the emotion.
    #[serde(rename = "EmotionConfidence", skip_serializing_if = "Option::is_none")]
    pub emotion_confidence: Option<f64>,

    /// The quality of the face.
    #[serde(rename = "FaceQuality", skip_serializing_if = "Option::is_none")]
    pub face_quality: Option<f64>,

    /// The boundary of the figure.
    #[serde(rename = "Boundary", skip_serializing_if = "Option::is_none")]
    pub boundary: Option<Boundary>,

    /// The state of the mouth.
    #[serde(rename = "Mouth", skip_serializing_if = "Option::is_none")]
    pub mouth: Option<String>,

    /// The confidence of the mouth state.
    #[serde(rename = "MouthConfidence", skip_serializing_if = "Option::is_none")]
    pub mouth_confidence: Option<f64>,

    /// The state of the beard.
    #[serde(rename = "Beard", skip_serializing_if = "Option::is_none")]
    pub beard: Option<String>,

    /// The confidence of the beard state.
    #[serde(rename = "BeardConfidence", skip_serializing_if = "Option::is_none")]
    pub beard_confidence: Option<f64>,

    /// The state of the hat.
    #[serde(rename = "Hat", skip_serializing_if = "Option::is_none")]
    pub hat: Option<String>,

    /// The confidence of the hat state.
    #[serde(rename = "HatConfidence", skip_serializing_if = "Option::is_none")]
    pub hat_confidence: Option<f64>,

    /// The state of the mask.
    #[serde(rename = "Mask", skip_serializing_if = "Option::is_none")]
    pub mask: Option<String>,

    /// The confidence of the mask state.
    #[serde(rename = "MaskConfidence", skip_serializing_if = "Option::is_none")]
    pub mask_confidence: Option<f64>,

    /// The state of the glasses.
    #[serde(rename = "Glasses", skip_serializing_if = "Option::is_none")]
    pub glasses: Option<String>,

    /// The confidence of the glasses state.
    #[serde(rename = "GlassesConfidence", skip_serializing_if = "Option::is_none")]
    pub glasses_confidence: Option<f64>,

    /// The sharpness of the figure.
    #[serde(rename = "Sharpness", skip_serializing_if = "Option::is_none")]
    pub sharpness: Option<f64>,

    /// The attractiveness of the figure.
    #[serde(rename = "Attractive", skip_serializing_if = "Option::is_none")]
    pub attractive: Option<f64>,

    /// The head pose of the figure.
    #[serde(rename = "HeadPose", skip_serializing_if = "Option::is_none")]
    pub head_pose: Option<HeadPose>,
}

/// The container that stores figures.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Figures {
    /// The figures.
    #[serde(rename = "Figure", default)]
    pub figures: Vec<Figure>,
}

/// A video stream.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct VideoStream {
    /// The index of the stream.
    #[serde(rename = "Index", skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,

    /// The language used in the stream.
    #[serde(rename = "Language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// The abbreviated name of the codec.
    #[serde(rename = "CodecName", skip_serializing_if = "Option::is_none")]
    pub codec_name: Option<String>,

    /// The full name of the codec.
    #[serde(rename = "CodecLongName", skip_serializing_if = "Option::is_none")]
    pub codec_long_name: Option<String>,

    /// The profile of the codec.
    #[serde(rename = "Profile", skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,

    /// The time base of the codec.
    #[serde(rename = "CodecTimeBase", skip_serializing_if = "Option::is_none")]
    pub codec_time_base: Option<String>,

    /// The tag string of the codec.
    #[serde(rename = "CodecTagString", skip_serializing_if = "Option::is_none")]
    pub codec_tag_string: Option<String>,

    /// The tag of the codec.
    #[serde(rename = "CodecTag", skip_serializing_if = "Option::is_none")]
    pub codec_tag: Option<String>,

    /// The width of the stream. Unit: pixel.
    #[serde(rename = "Width", skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,

    /// The height of the stream. Unit: pixel.
    #[serde(rename = "Height", skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,

    /// The number of B frames.
    #[serde(rename = "HasBFrames", skip_serializing_if = "Option::is_none")]
    pub has_b_frames: Option<i64>,

    /// The sample aspect ratio.
    #[serde(rename = "SampleAspectRatio", skip_serializing_if = "Option::is_none")]
    pub sample_aspect_ratio: Option<String>,

    /// The display aspect ratio.
    #[serde(rename = "DisplayAspectRatio", skip_serializing_if = "Option::is_none")]
    pub display_aspect_ratio: Option<String>,

    /// The pixel format.
    #[serde(rename = "PixelFormat", skip_serializing_if = "Option::is_none")]
    pub pixel_format: Option<String>,

    /// The level of the codec.
    #[serde(rename = "Level", skip_serializing_if = "Option::is_none")]
    pub level: Option<i64>,

    /// The frame rate.
    #[serde(rename = "FrameRate", skip_serializing_if = "Option::is_none")]
    pub frame_rate: Option<String>,

    /// The average frame rate.
    #[serde(rename = "AverageFrameRate", skip_serializing_if = "Option::is_none")]
    pub average_frame_rate: Option<String>,

    /// The time base.
    #[serde(rename = "TimeBase", skip_serializing_if = "Option::is_none")]
    pub time_base: Option<String>,

    /// The start time of the stream in seconds.
    #[serde(rename = "StartTime", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,

    /// The duration of the stream in seconds.
    #[serde(rename = "Duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// The bitrate. Unit: bit/s.
    #[serde(rename = "Bitrate", skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<i64>,

    /// The number of frames.
    #[serde(rename = "FrameCount", skip_serializing_if = "Option::is_none")]
    pub frame_count: Option<i64>,

    /// The rotation of the stream.
    #[serde(rename = "Rotate", skip_serializing_if = "Option::is_none")]
    pub rotate: Option<String>,

    /// The bit depth.
    #[serde(rename = "BitDepth", skip_serializing_if = "Option::is_none")]
    pub bit_depth: Option<i64>,

    /// The color space.
    #[serde(rename = "ColorSpace", skip_serializing_if = "Option::is_none")]
    pub color_space: Option<String>,

    /// The color range.
    #[serde(rename = "ColorRange", skip_serializing_if = "Option::is_none")]
    pub color_range: Option<String>,

    /// The color transfer characteristics.
    #[serde(rename = "ColorTransfer", skip_serializing_if = "Option::is_none")]
    pub color_transfer: Option<String>,

    /// The color primaries.
    #[serde(rename = "ColorPrimaries", skip_serializing_if = "Option::is_none")]
    pub color_primaries: Option<String>,
}

/// The container that stores video streams.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct VideoStreams {
    /// The video streams.
    #[serde(rename = "VideoStream", default)]
    pub video_streams: Vec<VideoStream>,
}

/// An audio stream.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct AudioStream {
    /// The index of the stream.
    #[serde(rename = "Index", skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,

    /// The language used in the stream.
    #[serde(rename = "Language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// The abbreviated name of the codec.
    #[serde(rename = "CodecName", skip_serializing_if = "Option::is_none")]
    pub codec_name: Option<String>,

    /// The full name of the codec.
    #[serde(rename = "CodecLongName", skip_serializing_if = "Option::is_none")]
    pub codec_long_name: Option<String>,

    /// The time base of the codec.
    #[serde(rename = "CodecTimeBase", skip_serializing_if = "Option::is_none")]
    pub codec_time_base: Option<String>,

    /// The tag string of the codec.
    #[serde(rename = "CodecTagString", skip_serializing_if = "Option::is_none")]
    pub codec_tag_string: Option<String>,

    /// The tag of the codec.
    #[serde(rename = "CodecTag", skip_serializing_if = "Option::is_none")]
    pub codec_tag: Option<String>,

    /// The time base.
    #[serde(rename = "TimeBase", skip_serializing_if = "Option::is_none")]
    pub time_base: Option<String>,

    /// The start time of the stream in seconds.
    #[serde(rename = "StartTime", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,

    /// The duration of the stream in seconds.
    #[serde(rename = "Duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// The bitrate. Unit: bit/s.
    #[serde(rename = "Bitrate", skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<i64>,

    /// The number of frames.
    #[serde(rename = "FrameCount", skip_serializing_if = "Option::is_none")]
    pub frame_count: Option<i64>,

    /// The lyrics.
    #[serde(rename = "Lyric", skip_serializing_if = "Option::is_none")]
    pub lyric: Option<String>,

    /// The sample format.
    #[serde(rename = "SampleFormat", skip_serializing_if = "Option::is_none")]
    pub sample_format: Option<String>,

    /// The sampling rate.
    #[serde(rename = "SampleRate", skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<i64>,

    /// The number of sound channels.
    #[serde(rename = "Channels", skip_serializing_if = "Option::is_none")]
    pub channels: Option<i64>,

    /// The channel layout.
    #[serde(rename = "ChannelLayout", skip_serializing_if = "Option::is_none")]
    pub channel_layout: Option<String>,
}

/// The container that stores audio streams.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct AudioStreams {
    /// The audio streams.
    #[serde(rename = "AudioStream", default)]
    pub audio_streams: Vec<AudioStream>,
}

/// A subtitle stream.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct SubtitleStream {
    /// The index of the stream.
    #[serde(rename = "Index", skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,

    /// The language used in the stream.
    #[serde(rename = "Language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// The abbreviated name of the codec.
    #[serde(rename = "CodecName", skip_serializing_if = "Option::is_none")]
    pub codec_name: Option<String>,

    /// The full name of the codec.
    #[serde(rename = "CodecLongName", skip_serializing_if = "Option::is_none")]
    pub codec_long_name: Option<String>,

    /// The tag string of the codec.
    #[serde(rename = "CodecTagString", skip_serializing_if = "Option::is_none")]
    pub codec_tag_string: Option<String>,

    /// The tag of the codec.
    #[serde(rename = "CodecTag", skip_serializing_if = "Option::is_none")]
    pub codec_tag: Option<String>,

    /// The start time of the stream in seconds.
    #[serde(rename = "StartTime", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,

    /// The duration of the stream in seconds.
    #[serde(rename = "Duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// The bitrate. Unit: bit/s.
    #[serde(rename = "Bitrate", skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<i64>,

    /// The content of the subtitle.
    #[serde(rename = "Content", skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// The width. Unit: pixel.
    #[serde(rename = "Width", skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,

    /// The height. Unit: pixel.
    #[serde(rename = "Height", skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,
}

/// The container that stores subtitle streams.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Subtitles {
    /// The subtitle streams.
    #[serde(rename = "Subtitle", default)]
    pub subtitles: Vec<SubtitleStream>,
}

/// An audio cover.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct AudioCover {
    /// The image width. Unit: pixel.
    #[serde(rename = "ImageWidth", skip_serializing_if = "Option::is_none")]
    pub image_width: Option<i64>,

    /// The image height. Unit: pixel.
    #[serde(rename = "ImageHeight", skip_serializing_if = "Option::is_none")]
    pub image_height: Option<i64>,

    /// The EXIF information of the image.
    #[serde(rename = "EXIF", skip_serializing_if = "Option::is_none")]
    pub exif: Option<String>,

    /// The image quality score.
    #[serde(rename = "ImageScore", skip_serializing_if = "Option::is_none")]
    pub image_score: Option<ImageScore>,

    /// The cropping suggestions.
    #[serde(
        rename = "CroppingSuggestions",
        skip_serializing_if = "Option::is_none"
    )]
    pub cropping_suggestions: Option<CroppingSuggestions>,

    /// The OCR recognition results.
    #[serde(rename = "OCRContents", skip_serializing_if = "Option::is_none")]
    pub ocr_contents: Option<OCRContents>,
}

/// The container that stores audio covers.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct AudioCovers {
    /// The audio covers.
    #[serde(rename = "AudioCover", default)]
    pub audio_covers: Vec<AudioCover>,
}

/// An element content.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ElementContent {
    /// The type of the content.
    #[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
    pub element_type: Option<String>,

    /// The URI of the content.
    #[serde(rename = "URI", skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    /// The value of the content.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// The container that stores element contents.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ElementContents {
    /// The element contents.
    #[serde(rename = "ElementContent", default)]
    pub element_contents: Vec<ElementContent>,
}

/// A relation between elements.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ElementRelation {
    /// The ID of the related object.
    #[serde(rename = "ObjectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,

    /// The type of the relation.
    #[serde(rename = "RelationType", skip_serializing_if = "Option::is_none")]
    pub relation_type: Option<String>,
}

/// The container that stores element relations.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ElementRelations {
    /// The element relations.
    #[serde(rename = "ElementRelation", default)]
    pub element_relations: Vec<ElementRelation>,
}

/// An element that is detected in a media file.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Element {
    /// The contents of the element.
    #[serde(rename = "ElementContents", skip_serializing_if = "Option::is_none")]
    pub element_contents: Option<ElementContents>,

    /// The ID of the object that contains the element.
    #[serde(rename = "ObjectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,

    /// The type of the element.
    #[serde(rename = "ElementType", skip_serializing_if = "Option::is_none")]
    pub element_type: Option<String>,

    /// The semantic similarity of the element.
    #[serde(rename = "SemanticSimilarity", skip_serializing_if = "Option::is_none")]
    pub semantic_similarity: Option<f64>,

    /// The relations of the element.
    #[serde(rename = "ElementRelations", skip_serializing_if = "Option::is_none")]
    pub element_relations: Option<ElementRelations>,
}

/// The container that stores elements.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Elements {
    /// The elements.
    #[serde(rename = "Element", default)]
    pub elements: Vec<Element>,
}

/// An element in a scene.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct SceneElement {
    /// The time range of the element, in milliseconds.
    #[serde(rename = "TimeRange", default)]
    pub time_range: Vec<i64>,

    /// The frame times of the element, in milliseconds.
    #[serde(rename = "FrameTimes", default)]
    pub frame_times: Vec<i64>,

    /// The index of the video stream.
    #[serde(rename = "VideoStreamIndex", skip_serializing_if = "Option::is_none")]
    pub video_stream_index: Option<i64>,

    /// The labels of the element.
    #[serde(rename = "Labels", skip_serializing_if = "Option::is_none")]
    pub labels: Option<Labels>,
}

/// The container that stores scene elements.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct SceneElements {
    /// The scene elements.
    #[serde(rename = "SceneElement", default)]
    pub scene_elements: Vec<SceneElement>,
}

/// The semantic types of a file.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct SemanticTypes {
    /// The semantic types.
    #[serde(rename = "SemanticType", default)]
    pub semantic_types: Vec<String>,
}

/// A tag of an object.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Tagging {
    /// The tag key.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The tag value.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// The container that stores the tags of an object.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Taggings {
    /// The tags.
    #[serde(rename = "Tagging", default)]
    pub taggings: Vec<Tagging>,
}

/// A user metadata item of an object.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct UserMeta {
    /// The key of the user metadata item.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The value of the user metadata item.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// The container that stores user metadata items.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct UserMetas {
    /// The user metadata items.
    #[serde(rename = "UserMeta", default)]
    pub user_metas: Vec<UserMeta>,
}

/// A custom label of an object.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct CustomLabel {
    /// The key of the custom label.
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The value of the custom label.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// The container that stores custom labels.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct CustomLabels {
    /// The custom labels.
    #[serde(rename = "Item", default)]
    pub items: Vec<CustomLabel>,
}

/// The description of an image file.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ImageInsight {
    /// A brief description.
    #[serde(rename = "Caption", skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,

    /// A detailed description.
    #[serde(rename = "Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// The description of a video file.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct VideoInsight {
    /// A brief description.
    #[serde(rename = "Caption", skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,

    /// A detailed description.
    #[serde(rename = "Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// The insights of a file.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Insights {
    /// The description of the video file.
    #[serde(rename = "Video", skip_serializing_if = "Option::is_none")]
    pub video: Option<VideoInsight>,

    /// The description of the image file.
    #[serde(rename = "Image", skip_serializing_if = "Option::is_none")]
    pub image: Option<ImageInsight>,
}

/// The information about an object that meets the query conditions.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct File {
    /// The ID of the object owner.
    #[serde(rename = "OwnerId", skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,

    /// The name of the dataset.
    #[serde(rename = "DatasetName", skip_serializing_if = "Option::is_none")]
    pub dataset_name: Option<String>,

    /// The type of the object.
    #[serde(rename = "ObjectType", skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,

    /// The ID of the object.
    #[serde(rename = "ObjectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,

    /// The time when the object was last updated.
    #[serde(rename = "UpdateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,

    /// The time when the object was created.
    #[serde(rename = "CreateTime", skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,

    /// The full path of the object.
    #[serde(rename = "URI", skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    /// The URI of the object in OSS.
    #[serde(rename = "OSSURI", skip_serializing_if = "Option::is_none")]
    pub oss_uri: Option<String>,

    /// The name of the object.
    #[serde(rename = "Filename", skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    /// The type of multimedia.
    #[serde(rename = "MediaType", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,

    /// The MIME type of the object.
    #[serde(rename = "ContentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// The object size.
    #[serde(rename = "Size", skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    /// The hash of the object.
    #[serde(rename = "FileHash", skip_serializing_if = "Option::is_none")]
    pub file_hash: Option<String>,

    /// The time when the object was last modified.
    #[serde(rename = "FileModifiedTime", skip_serializing_if = "Option::is_none")]
    pub file_modified_time: Option<String>,

    /// The time when the object was created.
    #[serde(rename = "FileCreateTime", skip_serializing_if = "Option::is_none")]
    pub file_create_time: Option<String>,

    /// The time when the object was last accessed.
    #[serde(rename = "FileAccessTime", skip_serializing_if = "Option::is_none")]
    pub file_access_time: Option<String>,

    /// The time when the object was produced.
    #[serde(rename = "ProduceTime", skip_serializing_if = "Option::is_none")]
    pub produce_time: Option<String>,

    /// The longitude and latitude information.
    #[serde(rename = "LatLong", skip_serializing_if = "Option::is_none")]
    pub lat_long: Option<String>,

    /// The time zone.
    #[serde(rename = "Timezone", skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,

    /// The addresses.
    #[serde(rename = "Addresses", skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Addresses>,

    /// The ID of the travel cluster.
    #[serde(rename = "TravelClusterId", skip_serializing_if = "Option::is_none")]
    pub travel_cluster_id: Option<String>,

    /// The orientation of the image.
    #[serde(rename = "Orientation", skip_serializing_if = "Option::is_none")]
    pub orientation: Option<i64>,

    /// The figures.
    #[serde(rename = "Figures", skip_serializing_if = "Option::is_none")]
    pub figures: Option<Figures>,

    /// The number of figures.
    #[serde(rename = "FigureCount", skip_serializing_if = "Option::is_none")]
    pub figure_count: Option<i64>,

    /// The labels.
    #[serde(rename = "Labels", skip_serializing_if = "Option::is_none")]
    pub labels: Option<Labels>,

    /// The title of the object.
    #[serde(rename = "Title", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The image width. Unit: pixel.
    #[serde(rename = "ImageWidth", skip_serializing_if = "Option::is_none")]
    pub image_width: Option<i64>,

    /// The image height. Unit: pixel.
    #[serde(rename = "ImageHeight", skip_serializing_if = "Option::is_none")]
    pub image_height: Option<i64>,

    /// The EXIF information of the image.
    #[serde(rename = "EXIF", skip_serializing_if = "Option::is_none")]
    pub exif: Option<String>,

    /// The image quality score.
    #[serde(rename = "ImageScore", skip_serializing_if = "Option::is_none")]
    pub image_score: Option<ImageScore>,

    /// The cropping suggestions.
    #[serde(
        rename = "CroppingSuggestions",
        skip_serializing_if = "Option::is_none"
    )]
    pub cropping_suggestions: Option<CroppingSuggestions>,

    /// The OCR recognition results.
    #[serde(rename = "OCRContents", skip_serializing_if = "Option::is_none")]
    pub ocr_contents: Option<OCRContents>,

    /// The video width. Unit: pixel.
    #[serde(rename = "VideoWidth", skip_serializing_if = "Option::is_none")]
    pub video_width: Option<i64>,

    /// The video height. Unit: pixel.
    #[serde(rename = "VideoHeight", skip_serializing_if = "Option::is_none")]
    pub video_height: Option<i64>,

    /// The video streams.
    #[serde(rename = "VideoStreams", skip_serializing_if = "Option::is_none")]
    pub video_streams: Option<VideoStreams>,

    /// The subtitle streams.
    #[serde(rename = "Subtitles", skip_serializing_if = "Option::is_none")]
    pub subtitles: Option<Subtitles>,

    /// The audio streams.
    #[serde(rename = "AudioStreams", skip_serializing_if = "Option::is_none")]
    pub audio_streams: Option<AudioStreams>,

    /// The artist.
    #[serde(rename = "Artist", skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,

    /// The album artist.
    #[serde(rename = "AlbumArtist", skip_serializing_if = "Option::is_none")]
    pub album_artist: Option<String>,

    /// The audio covers.
    #[serde(rename = "AudioCovers", skip_serializing_if = "Option::is_none")]
    pub audio_covers: Option<AudioCovers>,

    /// The composer.
    #[serde(rename = "Composer", skip_serializing_if = "Option::is_none")]
    pub composer: Option<String>,

    /// The performer.
    #[serde(rename = "Performer", skip_serializing_if = "Option::is_none")]
    pub performer: Option<String>,

    /// The language of the object content.
    #[serde(rename = "Language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// The album.
    #[serde(rename = "Album", skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,

    /// The number of pages of the document.
    #[serde(rename = "PageCount", skip_serializing_if = "Option::is_none")]
    pub page_count: Option<i64>,

    /// The ETag of the object.
    #[serde(rename = "ETag", skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,

    /// The web page caching behavior performed when the object is downloaded.
    #[serde(rename = "CacheControl", skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<String>,

    /// The name of the object when it is downloaded.
    #[serde(rename = "ContentDisposition", skip_serializing_if = "Option::is_none")]
    pub content_disposition: Option<String>,

    /// The content encoding format of the object when the object is downloaded.
    #[serde(rename = "ContentEncoding", skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<String>,

    /// The language of the object content.
    #[serde(rename = "ContentLanguage", skip_serializing_if = "Option::is_none")]
    pub content_language: Option<String>,

    /// The origins allowed in cross-origin requests.
    #[serde(
        rename = "AccessControlAllowOrigin",
        skip_serializing_if = "Option::is_none"
    )]
    pub access_control_allow_origin: Option<String>,

    /// The cross-origin request methods that are allowed.
    #[serde(
        rename = "AccessControlRequestMethod",
        skip_serializing_if = "Option::is_none"
    )]
    pub access_control_request_method: Option<String>,

    /// The server-side encryption algorithm used when the object was created.
    #[serde(
        rename = "ServerSideEncryptionCustomerAlgorithm",
        skip_serializing_if = "Option::is_none"
    )]
    pub server_side_encryption_customer_algorithm: Option<String>,

    /// The server-side encryption of the object.
    #[serde(
        rename = "ServerSideEncryption",
        skip_serializing_if = "Option::is_none"
    )]
    pub server_side_encryption: Option<String>,

    /// The algorithm used to encrypt objects.
    #[serde(
        rename = "ServerSideDataEncryption",
        skip_serializing_if = "Option::is_none"
    )]
    pub server_side_data_encryption: Option<String>,

    /// The ID of the customer master key (CMK) that is managed by KMS.
    #[serde(
        rename = "ServerSideEncryptionKeyId",
        skip_serializing_if = "Option::is_none"
    )]
    pub server_side_encryption_key_id: Option<String>,

    /// The storage class of the object.
    #[serde(rename = "OSSStorageClass", skip_serializing_if = "Option::is_none")]
    pub oss_storage_class: Option<String>,

    /// The CRC-64 value of the object.
    #[serde(rename = "OSSCRC64", skip_serializing_if = "Option::is_none")]
    pub osscrc64: Option<String>,

    /// The access control list (ACL) of the object.
    #[serde(rename = "ObjectACL", skip_serializing_if = "Option::is_none")]
    pub object_acl: Option<String>,

    /// The Content-MD5 value of the object.
    #[serde(rename = "ContentMd5", skip_serializing_if = "Option::is_none")]
    pub content_md5: Option<String>,

    /// The sequence number of the object.
    #[serde(rename = "SequenceNumber", skip_serializing_if = "Option::is_none")]
    pub sequence_number: Option<i64>,

    /// The semantic similarity of the object.
    #[serde(rename = "SemanticSimilarity", skip_serializing_if = "Option::is_none")]
    pub semantic_similarity: Option<f64>,

    /// The user metadata items.
    #[serde(rename = "OSSUserMeta", skip_serializing_if = "Option::is_none")]
    pub oss_user_meta: Option<UserMetas>,

    /// The number of tags of the object.
    #[serde(rename = "OSSTaggingCount", skip_serializing_if = "Option::is_none")]
    pub oss_tagging_count: Option<i64>,

    /// The tags of the object.
    #[serde(rename = "OSSTagging", skip_serializing_if = "Option::is_none")]
    pub oss_tagging: Option<Taggings>,

    /// The time when the object expires.
    #[serde(rename = "OSSExpiration", skip_serializing_if = "Option::is_none")]
    pub oss_expiration: Option<String>,

    /// The version ID of the object.
    #[serde(rename = "OSSVersionId", skip_serializing_if = "Option::is_none")]
    pub oss_version_id: Option<String>,

    /// The delete marker of the object.
    #[serde(rename = "OSSDeleteMarker", skip_serializing_if = "Option::is_none")]
    pub oss_delete_marker: Option<String>,

    /// The type of the object.
    #[serde(rename = "OSSObjectType", skip_serializing_if = "Option::is_none")]
    pub oss_object_type: Option<String>,

    /// The custom ID of the object.
    #[serde(rename = "CustomId", skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<String>,

    /// The custom labels of the object.
    #[serde(rename = "CustomLabels", skip_serializing_if = "Option::is_none")]
    pub custom_labels: Option<CustomLabels>,

    /// The number of streams.
    #[serde(rename = "StreamCount", skip_serializing_if = "Option::is_none")]
    pub stream_count: Option<i64>,

    /// The number of programs.
    #[serde(rename = "ProgramCount", skip_serializing_if = "Option::is_none")]
    pub program_count: Option<i64>,

    /// The name of the format.
    #[serde(rename = "FormatName", skip_serializing_if = "Option::is_none")]
    pub format_name: Option<String>,

    /// The long name of the format.
    #[serde(rename = "FormatLongName", skip_serializing_if = "Option::is_none")]
    pub format_long_name: Option<String>,

    /// The start time of the media file in seconds.
    #[serde(rename = "StartTime", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,

    /// The bitrate. Unit: bit/s.
    #[serde(rename = "Bitrate", skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<i64>,

    /// The total duration of the media file in seconds.
    #[serde(rename = "Duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// The semantic types of the object.
    #[serde(rename = "SemanticTypes", skip_serializing_if = "Option::is_none")]
    pub semantic_types: Option<SemanticTypes>,

    /// The elements.
    #[serde(rename = "Elements", skip_serializing_if = "Option::is_none")]
    pub elements: Option<Elements>,

    /// The scene elements.
    #[serde(rename = "SceneElements", skip_serializing_if = "Option::is_none")]
    pub scene_elements: Option<SceneElements>,

    /// The recognized text of the object.
    #[serde(rename = "OCRTexts", skip_serializing_if = "Option::is_none")]
    pub ocr_texts: Option<String>,

    /// The reason why the object is in its current state.
    #[serde(rename = "Reason", skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// The status of the object.
    #[serde(rename = "ObjectStatus", skip_serializing_if = "Option::is_none")]
    pub object_status: Option<String>,

    /// The insights of the object.
    #[serde(rename = "Insights", skip_serializing_if = "Option::is_none")]
    pub insights: Option<Insights>,
}

/// The container that stores the objects that meet the query conditions.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Files {
    /// The objects.
    #[serde(rename = "File", default)]
    pub files: Vec<File>,
}

/// A group of aggregation results.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct AggregationGroup {
    /// The value of the group.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// The number of results in the group.
    #[serde(rename = "Count", skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
}

/// The container that stores the groups of an aggregation.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct AggregationGroups {
    /// The groups.
    #[serde(rename = "Group", default)]
    pub groups: Vec<AggregationGroup>,
}

/// An aggregate operation.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Aggregation {
    /// The operator for the aggregate operation. Valid values: min, max,
    /// average, sum, count, distinct, group.
    #[serde(rename = "Operation", skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,

    /// The field name.
    #[serde(rename = "Field", skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,

    /// The result of the aggregate operation. Only returned in responses.
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// The grouped results. Only returned in responses.
    #[serde(rename = "Groups", default, skip_serializing)]
    pub groups: Option<AggregationGroups>,
}

/// The container that stores aggregate operations.
///
/// The metadata index library nests the operations under an `Aggregation`
/// element while the `aggregations` query parameter carries them as a plain
/// JSON array.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Aggregations {
    /// The aggregate operations.
    #[serde(rename = "Aggregation", default)]
    pub aggregations: Vec<Aggregation>,
}

impl Aggregations {
    /// Serializes the aggregate operations into the JSON array expected by the
    /// `aggregations` query parameter.
    pub fn to_parameter_value(&self) -> String {
        serde_json::to_string(&self.aggregations).unwrap_or_default()
    }
}

#[derive(Debug, Default, OssRequestModel)]
pub struct SimpleQueryRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the dataset.
    #[field(type = "query", rename = "datasetName")]
    pub dataset_name: String,

    /// The token that is used to retrieve the next page of results.
    #[field(type = "query", rename = "nextToken")]
    pub next_token: Option<String>,

    /// The maximum number of objects to return.
    #[field(type = "query", rename = "maxResults")]
    pub max_results: Option<i32>,

    /// The query conditions, as produced by
    /// [`SimpleQuery::to_parameter_value`].
    #[field(type = "query", rename = "query")]
    pub query: Option<String>,

    /// The field based on which the results are sorted.
    #[field(type = "query", rename = "sort")]
    pub sort: Option<String>,

    /// The sort order. Valid values: asc, desc.
    #[field(type = "query", rename = "order")]
    pub order: Option<String>,

    /// The aggregate operations, as produced by
    /// [`Aggregations::to_parameter_value`].
    #[field(type = "query", rename = "aggregations")]
    pub aggregations: Option<String>,

    /// The fields to return, as produced by [`WithFields::to_parameter_value`].
    #[field(type = "query", rename = "withFields")]
    pub with_fields: Option<String>,

    /// Specifies whether to return the total number of hits.
    #[field(type = "query", rename = "withoutTotalHits")]
    pub without_total_hits: Option<bool>,

    pub common: RequestCommon,
}

impl SimpleQueryRequest {
    /// Creates a request that queries the files of a dataset.
    pub fn new(bucket: &str, dataset_name: &str) -> Self {
        SimpleQueryRequest {
            bucket: bucket.to_string(),
            dataset_name: dataset_name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct SimpleQueryResult {
    /// The objects that meet the query conditions.
    #[serde(rename = "Files", skip_serializing_if = "Option::is_none")]
    pub files: Option<Files>,

    /// The token that is used to retrieve the next page of results.
    #[serde(rename = "NextToken", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,

    /// The maximum number of objects returned.
    #[serde(rename = "MaxResults", skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,

    /// The total number of objects that meet the query conditions.
    #[serde(rename = "TotalHits", skip_serializing_if = "Option::is_none")]
    pub total_hits: Option<i64>,

    /// The aggregate results.
    #[serde(rename = "Aggregations", skip_serializing_if = "Option::is_none")]
    pub aggregations: Option<Aggregations>,

    /// Common result fields
    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the files of a dataset by using the simple query feature.
    ///
    /// # Arguments
    ///
    /// * `request` - The `SimpleQueryRequest` containing the bucket and dataset
    ///   names and the query conditions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::dataprocess::{SimpleQuery, SimpleQueryRequest};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let query = SimpleQuery {
    ///     field: Some("Size".to_string()),
    ///     value: Some("1".to_string()),
    ///     operation: Some("gt".to_string()),
    ///     ..Default::default()
    /// };
    /// let request = SimpleQueryRequest {
    ///     query: Some(query.to_parameter_value()),
    ///     sort: Some("Size".to_string()),
    ///     ..SimpleQueryRequest::new("my-bucket", "my-dataset")
    /// };
    ///
    /// match client.simple_query(&request).await {
    ///     Ok(result) => println!("files: {:?}", result.files),
    ///     Err(error) => eprintln!("Failed to run simple query: {}", error),
    /// }
    /// # })
    /// ```
    pub async fn simple_query(
        &self,
        request: &SimpleQueryRequest,
    ) -> Result<SimpleQueryResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "SimpleQuery".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("metaQuery", ""), ("action", "simpleQuery")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_length],
        )?;

        let mut output: OperationOutput = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: SimpleQueryResult = quick_xml::de::from_str(&data_str)?;
        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_simple_query_to_parameter_value() {
        let query = SimpleQuery {
            field: Some("Size".to_string()),
            value: Some("1".to_string()),
            operation: Some("gt".to_string()),
            ..Default::default()
        };
        assert_eq!(
            query.to_parameter_value(),
            r#"{"Field":"Size","Value":"1","Operation":"gt"}"#
        );
    }

    #[test]
    fn test_simple_query_sub_queries_to_parameter_value() {
        let query = SimpleQuery {
            operation: Some("and".to_string()),
            sub_queries: vec![SimpleQuery {
                field: Some("MediaType".to_string()),
                value: Some("image".to_string()),
                operation: Some("eq".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(
            query.to_parameter_value(),
            r#"{"Operation":"and","SubQueries":[{"Field":"MediaType","Value":"image","Operation":"eq"}]}"#
        );
    }

    #[test]
    fn test_with_fields_to_parameter_value() {
        let with_fields = WithFields {
            with_field: vec!["Filename".to_string(), "Size".to_string()],
        };
        assert_eq!(with_fields.to_parameter_value(), r#"["Filename","Size"]"#);
    }

    #[test]
    fn test_media_types_to_parameter_value() {
        let media_types = MediaTypes {
            media_types: vec!["video".to_string(), "image".to_string()],
        };
        assert_eq!(media_types.to_parameter_value(), r#"["video","image"]"#);
    }

    #[test]
    fn test_aggregations_to_parameter_value_excludes_groups() {
        let aggregations = Aggregations {
            aggregations: vec![Aggregation {
                operation: Some("group".to_string()),
                field: Some("MediaType".to_string()),
                groups: Some(AggregationGroups {
                    groups: vec![AggregationGroup {
                        value: Some("document".to_string()),
                        count: Some(80),
                    }],
                }),
                ..Default::default()
            }],
        };
        assert_eq!(
            aggregations.to_parameter_value(),
            r#"[{"Operation":"group","Field":"MediaType"}]"#
        );
    }

    #[test]
    fn test_simple_query_request_query_map() {
        let request = SimpleQueryRequest {
            next_token: Some("next-token".to_string()),
            max_results: Some(99),
            query: Some(r#"{"Field":"Size","Value":"1","Operation":"gt"}"#.to_string()),
            sort: Some("Size".to_string()),
            order: Some("acs".to_string()),
            without_total_hits: Some(true),
            ..SimpleQueryRequest::new("bucket", "your_dataset")
        };
        let query = request.query_map();
        assert_eq!(
            query.get("datasetName").map(String::as_str),
            Some("your_dataset")
        );
        assert_eq!(query.get("maxResults").map(String::as_str), Some("99"));
        assert_eq!(query.get("sort").map(String::as_str), Some("Size"));
        assert_eq!(query.get("order").map(String::as_str), Some("acs"));
        assert_eq!(
            query.get("withoutTotalHits").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            query.get("query").map(String::as_str),
            Some(r#"{"Field":"Size","Value":"1","Operation":"gt"}"#)
        );
    }

    #[test]
    fn test_simple_query_result_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<MetaQuery>
  <NextToken>MTIzNDU2Nzg5MDEyMzQ1Njc4OTAx****</NextToken>
  <TotalHits>150</TotalHits>
  <Files>
    <File>
      <Filename>docs/report.pdf</Filename>
      <Size>5242880</Size>
      <URI>oss://examplebucket/docs/report.pdf</URI>
      <OSSURI>oss://examplebucket/docs/report.pdf</OSSURI>
      <MediaType>document</MediaType>
      <ContentType>application/pdf</ContentType>
      <FileModifiedTime>2025-12-01T10:30:00Z</FileModifiedTime>
      <PageCount>20</PageCount>
    </File>
  </Files>
  <Aggregations>
    <Aggregation>
      <Field>MediaType</Field>
      <Operation>group</Operation>
      <Groups>
        <Group>
          <Value>document</Value>
          <Count>80</Count>
        </Group>
        <Group>
          <Value>image</Value>
          <Count>70</Count>
        </Group>
      </Groups>
    </Aggregation>
  </Aggregations>
</MetaQuery>"#;
        let result: SimpleQueryResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(
            result.next_token.as_deref(),
            Some("MTIzNDU2Nzg5MDEyMzQ1Njc4OTAx****")
        );
        assert_eq!(result.total_hits, Some(150));

        let files = result.files.unwrap().files;
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename.as_deref(), Some("docs/report.pdf"));
        assert_eq!(files[0].size, Some(5242880));
        assert_eq!(
            files[0].uri.as_deref(),
            Some("oss://examplebucket/docs/report.pdf")
        );
        assert_eq!(
            files[0].oss_uri.as_deref(),
            Some("oss://examplebucket/docs/report.pdf")
        );
        assert_eq!(files[0].media_type.as_deref(), Some("document"));
        assert_eq!(files[0].content_type.as_deref(), Some("application/pdf"));
        assert_eq!(files[0].page_count, Some(20));

        let aggregations = result.aggregations.unwrap().aggregations;
        assert_eq!(aggregations.len(), 1);
        assert_eq!(aggregations[0].field.as_deref(), Some("MediaType"));
        assert_eq!(aggregations[0].operation.as_deref(), Some("group"));
        let groups = aggregations[0].groups.as_ref().unwrap().groups.clone();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].value.as_deref(), Some("document"));
        assert_eq!(groups[0].count, Some(80));
        assert_eq!(groups[1].value.as_deref(), Some("image"));
        assert_eq!(groups[1].count, Some(70));
    }

    #[test]
    fn test_simple_query_result_deserialize_media_metadata() {
        let xml = r#"<MetaQuery>
  <TotalHits>258</TotalHits>
  <Files>
    <File>
      <Filename>photos/sunset.jpg</Filename>
      <Size>2048000</Size>
      <MediaType>image</MediaType>
      <ImageWidth>3840</ImageWidth>
      <ImageHeight>2160</ImageHeight>
      <Orientation>1</Orientation>
      <Figures>
        <Figure>
          <FigureId>figure-1</FigureId>
          <FigureConfidence>0.98</FigureConfidence>
          <Boundary><Width>100</Width><Height>200</Height><Left>10</Left><Top>20</Top></Boundary>
        </Figure>
      </Figures>
      <Labels>
        <Label>
          <LabelName>sky</LabelName>
          <LabelConfidence>0.9</LabelConfidence>
        </Label>
      </Labels>
      <OSSTagging>
        <Tagging><Key>k</Key><Value>v</Value></Tagging>
      </OSSTagging>
      <VideoStreams>
        <VideoStream><Index>0</Index><CodecName>h264</CodecName><Bitrate>1000</Bitrate></VideoStream>
      </VideoStreams>
      <SemanticTypes><SemanticType>nature</SemanticType></SemanticTypes>
      <Insights><Image><Caption>a sunset</Caption></Image></Insights>
    </File>
  </Files>
</MetaQuery>"#;
        let result: SimpleQueryResult = quick_xml::de::from_str(xml).unwrap();
        let files = result.files.unwrap().files;
        let file = &files[0];
        assert_eq!(file.image_width, Some(3840));
        assert_eq!(file.orientation, Some(1));

        let figures = file.figures.as_ref().unwrap().figures.clone();
        assert_eq!(figures.len(), 1);
        assert_eq!(figures[0].figure_id.as_deref(), Some("figure-1"));
        assert_eq!(figures[0].figure_confidence, Some(0.98));
        assert_eq!(figures[0].boundary.as_ref().unwrap().width, Some(100));

        let labels = file.labels.as_ref().unwrap().labels.clone();
        assert_eq!(labels[0].label_name.as_deref(), Some("sky"));

        let taggings = file.oss_tagging.as_ref().unwrap().taggings.clone();
        assert_eq!(taggings[0].key.as_deref(), Some("k"));
        assert_eq!(taggings[0].value.as_deref(), Some("v"));

        let streams = file.video_streams.as_ref().unwrap().video_streams.clone();
        assert_eq!(streams[0].codec_name.as_deref(), Some("h264"));

        assert_eq!(
            file.semantic_types.as_ref().unwrap().semantic_types,
            vec!["nature".to_string()]
        );
        assert_eq!(
            file.insights
                .as_ref()
                .unwrap()
                .image
                .as_ref()
                .unwrap()
                .caption
                .as_deref(),
            Some("a sunset")
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_simple_query() {
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

        let result = client
            .simple_query(&SimpleQueryRequest::new(&config.bucket, "sdk-test-dataset"))
            .await;
        if let Err(error) = &result {
            eprintln!("simple_query rejected: {}", error);
        }
    }
}
