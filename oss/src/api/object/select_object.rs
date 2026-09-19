use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use alibabacloud_oss_sdk_rust_v2_api_model::{OssRequestModel, OssResultModel};
use base64::Engine;
use serde::Serialize;
use tokio::io::{AsyncRead, ReadBuf};

use crate::api::{RequestCommon, ResultCommon};
use crate::client::Client;
use crate::utils::{modify_request, update_content_length, update_content_md5};
use crate::{
    BodyContent, BodyStream, OperationInput, OperationOutput, HTTP_HEADER_CONTENT_TYPE,
};

// Frame types of the select object response framing protocol.
const DATA_FRAME_TYPE: i32 = 8388609;
const CONTINUOUS_FRAME_TYPE: i32 = 8388612;
const END_FRAME_TYPE: i32 = 8388613;
const META_END_FRAME_CSV_TYPE: i32 = 8388614;
const META_END_FRAME_JSON_TYPE: i32 = 8388615;

// CRC-32/ISO-HDLC (same as crc32.NewIEEE() in Go) for payload checksums.
static SELECT_CRC32: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

/// The request body of the SelectObject operation.
#[derive(Debug, Default, Clone, Serialize)]
pub struct SelectRequest {
    /// The SQL statement, base64 encoded before being sent.
    #[serde(rename = "Expression", skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,

    /// The format of the input object data.
    #[serde(rename = "InputSerialization")]
    pub input_serialization: InputSerializationSelect,

    /// The format of the output data.
    #[serde(rename = "OutputSerialization")]
    pub output_serialization: OutputSerializationSelect,

    /// The options of the select operation.
    #[serde(rename = "Options", skip_serializing_if = "Option::is_none")]
    pub options: Option<SelectOptions>,
}

/// The format of the input object data.
#[derive(Debug, Default, Clone, Serialize)]
pub struct InputSerializationSelect {
    /// The CSV input format.
    #[serde(rename = "CSV", skip_serializing_if = "Option::is_none")]
    pub csv: Option<CsvSelectInput>,

    /// The JSON input format.
    #[serde(rename = "JSON", skip_serializing_if = "Option::is_none")]
    pub json: Option<JsonSelectInput>,

    /// The compression type of the object. Valid values: None and GZIP.
    #[serde(rename = "CompressionType", skip_serializing_if = "Option::is_none")]
    pub compression_type: Option<String>,
}

/// The CSV input format.
#[derive(Debug, Default, Clone, Serialize)]
pub struct CsvSelectInput {
    /// The header information of the CSV object. Valid values: USE, IGNORE and NONE.
    #[serde(rename = "FileHeaderInfo", skip_serializing_if = "Option::is_none")]
    pub file_header_info: Option<String>,

    /// The record delimiter, base64 encoded before being sent.
    #[serde(rename = "RecordDelimiter", skip_serializing_if = "Option::is_none")]
    pub record_delimiter: Option<String>,

    /// The field delimiter, base64 encoded before being sent.
    #[serde(rename = "FieldDelimiter", skip_serializing_if = "Option::is_none")]
    pub field_delimiter: Option<String>,

    /// The quote character, base64 encoded before being sent.
    #[serde(rename = "QuoteCharacter", skip_serializing_if = "Option::is_none")]
    pub quote_character: Option<String>,

    /// The comment character, base64 encoded before being sent.
    #[serde(rename = "CommentCharacter", skip_serializing_if = "Option::is_none")]
    pub comment_character: Option<String>,

    /// The line range of the object to select, in the format of `start-end`.
    /// The `line-range=` prefix is added before being sent.
    #[serde(rename = "Range", skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,

    /// The split range of the object to select, in the format of `start-end`.
    /// It is converted to a `split-range=` prefixed `Range` before being sent.
    #[serde(skip)]
    pub split_range: Option<String>,

    /// Specifies whether the record delimiter can be enclosed in quotation marks.
    #[serde(
        rename = "AllowQuotedRecordDelimiter",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_quoted_record_delimiter: Option<bool>,
}

/// The JSON input format.
#[derive(Debug, Default, Clone, Serialize)]
pub struct JsonSelectInput {
    /// The type of the JSON object. Valid values: DOCUMENT and LINES.
    #[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
    pub json_type: Option<String>,

    /// The line range of the object to select, in the format of `start-end`.
    /// The `line-range=` prefix is added before being sent.
    #[serde(rename = "Range", skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,

    /// Specifies whether to parse JSON numbers as strings.
    #[serde(
        rename = "ParseJsonNumberAsString",
        skip_serializing_if = "Option::is_none"
    )]
    pub parse_json_number_as_string: Option<bool>,

    /// The split range of the object to select, in the format of `start-end`.
    /// It is converted to a `split-range=` prefixed `Range` before being sent.
    #[serde(skip)]
    pub split_range: Option<String>,
}

/// The format of the output data.
#[derive(Debug, Default, Clone, Serialize)]
pub struct OutputSerializationSelect {
    /// The CSV output format.
    #[serde(rename = "CSV", skip_serializing_if = "Option::is_none")]
    pub csv: Option<CsvSelectOutput>,

    /// The JSON output format.
    #[serde(rename = "JSON", skip_serializing_if = "Option::is_none")]
    pub json: Option<JsonSelectOutput>,

    /// Specifies whether to output raw data without the framing protocol.
    #[serde(rename = "OutputRawData", skip_serializing_if = "Option::is_none")]
    pub output_raw_data: Option<bool>,

    /// Specifies whether to return all columns in the output.
    #[serde(rename = "KeepAllColumns", skip_serializing_if = "Option::is_none")]
    pub keep_all_columns: Option<bool>,

    /// Specifies whether to enable the payload CRC-32 check for each frame.
    #[serde(rename = "EnablePayloadCrc", skip_serializing_if = "Option::is_none")]
    pub enable_payload_crc: Option<bool>,

    /// Specifies whether to output the header row in the first frame.
    #[serde(rename = "OutputHeader", skip_serializing_if = "Option::is_none")]
    pub output_header: Option<bool>,
}

/// The CSV output format.
#[derive(Debug, Default, Clone, Serialize)]
pub struct CsvSelectOutput {
    /// The record delimiter.
    #[serde(rename = "RecordDelimiter", skip_serializing_if = "Option::is_none")]
    pub record_delimiter: Option<String>,

    /// The field delimiter.
    #[serde(rename = "FieldDelimiter", skip_serializing_if = "Option::is_none")]
    pub field_delimiter: Option<String>,
}

/// The JSON output format.
#[derive(Debug, Default, Clone, Serialize)]
pub struct JsonSelectOutput {
    /// The record delimiter, base64 encoded before being sent.
    #[serde(rename = "RecordDelimiter", skip_serializing_if = "Option::is_none")]
    pub record_delimiter: Option<String>,
}

/// The options of the select operation.
#[derive(Debug, Default, Clone, Serialize)]
pub struct SelectOptions {
    /// Specifies whether to skip partially recorded data.
    #[serde(
        rename = "SkipPartialDataRecord",
        skip_serializing_if = "Option::is_none"
    )]
    pub skip_partial_data_record: Option<bool>,

    /// The maximum number of skipped records allowed.
    #[serde(
        rename = "MaxSkippedRecordsAllowed",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_skipped_records_allowed: Option<i32>,
}
#[derive(Debug, Default, OssRequestModel)]
pub struct SelectObjectRequest {
    /// The name of the bucket.
    pub bucket: String,

    /// The name of the object.
    pub key: String,

    /// The select request body.
    pub select_request: SelectRequest,

    pub common: RequestCommon,
}
#[derive(Default, OssResultModel)]
pub struct SelectObjectResult {
    /// The select result body. It is a reader that decodes the select framing
    /// protocol and yields the plain result data.
    pub body: Option<SelectObjectBodyReader>,

    /// Common result fields.
    pub common: ResultCommon,
}

impl std::fmt::Debug for SelectObjectResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectObjectResult")
            .field("body", &self.body)
            .field("common", &self.common)
            .finish()
    }
}

fn base64_encode(value: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(value.as_bytes())
}

/// Applies the same mutation as Go's `SelectRequest.encodeBase64`: dispatches
/// to the CSV or JSON variant based on whether a JSON input is present.
fn encode_base64_select(select_request: &mut SelectRequest) {
    if select_request.input_serialization.json.is_none() {
        csv_encode_base64(select_request);
    } else {
        json_encode_base64(select_request);
    }
}

/// Mirrors Go's `SelectRequest.csvEncodeBase64`.
fn csv_encode_base64(select_request: &mut SelectRequest) {
    if let Some(expression) = &select_request.expression {
        select_request.expression = Some(base64_encode(expression));
    }
    let csv = match select_request.input_serialization.csv.as_mut() {
        Some(csv) => csv,
        None => return,
    };
    if let Some(v) = &csv.record_delimiter {
        csv.record_delimiter = Some(base64_encode(v));
    }
    if let Some(v) = &csv.field_delimiter {
        csv.field_delimiter = Some(base64_encode(v));
    }
    if let Some(v) = &csv.quote_character {
        csv.quote_character = Some(base64_encode(v));
    }
    if let Some(v) = &csv.comment_character {
        csv.comment_character = Some(base64_encode(v));
    }
    if let Some(range) = &csv.range {
        if !range.is_empty() {
            csv.range = Some(format!("line-range={}", range));
        }
    }
    if let Some(split_range) = csv.split_range.take() {
        if !split_range.is_empty() {
            csv.range = Some(format!("split-range={}", split_range));
        } else {
            csv.split_range = Some(split_range);
        }
    }
}

/// Mirrors Go's `SelectRequest.jsonEncodeBase64`, including its early return
/// when no JSON output serialization is present.
fn json_encode_base64(select_request: &mut SelectRequest) {
    if let Some(expression) = &select_request.expression {
        select_request.expression = Some(base64_encode(expression));
    }
    match select_request.output_serialization.json.as_mut() {
        Some(json_output) => {
            if let Some(v) = &json_output.record_delimiter {
                json_output.record_delimiter = Some(base64_encode(v));
            }
        }
        None => return,
    }
    if let Some(json_input) = select_request.input_serialization.json.as_mut() {
        if let Some(range) = &json_input.range {
            json_input.range = Some(format!("line-range={}", range));
        }
        if let Some(split_range) = json_input.split_range.take() {
            if !split_range.is_empty() {
                json_input.range = Some(format!("split-range={}", split_range));
            } else {
                json_input.split_range = Some(split_range);
            }
        }
    }
}

/// The reader for the select object response body. It decodes the select
/// framing protocol and yields the plain result data.
///
/// Each frame on the wire has the following layout:
///
/// ```text
/// | frame type (4B) | payload length (4B) | header checksum (4B) | offset (8B) | payload | payload checksum (4B) |
/// ```
///
/// The payload length includes the 8-byte offset. Supported frame types are
/// Data (8388609), Continuous (8388612), End (8388613), MetaEndCSV (8388614)
/// and MetaEndJSON (8388615).
pub struct SelectObjectBodyReader {
    stream: BodyStream,
    in_buf: Vec<u8>,
    in_pos: usize,
    stream_done: bool,
    output_raw_data: bool,
    enable_payload_crc: bool,
    finished: bool,
    open_line: bool,
    awaiting_checksum: bool,
    frame_type: i32,
    payload_length: i64,
    consumed_bytes: i64,
    crc_digest: Option<crc::Digest<'static, u32>>,
    // Meta captured from End/MetaEnd frames.
    total_scanned: i64,
    http_status_code: i32,
    status: i32,
    splits_count: i32,
    rows_count: i64,
    columns_count: i32,
    error_msg: String,
    pending_error: Option<io::Error>,
}

impl std::fmt::Debug for SelectObjectBodyReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectObjectBodyReader")
            .field("finished", &self.finished)
            .field("output_raw_data", &self.output_raw_data)
            .field("enable_payload_crc", &self.enable_payload_crc)
            .field("total_scanned", &self.total_scanned)
            .field("http_status_code", &self.http_status_code)
            .field("error_msg", &self.error_msg)
            .finish()
    }
}

impl SelectObjectBodyReader {
    /// Creates a reader over the raw response body stream.
    pub(crate) fn new(
        stream: BodyStream,
        output_raw_data: bool,
        enable_payload_crc: bool,
    ) -> Self {
        SelectObjectBodyReader {
            stream,
            in_buf: Vec::new(),
            in_pos: 0,
            stream_done: false,
            output_raw_data,
            enable_payload_crc,
            finished: false,
            open_line: false,
            awaiting_checksum: false,
            frame_type: 0,
            payload_length: 0,
            consumed_bytes: 0,
            crc_digest: if enable_payload_crc {
                Some(SELECT_CRC32.digest())
            } else {
                None
            },
            total_scanned: 0,
            http_status_code: 0,
            status: 0,
            splits_count: 0,
            rows_count: 0,
            columns_count: 0,
            error_msg: String::new(),
            pending_error: None,
        }
    }

    /// The total scanned bytes reported by the End/MetaEnd frame.
    pub fn total_scanned(&self) -> i64 {
        self.total_scanned
    }

    /// The HTTP status code reported by the End frame.
    pub fn http_status_code(&self) -> i32 {
        self.http_status_code
    }

    /// The status reported by the MetaEnd frame.
    pub fn status(&self) -> i32 {
        self.status
    }

    /// The splits count reported by the MetaEnd frame.
    pub fn splits_count(&self) -> i32 {
        self.splits_count
    }

    /// The rows count reported by the MetaEnd frame.
    pub fn rows_count(&self) -> i64 {
        self.rows_count
    }

    /// The columns count reported by the MetaEnd CSV frame.
    pub fn columns_count(&self) -> i32 {
        self.columns_count
    }

    /// The error message reported by the End/MetaEnd frame.
    pub fn error_msg(&self) -> &str {
        &self.error_msg
    }

    /// Reads all remaining data. Convenience wrapper over
    /// `tokio::io::AsyncReadExt::read_to_end`.
    pub async fn read_all(&mut self) -> io::Result<Vec<u8>> {
        let mut data = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(self, &mut data).await?;
        Ok(data)
    }

    fn buffered(&self) -> &[u8] {
        &self.in_buf[self.in_pos..]
    }

    fn advance(&mut self, n: usize) {
        self.in_pos += n;
        if self.in_pos == self.in_buf.len() {
            self.in_buf.clear();
            self.in_pos = 0;
        }
    }

    fn feed_crc(&mut self, data: &[u8]) {
        if let Some(digest) = self.crc_digest.as_mut() {
            digest.update(data);
        }
    }

    /// Pulls one chunk from the underlying stream into the internal buffer.
    /// Returns Ok(true) if bytes were buffered, Ok(false) on stream end.
    fn poll_fill(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<bool>> {
        if self.stream_done {
            return Poll::Ready(Ok(false));
        }
        match self.stream.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                if chunk.is_empty() {
                    return self.poll_fill(cx);
                }
                self.in_buf.extend_from_slice(&chunk);
                Poll::Ready(Ok(true))
            }
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e)))
            }
            Poll::Ready(None) => {
                self.stream_done = true;
                Poll::Ready(Ok(false))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    /// Ensures at least `n` bytes are buffered. Returns Ok(false) if the
    /// stream ended before `n` bytes were available.
    fn poll_ensure(&mut self, cx: &mut Context<'_>, n: usize) -> Poll<io::Result<bool>> {
        while self.buffered().len() < n {
            match self.poll_fill(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Ready(Ok(false)) => return Poll::Ready(Ok(false)),
                Poll::Ready(Ok(true)) => {}
            }
        }
        Poll::Ready(Ok(true))
    }

    /// Parses the 20-byte frame header. Mirrors Go's `analysisHeader`: the
    /// first byte of the frame type is a version byte and is masked out, the
    /// header checksum is not validated, and the 8-byte offset is included in
    /// the payload CRC.
    fn parse_header(&mut self) -> io::Result<()> {
        let start = self.in_pos;
        let header = &self.in_buf[start..start + 20];
        let frame_type =
            u32::from_be_bytes([0, header[1], header[2], header[3]]) as i32;
        if frame_type != DATA_FRAME_TYPE
            && frame_type != CONTINUOUS_FRAME_TYPE
            && frame_type != END_FRAME_TYPE
            && frame_type != META_END_FRAME_CSV_TYPE
            && frame_type != META_END_FRAME_JSON_TYPE
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected frame type: {}", frame_type),
            ));
        }
        self.frame_type = frame_type;
        self.payload_length =
            i32::from_be_bytes([header[4], header[5], header[6], header[7]]) as i64;
        let offset_bytes: [u8; 8] = [
            header[12], header[13], header[14], header[15], header[16], header[17],
            header[18], header[19],
        ];
        self.feed_crc(&offset_bytes);
        self.advance(20);
        self.open_line = true;
        Ok(())
    }

    /// Parses the End frame payload. Mirrors Go's `analysisEndFrame`.
    fn parse_end_frame(&mut self) {
        let start = self.in_pos;
        let len = (self.payload_length - 8) as usize;
        let payload = &self.in_buf[start..start + len];
        self.total_scanned = i64::from_be_bytes(payload[0..8].try_into().unwrap());
        self.http_status_code = i32::from_be_bytes(payload[8..12].try_into().unwrap());
        let error_msg = String::from_utf8_lossy(&payload[12..]).into_owned();
        self.error_msg = error_msg;
        let crc_bytes: Vec<u8> = payload.to_vec();
        self.feed_crc(&crc_bytes);
        self.advance(len);
    }

    /// Parses the MetaEnd CSV/JSON frame payloads. Mirrors Go's
    /// `analysisMetaEndFrameCSV` and `analysisMetaEndFrameJSON`.
    fn parse_meta_end_frame(&mut self) {
        let start = self.in_pos;
        let len = (self.payload_length - 8) as usize;
        let payload = &self.in_buf[start..start + len];
        self.total_scanned = i64::from_be_bytes(payload[0..8].try_into().unwrap());
        self.status = i32::from_be_bytes(payload[8..12].try_into().unwrap());
        self.splits_count = i32::from_be_bytes(payload[12..16].try_into().unwrap());
        self.rows_count = i64::from_be_bytes(payload[16..24].try_into().unwrap());
        let error_msg = if self.frame_type == META_END_FRAME_CSV_TYPE {
            self.columns_count = i32::from_be_bytes(payload[24..28].try_into().unwrap());
            String::from_utf8_lossy(&payload[28..]).into_owned()
        } else {
            String::from_utf8_lossy(&payload[24..]).into_owned()
        };
        self.error_msg = error_msg;
        let crc_bytes: Vec<u8> = payload.to_vec();
        self.feed_crc(&crc_bytes);
        self.advance(len);
    }

    /// Reads the 4-byte payload checksum, verifies it when payload CRC is
    /// enabled, and finalizes the current frame. Mirrors Go's
    /// `checkPayloadSum`. Returns an End-frame error to surface to the caller
    /// when the server reported a non-2xx status code.
    fn finalize_frame(&mut self) -> io::Result<Option<io::Error>> {
        let start = self.in_pos;
        let server_crc32 = u32::from_be_bytes(
            self.in_buf[start..start + 4].try_into().unwrap(),
        );
        self.advance(4);
        if self.enable_payload_crc && server_crc32 != 0 {
            let client_crc32 = match self.crc_digest.take() {
                Some(digest) => digest.finalize(),
                None => 0,
            };
            if server_crc32 != client_crc32 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "unexpected frame type: {}, client {} but server {}",
                        self.frame_type, client_crc32, server_crc32
                    ),
                ));
            }
        }
        // Deviation from Go v2 (which only records the End frame status):
        // a non-2xx End frame status is surfaced as a read error carrying the
        // server-provided error message.
        let end_error = if self.frame_type == END_FRAME_TYPE
            && (self.http_status_code < 200 || self.http_status_code >= 300)
        {
            Some(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "select object failed, http status code: {}, error message: {}",
                    self.http_status_code, self.error_msg
                ),
            ))
        } else {
            None
        };
        match self.frame_type {
            END_FRAME_TYPE | META_END_FRAME_CSV_TYPE | META_END_FRAME_JSON_TYPE => {
                self.finished = true;
            }
            _ => self.reset_frame(),
        }
        Ok(end_error)
    }

    /// Resets the per-frame state. Mirrors Go's `emptyFrame`.
    fn reset_frame(&mut self) {
        self.open_line = false;
        self.awaiting_checksum = false;
        self.frame_type = 0;
        self.payload_length = 0;
        self.consumed_bytes = 0;
        self.crc_digest = if self.enable_payload_crc {
            Some(SELECT_CRC32.digest())
        } else {
            None
        };
    }
}

impl AsyncRead for SelectObjectBodyReader {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if buf.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }
        let this = self.get_mut();
        if let Some(err) = this.pending_error.take() {
            return Poll::Ready(Err(err));
        }
        if this.finished {
            // EOF
            return Poll::Ready(Ok(()));
        }
        let start_filled = buf.filled().len();

        macro_rules! pending_or_ready {
            () => {
                if buf.filled().len() > start_filled {
                    return Poll::Ready(Ok(()));
                } else {
                    return Poll::Pending;
                }
            };
        }
        macro_rules! ret_err {
            ($e:expr) => {{
                let e: io::Error = $e;
                if buf.filled().len() > start_filled {
                    // Deliver the bytes already produced and surface the error
                    // on the next read, matching Go's (n, err) semantics.
                    this.pending_error = Some(e);
                    return Poll::Ready(Ok(()));
                } else {
                    return Poll::Ready(Err(e));
                }
            }};
        }

        loop {
            if this.output_raw_data {
                // The server returned raw data: pass the stream through
                // without decoding frames.
                let avail = this.buffered().len().min(buf.remaining());
                if avail > 0 {
                    buf.put_slice(&this.buffered()[..avail]);
                    this.advance(avail);
                    return Poll::Ready(Ok(()));
                }
                match this.poll_fill(cx) {
                    Poll::Pending => pending_or_ready!(),
                    Poll::Ready(Err(e)) => ret_err!(e),
                    Poll::Ready(Ok(false)) => return Poll::Ready(Ok(())),
                    Poll::Ready(Ok(true)) => {
                        let avail = this.buffered().len().min(buf.remaining());
                        buf.put_slice(&this.buffered()[..avail]);
                        this.advance(avail);
                        return Poll::Ready(Ok(()));
                    }
                }
            }

            if this.awaiting_checksum {
                match this.poll_ensure(cx, 4) {
                    Poll::Pending => pending_or_ready!(),
                    Poll::Ready(Err(e)) => ret_err!(e),
                    Poll::Ready(Ok(false)) => ret_err!(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "read checksum error"
                    )),
                    Poll::Ready(Ok(true)) => match this.finalize_frame() {
                        Err(e) => ret_err!(e),
                        Ok(Some(e)) => ret_err!(e),
                        Ok(None) => {
                            if this.finished {
                                return Poll::Ready(Ok(()));
                            }
                        }
                    },
                }
                continue;
            }

            if !this.open_line {
                match this.poll_ensure(cx, 20) {
                    Poll::Pending => pending_or_ready!(),
                    Poll::Ready(Err(e)) => ret_err!(e),
                    Poll::Ready(Ok(false)) => ret_err!(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "read response frame header failure"
                    )),
                    Poll::Ready(Ok(true)) => {
                        if let Err(e) = this.parse_header() {
                            ret_err!(e);
                        }
                    }
                }
            }

            match this.frame_type {
                DATA_FRAME_TYPE => {
                    let frame_remaining =
                        (this.payload_length - 8 - this.consumed_bytes) as usize;
                    let want = frame_remaining.min(buf.remaining());
                    let avail = want.min(this.buffered().len());
                    if avail > 0 {
                        let start = this.in_pos;
                        if let Some(digest) = this.crc_digest.as_mut() {
                            digest.update(&this.in_buf[start..start + avail]);
                        }
                        buf.put_slice(&this.in_buf[start..start + avail]);
                        this.advance(avail);
                        this.consumed_bytes += avail as i64;
                        if this.consumed_bytes == this.payload_length - 8 {
                            this.awaiting_checksum = true;
                        }
                        if buf.remaining() == 0 {
                            return Poll::Ready(Ok(()));
                        }
                    } else if frame_remaining == 0 {
                        this.awaiting_checksum = true;
                    } else {
                        match this.poll_fill(cx) {
                            Poll::Pending => pending_or_ready!(),
                            Poll::Ready(Err(e)) => ret_err!(e),
                            Poll::Ready(Ok(false)) => ret_err!(io::Error::new(
                                io::ErrorKind::UnexpectedEof,
                                "read frame data error"
                            )),
                            Poll::Ready(Ok(true)) => {}
                        }
                    }
                }
                CONTINUOUS_FRAME_TYPE => {
                    // The payload of a Continuous frame is only the 8-byte
                    // offset, which is already part of the frame header.
                    this.awaiting_checksum = true;
                }
                END_FRAME_TYPE => {
                    let need = (this.payload_length - 8) as usize;
                    match this.poll_ensure(cx, need) {
                        Poll::Pending => pending_or_ready!(),
                        Poll::Ready(Err(e)) => ret_err!(e),
                        Poll::Ready(Ok(false)) => ret_err!(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "read end frame error"
                        )),
                        Poll::Ready(Ok(true)) => {
                            this.parse_end_frame();
                            this.awaiting_checksum = true;
                        }
                    }
                }
                META_END_FRAME_CSV_TYPE | META_END_FRAME_JSON_TYPE => {
                    let need = (this.payload_length - 8) as usize;
                    match this.poll_ensure(cx, need) {
                        Poll::Pending => pending_or_ready!(),
                        Poll::Ready(Err(e)) => ret_err!(e),
                        Poll::Ready(Ok(false)) => ret_err!(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "read meta end frame error"
                        )),
                        Poll::Ready(Ok(true)) => {
                            this.parse_meta_end_frame();
                            this.awaiting_checksum = true;
                        }
                    }
                }
                _ => ret_err!(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unexpected frame type: {}", this.frame_type)
                )),
            }
        }
    }
}

impl Client {
    /// Executes SQL statements to perform operations on an object and obtains
    /// the execution results.
    ///
    /// # Arguments
    ///
    /// * `request` - The `SelectObjectRequest` containing the bucket name,
    ///   object key and the select request body.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use alibabacloud_oss_sdk_rust_v2::api::object::{
    /// #     SelectObjectRequest, SelectRequest, InputSerializationSelect,
    /// #     OutputSerializationSelect, CsvSelectInput, CsvSelectOutput,
    /// # };
    /// # use alibabacloud_oss_sdk_rust_v2::client::Client;
    /// # use alibabacloud_oss_sdk_rust_v2::config::Config;
    /// # use tokio::io::AsyncReadExt;
    /// #
    /// # tokio_test::block_on(async {
    /// let client = Client::new(&Config::default());
    /// let request = SelectObjectRequest {
    ///     bucket: "my-bucket".to_string(),
    ///     key: "my-object.csv".to_string(),
    ///     select_request: SelectRequest {
    ///         expression: Some("select * from ossobject".to_string()),
    ///         input_serialization: InputSerializationSelect {
    ///             csv: Some(CsvSelectInput {
    ///                 record_delimiter: Some("\n".to_string()),
    ///                 field_delimiter: Some(",".to_string()),
    ///                 ..Default::default()
    ///             }),
    ///             ..Default::default()
    ///         },
    ///         output_serialization: OutputSerializationSelect {
    ///             csv: Some(CsvSelectOutput {
    ///                 record_delimiter: Some("\n".to_string()),
    ///                 field_delimiter: Some(",".to_string()),
    ///             }),
    ///             ..Default::default()
    ///         },
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// match client.select_object(&request).await {
    ///     Ok(mut result) => {
    ///         if let Some(mut reader) = result.body.take() {
    ///             let mut data = Vec::new();
    ///             let _ = reader.read_to_end(&mut data).await;
    ///             println!("{}", String::from_utf8_lossy(&data));
    ///         }
    ///     }
    ///     Err(error) => {
    ///         eprintln!("Failed to select object: {}", error);
    ///     }
    /// }
    /// # })
    /// ```
    pub async fn select_object(
        &self,
        request: &SelectObjectRequest,
    ) -> Result<SelectObjectResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut select_request = request.select_request.clone();
        let process = if select_request.input_serialization.json.is_none() {
            "csv/select"
        } else {
            "json/select"
        };
        encode_base64_select(&mut select_request);
        let xml_body = quick_xml::se::to_string_with_root("SelectRequest", &select_request)?;

        let mut input = OperationInput {
            op_name: "SelectObject".to_string(),
            method: http::Method::POST,
            bucket: Some(request.bucket.clone()),
            key: Some(request.key.clone()),
            parameters: [("x-oss-process", process)]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: [(HTTP_HEADER_CONTENT_TYPE, "application/xml")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Default::default()
        };
        input.body = Some(BodyContent::from_text(xml_body, None));

        modify_request(
            &mut input,
            request.header_map(),
            request.query_map(),
            vec![update_content_md5, update_content_length],
        )?;

        let output = self.invoke_operation(input, vec![]).await?;

        let output_raw_data = output
            .headers
            .get("x-oss-select-output-raw")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let enable_payload_crc = request
            .select_request
            .output_serialization
            .enable_payload_crc
            == Some(true);

        let mut result = SelectObjectResult::default();
        result.update_result(&output);
        if let Some(stream) = output.body {
            result.body = Some(SelectObjectBodyReader::new(
                stream,
                output_raw_data,
                enable_payload_crc,
            ));
        }

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
    use tokio::io::AsyncReadExt;

    fn stream_from_chunks(chunks: Vec<Vec<u8>>) -> BodyStream {
        let items: Vec<Result<bytes::Bytes, reqwest::Error>> = chunks
            .into_iter()
            .map(|c| Ok(bytes::Bytes::from(c)))
            .collect();
        Box::pin(futures_util::stream::iter(items))
    }

    fn stream_from(data: Vec<u8>) -> BodyStream {
        stream_from_chunks(vec![data])
    }

    fn payload_crc32(offset: u64, payload: &[u8]) -> u32 {
        let crc = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
        let mut digest = crc.digest();
        digest.update(&offset.to_be_bytes());
        digest.update(payload);
        digest.finalize()
    }

    /// Builds one wire frame: type(4B) | payload length(4B, offset included)
    /// | header checksum(4B) | offset(8B) | payload | payload checksum(4B).
    fn build_frame(
        frame_type: i32,
        offset: u64,
        payload: &[u8],
        payload_checksum: Option<u32>,
    ) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&(frame_type as u32).to_be_bytes());
        v.extend_from_slice(&((payload.len() as u32 + 8).to_be_bytes()));
        v.extend_from_slice(&0u32.to_be_bytes());
        v.extend_from_slice(&offset.to_be_bytes());
        v.extend_from_slice(payload);
        v.extend_from_slice(&payload_checksum.unwrap_or(0).to_be_bytes());
        v
    }

    fn data_frame(offset: u64, data: &[u8], with_crc: bool) -> Vec<u8> {
        let checksum = if with_crc {
            Some(payload_crc32(offset, data))
        } else {
            None
        };
        build_frame(DATA_FRAME_TYPE, offset, data, checksum)
    }

    fn end_frame(offset: u64, total_scanned: i64, status: i32, error_msg: &str) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&total_scanned.to_be_bytes());
        payload.extend_from_slice(&status.to_be_bytes());
        payload.extend_from_slice(error_msg.as_bytes());
        build_frame(END_FRAME_TYPE, offset, &payload, None)
    }

    #[tokio::test]
    async fn test_select_reader_data_and_end_frame() {
        let mut bytes = data_frame(0, b"hello", false);
        bytes.extend_from_slice(&data_frame(5, b" world", false));
        bytes.extend_from_slice(&end_frame(11, 1024, 200, ""));

        let mut reader = SelectObjectBodyReader::new(stream_from(bytes), false, false);
        let mut out = Vec::new();
        reader.read_to_end(&mut out).await.unwrap();
        assert_eq!(out, b"hello world");
        assert_eq!(reader.http_status_code(), 200);
        assert_eq!(reader.total_scanned(), 1024);

        // Subsequent reads return EOF.
        let mut out2 = Vec::new();
        reader.read_to_end(&mut out2).await.unwrap();
        assert!(out2.is_empty());
    }

    #[tokio::test]
    async fn test_select_reader_error_end_frame() {
        let mut bytes = data_frame(0, b"partial", false);
        bytes.extend_from_slice(&end_frame(7, 512, 400, "InvalidSqlStatement"));

        let mut reader = SelectObjectBodyReader::new(stream_from(bytes), false, false);
        let mut out = Vec::new();
        let err = reader.read_to_end(&mut out).await.unwrap_err();
        assert!(
            err.to_string().contains("InvalidSqlStatement"),
            "unexpected error: {}",
            err
        );
        // The data produced before the error end frame is still delivered.
        assert_eq!(out, b"partial");
        assert_eq!(reader.error_msg(), "InvalidSqlStatement");
    }

    #[tokio::test]
    async fn test_select_reader_error_end_frame_without_data() {
        let bytes = end_frame(0, 0, 403, "AccessDenied");
        let mut reader = SelectObjectBodyReader::new(stream_from(bytes), false, false);
        let mut out = Vec::new();
        let err = reader.read_to_end(&mut out).await.unwrap_err();
        assert!(err.to_string().contains("AccessDenied"));
        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn test_select_reader_chunked_stream() {
        let mut bytes = data_frame(0, b"chunked-data-payload", false);
        bytes.extend_from_slice(&end_frame(20, 64, 206, ""));
        // Split the wire bytes into small chunks, cutting frames mid-header
        // and mid-payload.
        let chunks: Vec<Vec<u8>> = bytes.chunks(3).map(|c| c.to_vec()).collect();

        let mut reader = SelectObjectBodyReader::new(stream_from_chunks(chunks), false, false);
        let mut out = Vec::new();
        reader.read_to_end(&mut out).await.unwrap();
        assert_eq!(out, b"chunked-data-payload");
        assert_eq!(reader.http_status_code(), 206);
    }

    #[tokio::test]
    async fn test_select_reader_small_read_buffer() {
        let mut bytes = data_frame(0, b"0123456789abcdef", false);
        bytes.extend_from_slice(&end_frame(16, 16, 200, ""));

        let mut reader = SelectObjectBodyReader::new(stream_from(bytes), false, false);
        let mut collected = Vec::new();
        let mut buf = [0u8; 4];
        loop {
            let n = reader.read(&mut buf).await.unwrap();
            if n == 0 {
                break;
            }
            collected.extend_from_slice(&buf[..n]);
        }
        assert_eq!(collected, b"0123456789abcdef");
    }

    #[tokio::test]
    async fn test_select_reader_continuous_frame() {
        let mut bytes = data_frame(0, b"ab", false);
        bytes.extend_from_slice(&build_frame(CONTINUOUS_FRAME_TYPE, 2, &[], None));
        bytes.extend_from_slice(&data_frame(2, b"cd", false));
        bytes.extend_from_slice(&end_frame(4, 4, 200, ""));

        let mut reader = SelectObjectBodyReader::new(stream_from(bytes), false, false);
        let mut out = Vec::new();
        reader.read_to_end(&mut out).await.unwrap();
        assert_eq!(out, b"abcd");
    }

    #[tokio::test]
    async fn test_select_reader_payload_crc_ok() {
        let mut bytes = data_frame(0, b"with-crc", true);
        bytes.extend_from_slice(&end_frame(8, 8, 200, ""));

        let mut reader = SelectObjectBodyReader::new(stream_from(bytes), false, true);
        let mut out = Vec::new();
        reader.read_to_end(&mut out).await.unwrap();
        assert_eq!(out, b"with-crc");
    }

    #[tokio::test]
    async fn test_select_reader_payload_crc_mismatch() {
        let mut frame = data_frame(0, b"bad-crc", true);
        // Corrupt the trailing payload checksum.
        let n = frame.len();
        frame[n - 1] ^= 0xff;
        frame.extend_from_slice(&end_frame(7, 7, 200, ""));

        let mut reader = SelectObjectBodyReader::new(stream_from(frame), false, true);
        let mut out = Vec::new();
        let err = reader.read_to_end(&mut out).await.unwrap_err();
        assert!(
            err.to_string().contains("client"),
            "unexpected error: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_select_reader_raw_passthrough() {
        let raw = b"raw,output,not,framed\n1,2,3,4\n".to_vec();
        let mut reader = SelectObjectBodyReader::new(stream_from(raw.clone()), true, false);
        let mut out = Vec::new();
        reader.read_to_end(&mut out).await.unwrap();
        assert_eq!(out, raw);
    }

    #[tokio::test]
    async fn test_select_reader_unexpected_frame_type() {
        let bytes = build_frame(12345, 0, &[], None);
        let mut reader = SelectObjectBodyReader::new(stream_from(bytes), false, false);
        let mut out = Vec::new();
        let err = reader.read_to_end(&mut out).await.unwrap_err();
        assert!(
            err.to_string().contains("unexpected frame type"),
            "unexpected error: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_select_reader_truncated_stream() {
        // Only half of a data frame, then the stream ends.
        let mut bytes = data_frame(0, b"truncated", false);
        bytes.truncate(20 + 3);

        let mut reader = SelectObjectBodyReader::new(stream_from(bytes), false, false);
        let mut out = Vec::new();
        let err = reader.read_to_end(&mut out).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_select_request_csv_base64_and_range_encoding() {
        let mut select_request = SelectRequest {
            expression: Some("select * from ossobject".to_string()),
            input_serialization: InputSerializationSelect {
                csv: Some(CsvSelectInput {
                    file_header_info: Some("USE".to_string()),
                    record_delimiter: Some("\n".to_string()),
                    field_delimiter: Some(",".to_string()),
                    quote_character: Some("\"".to_string()),
                    range: Some("1-100".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            output_serialization: OutputSerializationSelect {
                csv: Some(CsvSelectOutput {
                    record_delimiter: Some("\n".to_string()),
                    field_delimiter: Some(",".to_string()),
                }),
                enable_payload_crc: Some(true),
                ..Default::default()
            },
            ..Default::default()
        };

        encode_base64_select(&mut select_request);
        let xml =
            quick_xml::se::to_string_with_root("SelectRequest", &select_request).unwrap();

        assert!(xml.contains("<SelectRequest>"));
        // base64("select * from ossobject")
        assert!(xml.contains(
            "<Expression>c2VsZWN0ICogZnJvbSBvc3NvYmplY3Q=</Expression>"
        ));
        // base64("\n") == "Cg==", base64(",") == "LA==", base64("\"") == "Ig=="
        assert!(xml.contains("<RecordDelimiter>Cg==</RecordDelimiter>"));
        assert!(xml.contains("<FieldDelimiter>LA==</FieldDelimiter>"));
        assert!(xml.contains("<QuoteCharacter>Ig==</QuoteCharacter>"));
        assert!(xml.contains("<Range>line-range=1-100</Range>"));
        assert!(xml.contains("<EnablePayloadCrc>true</EnablePayloadCrc>"));
    }

    #[test]
    fn test_select_request_split_range_encoding() {
        let mut select_request = SelectRequest {
            expression: Some("select * from ossobject".to_string()),
            input_serialization: InputSerializationSelect {
                csv: Some(CsvSelectInput {
                    split_range: Some("0-3".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        encode_base64_select(&mut select_request);
        let xml =
            quick_xml::se::to_string_with_root("SelectRequest", &select_request).unwrap();

        assert!(xml.contains("<Range>split-range=0-3</Range>"));
        // SplitRange itself is not serialized.
        assert!(!xml.contains("SplitRange"));
    }

    #[test]
    fn test_select_request_json_encoding() {
        let mut select_request = SelectRequest {
            expression: Some("select s.a from ossobject s".to_string()),
            input_serialization: InputSerializationSelect {
                json: Some(JsonSelectInput {
                    json_type: Some("LINES".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            output_serialization: OutputSerializationSelect {
                json: Some(JsonSelectOutput {
                    record_delimiter: Some("\n".to_string()),
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        encode_base64_select(&mut select_request);
        let xml =
            quick_xml::se::to_string_with_root("SelectRequest", &select_request).unwrap();

        // base64("select s.a from ossobject s")
        assert!(xml.contains(
            "<Expression>c2VsZWN0IHMuYSBmcm9tIG9zc29iamVjdCBz</Expression>"
        ));
        assert!(xml.contains("<Type>LINES</Type>"));
        assert!(xml.contains("<RecordDelimiter>Cg==</RecordDelimiter>"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_select_object() {
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

        let object_name = crate::test_utils::generate_unique_object_name("select");
        let csv_content = "name,score\nAlice,90\nBob,80\nCarol,70\n";

        // Prepare a CSV object.
        client
            .put_object(crate::api::object::PutObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name.clone(),
                body: Some(BodyContent::from_text(csv_content.to_string(), None)),
                ..Default::default()
            })
            .await
            .unwrap();

        let request = SelectObjectRequest {
            bucket: config.bucket.clone(),
            key: object_name.clone(),
            select_request: SelectRequest {
                expression: Some(
                    "select * from ossobject where score > 75".to_string(),
                ),
                input_serialization: InputSerializationSelect {
                    csv: Some(CsvSelectInput {
                        file_header_info: Some("USE".to_string()),
                        record_delimiter: Some("\n".to_string()),
                        field_delimiter: Some(",".to_string()),
                        quote_character: Some("\"".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                output_serialization: OutputSerializationSelect {
                    csv: Some(CsvSelectOutput {
                        record_delimiter: Some("\n".to_string()),
                        field_delimiter: Some(",".to_string()),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let result = client.select_object(&request).await;
        assert!(result.is_ok(), "select_object failed: {:?}", result.err());

        let mut result = result.unwrap();
        let mut reader = result.body.take().expect("missing select result body");
        let data = reader.read_all().await.unwrap();
        let text = String::from_utf8_lossy(&data);
        assert!(text.contains("Alice,90"), "unexpected result: {}", text);
        assert!(text.contains("Bob,80"), "unexpected result: {}", text);
        assert!(!text.contains("Carol"), "unexpected result: {}", text);

        // Clean up.
        let _ = client
            .delete_object(crate::api::object::DeleteObjectRequest {
                bucket: config.bucket.clone(),
                key: object_name,
                ..Default::default()
            })
            .await;
    }
}
