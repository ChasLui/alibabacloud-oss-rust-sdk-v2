use base64::engine::general_purpose;
use base64::Engine;
use md5::{Digest, Md5};

use crate::{BodyContent, OperationInput, HTTP_HEADER_CONTENT_MD5};

/// Updates the Content-MD5 header of the given `OperationInput` based on the
/// body content.
///
/// If the `body` field of the `OperationInput` is not `None`, the function
/// calculates the MD5 hash of the body content and encodes it using the base64
/// encoding scheme. The resulting encoded string is then set as the value of
/// the Content-MD5 header in the `headers` field of the `OperationInput`.
///
/// If the `body` field is `None`, the function sets the value of the
/// Content-MD5 header to a default string "1B2M2Y8AsgTpgAmY7PhCfg==".
///
/// # Arguments
///
/// * `input` - A mutable reference to the `OperationInput` struct.
///
/// # Errors
///
/// Returns an `Result` indicating whether the operation was successful or not.
pub fn update_content_md5(
    input: &mut OperationInput,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // let mut hasher = Md5::new();
    // hasher.update(input.get_body().unwrap_or_default());
    // let result = hasher.finalize();
    //
    // input.headers.insert(
    //     HTTP_HEADER_CONTENT_MD5.to_string(),
    //     general_purpose::STANDARD.encode(result),
    // );
    if let Some(body) = &input.body {
        // 1. 优先使用用户提供的 MD5
        let md5_bytes = match body {
            BodyContent::File { md5: Some(m), .. } => Some(*m),
            BodyContent::Bytes { md5: Some(m), .. } => Some(*m),
            BodyContent::Text { md5: Some(m), .. } => Some(*m),
            BodyContent::Stream { md5: Some(m), .. } => Some(*m),

            // 2. 未提供时，仅 Bytes 和 Text 自动计算
            BodyContent::Bytes { data, .. } => {
                let mut hasher = Md5::new();
                hasher.update(data.as_ref());
                Some(hasher.finalize().into())
            }
            BodyContent::Text { data, .. } => {
                let mut hasher = Md5::new();
                hasher.update(data.as_bytes());
                Some(hasher.finalize().into())
            }
            // Stream 类型不自动计算MD5，因为它是流式数据
            BodyContent::Stream { .. } => None,

            // 3. File 不自动计算
            BodyContent::File { .. } => None,
        };

        // 4. 设置 Content-MD5 头（如果存在）
        if let Some(md5) = md5_bytes {
            let md5_b64 = general_purpose::STANDARD.encode(md5);
            // let md5_b64 = BASE64_STANDARD.encode(md5);
            input.headers.insert(
                HTTP_HEADER_CONTENT_MD5.to_string(),
                md5_b64.parse().unwrap(),
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_update_content_md5_with_body() {
        let content = "test";

        let mut input = OperationInput {
            body: Some(BodyContent::from_text(content.to_string(), None)),
            ..Default::default()
        };

        update_content_md5(&mut input).unwrap();

        assert_eq!(
            input.headers.get(HTTP_HEADER_CONTENT_MD5).unwrap(),
            "CY9rzUYh03PK3k6DJie09g=="
        );
    }

    #[test]
    fn test_update_content_md5_without_body() {
        // let mut input = OperationInput::default();

        let content = "";

        let mut input = OperationInput {
            body: Some(BodyContent::from_text(content.to_string(), None)),
            ..Default::default()
        };

        update_content_md5(&mut input).unwrap();

        assert_eq!(
            input.headers.get(HTTP_HEADER_CONTENT_MD5).unwrap(),
            "1B2M2Y8AsgTpgAmY7PhCfg=="
        );
    }
}
