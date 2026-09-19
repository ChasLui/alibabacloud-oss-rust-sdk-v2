use std::rc::Rc;

use super::{LogLevel, LogPrinter, Logger, StandardLogPrinter};

/// This module contains the implementation of the `StandardLogger` struct,
/// which is a logger that provides standard logging functionality. It allows
/// logging messages to various outputs such as standard output, files, or
/// custom writers.
///
/// # Example
///
/// ```
/// # use std::rc::Rc;
/// #
/// # use alibabacloud_oss_sdk_rust_v2::log::{
/// #     LogLevel, LogOutput, LogPrinter, Logger, StandardLogPrinter, StandardLogger,
/// # };
/// #
/// // Create a new `StandardLogger` instance with a specific output and log level
/// let logger = StandardLogger::new(
///     Rc::new(StandardLogPrinter::new(LogOutput::Stdout)),
///     LogLevel::Debug,
/// );
///
/// // Log messages at different log levels
/// logger.debug("This is a debug message");
/// logger.info("This is an info message");
/// logger.warn("This is a warning message");
/// logger.error("This is an error message");
/// ```
///
/// # Implementation Details
///
/// The `StandardLogger` struct implements the `LogPrinter` and `Logger` traits.
/// It uses a `RwLock` to ensure thread-safe access to the log output. The
/// `print` method is responsible for printing the log message to the
/// appropriate output based on the configured `LogOutput`. The `debug`, `info`,
/// `warn`, and `error` methods are used to log messages at different log
/// levels, depending on the configured log level. The `level` method returns
/// the current log level of the logger.
pub struct StandardLogger {
    printer: Rc<dyn LogPrinter>,
    level: LogLevel,
}

impl StandardLogger {
    /// Creates a new `StandardLogger` instance with the specified `output` and
    /// `level`.
    ///
    /// # Arguments
    ///
    /// * `printer` - The log printer to use for printing log messages.
    /// * `level` - The log level to use (e.g., `LogLevel::Debug`,
    ///   `LogLevel::Info`, etc.).
    ///
    /// # Returns
    ///
    /// A new `StandardLogger` instance.
    pub fn new(printer: Rc<dyn LogPrinter>, level: LogLevel) -> Self {
        Self { printer, level }
    }
}

impl Default for StandardLogger {
    fn default() -> Self {
        Self::new(Rc::new(StandardLogPrinter::default()), LogLevel::Info)
    }
}

impl Logger for StandardLogger {
    /// Logs a debug `message` if the log level is set to `LogLevel::Debug` or
    /// higher.
    ///
    /// # Arguments
    ///
    /// * `message` - The debug message to log.
    fn debug(&self, message: &str) {
        if self.level >= LogLevel::Debug {
            self.printer
                .print(format!("[DEBUG] {}\n", message).as_str());
        }
    }

    /// Logs an info `message` if the log level is set to `LogLevel::Info` or
    /// higher.
    ///
    /// # Arguments
    ///
    /// * `message` - The info message to log.
    fn info(&self, message: &str) {
        if self.level >= LogLevel::Info {
            self.printer
                .print(format!("[INFO]  {}\n", message).as_str());
        }
    }

    /// Logs a warning `message` if the log level is set to `LogLevel::Warn` or
    /// higher.
    ///
    /// # Arguments
    ///
    /// * `message` - The warning message to log.
    fn warn(&self, message: &str) {
        if self.level >= LogLevel::Warn {
            self.printer
                .print(format!("[WARN]  {}\n", message).as_str());
        }
    }

    /// Logs an error `message` if the log level is set to `LogLevel::Error` or
    /// higher.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message to log.
    fn error(&self, message: &str) {
        if self.level >= LogLevel::Error {
            self.printer
                .print(format!("[ERROR] {}\n", message).as_str());
        }
    }

    /// Returns the current log level of the logger.
    ///
    /// # Returns
    ///
    /// The current log level.
    fn level(&self) -> LogLevel {
        self.level
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use std::sync::{Arc, RwLock};

    use super::*;
    use crate::log::{LogOutput, StandardLogPrinter, StringWriter};

    #[test]
    fn test_logger_impl_stdout_single_level() {
        let logger = StandardLogger::new(
            Rc::new(StandardLogPrinter::new(LogOutput::Stdout)),
            LogLevel::Error,
        );
        logger.debug("test debug");
        logger.info("test info");
        logger.warn("test warn");
        logger.error("test error");
    }

    #[test]
    fn test_logger_impl_file_all_level() {
        let path = "test_logger_impl_file_all_level.log";

        // delete file in path if exists
        if std::path::Path::new(path).exists() {
            std::fs::remove_file(path).unwrap();
        }

        // write content
        let logger = StandardLogger::new(
            Rc::new(StandardLogPrinter::new(LogOutput::File(path.to_string()))),
            LogLevel::Debug,
        );
        logger.debug("test debug");
        logger.info("test info");
        logger.warn("test warn");
        logger.error("test error");

        // compare file content with output content
        let mut file = File::open(path).unwrap();
        let mut file_content = String::new();
        file.read_to_string(&mut file_content).unwrap();
        assert_eq!(
            file_content,
            "[DEBUG] test debug\n\n[INFO]  test info\n\n[WARN]  test warn\n\n[ERROR] test \
             error\n\n"
        );

        // delete file
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_logger_impl_file_partial_level() {
        let path = "test_logger_impl_file_partial_level.log";

        // delete file in path if exists
        if std::path::Path::new(path).exists() {
            std::fs::remove_file(path).unwrap();
        }

        // write content
        let logger = StandardLogger::new(
            Rc::new(StandardLogPrinter::new(LogOutput::File(path.to_string()))),
            LogLevel::Warn,
        );
        logger.debug("test debug");
        logger.info("test info");
        logger.warn("test warn");
        logger.error("test error");

        // compare file content with output content
        let mut file = File::open(path).unwrap();
        let mut file_content = String::new();
        file.read_to_string(&mut file_content).unwrap();
        assert_eq!(file_content, "[WARN]  test warn\n\n[ERROR] test error\n\n");

        // delete file
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_logger_impl_string_writer() {
        let writer = Arc::new(RwLock::new(StringWriter::new()));
        let logger = StandardLogger::new(
            Rc::new(StandardLogPrinter::new(LogOutput::Custom(writer.clone()))),
            LogLevel::Info,
        );
        logger.debug("test debug");
        logger.info("test info");
        logger.warn("test warn");
        logger.error("test error");

        assert_eq!(
            writer.read().unwrap().get_logged_string(),
            "[INFO]  test info\n\n[WARN]  test warn\n\n[ERROR] test error\n\n"
        );
    }

    #[test]
    fn test_get_level() {
        let logger = StandardLogger::new(Rc::new(StandardLogPrinter::default()), LogLevel::Debug);
        assert_eq!(logger.level(), LogLevel::Debug);
    }

    #[test]
    fn test_default_logger() {
        let logger = StandardLogger::default();
        assert_eq!(logger.level(), LogLevel::Info);
    }
}
