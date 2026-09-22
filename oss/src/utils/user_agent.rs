lazy_static::lazy_static! {
    pub(crate) static ref DEFAULT_USER_AGENT: String = default_user_agent();
}

fn default_user_agent() -> String {
    format!(
        "{}/{} ({}/{}/{};{})",
        option_env!("CARGO_PKG_NAME").unwrap_or("alibabacloud-oss-sdk-rust-v2"),
        option_env!("CARGO_PKG_VERSION").unwrap_or("1.0.0"),
        std::env::consts::OS,
        match std::env::consts::OS {
            "windows" => option_env!("WINDOWS_VERSION").unwrap_or("-"),
            _ => option_env!("UNIX_KERNEL_RELEASE").unwrap_or("-"),
        },
        std::env::consts::ARCH,
        option_env!("RUSTC_VERSION").unwrap_or("-"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_user_agent() {
        assert!(DEFAULT_USER_AGENT.starts_with("alibabacloud-oss-sdk-rust-v2/"));
    }
}
