//! Transfer progress reporting.
//!
//! The callback declared on `PutObjectRequest`, `UploadPartRequest` and
//! `GetObjectRequest` was previously never invoked: a caller could set one and
//! simply never hear from it. It is wired through the same body-tracking path
//! the CRC64 checks use, which is the only place the SDK sees every byte.

use std::any::Any;
use std::rc::Rc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use crate::constants::{OP_META_KEY_PROGRESS_TRACKER, OP_META_KEY_RESPONSE_PROGRESS_TRACKER};
use crate::types::operation::{BodyTracker, OperationInput};

/// Reports transferred bytes to a caller-supplied callback.
///
/// The counters live behind an `Arc` so a handle can be registered in
/// `OperationMetadata` (which stores `Rc`) while another rides the body stream,
/// which requires `Send`.
pub(crate) struct ProgressTracker {
    inner: Arc<ProgressInner>,
}

pub(crate) struct ProgressInner {
    /// Called with `(transferred, total)` after each chunk.
    callback: Box<dyn Fn(i64, i64) + Send + Sync>,
    transferred: AtomicI64,
    /// The size of the transfer, or 0 while it is still unknown.
    total: AtomicI64,
}

impl ProgressTracker {
    fn new(callback: Box<dyn Fn(i64, i64) + Send + Sync>, total: i64) -> Self {
        ProgressTracker {
            inner: Arc::new(ProgressInner {
                callback,
                transferred: AtomicI64::new(0),
                total: AtomicI64::new(total),
            }),
        }
    }
}

impl ProgressTracker {
    /// A handle that can ride a `Send` body stream.
    ///
    /// The tracker itself cannot be `Clone` (its callback is a `Box<dyn Fn>`),
    /// but the state behind it can be shared.
    pub(crate) fn handle(&self) -> Arc<dyn BodyTracker> {
        Arc::new(ProgressTracker {
            inner: self.inner.clone(),
        })
    }
}

impl BodyTracker for ProgressTracker {
    fn update(&self, chunk: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let transferred = self
            .inner
            .transferred
            .fetch_add(chunk.len() as i64, Ordering::Relaxed)
            + chunk.len() as i64;
        (self.inner.callback)(transferred, self.inner.total.load(Ordering::Relaxed));
        Ok(())
    }

    fn reset(&self) {
        // A retried body starts over, so the count does too: a caller drawing a
        // progress bar would otherwise watch it run past 100%.
        self.inner.transferred.store(0, Ordering::Relaxed);
    }
}

/// Attaches `callback` to `input` so it reports the bytes of a request body.
///
/// `total` is what the callback receives as the second argument; pass 0 when
/// the size is not known in advance.
pub(crate) fn add_progress_tracker(
    input: &mut OperationInput,
    callback: Box<dyn Fn(i64, i64) + Send + Sync>,
    total: i64,
) {
    let tracker = Rc::new(ProgressTracker::new(callback, total));
    input
        .op_metadata
        .set(OP_META_KEY_PROGRESS_TRACKER, tracker as Rc<dyn Any>);
}

/// Attaches `callback` to `input` so it reports the bytes of a response body.
///
/// A download's size is only known once the response arrives, so the callback
/// is handed the `Content-Length` of that response as its total.
pub(crate) fn add_response_progress_tracker(
    input: &mut OperationInput,
    callback: Box<dyn Fn(i64, i64) + Send + Sync>,
) {
    let tracker = Rc::new(ProgressTracker::new(callback, 0));
    input.op_metadata.set(
        OP_META_KEY_RESPONSE_PROGRESS_TRACKER,
        tracker as Rc<dyn Any>,
    );
}

/// Takes the response-side tracker out of `input`, if one was registered.
///
/// Returns a handle that can ride a `Send` stream. Its total is filled in from
/// the response's `Content-Length` by [`set_total`] before the stream is built.
pub(crate) fn take_response_progress_tracker(input: &OperationInput) -> Option<Arc<ProgressInner>> {
    input
        .op_metadata
        .values(OP_META_KEY_RESPONSE_PROGRESS_TRACKER)
        .into_iter()
        .flatten()
        .find_map(|value| value.clone().downcast::<ProgressTracker>().ok())
        .map(|tracker| tracker.inner.clone())
}

/// Records the size of a response so the callback can report a percentage.
pub(crate) fn set_total(inner: &ProgressInner, total: i64) {
    inner.total.store(total, Ordering::Relaxed);
}

/// Feeds a chunk to a response-side tracker.
pub(crate) fn update_response_progress(inner: &ProgressInner, chunk: &[u8]) {
    let transferred = inner
        .transferred
        .fetch_add(chunk.len() as i64, Ordering::Relaxed)
        + chunk.len() as i64;
    (inner.callback)(transferred, inner.total.load(Ordering::Relaxed));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Records every callback invocation as `(transferred, total)`.
    fn recorder() -> (Box<dyn Fn(i64, i64) + Send + Sync>, Arc<Mutex<Vec<(i64, i64)>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = seen.clone();
        let callback = Box::new(move |transferred: i64, total: i64| {
            sink.lock().expect("lock").push((transferred, total));
        });
        (callback, seen)
    }

    #[test]
    fn reports_cumulative_bytes() {
        let (callback, seen) = recorder();
        let tracker = ProgressTracker::new(callback, 100);

        tracker.update(&[0u8; 10]).expect("update");
        tracker.update(&[0u8; 5]).expect("update");

        assert_eq!(*seen.lock().expect("lock"), vec![(10, 100), (15, 100)]);
    }

    #[test]
    fn a_retried_body_restarts_the_count() {
        let (callback, seen) = recorder();
        let tracker = ProgressTracker::new(callback, 100);

        tracker.update(&[0u8; 10]).expect("update");
        tracker.reset();
        tracker.update(&[0u8; 10]).expect("update");

        // Without the reset the caller would see 10 then 20 against a total of
        // 100, and a retry would look like progress it did not make.
        assert_eq!(*seen.lock().expect("lock"), vec![(10, 100), (10, 100)]);
    }

    #[test]
    fn a_response_total_is_filled_in_before_the_stream_runs() {
        let (callback, seen) = recorder();
        let tracker = ProgressTracker::new(callback, 0);
        let inner = tracker.inner.clone();

        set_total(&inner, 4096);
        update_response_progress(&inner, &[0u8; 1024]);

        assert_eq!(*seen.lock().expect("lock"), vec![(1024, 4096)]);
    }
}
