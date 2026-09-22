use std::time::Duration;

use super::backoff::*;
use super::error_retryable::{
    ConnectionErrorRetryable, HTTPStatusCodeRetryable, ServiceErrorCodeRetryable,
};
use super::traits::{BackoffDelayer, ErrorRetryable, Retryer};

pub const DEFAULT_MAX_ATTEMPTS: u32 = 3;
pub const DEFAULT_MAX_BACKOFF: Duration = Duration::from_secs(20);
pub const DEFAULT_BASE_DELAY: Duration = Duration::from_millis(200);

/// Struct representing the retry options for the Standard retryer.
pub struct RetryOptions {
    pub max_attempts: usize,
    pub max_backoff: Duration,
    pub base_delay: Duration,
    pub backoff: Box<dyn BackoffDelayer>,
    pub error_retryables: Vec<Box<dyn ErrorRetryable>>,
}

/// Struct representing the Standard retryer.
pub struct Standard {
    max_attempts: u32,
    retryables: Vec<Box<dyn ErrorRetryable>>,
    backoff: Box<dyn BackoffDelayer>,
}

impl Default for Standard {
    /// Creates a new instance of the Standard retryer with default values.
    fn default() -> Self {
        Standard {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            retryables: vec![
                Box::new(HTTPStatusCodeRetryable),
                Box::new(ServiceErrorCodeRetryable),
                Box::new(ConnectionErrorRetryable),
            ],
            backoff: Box::new(FullJitterBackoff::new(
                DEFAULT_BASE_DELAY,
                DEFAULT_MAX_BACKOFF,
            )),
        }
    }
}

impl Standard {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of retry attempts for the Standard retryer.
    ///
    /// A value of zero is invalid — at least one attempt is always made — so
    /// it falls back to [DEFAULT_MAX_ATTEMPTS]. Mirrors Go `NewStandard`,
    /// which normalizes a non-positive `MaxAttempts` the same way.
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = if max_attempts == 0 {
            DEFAULT_MAX_ATTEMPTS
        } else {
            max_attempts
        };
        self
    }

    /// Sets the error retryables for the Standard retryer.
    pub fn with_retryables(mut self, retryables: Vec<Box<dyn ErrorRetryable>>) -> Self {
        self.retryables = retryables;
        self
    }

    /// Sets the backoff strategy for the Standard retryer.
    pub fn with_backoff(mut self, backoff: Box<dyn BackoffDelayer>) -> Self {
        self.backoff = backoff;
        self
    }
}

impl Retryer for Standard {
    /// Returns the maximum number of retry attempts for the Standard retryer.
    fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    /// Checks if an error is retryable, leveraging the implemented retryables.
    fn is_error_retryable(&self, err: &(dyn std::error::Error + 'static)) -> bool {
        self.retryables
            .iter()
            .any(|retryable| retryable.is_error_retryable(err))
    }

    /// Delegates to the backoff strategy's method to determine the retry delay.
    fn retry_delay(
        &self,
        attempt: u32,
        _err: &dyn std::error::Error,
    ) -> Result<Duration, Box<dyn std::error::Error + Send + Sync>> {
        self.backoff.backoff_delay(attempt)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Error;
    use std::io::ErrorKind::{ConnectionReset, Other};

    use super::*;

    #[test]
    fn test_standard_retryer() {
        let standard_retryer = Standard::default();

        // test default retryer
        assert!(!standard_retryer.is_error_retryable(&Error::new(Other, "not retryable",)));
        assert_eq!(standard_retryer.max_attempts(), 3);
        let delay = standard_retryer
            .retry_delay(DEFAULT_MAX_ATTEMPTS, &Error::new(Other, "arbitrary error"))
            .unwrap();
        assert!(delay > Duration::ZERO && delay < DEFAULT_MAX_BACKOFF);

        // test other fields
        let standard_retryer = Standard::default()
            .with_max_attempts(5)
            .with_retryables(vec![Box::new(ConnectionErrorRetryable)]);
        assert_eq!(standard_retryer.max_attempts(), 5);
        assert!(
            standard_retryer.is_error_retryable(&Error::new(ConnectionReset, "connection reset"))
        );

        // A zero attempt budget would otherwise send nothing at all.
        assert_eq!(
            Standard::default().with_max_attempts(0).max_attempts(),
            DEFAULT_MAX_ATTEMPTS
        );

        // test fixed-delay-backoff retryer
        let standard_retryer =
            Standard::default().with_backoff(Box::new(FixedDelayBackoff::new(DEFAULT_BASE_DELAY)));
        assert_eq!(
            standard_retryer
                .retry_delay(rand::random(), &Error::new(Other, "arbitrary error"))
                .unwrap(),
            DEFAULT_BASE_DELAY
        );
    }
}
