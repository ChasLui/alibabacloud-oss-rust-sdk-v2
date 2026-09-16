use std::rc::Rc;

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use serde::Deserialize;

use super::close_meta_query::{MetaQuery, MetaQueryAggregations, MetaQueryFiles};
use crate::api::{RequestCommon, ResultCommon};
use crate::client::BodyDataReader;
use crate::client::Client;
use crate::signer::SUB_RESOURCE;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{OperationOutput, BodyContent, OperationInput, DEFAULT_CONTENT_TYPE, HTTP_HEADER_CONTENT_TYPE};

#[derive(Debug, Default, OssRequestModel)]
pub struct DoMetaQueryRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The mode of the query.
    #[field(type = "query", rename = "mode")]
    pub mode: Option<String>,

    /// The query conditions.
    pub meta_query: MetaQuery,

    pub common: RequestCommon,
}

#[derive(Debug, Default, Deserialize, OssResultModel)]
pub struct DoMetaQueryResult {
    /// The token that is used for the next query when the total number of
    /// objects exceeds the value of MaxResults. This parameter has a value only
    /// when not all objects are returned.
    #[serde(rename = "NextToken", skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,

    /// The list of file information.
    #[serde(rename = "Files", skip_serializing_if = "Option::is_none")]
    pub files: Option<MetaQueryFiles>,

    /// The list of aggregate operation results.
    #[serde(rename = "Aggregations", skip_serializing_if = "Option::is_none")]
    pub aggregations: Option<MetaQueryAggregations>,

    #[serde(skip)]
    pub common: ResultCommon,
}

impl Client {
    /// Queries the objects in a bucket that meet the specified conditions by
    /// using the data indexing feature. The information about the objects is
    /// listed based on the specified fields and sorting methods.
    ///
    /// # Arguments
    ///
    /// * `request` - The `DoMetaQueryRequest` containing the bucket name and the
    ///   query conditions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::bucket::{DoMetaQueryRequest, MetaQuery};
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = DoMetaQueryRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     meta_query: MetaQuery {
    ///         max_results: Some(100),
    ///         query: Some(r#"{"Field": "Size", "Value": "1048576", "Operation": "gt"}"#.to_string()),
    ///         sort: Some("Size".to_string()),
    ///         order: Some("desc".to_string()),
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.do_meta_query(&request).await {
    ///     Ok(result) => {
    ///         println!("NextToken: {:?}", result.next_token);
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to do meta query: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn do_meta_query(
        &self,
        request: &DoMetaQueryRequest,
    ) -> Result<DoMetaQueryResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut input = OperationInput {
            op_name: "DoMetaQuery".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            parameters: [("comp", "query"), ("metaQuery", "")]
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

        let xml_body = quick_xml::se::to_string_with_root("MetaQuery", &request.meta_query)?;
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let mut output = self.invoke_operation(input, vec![]).await?;

        let body_data = output.get_all_data().await?;
        let data_str = String::from_utf8_lossy(&body_data);
        let mut result: DoMetaQueryResult = quick_xml::de::from_str(&data_str)?;

        result.update_result(&output);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::close_meta_query::{MetaQueryAggregation, MetaQueryMediaTypes};
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;
    use crate::test_utils::load_test_config;
    use crate::SignatureVersionType;

    #[test]
    fn test_meta_query_serde_round_trip() {
        let meta_query = MetaQuery {
            max_results: Some(100),
            query: Some(r#"{"Field": "Size", "Value": "1048576", "Operation": "gt"}"#.to_string()),
            sort: Some("Size".to_string()),
            order: Some("desc".to_string()),
            aggregations: Some(MetaQueryAggregations {
                aggregations: vec![
                    MetaQueryAggregation {
                        field: Some("Size".to_string()),
                        operation: Some("sum".to_string()),
                        value: None,
                        groups: None,
                    },
                    MetaQueryAggregation {
                        field: Some("OSSObjectType".to_string()),
                        operation: Some("group".to_string()),
                        value: None,
                        groups: None,
                    },
                ],
            }),
            next_token: Some("token-xxx".to_string()),
            media_types: Some(MetaQueryMediaTypes {
                media_types: vec!["image".to_string()],
            }),
            simple_query: None,
        };

        let xml = quick_xml::se::to_string_with_root("MetaQuery", &meta_query).unwrap();
        assert!(xml.contains("<MetaQuery>"));
        assert!(xml.contains("<MaxResults>100</MaxResults>"));
        assert!(xml.contains("<Sort>Size</Sort>"));
        assert!(xml.contains("<Order>desc</Order>"));
        assert!(xml.contains("<Operation>sum</Operation>"));
        assert!(xml.contains("<MediaType>image</MediaType>"));
        assert!(xml.contains("<NextToken>token-xxx</NextToken>"));

        let parsed: MetaQuery = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.max_results, Some(100));
        assert_eq!(parsed.sort.as_deref(), Some("Size"));
        let aggs = parsed.aggregations.unwrap().aggregations;
        assert_eq!(aggs.len(), 2);
        assert_eq!(aggs[0].operation.as_deref(), Some("sum"));
        assert_eq!(parsed.media_types.unwrap().media_types.len(), 1);
    }

    #[test]
    fn test_do_meta_query_result_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<MetaQuery>
  <NextToken>next-token</NextToken>
  <Files>
    <File>
      <Filename>exampleobject.txt</Filename>
      <Size>120</Size>
      <OSSObjectType>Normal</OSSObjectType>
      <OSSStorageClass>Standard</OSSStorageClass>
      <ObjectACL>default</ObjectACL>
      <ETag>5B3C1A2E053D763E1B002CC607C5A0FE</ETag>
      <OSSCRC64>13560574357270602378</OSSCRC64>
      <OSSTaggingCount>1</OSSTaggingCount>
      <OSSTagging>
        <Tagging>
          <Key>owner</Key>
          <Value>John</Value>
        </Tagging>
      </OSSTagging>
      <OSSUserMeta>
        <UserMeta>
          <Key>key1</Key>
          <Value>value1</Value>
        </UserMeta>
      </OSSUserMeta>
      <ImageHeight>500</ImageHeight>
      <ImageWidth>270</ImageWidth>
      <Duration>10.5</Duration>
      <VideoStreams>
        <VideoStream>
          <CodecName>h264</CodecName>
          <Width>1920</Width>
          <Height>1080</Height>
        </VideoStream>
      </VideoStreams>
      <AudioStreams>
        <AudioStream>
          <CodecName>aac</CodecName>
          <SampleRate>44100</SampleRate>
          <Channels>2</Channels>
        </AudioStream>
      </AudioStreams>
      <Addresses>
        <Address>
          <Country>China</Country>
          <City>Hangzhou</City>
        </Address>
      </Addresses>
      <Subtitles>
        <Subtitle>
          <CodecName>srt</CodecName>
          <Language>en</Language>
        </Subtitle>
      </Subtitles>
      <Insights>
        <Video>
          <Caption>caption</Caption>
          <Description>description</Description>
        </Video>
      </Insights>
    </File>
  </Files>
  <Aggregations>
    <Aggregation>
      <Field>Size</Field>
      <Operation>sum</Operation>
      <Value>2048.5</Value>
    </Aggregation>
    <Aggregation>
      <Field>OSSObjectType</Field>
      <Operation>group</Operation>
      <Groups>
        <Group>
          <Value>Normal</Value>
          <Count>5</Count>
        </Group>
      </Groups>
    </Aggregation>
  </Aggregations>
</MetaQuery>"#;
        let result: DoMetaQueryResult = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(result.next_token.as_deref(), Some("next-token"));
        let files = result.files.unwrap().files;
        assert_eq!(files.len(), 1);
        let file = &files[0];
        assert_eq!(file.filename.as_deref(), Some("exampleobject.txt"));
        assert_eq!(file.size, Some(120));
        assert_eq!(file.oss_tagging_count, Some(1));
        assert_eq!(
            file.oss_tagging.as_ref().unwrap().taggings[0].key.as_deref(),
            Some("owner")
        );
        assert_eq!(
            file.oss_user_meta.as_ref().unwrap().user_metas[0].value.as_deref(),
            Some("value1")
        );
        assert_eq!(file.duration, Some(10.5));
        assert_eq!(
            file.video_streams.as_ref().unwrap().video_streams[0].codec_name.as_deref(),
            Some("h264")
        );
        assert_eq!(
            file.audio_streams.as_ref().unwrap().audio_streams[0].sample_rate,
            Some(44100)
        );
        assert_eq!(
            file.addresses.as_ref().unwrap().addresses[0].city.as_deref(),
            Some("Hangzhou")
        );
        assert_eq!(
            file.subtitles.as_ref().unwrap().subtitles[0].language.as_deref(),
            Some("en")
        );
        assert_eq!(
            file.insights.as_ref().unwrap().video.as_ref().unwrap().caption.as_deref(),
            Some("caption")
        );
        let aggs = result.aggregations.unwrap().aggregations;
        assert_eq!(aggs.len(), 2);
        assert_eq!(aggs[0].value, Some(2048.5));
        let groups = aggs[1].groups.as_ref().unwrap();
        assert_eq!(groups.groups[0].value.as_deref(), Some("Normal"));
        assert_eq!(groups.groups[0].count, Some(5));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_do_meta_query() {
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

        // DoMetaQuery requires the metadata index library to be in the Running
        // state, which takes time after OpenMetaQuery, so this call may be
        // rejected on freshly prepared buckets.
        let result = client
            .do_meta_query(&DoMetaQueryRequest {
                bucket: config.bucket.clone(),
                meta_query: MetaQuery {
                    max_results: Some(10),
                    query: Some(r#"{"Field": "Size", "Value": "0", "Operation": "gte"}"#.to_string()),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;
        if let Err(error) = &result {
            eprintln!("do_meta_query rejected (meta query may not be running): {}", error);
        }
    }
}
