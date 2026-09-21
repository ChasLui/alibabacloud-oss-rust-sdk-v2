use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use super::{
    TransportConfig, DEFAULT_CONNECT_TIMEOUT, DEFAULT_KEEP_ALIVE_TIMEOUT,
    DEFAULT_READ_WRITE_TIMEOUT,
};

#[allow(clippy::type_complexity)]
#[derive(Clone)]
/// Represents a dialer used for establishing network connections.
pub struct Dialer {
    pub connect_timeout: Duration,
    pub keep_alive_timeout: Duration,
    /// The timeout for read and write operations.
    pub read_write_timeout: Duration,
    /// A list of functions to be executed after reading from the connection.
    ///
    /// Each callback receives the result of the read and the number of bytes
    /// it moved, so a bandwidth limiter can pace the traffic.
    pub post_read: Vec<Arc<dyn Fn(&io::Result<usize>, usize) + Send + Sync>>,
    /// A list of functions to be executed after writing to the connection.
    ///
    /// Each callback receives the result of the write and the number of bytes
    /// it moved, so a bandwidth limiter can pace the traffic.
    pub post_write: Vec<Arc<dyn Fn(&io::Result<usize>, usize) + Send + Sync>>,
}

impl Dialer {
    /// Creates a new `Dialer` instance with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the dialer.
    ///
    /// # Returns
    ///
    /// A new `Dialer` instance.
    pub fn new(config: &TransportConfig) -> Self {
        Dialer {
            connect_timeout: config.connect_timeout.unwrap_or(DEFAULT_CONNECT_TIMEOUT),
            keep_alive_timeout: config
                .keep_alive_timeout
                .unwrap_or(DEFAULT_KEEP_ALIVE_TIMEOUT),
            read_write_timeout: config
                .read_write_timeout
                .unwrap_or(DEFAULT_READ_WRITE_TIMEOUT),
            post_read: config.post_read.clone().unwrap_or_default(),
            post_write: config.post_write.clone().unwrap_or_default(),
        }
    }

    /// Establishes a connection to the specified socket address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The socket address to connect to.
    ///
    /// # Returns
    ///
    /// A result containing the connected stream with timeout.
    pub fn dial(&self, addr: &SocketAddr) -> io::Result<TimeoutConnection> {
        let stream = TcpStream::connect_timeout(addr, self.connect_timeout)?;
        stream.set_nodelay(true)?;
        Ok(TimeoutConnection {
            stream,
            timeout: self.read_write_timeout,
            dialer: self.clone(),
        })
    }
}

/// Represents a stream with a timeout.
pub struct TimeoutConnection {
    stream: TcpStream,
    timeout: Duration,
    dialer: Dialer,
}

impl TimeoutConnection {
    /// Nudges the deadline of the underlying stream by setting read and write
    /// timeouts.
    ///
    /// # Errors
    ///
    /// This function will return an `io::Result<()>` indicating success or an
    /// error if setting the timeouts fails.
    fn nudge_deadline(&self) -> io::Result<()> {
        self.stream.set_read_timeout(Some(self.timeout))?;
        self.stream.set_write_timeout(Some(self.timeout))?;
        Ok(())
    }
}

impl Read for TimeoutConnection {
    /// Reads data from the underlying stream into the provided buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - A mutable reference to the buffer where the read data will be
    ///   stored.
    ///
    /// # Returns
    ///
    /// The number of bytes read from the stream, or an `io::Error` if an error
    /// occurred.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = self.stream.read(buf);
        let n = res.as_ref().map(|&n| n).unwrap_or(0);
        for callback in &self.dialer.post_read {
            callback(&res, n);
        }
        if res.is_ok() && n > 0 && self.timeout > Duration::from_secs(0) {
            self.nudge_deadline()?;
        }
        res
    }
}

impl Write for TimeoutConnection {
    /// Writes a buffer into the underlying stream with a timeout.
    ///
    /// This method writes the contents of `buf` into the underlying stream and
    /// returns the number of bytes written. It also invokes any registered
    /// post-write callbacks. If the write operation is successful and at
    /// least one byte is written, and the timeout duration is greater than
    /// zero, it nudges the deadline for the timeout.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer containing the data to be written.
    ///
    /// # Returns
    ///
    /// The number of bytes written if the write operation is successful, or an
    /// `io::Error` if an error occurs.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = self.stream.write(buf);
        let n = res.as_ref().map(|&n| n).unwrap_or(0);
        for callback in &self.dialer.post_write {
            callback(&res, n);
        }
        if res.is_ok() && n > 0 && self.timeout > Duration::from_secs(0) {
            self.nudge_deadline()?;
        }
        res
    }

    /// Flushes the underlying stream.
    ///
    /// This method flushes any buffered data in the underlying stream. It
    /// ensures that all previously written data is sent to the receiver.
    ///
    /// # Returns
    ///
    /// An `io::Result` indicating the success or failure of the flush
    /// operation.
    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;
    use std::thread;

    use super::*;

    #[test]
    fn test_server() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut buffer = [0; 1024];
                let _ = stream.read(&mut buffer).unwrap();

                let response = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
                let _ = stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
        });

        assert!(addr.ip().is_loopback());
        assert_ne!(addr.port(), 0);

        // Dialer with default config
        let cfg = TransportConfig::default();
        let dialer = Dialer::new(&cfg);
        assert_eq!(dialer.read_write_timeout, DEFAULT_READ_WRITE_TIMEOUT);
        assert_eq!(dialer.connect_timeout, DEFAULT_CONNECT_TIMEOUT);
        assert_eq!(dialer.keep_alive_timeout, DEFAULT_KEEP_ALIVE_TIMEOUT);

        // Dial result with default config
        let dial_result = dialer.dial(&addr);
        assert!(dial_result.is_ok());
        let connection = dial_result.unwrap();
        assert_eq!(connection.timeout, DEFAULT_READ_WRITE_TIMEOUT);
        assert!(connection.dialer.post_read.is_empty());
        assert!(connection.dialer.post_write.is_empty());

        // Specified config
        let rw_timeout = Duration::from_secs(10);
        let config_with_one_post_read = TransportConfig {
            read_write_timeout: Some(rw_timeout),
            post_read: Some(vec![Arc::new(|_, _| {})]),
            post_write: Some(vec![Arc::new(|_, _| {}), Arc::new(|_, _| {})]),
            ..Default::default()
        };
        let connection = Dialer::new(&config_with_one_post_read).dial(&addr).unwrap();
        assert_eq!(connection.timeout, rw_timeout);
        assert_eq!(connection.dialer.post_read.len(), 1);
        assert_eq!(connection.dialer.post_write.len(), 2);
    }
}
