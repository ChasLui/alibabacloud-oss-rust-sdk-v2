use std::any::Any;
use std::fs::File;
use std::io::Write;
use std::ops::DerefMut;
use std::sync::RwLock;

use super::LogOutput;

/// The `LogPrinter` trait defines the behavior of a log printer.
pub trait LogPrinter: Any {
    /// Prints the log message with the specified format.
    ///
    /// # Arguments
    ///
    /// * `format` - The format string for the log message.
    fn print(&self, format: &str);
}

/// A struct representing a standard log printer.
#[derive(Default)]
pub struct StandardLogPrinter {
    output: RwLock<LogOutput>,
}

impl StandardLogPrinter {
    /// Creates a new `StandardLogPrinter` with the specified `LogOutput`.
    ///
    /// # Arguments
    ///
    /// * `output` - The output destination for the log messages.
    pub fn new(output: LogOutput) -> Self {
        Self {
            output: RwLock::new(output),
        }
    }
}

impl LogPrinter for StandardLogPrinter {
    /// Prints the specified log `message` to the appropriate output based on
    /// the configured `LogOutput`.
    ///
    /// # Arguments
    ///
    /// * `message` - The log message to print.
    ///
    /// # Note
    ///
    /// We use `println!` to print to stdout instead of `std::io::stdout()` to
    /// avoid too many output during the tests. This is a known issue of rust,
    /// check [#12309](https://github.com/rust-lang/rust/issues/12309) and
    /// [#90785](https://github.com/rust-lang/rust/issues/90785) for more.
    fn print(&self, message: &str) {
        let mut output_lock = self
            .output
            .write()
            .expect("Already locked by current thread");

        match output_lock.deref_mut() {
            LogOutput::Stdout => {
                // let mut handle = std::io::stdout().lock();
                // writeln!(handle, "{}", message).expect("Failed to write to
                // stdout");
                println!("{}", message);
            }
            LogOutput::File(ref path) => {
                let mut file = File::options()
                    .create(true)
                    .append(true)
                    .open(path)
                    .expect("Failed to open file");
                writeln!(file, "{}", message).expect("Failed to write to file");
            }
            LogOutput::Custom(ref mut custom_writer) => {
                let mut writer = custom_writer
                    .write()
                    .expect("Already locked by current thread");
                writeln!(writer.deref_mut(), "{}", message)
                    .expect("Failed to write to custom writer");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::ops::Deref;
    use std::sync::{Arc, RwLock};

    use super::*;
    use crate::log::StringWriter;

    #[test]
    fn test_standard_log_printer_stdout() {
        let printer = StandardLogPrinter::new(LogOutput::Stdout);
        printer.print("This is a test message");
    }

    #[test]
    fn test_standard_log_printer_file() {
        let path = "test_standard_log_printer_file.log";

        // delete file in path if exists
        if std::path::Path::new(path).exists() {
            std::fs::remove_file(path).unwrap();
        }

        let printer = StandardLogPrinter::new(LogOutput::File(path.to_string()));
        printer.print("This is a test message");

        // compare file content with output content
        let mut file = File::open(path).unwrap();
        let mut file_content = String::new();
        file.read_to_string(&mut file_content).unwrap();
        assert_eq!(file_content, "This is a test message\n");

        // delete file
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_standard_log_printer_custom() {
        let writer = Arc::new(RwLock::new(StringWriter::new()));
        let printer = StandardLogPrinter::new(LogOutput::Custom(writer.clone()));
        printer.print("This is a test message");

        let output = printer.output.read().unwrap();
        match output.deref() {
            LogOutput::Custom(_) => {
                let writer = writer.read().unwrap();
                assert_eq!(writer.get_logged_string(), "This is a test message\n");
            }
            _ => panic!("Unexpected log output"),
        }
    }
}
