//! Integration test for the log level ordering that filtering depends on.

use alibabacloud_oss_sdk_rust_v2::log::LogLevel;

/// Levels are ordered by verbosity, not by severity: a higher value logs more,
/// and each call emits when the configured level is at least its own. That is
/// what makes `Off` silent and `Debug` verbose.
///
/// The ordering comes from the enum's declaration order, which is easy to
/// disturb by inserting a variant in the wrong place, so it is pinned here.
#[test]
fn log_levels_are_ordered_by_verbosity() {
    assert!(LogLevel::Off < LogLevel::Error);
    assert!(LogLevel::Error < LogLevel::Warn);
    assert!(LogLevel::Warn < LogLevel::Info);
    assert!(LogLevel::Info < LogLevel::Debug);
}
