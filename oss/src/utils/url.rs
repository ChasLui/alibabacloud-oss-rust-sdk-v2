use crate::client::ClientOptions;
use crate::utils::escape_path;
use crate::{OperationInput, UrlStyleType};

pub(crate) fn build_url(input: &OperationInput, opts: &ClientOptions) -> (String, String) {
    if opts.endpoint.is_none() {
        return ("".to_string(), "".to_string());
    }

    let endpoint = opts.endpoint.as_ref().expect("Endpoint not set");
    let endpoint_host = endpoint.host_str().expect("Endpoint host not set");
    // `host_str()` drops the port, which breaks non-default endpoints such as
    // local mocks (`http://127.0.0.1:9000`); match Go, which keeps it.
    let endpoint_string = match endpoint.port() {
        Some(port) => format!("{}:{}", endpoint_host, port),
        None => endpoint_host.to_string(),
    };

    let mut paths = vec![];

    let host = if let Some(bucket) = input.bucket.as_ref() {
        match opts.url_style {
            UrlStyleType::VirtualHosted => {
                format!("{}.{}", bucket, endpoint_string)
            }
            UrlStyleType::Path => {
                paths.push(bucket.clone());
                endpoint_string
            }
            UrlStyleType::CName => endpoint_string,
        }
    } else {
        endpoint_string
    };

    if let Some(key) = &input.key {
        paths.push(escape_path(key, false));
    }

    let path = format!("/{}", paths.join("/"));
    (host, path)
}
