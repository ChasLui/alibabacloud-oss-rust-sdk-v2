// Integration tests for logging functionality

use alibabacloud_oss_sdk_rust_v2::log::{LogLevel, Logger};

#[test]
fn test_log_level_variants() {
    // Test all log level variants exist
    let _debug = LogLevel::Debug;
    let _info = LogLevel::Info;
    let _warn = LogLevel::Warn;
    let _error = LogLevel::Error;
}

#[test]
fn test_log_level_ordering() {
    assert!(LogLevel::Debug < LogLevel::Info);
    assert!(LogLevel::Info < LogLevel::Warn);
    assert!(LogLevel::Warn < LogLevel::Error);
}

#[test]
fn test_log_level_equality() {
    assert_eq!(LogLevel::Debug, LogLevel::Debug);
    assert_eq!(LogLevel::Info, LogLevel::Info);
    assert_ne!(LogLevel::Debug, LogLevel::Info);
}

#[test]
fn test_log_level_debug_format() {
    let level = LogLevel::Info;
    let debug_str = format!("{:?}", level);
    assert!(debug_str.contains("Info"));
}

#[test]
fn test_logger_creation() {
    let logger = Logger::new();
    assert!(logger.is_ok());
}

#[test]
fn test_logger_default_level() {
    let logger = Logger::new().unwrap();
    // Default level should be Info or higher
    assert!(logger.level() >= LogLevel::Info);
}

#[test]
fn test_logger_set_level() {
    let mut logger = Logger::new().unwrap();
    logger.set_level(LogLevel::Debug);
    assert_eq!(logger.level(), LogLevel::Debug);
}

#[test]
fn test_logger_filter_messages() {
    let mut logger = Logger::new().unwrap();
    logger.set_level(LogLevel::Warn);
    
    // Debug and Info should be filtered out
    assert!(!logger.should_log(LogLevel::Debug));
    assert!(!logger.should_log(LogLevel::Info));
    
    // Warn and Error should pass through
    assert!(logger.should_log(LogLevel::Warn));
    assert!(logger.should_log(LogLevel::Error));
}

#[test]
fn test_log_level_from_string() {
    // Test parsing log levels from strings
    assert!("debug".parse::<LogLevel>().is_ok());
    assert!("info".parse::<LogLevel>().is_ok());
    assert!("warn".parse::<LogLevel>().is_ok());
    assert!("error".parse::<LogLevel>().is_ok());
}

#[test]
fn test_log_level_display() {
    let level = LogLevel::Error;
    let display = format!("{}", level);
    assert!(!display.is_empty());
}
