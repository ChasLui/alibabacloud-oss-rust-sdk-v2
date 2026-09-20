use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use chrono::TimeDelta;
use url::Url;

use crate::credential::CredentialsProvider;
use crate::log::Logger;
use crate::retry::Retryer;
use crate::signer::Signer;
use crate::utils::BwTokenBuckets;
use crate::{AuthMethodType, FeatureFlagsType, UrlStyleType};
use crate::client::OssResponse;

/// A callback invoked on the response before it is returned to the caller.
///
/// Handlers may reject a response by returning an error, which is how
/// integrity checks (CRC64) and error conversion surface failures. This is
/// the `OssResponse`-based counterpart of Go's
/// `func(*http.Response) error` response handlers.
pub type ResponseHandler =
    Rc<dyn Fn(&OssResponse) -> Result<(), Box<dyn std::error::Error + Send + Sync>>>;

/// A batch of [`ResponseHandler`]s, as stored in `OperationMetadata` under
/// `OP_META_KEY_RESPONSE_HANDLER`.
pub type ResponseHandlers = Vec<ResponseHandler>;


#[derive(Clone, Default)]
pub struct ClientOptions {
    pub product: String,
    pub region: String,
    pub endpoint: Option<Url>,
    pub retry_max_attempts: Option<u32>,
    pub retryer: Option<Rc<dyn Retryer>>,
    pub signer: Option<Rc<dyn Signer>>,
    pub credentials_provider: Option<Rc<dyn CredentialsProvider>>,
    pub http_client: Option<reqwest::Client>,
    pub response_handlers: ResponseHandlers,
    pub url_style: UrlStyleType,
    pub feature_flags: FeatureFlagsType,
    pub op_read_write_timeout: Option<Duration>,
    pub auth_method: Option<AuthMethodType>,
    pub additional_headers: Vec<String>,
}

pub fn op_read_write_timeout(value: Duration) -> impl Fn(&mut ClientOptions) {
    move |client_options: &mut ClientOptions| {
        client_options.op_read_write_timeout = Some(value);
    }
}

#[derive(Clone, Default)]
pub struct ClientInnerOptions {
    pub bw_token_buckets: BwTokenBuckets,
    /// Clock correction learned from `RequestTimeTooSkewed` responses.
    ///
    /// Signed (`TimeDelta`) because the server clock can run either ahead of
    /// or behind the local one. `Cell` because the correction is written while
    /// a request is being retried through a shared `&Client`, and the client
    /// is single-threaded (`Rc`-based).
    pub clock_offset: Cell<TimeDelta>,
    pub logger: Option<Rc<dyn Logger>>,
    pub user_agent: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_read_write_timeout() {
        let mut options = ClientOptions::default();
        let timeout = Duration::from_secs(5);
        let set_timeout = op_read_write_timeout(timeout);
        set_timeout(&mut options);
        assert_eq!(options.op_read_write_timeout.unwrap(), timeout);
    }
}
