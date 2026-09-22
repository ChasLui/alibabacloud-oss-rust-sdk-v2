//! Client-side bandwidth limiting.
//!
//! Mirrors Go's `BwTokenBucket` in `oss/limiter.go`, with one structural
//! difference: Go paces the socket, while this paces the body streams handed
//! to and taken from reqwest. reqwest owns its sockets, so those streams are
//! the only places the SDK sees the bytes at all.
//!
//! Two decisions are worth stating:
//!
//! - The limiter is a token bucket whose burst is a megabyte-scale window, not
//!   one packet. Waiting per small chunk would add a scheduling round trip to
//!   every read and cap throughput far below the configured rate; a window
//!   lets a burst through and slows the average down, which is what a
//!   bandwidth limit means.
//! - Waiting is asynchronous, because the stream runs on the caller's runtime.
//!   A blocking sleep there stalls every other task that runtime is driving —
//!   measurably, a download that should have taken seconds ran into its own
//!   request timeout.

use std::time::{Duration, Instant};

/// A token bucket that admits bytes at a configured rate.
///
/// The bucket refills continuously rather than in ticks, so a limit is spread
/// evenly over time instead of arriving in bursts once per tick.
pub struct BwTokenBucket {
    /// The limit in bytes per second.
    bandwidth: u64,
    /// The largest burst the bucket allows, in bytes.
    max_burst: u64,
    /// The tokens available right now, and when they were last topped up.
    ///
    /// Behind a mutex because the bucket is shared: a download's body stream
    /// can be polled from more than one task.
    state: std::sync::Mutex<BucketState>,
}

/// The mutable half of the bucket.
struct BucketState {
    tokens: f64,
    last_refill: Instant,
}

impl BwTokenBucket {
    /// Builds a bucket limited to `bandwidth` bytes per second.
    ///
    /// Mirrors Go's burst calculation: a quarter of a second's worth of bytes,
    /// with a floor of 4 MiB. The floor matters for a low limit on a fast
    /// link, where a proportionally sized burst would be too small to let a
    /// single request make progress.
    pub fn new(bandwidth: u64) -> Self {
        const DEFAULT_MAX_BURST_SIZE: u64 = 4 * 1024 * 1024;
        let scaled = bandwidth.saturating_mul(DEFAULT_MAX_BURST_SIZE) / (256 * 1024 * 1024);
        let max_burst = scaled.max(DEFAULT_MAX_BURST_SIZE);

        BwTokenBucket {
            bandwidth,
            max_burst,
            state: std::sync::Mutex::new(BucketState {
                // Starts full, so the first bytes are not delayed.
                tokens: max_burst as f64,
                last_refill: Instant::now(),
            }),
        }
    }

    /// The configured rate, in bytes per second.
    pub fn bandwidth(&self) -> u64 {
        self.bandwidth
    }

    /// The largest burst the bucket admits, in bytes.
    pub fn max_burst(&self) -> u64 {
        self.max_burst
    }

    /// Adds the tokens earned since the last refill.
    fn refill(state: &mut BucketState, bandwidth: u64, max_burst: u64) {
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_refill).as_secs_f64();
        state.last_refill = now;
        if bandwidth == 0 {
            state.tokens = max_burst as f64;
            return;
        }
        state.tokens = (state.tokens + elapsed * bandwidth as f64).min(max_burst as f64);
    }

    /// Waits until `n` bytes may be sent.
    ///
    /// A zero bandwidth is treated as unlimited, matching Go's behaviour when
    /// the option is left unset. The wait awaits rather than blocking, because
    /// the limiter runs on a body stream: a blocking sleep there would stall
    /// every other task the runtime is driving.
    pub async fn limit_bandwidth(&self, n: usize) {
        if self.bandwidth == 0 || n == 0 {
            return;
        }
        let needed = n as f64;
        loop {
            let wait = {
                let mut state = match self.state.lock() {
                    Ok(state) => state,
                    // A poisoned lock only means another poll panicked; the
                    // counters are still usable, so pacing continues rather
                    // than turning into a panic here.
                    Err(poisoned) => poisoned.into_inner(),
                };
                Self::refill(&mut state, self.bandwidth, self.max_burst);
                if state.tokens >= needed {
                    state.tokens -= needed;
                    return;
                }
                let missing = needed - state.tokens;
                missing / self.bandwidth as f64
            };
            // A read can be larger than the current tokens; waiting for the
            // exact shortfall keeps the average at the configured rate.
            tokio::time::sleep(Duration::from_secs_f64(wait.clamp(0.000_001, 1.0))).await;
        }
    }

    /// The waiting counterpart, for callers on a non-async thread.
    pub fn limit_bandwidth_blocking(&self, n: usize) {
        if self.bandwidth == 0 || n == 0 {
            return;
        }
        let needed = n as f64;
        loop {
            let wait = {
                let mut state = match self.state.lock() {
                    Ok(state) => state,
                    Err(poisoned) => poisoned.into_inner(),
                };
                Self::refill(&mut state, self.bandwidth, self.max_burst);
                if state.tokens >= needed {
                    state.tokens -= needed;
                    return;
                }
                let missing = needed - state.tokens;
                missing / self.bandwidth as f64
            };
            std::thread::sleep(Duration::from_secs_f64(wait.clamp(0.000_001, 1.0)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bucket must start full, so the first bytes are not delayed.
    #[tokio::test]
    async fn test_starts_full() {
        let bucket = BwTokenBucket::new(1024 * 1024);
        let state = bucket.state.lock().expect("lock");
        assert_eq!(state.tokens, bucket.max_burst as f64);
        assert_eq!(bucket.bandwidth(), 1024 * 1024);
    }

    /// A low limit still gets the minimum burst, which is what keeps a single
    /// request able to make progress on a fast link.
    #[tokio::test]
    async fn test_small_bandwidth_gets_minimum_burst() {
        let bucket = BwTokenBucket::new(1024);
        assert_eq!(
            bucket.max_burst(),
            4 * 1024 * 1024,
            "a small limit must not shrink the burst below the floor"
        );
    }

    /// A large limit scales the burst up, so a high-rate transfer is not held
    /// to the floor.
    #[tokio::test]
    async fn test_large_bandwidth_scales_burst() {
        let bandwidth = 1024u64 * 1024 * 1024; // 1 GiB/s
        let bucket = BwTokenBucket::new(bandwidth);
        assert!(
            bucket.max_burst() > 4 * 1024 * 1024,
            "a high limit must be allowed a proportionally larger burst"
        );
    }

    /// Sending within the burst must not block; sending beyond it must.
    #[tokio::test]
    async fn test_limits_beyond_burst() {
        // 1 MiB/s with the 4 MiB burst floor: the first 4 MiB is free, and the
        // next megabyte has to wait about a second.
        let bucket = BwTokenBucket::new(1024 * 1024);
        let burst = bucket.max_burst() as usize;

        let start = Instant::now();
        bucket.limit_bandwidth(burst).await;
        let first = start.elapsed();
        assert!(
            first < Duration::from_millis(100),
            "a transfer within the burst must not be delayed, took {first:?}"
        );

        let start = Instant::now();
        bucket.limit_bandwidth(1024 * 1024).await;
        let second = start.elapsed();
        assert!(
            second >= Duration::from_millis(500),
            "exceeding the burst must be paced, only waited {second:?}"
        );
    }

    /// A zero bandwidth means unlimited: the option is unset in that case, and
    /// blocking would hang every transfer.
    #[tokio::test]
    async fn test_zero_bandwidth_is_unlimited() {
        let bucket = BwTokenBucket::new(0);
        let start = Instant::now();
        bucket.limit_bandwidth(64 * 1024 * 1024).await;
        assert!(
            start.elapsed() < Duration::from_millis(50),
            "an unset limit must not throttle"
        );
    }

    /// A zero-byte chunk must not be delayed either.
    #[tokio::test]
    async fn test_zero_bytes_does_not_wait() {
        let bucket = BwTokenBucket::new(1024);
        let start = Instant::now();
        bucket.limit_bandwidth(0).await;
        assert!(start.elapsed() < Duration::from_millis(50));
    }
}
