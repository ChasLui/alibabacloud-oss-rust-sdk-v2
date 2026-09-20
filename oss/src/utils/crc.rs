use std::any::Any;
use std::rc::Rc;

use crate::client::{OssResponse, ResponseHandler, ResponseHandlers};
use crate::{
    OperationInput, OP_META_KEY_REQUEST_BODY_TRACKER, OP_META_KEY_RESPONSE_HANDLER,
    HEADER_OSS_CRC64,
};
use crate::Crc64Tracker;

/// Represents a Crc64 calculator.
///
/// `size`/`block_size`/`append_sum` complete the `hash.Hash64` surface that Go
/// exposes; they are kept for parity with `NewCRC64` even though the SDK's own
/// paths only need `write` and `sum64`.
#[allow(dead_code)]
pub(crate) struct Crc64 {
    init: u64,
    crc: u64,
    crc_instance: crc::Crc<u64>,
}

#[allow(dead_code)]
impl Crc64 {
    /// Creates a new `Crc64` instance.
    ///
    /// # Arguments
    ///
    /// * `init` - The initial CRC value.
    pub fn new(init: u64) -> Self {
        Crc64 {
            init,
            crc: init,
            crc_instance: crc::Crc::<u64>::new(&crc::CRC_64_ECMA_182),
        }
    }

    /// Returns the size of the CRC in bytes.
    pub fn size(&self) -> u8 {
        self.crc_instance.algorithm.width / 8
    }

    /// Returns the block size for CRC calculation.
    pub fn block_size(&self) -> usize {
        1
    }

    /// Resets the CRC to its initial value.
    pub fn reset(&mut self) {
        self.crc = self.init;
    }

    /// Writes the given buffer to the CRC calculation.
    ///
    /// Continues from the current value, so a body sent as several chunks
    /// yields the same CRC as the concatenation of those chunks. Recomputing
    /// `checksum(buf)` from scratch would discard everything written earlier.
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
        let mut digest = self.crc_instance.digest_with_initial(self.crc);
        digest.update(buf);
        self.crc = digest.finalize();
        Ok(buf.len())
    }

    /// Returns the current CRC value.
    pub fn sum64(&self) -> u64 {
        self.crc
    }

    /// Appends the current CRC value to the given byte vector.
    ///
    /// # Arguments
    ///
    /// * `in_bytes` - The byte vector to append the CRC value to.
    pub fn append_sum(&self, in_bytes: &mut Vec<u8>) {
        in_bytes.extend_from_slice(&self.sum64().to_be_bytes());
    }
}

/// Attaches a CRC64 upload check to `input`, if the client enabled it.
///
/// Registers a tracker that observes the bytes actually sent, plus a response
/// handler that compares the tracker's checksum against the server's
/// `x-oss-hash-crc64ecma`. A mismatch is reported as an error whose message
/// contains `"crc is inconsistent"`, which
/// `ConnectionErrorRetryable` already recognises as retryable. Mirrors Go
/// `Client.addCrcCheck` and `checkResponseHeaderCRC64`.
///
/// # Arguments
///
/// * `input` - The operation input to attach the check to.
/// * `init` - The initial CRC64 value; `AppendObject` resumes from the value
///   returned by the previous append, other operations start at `0`.
/// * `enabled` - Whether `ENABLE_CRC64_CHECK_UPLOAD` is set on the client.
pub fn add_crc64_check(input: &mut OperationInput, init: u64, enabled: bool) {
    if !enabled {
        return;
    }
    // A body-less upload has nothing to verify.
    if input.body.is_none() {
        return;
    }

    let tracker = Rc::new(Crc64Tracker::new(init));
    input.op_metadata.set(
        OP_META_KEY_REQUEST_BODY_TRACKER,
        tracker.clone() as Rc<dyn Any>,
    );

    let handler: ResponseHandler = Rc::new(
        move |response: &OssResponse| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let server_crc = match response {
                OssResponse::SucResponse(response) => response
                    .headers()
                    .get(HEADER_OSS_CRC64)
                    .and_then(|v| v.to_str().ok()),
                OssResponse::ErrResponse { headers, .. } => {
                    headers.get(HEADER_OSS_CRC64).and_then(|v| v.to_str().ok())
                }
            };

            let client_crc = tracker.sum64().to_string();
            if let Some(server_crc) = server_crc {
                if !server_crc.is_empty() && server_crc != client_crc {
                    return Err(format!(
                        "crc is inconsistent, client {}, server {}",
                        client_crc, server_crc
                    )
                    .into());
                }
            }
            Ok(())
        },
    );
    // The metadata slot holds one `Vec` of handlers, matching the contract in
    // `apply_operation_metadata` (it downcasts the single value), so the check
    // is pushed as a one-element vector.
    let handlers: ResponseHandlers = vec![handler];
    input.op_metadata.add(OP_META_KEY_RESPONSE_HANDLER, Rc::new(handlers));
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &[u8; 5] = b"Hello";
    const EXAMPLE_CHECKSUM: u64 = 0xFFAD3236B47900AB;

    #[test]
    fn test_new() {
        let crc = Crc64::new(0x123);
        assert_eq!(crc.init, 0x123);
        assert_eq!(crc.crc, 0x123);
    }

    #[test]
    fn test_size() {
        let crc = Crc64::new(0);
        assert_eq!(crc.size(), 8);
    }

    #[test]
    fn test_block_size() {
        let crc = Crc64::new(0);
        assert_eq!(crc.block_size(), 1);
    }

    #[test]
    fn test_reset() {
        let mut crc = Crc64::new(0);
        crc.write(b"Anything").unwrap();
        crc.reset();
        assert_eq!(crc.crc, 0);

        let mut crc = Crc64::new(0x12);
        crc.write(b"Another anything").unwrap();
        crc.reset();
        assert_eq!(crc.crc, 0x12);
    }

    #[test]
    fn test_write() {
        let mut crc = Crc64::new(0);
        crc.write(EXAMPLE_INPUT).unwrap();
        assert_eq!(crc.crc, EXAMPLE_CHECKSUM);
    }

    /// Chunked writes must equal a single whole-buffer write. Previously each
    /// `write` recomputed the checksum from scratch, so a chunked body
    /// silently reported only its last chunk's CRC.
    #[test]
    fn test_write_accumulates_across_chunks() {
        let mut whole = Crc64::new(0);
        whole.write(EXAMPLE_INPUT).unwrap();

        let mut chunked = Crc64::new(0);
        chunked.write(b"He").unwrap();
        chunked.write(b"ll").unwrap();
        chunked.write(b"o").unwrap();

        assert_eq!(chunked.sum64(), whole.sum64());
        assert_eq!(chunked.sum64(), EXAMPLE_CHECKSUM);
    }

    /// A non-zero init participates in the accumulation, which mirrors Go's
    /// `hashCRC64{crc: init}` + `crc64.Update(d.crc, tab, p)` resume semantics.
    #[test]
    fn test_write_resumes_from_init() {
        // The init is the running value, not a "reset to" constant: starting
        // from a mid-stream value and finishing the input must reproduce the
        // whole-buffer checksum.
        let mut first_half = Crc64::new(0);
        first_half.write(b"He").unwrap();

        let mut second_half = Crc64::new(first_half.sum64());
        second_half.write(b"llo").unwrap();

        assert_eq!(second_half.sum64(), EXAMPLE_CHECKSUM);

        // A non-zero init that is not a real resume point still changes the
        // digest, proving the value feeds into the calculation.
        let mut off_init = Crc64::new(0x1234);
        off_init.write(EXAMPLE_INPUT).unwrap();
        assert_ne!(off_init.sum64(), EXAMPLE_CHECKSUM);
    }

    #[test]
    fn test_sum64() {
        let mut crc = Crc64::new(0);
        assert_eq!(crc.sum64(), 0);

        crc.write(EXAMPLE_INPUT).unwrap();
        assert_eq!(crc.crc, EXAMPLE_CHECKSUM);
    }

    #[test]
    fn test_append_sum() {
        let crc = Crc64::new(0);
        let mut in_bytes = vec![];
        crc.append_sum(&mut in_bytes);
        assert_eq!(in_bytes, [0; 8]);

        let mut crc = Crc64::new(0);
        let mut in_bytes = EXAMPLE_INPUT.clone().to_vec();
        crc.write(EXAMPLE_INPUT).unwrap();
        crc.append_sum(&mut in_bytes);
        assert_eq!(
            in_bytes,
            [EXAMPLE_INPUT, EXAMPLE_CHECKSUM.to_be_bytes().as_ref()].concat()
        );
    }
}
