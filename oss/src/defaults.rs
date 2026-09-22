use crate::SignatureVersionType;

pub const MAX_UPLOAD_PARTS: i32 = 10000;

// Max part size, 5GB, For UploadPart
pub const MAX_PART_SIZE: i64 = 5 * 1024 * 1024 * 1024;

// Min part size, 100KB, For UploadPart
pub const MIN_PART_SIZE: i64 = 100 * 1024;

// Default part size, 6M
pub const DEFAULT_PART_SIZE: i64 = 6 * 1024 * 1024;

// Default part size for uploader uploads data
pub const DEFAULT_UPLOAD_PART_SIZE: i64 = DEFAULT_PART_SIZE;

// Default part size for downloader downloads object
pub const DEFAULT_DOWNLOAD_PART_SIZE: i64 = DEFAULT_PART_SIZE;

// Default part size for copier object, 64M
pub const DEFAULT_COPY_PART_SIZE: i64 = 64 * 1024 * 1024;

// Default parallel
pub const DEFAULT_PARALLEL: i32 = 3;

// Default parallel for uploader uploads data
pub const DEFAULT_UPLOAD_PARALLEL: i32 = DEFAULT_PARALLEL;

// Default parallel for downloader downloads object
pub const DEFAULT_DOWNLOAD_PARALLEL: i32 = DEFAULT_PARALLEL;

// Default parallel for copier copies object
pub const DEFAULT_COPY_PARALLEL: i32 = DEFAULT_PARALLEL;

// Default prefetch threshold to switch to async read in ReadOnlyFile
pub const DEFAULT_PREFETCH_THRESHOLD: i64 = 20 * 1024 * 1024;

// Default prefetch number for async read in ReadOnlyFile
pub const DEFAULT_PREFETCH_NUM: i32 = DEFAULT_PARALLEL;

// Default prefetch chunk size for async read in ReadOnlyFile
pub const DEFAULT_PREFETCH_CHUNK_SIZE: i64 = DEFAULT_PART_SIZE;

// Default threshold to use multipart copy in Copier, 256M
pub const DEFAULT_COPY_THRESHOLD: i64 = 256 * 1024 * 1024;

// File permission
pub const FILE_PERM_MODE: u32 = 0o664;

// Temp file suffix
pub const TEMP_FILE_SUFFIX: &str = ".temp";

// Checkpoint file suffix for Downloader
pub const CHECKPOINT_FILE_SUFFIX_DOWNLOADER: &str = ".dcp";

// Checkpoint file suffix for Uploader
pub const CHECKPOINT_FILE_SUFFIX_UPLOADER: &str = ".ucp";

// Checkpoint file Magic
pub const CHECKPOINT_MAGIC: &str = "92611BED-89E2-46B6-89E5-72F273D4B0A3";

// Product for signing
pub const DEFAULT_PRODUCT: &str = "oss";

// Product of cloud box for signing
pub const CLOUD_BOX_PRODUCT: &str = "oss-cloudbox";

// The URL's scheme, default is https
pub const DEFAULT_ENDPOINT_SCHEME: &str = "https";

// Default signature version is v4
// Assuming `SIGNATURE_VERSION_V4` is defined elsewhere in the code
pub const DEFAULT_SIGNATURE_VERSION: SignatureVersionType = SignatureVersionType::V4;

pub const DEFAULT_CONTENT_TYPE: &str = "application/octet-stream";
