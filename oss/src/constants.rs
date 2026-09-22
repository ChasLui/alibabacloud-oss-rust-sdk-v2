// OSS headers
pub const HEADER_OSS_PREFIX: &str = "X-Oss-";
pub const HEADER_OSS_META_PREFIX: &str = "X-Oss-Meta-";
pub const HEADER_OSS_ACL: &str = "X-Oss-Ccl";
pub const HEADER_OSS_OBJECT_ACL: &str = "X-Oss-Object-Acl";
pub const HEADER_OSS_OBJECT_TYPE: &str = "X-Oss-Object-Type";
pub const HEADER_OSS_SECURITY_TOKEN: &str = "X-Oss-Security-Token";
pub const HEADER_OSS_SERVER_SIDE_ENCRYPTION: &str = "X-Oss-Server-Side-Encryption";
pub const HEADER_OSS_SERVER_SIDE_ENCRYPTION_KEY_ID: &str = "X-Oss-Server-Side-Encryption-Key-Id";
pub const HEADER_OSS_SERVER_SIDE_DATA_ENCRYPTION: &str = "X-Oss-Server-Side-Data-Encryption";
pub const HEADER_OSS_SSE_C_ALGORITHM: &str = "X-Oss-Server-Side-Encryption-Customer-Algorithm";
pub const HEADER_OSS_SSE_C_KEY: &str = "X-Oss-Server-Side-Encryption-Customer-Key";
pub const HEADER_OSS_SSE_C_KEY_MD5: &str = "X-Oss-Server-Side-Encryption-Customer-Key-Md5";
pub const HEADER_OSS_COPY_SOURCE: &str = "X-Oss-Copy-Source";
pub const HEADER_OSS_COPY_SOURCE_RANGE: &str = "X-Oss-Copy-Source-Range";
pub const HEADER_OSS_COPY_SOURCE_IF_MATCH: &str = "X-Oss-Copy-Source-If-Match";
pub const HEADER_OSS_COPY_SOURCE_IF_NONE_MATCH: &str = "X-Oss-Copy-Source-If-None-Match";
pub const HEADER_OSS_COPY_SOURCE_IF_MODIFIED_SINCE: &str = "X-Oss-Copy-Source-If-Modified-Since";
pub const HEADER_OSS_COPY_SOURCE_IF_UNMODIFIED_SINCE: &str =
    "X-Oss-Copy-Source-If-Unmodified-Since";
pub const HEADER_OSS_METADATA_DIRECTIVE: &str = "X-Oss-Metadata-Directive";
pub const HEADER_OSS_NEXT_APPEND_POSITION: &str = "X-Oss-Next-Append-Position";
pub const HEADER_OSS_REQUEST_ID: &str = "X-Oss-Request-Id";
pub const HEADER_OSS_CRC64: &str = "X-Oss-Hash-Crc64ecma";
pub const HEADER_OSS_SYMLINK_TARGET: &str = "X-Oss-Symlink-Target";
pub const HEADER_OSS_STORAGE_CLASS: &str = "X-Oss-Storage-Class";
pub const HEADER_OSS_CALLBACK: &str = "X-Oss-Callback";
pub const HEADER_OSS_CALLBACK_VAR: &str = "X-Oss-Callback-Var";
pub const HEADER_OSS_REQUESTER: &str = "X-Oss-Request-Payer";
pub const HEADER_OSS_TAGGING: &str = "X-Oss-Tagging";
pub const HEADER_OSS_TAGGING_DIRECTIVE: &str = "X-Oss-Tagging-Directive";
pub const HEADER_OSS_TRAFFIC_LIMIT: &str = "X-Oss-Traffic-Limit";
pub const HEADER_OSS_FORBID_OVER_WRITE: &str = "X-Oss-Forbid-Overwrite";
pub const HEADER_OSS_RANGE_BEHAVIOR: &str = "X-Oss-Range-Behavior";
pub const HEADER_OSS_ALLOW_SAME_ACTION_OVER_LAP: &str = "X-Oss-Allow-Same-Action-Overlap";
pub const HEADER_OSS_DATE: &str = "X-Oss-Date";
pub const HEADER_OSS_CONTENT_SHA256: &str = "X-Oss-Content-Sha256";
pub const HEADER_OSS_EC: &str = "X-Oss-Ec";
pub const HEADER_OSS_ERR: &str = "X-Oss-Err";

// OSS query parameters
pub const HEADER_OSS_EXPIRES: &str = "X-Oss-Expires";
pub const HEADER_OSS_CREDENTIAL: &str = "X-Oss-Credential";
pub const HEADER_OSS_SIGNATURE: &str = "X-Oss-Signature";
pub const HEADER_OSS_SIGNATURE_VERSION: &str = "X-Oss-Signature-Version";
pub const HEADER_OSS_ADDITIONAL_HEADERS: &str = "X-Oss-Additional-Headers";

// OSS headers for client-Side encryption
pub const OSS_CLIENT_SIDE_ENCRYPTION_KEY: &str = "X-Oss-Meta-Client-Side-Encryption-Key";
pub const OSS_CLIENT_SIDE_ENCRYPTION_START: &str = "X-Oss-Meta-Client-Side-Encryption-Start";
pub const OSS_CLIENT_SIDE_ENCRYPTION_CEK_ALG: &str = "X-Oss-Meta-Client-Side-Encryption-Cek-Alg";
pub const OSS_CLIENT_SIDE_ENCRYPTION_WRAP_ALG: &str = "X-Oss-Meta-Client-Side-Encryption-Wrap-Alg";
pub const OSS_CLIENT_SIDE_ENCRYPTION_MAT_DESC: &str = "X-Oss-Meta-Client-Side-Encryption-Matdesc";
pub const OSS_CLIENT_SIDE_ENCRYPTION_UNENCRYPTED_CONTENT_LENGTH: &str =
    "X-Oss-Meta-Client-Side-Encryption-Unencrypted-Content-Length";
pub const OSS_CLIENT_SIDE_ENCRYPTION_UNENCRYPTED_CONTENT_MD5: &str =
    "X-Oss-Meta-Client-Side-Encryption-Unencrypted-Content-Md5";
pub const OSS_CLIENT_SIDE_ENCRYPTION_DATA_SIZE: &str =
    "X-Oss-Meta-Client-Side-Encryption-Data-Size";
pub const OSS_CLIENT_SIDE_ENCRYPTION_PART_SIZE: &str =
    "X-Oss-Meta-Client-Side-Encryption-Part-Size";

// HTTP headers
pub const HTTP_HEADER_ACCEPT_ENCODING: &str = "Accept-Encoding";
pub const HTTP_HEADER_AUTHORIZATION: &str = "Authorization";
pub const HTTP_HEADER_CACHE_CONTROL: &str = "Cache-Control";
pub const HTTP_HEADER_CONTENT_DISPOSITION: &str = "Content-Disposition";
pub const HTTP_HEADER_CONTENT_ENCODING: &str = "Content-Encoding";
pub const HTTP_HEADER_CONTENT_LENGTH: &str = "Content-Length";
pub const HTTP_HEADER_CONTENT_MD5: &str = "Content-MD5";
pub const HTTP_HEADER_CONTENT_TYPE: &str = "Content-Type";
pub const HTTP_HEADER_CONTENT_LANGUAGE: &str = "Content-Language";
pub const HTTP_HEADER_CONTENT_RANGE: &str = "Content-Range";
pub const HTTP_HEADER_DATE: &str = "Date";
pub const HTTP_HEADER_ETAG: &str = "ETag";
pub const HTTP_HEADER_EXPIRES: &str = "Expires";
pub const HTTP_HEADER_HOST: &str = "Host";
pub const HTTP_HEADER_LAST_MODIFIED: &str = "Last-Modified";
pub const HTTP_HEADER_RANGE: &str = "Range";
pub const HTTP_HEADER_LOCATION: &str = "Location";
pub const HTTP_HEADER_USER_AGENT: &str = "User-Agent";
pub const HTTP_HEADER_IF_MODIFIED_SINCE: &str = "If-Modified-Since";
pub const HTTP_HEADER_IF_UNMODIFIED_SINCE: &str = "If-Unmodified-Since";
pub const HTTP_HEADER_IF_MATCH: &str = "If-Match";
pub const HTTP_HEADER_IF_NONE_MATCH: &str = "If-None-Match";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum UrlStyleType {
    #[default]
    VirtualHosted,
    Path,
    CName,
}

impl UrlStyleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            UrlStyleType::VirtualHosted => "virtual-hosted-style",
            UrlStyleType::Path => "path-style",
            UrlStyleType::CName => "cname-style",
        }
    }
}

bitflags::bitflags! {
    #[derive(Clone)]
    pub struct FeatureFlagsType: u32 {
        /// If the client time is different from server time by more than about 15 minutes, the
        /// requests your application makes will be signed with the incorrect time, and the server
        /// will reject them. The feature to help to identify this case, and SDK will correct for
        /// clock skew.
        const CORRECT_CLOCK_SKEW = 1 << 0;
        const ENABLE_MD5 = 1 << 1;
        /// Content-Type is automatically added based on the object name if not specified. This
        /// feature takes effect for PutObject, AppendObject and InitiateMultipartUpload
        const AUTO_DETECT_MIME_TYPE = 1 << 2;
        /// Check data integrity of uploads via the crc64. This feature takes effect for PutObject,
        /// AppendObject, UploadPart, Uploader.UploadFrom and Uploader.UploadFile
        const ENABLE_CRC64_CHECK_UPLOAD = 1 << 3;
        /// Check data integrity of downloads via the crc64. This feature takes effect for
        /// Downloader.DownloadFile
        const ENABLE_CRC64_CHECK_DOWNLOAD = 1 << 4;
        const DEFAULT = Self::CORRECT_CLOCK_SKEW.bits() |
                        Self::AUTO_DETECT_MIME_TYPE.bits() |
                        Self::ENABLE_CRC64_CHECK_UPLOAD.bits() |
                        Self::ENABLE_CRC64_CHECK_DOWNLOAD.bits();
    }
}

impl Default for FeatureFlagsType {
    fn default() -> Self {
        FeatureFlagsType::DEFAULT
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatureVersionType {
    V1,
    V4,
}

impl SignatureVersionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignatureVersionType::V4 => "OSS Signature Version 4",
            SignatureVersionType::V1 => "OSS Signature Version 1",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthMethodType {
    Header,
    Query,
}

impl AuthMethodType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthMethodType::Query => "authentication in query",
            AuthMethodType::Header => "authentication in header",
        }
    }
}

// OperationMetadata Keys
pub const OP_META_KEY_RESPONSE_HANDLER: &str = "opm-response-handler";
pub const OP_META_KEY_REQUEST_BODY_TRACKER: &str = "opm-request-body-tracker";
pub const OP_META_KEY_PROGRESS_TRACKER: &str = "opm-progress-tracker";
pub const OP_META_KEY_RESPONSE_PROGRESS_TRACKER: &str = "opm-response-progress-tracker";

// Environment variables
pub const ENV_OSS_SDK_LOG_LEVEL: &str = "OSS_SDK_LOG_LEVEL";
pub const ENV_OSS_ACCESS_KEY_ID: &str = "OSS_ACCESS_KEY_ID";
pub const ENV_OSS_ACCESS_KEY_SECRET: &str = "OSS_ACCESS_KEY_SECRET";
pub const ENV_OSS_SESSION_TOKEN: &str = "OSS_SESSION_TOKEN";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utl_style_type_as_str() {
        assert_eq!(UrlStyleType::VirtualHosted.as_str(), "virtual-hosted-style");
        assert_eq!(UrlStyleType::Path.as_str(), "path-style");
        assert_eq!(UrlStyleType::CName.as_str(), "cname-style");
    }

    #[test]
    fn test_signature_version_type_as_str() {
        assert_eq!(SignatureVersionType::V1.as_str(), "OSS Signature Version 1");
        assert_eq!(SignatureVersionType::V4.as_str(), "OSS Signature Version 4");
    }

    #[test]
    fn test_auth_method_type_as_str() {
        assert_eq!(AuthMethodType::Header.as_str(), "authentication in header");
        assert_eq!(AuthMethodType::Query.as_str(), "authentication in query");
    }
}
