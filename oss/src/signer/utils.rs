use std::collections::HashMap;

pub fn get_query(request: &reqwest::Request) -> HashMap<String, String> {
    request.url().query_pairs().into_owned().collect()
}

// TODO impl trait for url query
pub fn get_decoded_query_from_str(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    for pair in query.split('&') {
        let mut iter = pair.split('=');
        if let Some(key) = iter.next() {
            // Skip empty pairs (e.g. from an empty query string or "a=1&&b=2")
            if key.is_empty() {
                continue;
            }
            let decoded_key = urlencoding::decode(key).unwrap_or_else(|_| key.to_string().into());
            let decoded_value = if let Some(value) = iter.next() {
                urlencoding::decode(value)
                    .unwrap_or_else(|_| value.to_string().into())
                    .into_owned()
            } else {
                "".to_string()
            };
            params.insert(decoded_key.into(), decoded_value);
        }
    }
    params
}

pub fn get_encoded_query_from_str(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    for pair in query.split('&') {
        let mut iter = pair.split('=');
        if let Some(key) = iter.next() {
            // Skip empty pairs (e.g. from an empty query string or "a=1&&b=2")
            if key.is_empty() {
                continue;
            }
            let encoded_key = urlencoding::encode(key);
            let encoded_value = if let Some(value) = iter.next() {
                urlencoding::encode(value).into_owned()
            } else {
                "".to_string()
            };
            params.insert(encoded_key.into(), encoded_value);
        }
    }
    params
}

pub fn set_query(request: &mut reqwest::Request, query: &HashMap<String, String>) {
    request.url_mut().set_query(
        query
            .iter()
            .map(|(k, v)| {
                if !v.is_empty() {
                    format!("{}={}", k, v)
                } else {
                    k.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("&")
            .as_str()
            .into(),
    );
}
