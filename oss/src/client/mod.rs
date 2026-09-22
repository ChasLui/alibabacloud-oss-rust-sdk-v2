mod applier;
mod checkpoint;
mod copier;
mod downloader;
mod encryption;
mod error_handler;
mod extension;
mod filelike;
mod filelike_writeonly;
mod invoker;
mod options;
mod paginators;
mod presign;
mod resolver;
mod uploader;

use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use http::{HeaderMap, StatusCode};
use log::error;
use reqwest::Response;

use self::applier::*;
pub use self::checkpoint::*;
pub use self::copier::*;
pub use self::downloader::*;
pub use self::encryption::*;
pub use self::error_handler::*;
pub use self::extension::*;
pub use self::filelike::*;
pub use self::filelike_writeonly::*;
pub use self::options::*;
pub use self::paginators::*;
pub use self::presign::*;
pub use self::resolver::*;
pub use self::uploader::*;
use crate::config::Config;
use crate::log::{StandardLogPrinter, StandardLogger};
use crate::{FeatureFlagsType, DEFAULT_PRODUCT};
// pub type ByteStream = BoxStream<'static, Result<bytes::Bytes, Box<dyn
// std::error::Error + Send + Sync>>>;
#[allow(clippy::type_complexity)] // nested boxing is intentional: stream errors
                                  // are erased
pub struct ByteStream(
    Pin<Box<dyn Stream<Item = Result<Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send>>,
);
// SAFETY: ByteStream is never polled concurrently.
// The underlying stream is only accessed via &mut during poll,
// and reqwest guarantees sequential polling.
unsafe impl Sync for ByteStream {}

impl Stream for ByteStream {
    type Item = Result<Bytes, Box<dyn std::error::Error + Send + Sync>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0.as_mut().poll_next(cx)
    }
}

impl ByteStream {
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
    {
        Self(Box::pin(stream))
    }
}

// 定义一个通用 trait 用于从包含 body 流的结构中读取数据
#[allow(async_fn_in_trait)] // impls are concrete types; no `dyn` + `Send` bound
                            // is needed
pub trait BodyDataReader {
    /// 读取 body 流中的所有数据并返回字节数组
    async fn get_all_data(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(mut body_stream) = self.take_body() {
            let mut full_data = Vec::new();

            while let Some(chunk_result) = body_stream.next().await {
                match chunk_result {
                    Ok(chunk) => full_data.extend_from_slice(&chunk),
                    Err(e) => {
                        error!("Error reading body chunk: {}", e);
                        return Err(e.into());
                    }
                }
            }

            Ok(full_data)
        } else {
            Ok(Vec::new())
        }
    }

    /// 获取 body 流，如果有的话
    fn take_body(&mut self) -> Option<BodyStream>;

    /// set body 流，如果有的话
    fn set_body(&mut self, body: Option<BodyStream>);

    /// 尝试获取 body 流中的下一个字节块，用于流式处理
    /// 返回 Result<Option<Bytes>, Error>，其中 None 表示流结束
    /// 默认实现会尝试从流中获取下一个字节块，但一旦流被消费，
    /// 就无法通过此方法继续访问 因此，在调用 try_next 后，后续调用将返回
    /// None，除非实现类提供自己的逻辑来管理状态
    async fn try_next(
        &mut self,
    ) -> Result<Option<Bytes>, Box<dyn std::error::Error + Send + Sync>> {
        // 获取 body 流并从中获取下一块数据
        // 注意：由于 take_body 会消费流，我们需要实现类自行处理状态管理
        // 默认实现仅适用于一次性读取
        if let Some(mut body_stream) = self.take_body() {
            match body_stream.next().await {
                Some(result) => {
                    match result {
                        Ok(bytes) => {
                            // 由于我们无法将流放回原处，我们将其丢弃
                            // 后续调用 try_next 将返回 None
                            self.set_body(Some(body_stream));
                            Ok(Some(bytes))
                        }
                        Err(e) => {
                            // 发生错误
                            Err(Box::new(e))
                        }
                    }
                }
                None => {
                    // 流已经结束
                    Ok(None)
                }
            }
        } else {
            // 没有 body 流
            Ok(None)
        }
    }
}

// 假设你的 SDK 错误类型是 SdkError
pub type BodyStream = Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>;

#[derive(Clone)]
pub struct Client {
    pub options: ClientOptions,
    pub inner_options: ClientInnerOptions,
}

pub enum OssResponse<'a> {
    SucResponse(&'a Response),
    ErrResponse {
        status: StatusCode,
        // method: String,
        body: String,
        headers: HeaderMap,
        url: String,
    },
}

// impl OssResponse {
//     pub async fn from_response_and_context(
//         response: Response,
//         // method: &str,
//     ) -> OssResponse {
//         let status = response.status();
//         if !status.is_success() {
//             let url = response.url().to_string();
//             let headers = response.headers().clone();
//             let body = response.text().await?; // 注意：这会消耗 response
//
//             return OssResponse::ErrResponse {
//                 status,
//                 // method: method.to_string(),
//                 body,
//                 headers,
//                 url,
//             }
//         }
//         panic!("it will not go to here")
//     }
// }

impl Client {
    /// Mutable access to the resolved client options.
    ///
    /// Products whose addressing rules differ from OSS's use this after
    /// [`Client::new`] to install a [`BucketNameResolver`] or an
    /// [`EndpointProvider`].
    pub fn options_mut(&mut self) -> &mut ClientOptions {
        &mut self.options
    }

    /// Creates a client for a configuration and then applies `option_modifiers`
    /// on top of the resolved options.
    #[allow(clippy::type_complexity)] // boxed closures are the point: they
                                      // capture the product's provider
    pub fn new_with_options(
        config: &Config,
        option_modifiers: Vec<Box<dyn Fn(&mut ClientOptions)>>,
    ) -> Self {
        let mut client = Client::new(config);
        for modifier in option_modifiers {
            modifier(&mut client.options);
        }
        client
    }

    pub fn new(config: &Config) -> Self {
        let mut options = ClientOptions {
            product: DEFAULT_PRODUCT.to_string(),
            region: config.region.clone().unwrap_or_default(),
            retry_max_attempts: config.retry_max_attempts,
            retryer: config.retryer.clone(),
            credentials_provider: config.credentials_provider.clone(),
            http_client: config.http_client.clone(),
            feature_flags: FeatureFlagsType::DEFAULT,
            additional_headers: config.additional_headers.clone(),
            // Empty keys or values would panic when the header is set on the
            // request, so they are dropped here, once. Mirrors Go's
            // `resolveDefaultRequestHeaders`.
            default_request_headers: config
                .default_request_headers
                .iter()
                .filter(|(key, value)| !key.is_empty() && !value.is_empty())
                .cloned()
                .collect(),
            account_id: config.account_id.clone(),
            ..Default::default()
        };

        let inner_options = ClientInnerOptions {
            logger: Some(Rc::new(StandardLogger::new(
                config
                    .log_printer
                    .clone()
                    .unwrap_or(Rc::new(StandardLogPrinter::default())),
                config.log_level.unwrap_or_default(),
            ))),
            user_agent: build_user_agent(config),
            ..Default::default()
        };

        resolve_endpoint(config, &mut options);
        resolve_retryer(config, &mut options);
        resolve_http_client(config, &mut options);
        resolve_bandwidth_limit(config, &mut options);
        resolve_signer(config, &mut options);
        resolve_url_style(config, &mut options);
        resolve_feature_flags(config, &mut options);
        resolve_cloud_box(config, &mut options);

        // TODO add opt functions

        Self {
            options,
            inner_options,
        }
    }
}
