use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::io::Read;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::utils::{is_valid_bucket_name, is_valid_method, is_valid_object_name, read_to_string};
use futures_util::StreamExt;
use std::pin::Pin;
use bytes::Bytes;
use std::path::PathBuf;
use futures_util::stream::TryStreamExt;
use reqwest;
use tokio::fs;
use futures_util::stream::{BoxStream, Stream};
use crate::client::ByteStream;

/// 用户可传入的数据源类型，支持惰性上传
pub enum BodyContent {
    File {
        path: PathBuf,
        len: u64,
        md5: Option<[u8; 16]>, // 自动计算
    },
    Bytes {
        data: Bytes,
        len: u64,
        md5: Option<[u8; 16]>, // 自动计算
    },
    Text {
        data: String,
        len: u64,
        md5: Option<[u8; 16]>, // 自动计算
    },
    Stream{
        stream:ByteStream,
        len: u64,
        md5: Option<[u8; 16]>, //非必选
    },
}

impl fmt::Debug for BodyContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BodyContent::File { path, len, md5: _ } => f.debug_struct("BodyContent::File")
                .field("path", path)
                .field("len", len)
                .finish(),
            BodyContent::Bytes { data, len, md5: _ } => f.debug_struct("BodyContent::Bytes")
                .field("data_len", &data.len())
                .field("len", len)
                .finish(),
            BodyContent::Text { data, len, md5: _ } => f.debug_struct("BodyContent::Text")
                .field("text_len", &data.len())
                .field("len", len)
                .finish(),
            BodyContent::Stream { stream: _, len, md5: _ } => f.debug_struct("BodyContent::Stream")
                .field("len", len)
                .finish(),
        }
    }
}

impl BodyContent {
    pub async fn into_reqwest_body(self) -> Result<reqwest::Body, std::io::Error> {
        self.into_reqwest_body_with_trackers(Vec::new()).await
    }

    /// Same as [`BodyContent::into_reqwest_body`], but feeds every chunk read
    /// from the body through `trackers` first.
    ///
    /// This is how upload integrity checks observe the bytes actually sent:
    /// the request body is consumed by reqwest, so a tracker has to sit in the
    /// read path rather than on a buffer. Mirrors Go `TeeReadNopCloser`, which
    /// tees `input.Body` into the writers registered under
    /// `OpMetaKeyRequestBodyTracker`.
    pub async fn into_reqwest_body_with_trackers(
        self,
        trackers: Vec<Arc<dyn BodyTracker>>,
    ) -> Result<reqwest::Body, std::io::Error> {
        self.into_reqwest_body_with_limit(trackers, None).await
    }

    /// Builds the request body, pacing it through `limiter` when one is set.
    ///
    /// The limit is applied per chunk as it leaves the SDK, which is the only
    /// point where an outgoing body can be throttled: reqwest owns the socket.
    pub async fn into_reqwest_body_with_limit(
        self,
        trackers: Vec<Arc<dyn BodyTracker>>,
        limiter: Option<Arc<crate::utils::BwTokenBucket>>,
    ) -> Result<reqwest::Body, std::io::Error> {
        use futures_util::stream::StreamExt;

        // Tees each chunk into the trackers before it reaches reqwest. Shared
        // so both the plain and the paced paths can use it.
        let tee = std::sync::Arc::new(
            move |chunk: Bytes| -> Result<Bytes, Box<dyn std::error::Error + Send + Sync>> {
                for tracker in &trackers {
                    tracker.update(&chunk)?;
                }
                Ok(chunk)
            },
        );

        match self {
            BodyContent::File { path, .. } => {
                let file = tokio::fs::File::open(path).await?;
                let stream = tokio_util::codec::FramedRead::new(file, tokio_util::codec::BytesCodec::new());
                let limiter_for_file = limiter.clone();
                let mapped = stream
                    .map(|result| {
                        result
                            .map(|bytes| bytes.freeze())
                            .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e))
                    })
                    .then(move |result| {
                        let limiter = limiter_for_file.clone();
                        let tee = tee.clone();
                        async move {
                            if let (Some(limiter), Ok(ref chunk)) = (&limiter, &result) {
                                limiter.limit_bandwidth(chunk.len()).await;
                            }
                            result.and_then(|chunk| tee(chunk))
                        }
                    });
                let byte_stream = ByteStream::new(mapped);
                Ok(reqwest::Body::wrap_stream(byte_stream))
            }
            BodyContent::Bytes { data, .. } => {
                let tracked = tee(data);
                let stream = futures_util::stream::once(async { tracked });
                let byte_stream = ByteStream::new(stream);
                Ok(reqwest::Body::wrap_stream(byte_stream))
            }
            BodyContent::Text { data, .. } => {
                let tracked = tee(Bytes::from(data.into_bytes()));
                let stream = futures_util::stream::once(async { tracked });
                let byte_stream = ByteStream::new(stream);
                Ok(reqwest::Body::wrap_stream(byte_stream))
            }
            BodyContent::Stream { stream, .. } => {
                // 对于Stream类型，直接使用传入的stream（现在拥有所有权）
                let tee = tee.clone();
                let mapped = stream.map(move |result| result.and_then(|chunk| tee(chunk)));
                Ok(reqwest::Body::wrap_stream(ByteStream::new(mapped)))
            }
        }
    }
}

impl BodyContent {
    pub async fn from_file_path(
        path: impl Into<PathBuf>,
        len: u64,
        md5: Option<[u8; 16]>,
    ) -> Result<Self, std::io::Error> {
        let path = path.into();
        // 可选：验证 len 是否匹配文件实际大小（非必须）
        Ok(BodyContent::File { path, len, md5 })
    }

    pub fn from_bytes(data: impl Into<Bytes>, md5: Option<[u8; 16]>) -> Self {
        let data = data.into();
        let len = data.len() as u64;
        BodyContent::Bytes { data, len, md5 }
    }

    pub fn from_text(s: String, md5: Option<[u8; 16]>) -> Self {
        let len = s.len() as u64;
        BodyContent::Text { data: s, len, md5 }
    }

    pub fn from_stream(stream: ByteStream, len: u64, md5: Option<[u8; 16]>) -> Self {
        BodyContent::Stream { stream, len, md5 }
    }



    pub fn content_length(&self) -> Option<u64> {
        match self {
            BodyContent::File { len, .. } => Some(*len),
            BodyContent::Bytes { len, .. } => Some(*len),
            BodyContent::Text { len, .. } => Some(*len),
            BodyContent::Stream { len, .. } => Some(*len),
        }
    }

    pub fn content_md5(&self) -> Option<[u8; 16]> {
        match self {
            BodyContent::File { md5, .. } => *md5,
            BodyContent::Bytes { md5, .. } => *md5,
            BodyContent::Text { md5, .. } => *md5,
            BodyContent::Stream { md5, .. } => *md5,
        }
    }

    /// Returns a fresh copy of this body for another send attempt.
    ///
    /// `File`, `Bytes`, and `Text` bodies can be rebuilt; an owned `Stream` is
    /// consumed by its first send and cannot be replayed. Mirrors Go's
    /// `teeReadNopCloser.IsSeekable` check in `Client.sendHttpRequest`.
    pub fn try_clone(&self) -> Option<BodyContent> {
        match self {
            BodyContent::File { path, len, md5 } => Some(BodyContent::File {
                path: path.clone(),
                len: *len,
                md5: *md5,
            }),
            BodyContent::Bytes { data, len, md5 } => Some(BodyContent::Bytes {
                data: data.clone(),
                len: *len,
                md5: *md5,
            }),
            BodyContent::Text { data, len, md5 } => Some(BodyContent::Text {
                data: data.clone(),
                len: *len,
                md5: *md5,
            }),
            BodyContent::Stream { .. } => None,
        }
    }
}

/// Observes the bytes of a request body as they are sent.
///
/// The body is consumed by reqwest, so integrity checks (CRC64) have to see
/// chunks on the read path. Trackers ride the request stream, which requires
/// `Send`, so they are shared through `Arc` and mutate through a lock; the
/// same instance is queried afterwards by the response check. Mirrors the
/// `io.Writer` registered under Go's `OpMetaKeyRequestBodyTracker`.
pub trait BodyTracker: Send + Sync {
    /// Feeds the next chunk of the request body.
    fn update(&self, chunk: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Resets to the initial state so a retried body is tracked from scratch.
    fn reset(&self);
}

/// A [`BodyTracker`] that accumulates a CRC64 over the request body.
///
/// The accumulator lives behind an `Arc` so a handle can be registered in
/// `OperationMetadata` (which stores `Rc`) while another handle rides the
/// request stream, which requires `Send` and therefore `Arc`.
#[derive(Clone)]
pub(crate) struct Crc64Tracker {
    crc: Arc<Mutex<crate::utils::Crc64>>,
}

impl Crc64Tracker {
    pub(crate) fn new(init: u64) -> Self {
        Crc64Tracker {
            crc: Arc::new(Mutex::new(crate::utils::Crc64::new(init))),
        }
    }

    /// Returns the CRC64 of everything written so far, as OSS reports it in
    /// `x-oss-hash-crc64ecma`.
    pub(crate) fn sum64(&self) -> u64 {
        self.crc.lock().expect("crc tracker lock poisoned").sum64()
    }
}

impl BodyTracker for Crc64Tracker {
    fn update(&self, chunk: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.crc
            .lock()
            .expect("crc tracker lock poisoned")
            .write(chunk)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("{}", e).into() })?;
        Ok(())
    }

    fn reset(&self) {
        self.crc.lock().expect("crc tracker lock poisoned").reset();
    }
}

// 假设你的 SDK 错误类型是 SdkError
pub type BodyStream = Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>;

// 定义一个通用 trait 用于从包含 body 流的结构中读取数据
pub trait BodyDataReader {
    /// 读取 body 流中的所有数据并返回字节数组
    /// 这会消费 body 流并为后续访问缓存结果
    async fn get_all_data(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // 如果已经缓存了数据，直接返回
        if let Some(cached_data) = self.get_cached_body_data() {
            return Ok(cached_data.into_bytes());
        }
        
        // 如果没有缓存数据但有body流，则读取流并缓存
        if let Some(body_stream) = self.take_body() {
            let mut full_data = Vec::new();
            
            // 使用 StreamExt 来处理流
            futures_util::pin_mut!(body_stream);
            while let Some(chunk_result) = body_stream.next().await {
                let chunk = chunk_result?;
                full_data.extend_from_slice(&chunk);
            }
            
            // 缓存数据以便将来访问（如果是有效的UTF-8字符串）
            if let Ok(s) = String::from_utf8(full_data.clone()) {
                self.set_body_data(Arc::new(s));
            }
            
            Ok(full_data)
        } else if let Some(cached_data) = self.get_cached_body_data() {
            // 再次检查以防万一
            Ok(cached_data.into_bytes())
        } else {
            // 没有body数据的情况
            Ok(Vec::new())
        }
    }
    
    /// 读取 body 流中的所有数据并返回字符串（安全版本）
    /// 这会消费 body 流并为后续访问缓存结果
    async fn get_all_data_as_string(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self.get_all_data().await {
            Ok(bytes) => {
                // 使用 from_utf8_lossy 来安全地处理非UTF-8序列
                Ok(String::from_utf8_lossy(&bytes).into_owned())
            }
            Err(e) => Err(e),
        }
    }
    
    /// 获取 body 流，如果有的话
    fn take_body(&mut self) -> Option<BodyStream>;
    
    /// 设置 body 数据缓存
    fn set_body_data(&mut self, data: Arc<String>);
    
    /// 获取已缓存的 body 数据
    fn get_cached_body_data(&self) -> Option<String>;
}

#[derive(Default, Debug, Clone)]
pub struct OperationMetadata {
    values: HashMap<String, Vec<Rc<dyn Any>>>,
}

impl OperationMetadata {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Rc<dyn Any>> {
        if let Some(v) = self.values.get(key) {
            return v.first();
        }
        None
    }

    pub fn values(&self, key: &str) -> Option<&Vec<Rc<dyn Any>>> {
        self.values.get(key)
    }

    pub fn add(&mut self, key: &str, value: Rc<dyn Any>) {
        self.values.entry(key.to_string()).or_default().push(value);
    }

    pub fn set(&mut self, key: &str, value: Rc<dyn Any>) {
        self.values.insert(key.to_string(), vec![value]);
    }

    pub fn has(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
}

#[derive(Default)]
pub struct OperationInput {
    pub op_name: String,
    pub method: http::Method,
    pub headers: HashMap<String, String>,
    pub parameters: HashMap<String, String>,
    /// Body reader. You may use `crate::utils::read_to_string` to read the
    /// content from a `dyn Read`.
    pub body: Option<BodyContent>,
    pub bucket: Option<String>,
    /// Object name
    pub key: Option<String>,
    pub op_metadata: OperationMetadata,
}

impl Clone for OperationInput {
    fn clone(&self) -> Self {
        OperationInput {
            op_name: self.op_name.clone(),
            method: self.method.clone(),
            headers: self.headers.clone(),
            parameters: self.parameters.clone(),
            body: None, // BodyContent cannot be cloned safely
            bucket: self.bucket.clone(),
            key: self.key.clone(),
            op_metadata: self.op_metadata.clone(),
        }
    }
}

#[derive(Default)]
pub struct OperationOutput {
    pub input: Option<Rc<OperationInput>>,
    pub status: http::StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Option<BodyStream>,
    // pub body_data: Option<Arc<String>>, // 添加缓存字段，使用 Arc 使其可共享
    pub op_metadata: OperationMetadata,
    // pub http_request: Option<Rc<reqwest::Request>>,
}

impl std::fmt::Debug for OperationOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationOutput")
            .field("input", &self.input)
            .field("status", &self.status)
            .field("headers", &self.headers)
            .field("body", &"<stream>") // Don't print the stream itself
            // .field("body_data", &self.body_data)
            .field("op_metadata", &self.op_metadata)
            // .field("http_request", &self.http_request)
            .finish()
    }
}

impl crate::client::BodyDataReader for OperationOutput {
    fn take_body(&mut self) -> Option<crate::client::BodyStream> {
        self.body.take()
    }

    fn set_body(&mut self, body: Option<crate::client::BodyStream>){
        self.body = body;
    }
}

// impl OperationOutput {
//     /// 获取已缓存的body数据（如果有的话），不消费body流
//     pub fn get_cached_body_data(&self) -> Option<String> {
//         self.body_data.as_ref().map(|data| data.as_str().to_string())
//     }
// }

// 实现 Clone trait，跳过无法克隆的字段
impl Clone for OperationOutput {
    fn clone(&self) -> Self {
        OperationOutput {
            input: self.input.clone(),
            status: self.status.clone(),
            headers: self.headers.clone(),
            body: None, // 无法克隆流，所以设置为 None
            // body_data: self.body_data.clone(), // Arc 可以安全克隆
            op_metadata: self.op_metadata.clone(),
            // http_request: self.http_request.clone(),
        }
    }
}

impl OperationInput {
    /// Validates the input fields of the `OperationInput` struct.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all input fields are valid.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the input fields are invalid.
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self
            .bucket
            .as_ref()
            .map_or(false, |b| !is_valid_bucket_name(b))
        {
            return Err("input.bucket is invalid".into());
        }
        if self
            .key
            .as_ref()
            .map_or(false, |b| !is_valid_object_name(b))
        {
            return Err("input.key is invalid".into());
        }
        if !is_valid_method(self.method.as_str()) {
            return Err("input.method is empty".into());
        }

        Ok(())
    }

    /// Retrieves the body content as a `String` from the `OperationInput`
    /// struct.
    ///
    pub fn get_body(&self) -> Option<String> {
        // Currently only returns the string representation of the body if it exists
        match &self.body {
            Some(BodyContent::Text { data, .. }) => Some(data.clone()),
            Some(BodyContent::Bytes { data, .. }) => Some(String::from_utf8_lossy(data.as_ref()).to_string()),
            Some(BodyContent::Stream { .. }) => {
                // 对于流类型，我们不能直接获取内容，因为它可能很大或不可用
                None
            },
            _ => None,
        }
    }
}

impl fmt::Debug for OperationInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OperationInput")
            .field("op_name", &self.op_name)
            .field("method", &self.method)
            .field("headers", &self.headers)
            .field("parameters", &self.parameters)
            .field("body", &self.body)
            .field("bucket", &self.bucket)
            .field("key", &self.key)
            .field("op_metadata", &self.op_metadata)
            .finish()
    }
}