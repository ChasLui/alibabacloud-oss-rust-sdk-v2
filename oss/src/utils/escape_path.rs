lazy_static::lazy_static! {
    static ref NO_ESCAPE: [bool; 256] = {
        let mut no_escape = [false; 256];
        for (index, item) in no_escape.iter_mut().enumerate() {
            let c = index as u8;
            *item = c.is_ascii_alphanumeric() || c == b'-' || c == b'.' || c == b'_' || c == b'~';
        }
        no_escape
    };
}

/// Escapes a path by replacing characters that need to be escaped with their
/// percent-encoded representation. If `encode_sep` is `true`, the path
/// separator '/' will also be percent-encoded.
///
/// Operates on UTF-8 bytes (like the Go SDK), so multi-byte characters are
/// percent-encoded byte by byte and never index outside the ASCII table.
pub(crate) fn escape_path(path: &str, encode_sep: bool) -> String {
    let mut escaped = String::with_capacity(path.len());
    for byte in path.bytes() {
        if NO_ESCAPE[byte as usize] || (byte == b'/' && !encode_sep) {
            escaped.push(byte as char);
        } else {
            escaped.push_str(&format!("%{:02X}", byte));
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_path_no_escape() {
        let path = "abc-123";
        let encoded = escape_path(path, true);
        assert_eq!(encoded, "abc-123");
    }

    #[test]
    fn test_escape_path_escape() {
        let path = "abc!@#";
        let encoded = escape_path(path, true);
        assert_eq!(encoded, "abc%21%40%23");
    }

    #[test]
    fn test_escape_path_no_encode_sep() {
        let path = "abc/123";
        let encoded = escape_path(path, false);
        assert_eq!(encoded, "abc/123");
    }

    #[test]
    fn test_escape_path_encode_sep() {
        let path = "abc/123";
        let encoded = escape_path(path, true);
        assert_eq!(encoded, "abc%2F123");
    }

    #[test]
    fn test_escape_path_non_ascii() {
        // Multi-byte characters must be percent-encoded per UTF-8 byte, not
        // panicked on (regression: `chars()` + `c as usize` indexed out of
        // the 256-entry table).
        let path = "中文.txt";
        let encoded = escape_path(path, false);
        assert_eq!(encoded, "%E4%B8%AD%E6%96%87.txt");
    }
}
