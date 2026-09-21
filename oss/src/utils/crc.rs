use std::any::Any;
use std::rc::Rc;

use crate::client::{OssResponse, ResponseHandler, ResponseHandlers};
use crate::Crc64Tracker;
use crate::{
    OperationInput, HEADER_OSS_CRC64, OP_META_KEY_REQUEST_BODY_TRACKER,
    OP_META_KEY_RESPONSE_HANDLER,
};

/// Reflected CRC-64 polynomial, as Go's `crc64.ECMA` defines it.
///
/// Go stores the polynomial in its reflected form (`0xC96C5795D7870F42`);
/// OSS's `x-oss-hash-crc64ecma` is the value of `hash/crc64` running that
/// table, so the reflected form is what must be implemented here. (The
/// non-reflected variant is a *different* checksum — see the tests.)
const CRC64_POLY: u64 = 0xC96C5795D7870F42;

const fn crc64_table() -> [u64; 256] {
    let mut table = [0u64; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut crc = i as u64;
        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ CRC64_POLY
            } else {
                crc >> 1
            };
            bit += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

static CRC64_TABLE: [u64; 256] = crc64_table();

/// One step of Go's `hash/crc64.Update`: the incoming state is a *final* CRC,
/// so it is complemented on entry and the result is complemented on exit.
/// Passing the previous `sum64()` back in therefore continues the checksum.
fn crc64_update(state: u64, data: &[u8]) -> u64 {
    let mut crc = !state;
    for &byte in data {
        crc = CRC64_TABLE[((crc ^ byte as u64) & 0xff) as usize] ^ (crc >> 8);
    }
    !crc
}

/// Represents a Crc64 calculator.
///
/// `size`/`block_size`/`append_sum` complete the `hash.Hash64` surface that Go
/// exposes; they are kept for parity with `NewCRC64` even though the SDK's own
/// paths only need `write` and `sum64`.
#[allow(dead_code)]
pub(crate) struct Crc64 {
    init: u64,
    crc: u64,
}

#[allow(dead_code)]
impl Crc64 {
    /// Creates a new `Crc64` instance.
    ///
    /// # Arguments
    ///
    /// * `init` - The initial CRC value.
    pub fn new(init: u64) -> Self {
        Crc64 { init, crc: init }
    }

    /// Returns the size of the CRC in bytes.
    pub fn size(&self) -> u8 {
        8
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
    /// yields the same CRC as the concatenation of those chunks. Mirrors Go
    /// `hashCRC64.Write`.
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
        self.crc = crc64_update(self.crc, buf);
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

/// GF(2) matrix times vector, used by [`crc64_combine`].
fn gf2_matrix_times(matrix: &[u64; 64], mut vector: u64) -> u64 {
    let mut sum = 0u64;
    let mut i = 0;
    while vector != 0 {
        if vector & 1 != 0 {
            sum ^= matrix[i];
        }
        vector >>= 1;
        i += 1;
    }
    sum
}

/// GF(2) matrix square, used by [`crc64_combine`].
fn gf2_matrix_square(square: &mut [u64; 64], matrix: &[u64; 64]) {
    for n in 0..64 {
        square[n] = gf2_matrix_times(matrix, matrix[n]);
    }
}

/// Combines two CRC64 checksums into the checksum of their concatenation.
///
/// `crc1` is the checksum of the first part, `crc2` of the second part, and
/// `len2` is the byte length of the second part. This lets a caller verify a
/// whole object while fetching it in pieces — each part's CRC is computed
/// independently and then folded into one value.
///
/// Mirrors Go `CRC64Combine`.
pub(crate) fn crc64_combine(crc1: u64, crc2: u64, len2: u64) -> u64 {
    if len2 == 0 {
        return crc1;
    }

    let mut even = [0u64; 64];
    let mut odd = [0u64; 64];

    // Operator for one zero bit.
    odd[0] = CRC64_POLY;
    let mut row = 1u64;
    for n in 1..64 {
        odd[n] = row;
        row <<= 1;
    }

    // Squares give operators for 2, then 4, zero bits.
    gf2_matrix_square(&mut even, &odd);
    gf2_matrix_square(&mut odd, &even);

    let mut crc1 = crc1;
    let mut len2 = len2;
    loop {
        gf2_matrix_square(&mut even, &odd);
        if len2 & 1 != 0 {
            crc1 = gf2_matrix_times(&even, crc1);
        }
        len2 >>= 1;
        if len2 == 0 {
            break;
        }

        gf2_matrix_square(&mut odd, &even);
        if len2 & 1 != 0 {
            crc1 = gf2_matrix_times(&odd, crc1);
        }
        len2 >>= 1;
        if len2 == 0 {
            break;
        }
    }

    crc1 ^ crc2
}

/// Compares a locally computed CRC64 against the server's
/// `x-oss-hash-crc64ecma` value.
///
/// A missing or empty server value means the server did not report a
/// checksum; there is then nothing to verify and the check passes. Mirrors Go
/// `checkResponseHeaderCRC64`.
pub fn check_crc64(client_crc: u64, server_crc: Option<&str>) -> Result<(), String> {
    if let Some(server_crc) = server_crc {
        if !server_crc.is_empty() && server_crc != client_crc.to_string() {
            return Err(format!(
                "crc is inconsistent, client {}, server {}",
                client_crc, server_crc
            ));
        }
    }
    Ok(())
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

            if let Err(e) = check_crc64(tracker.sum64(), server_crc) {
                return Err(e.into());
            }
            Ok(())
        },
    );
    // The metadata slot holds one `Vec` of handlers, matching the contract in
    // `apply_operation_metadata` (it downcasts the single value), so the check
    // is pushed as a one-element vector.
    let handlers: ResponseHandlers = vec![handler];
    input
        .op_metadata
        .add(OP_META_KEY_RESPONSE_HANDLER, Rc::new(handlers));
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &[u8; 5] = b"Hello";

    /// Every constant below is Go's `hash/crc64` running `crc64.ECMA` — the
    /// function OSS uses for `x-oss-hash-crc64ecma`. They are cross-language
    /// oracle values: if the polynomial, its reflection, or the complement
    /// convention drifts, these fail while a self-computed expectation would
    /// happily agree with a wrong implementation.
    const EXAMPLE_CHECKSUM: u64 = 0x51CF5C3BC87BACC8; // crc(0, "Hello")
    const GO_TEST_VECTOR: u64 = 0x995DC9BBDF1939FA; // Go utils_crc_test.go: crc(0, "123456789")
    const GO_TEST_VECTOR_2: u64 = 0x27DB187FC15BBC72; // Go utils_crc_test.go: crc(0, "This is a test...")
    const COMBINED_VECTOR: u64 = 0x6A7AA19FCBF48688; // crc(0, "123456789" + "This is a test...")

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

    /// The two vectors Go asserts in `utils_crc_test.go`. Running them here is
    /// what pins this implementation to `crc64.ECMA` rather than the
    /// non-reflected CRC-64/ECMA-182, which produces different values for the
    /// same input and would make every server-side comparison fail.
    #[test]
    fn test_matches_go_reference_vectors() {
        let mut crc = Crc64::new(0);
        crc.write(b"123456789").unwrap();
        assert_eq!(crc.sum64(), GO_TEST_VECTOR);

        let mut crc = Crc64::new(0);
        crc.write(b"This is a test of the emergency broadcast system.")
            .unwrap();
        assert_eq!(crc.sum64(), GO_TEST_VECTOR_2);

        // ...and that the non-reflected variant really is a different function,
        // so a future "simplification" back to it cannot pass unnoticed.
        let mut wrong = Crc64::new(0);
        wrong.write(b"123456789").unwrap();
        assert_ne!(wrong.sum64(), 0x6C40DF5F0B497347);
    }

    /// Resuming from a previous checksum must equal hashing the concatenation,
    /// which is Go's `NewCRC64(sum).Write(rest)` usage for append uploads.
    #[test]
    fn test_resume_matches_concatenation() {
        let first = b"123456789";
        let second = b"This is a test of the emergency broadcast system.";

        let mut upper = Crc64::new(0);
        upper.write(first).unwrap();
        upper.write(second).unwrap();

        let mut resumed = Crc64::new(GO_TEST_VECTOR);
        resumed.write(second).unwrap();

        assert_eq!(upper.sum64(), COMBINED_VECTOR);
        assert_eq!(resumed.sum64(), COMBINED_VECTOR);
    }

    #[test]
    fn test_combine_matches_concatenation() {
        // 49 is the byte length of the second part; using the wrong length here
        // is exactly the mistake that would corrupt a multi-part download, so
        // the test pins it rather than deriving it silently.
        assert_eq!(
            b"This is a test of the emergency broadcast system.".len(),
            49
        );
        assert_eq!(
            crc64_combine(GO_TEST_VECTOR, GO_TEST_VECTOR_2, 49),
            COMBINED_VECTOR
        );

        // Folding in nothing must leave the checksum untouched, otherwise a
        // partially-downloaded object would verify against a wrong value.
        assert_eq!(
            crc64_combine(GO_TEST_VECTOR, GO_TEST_VECTOR_2, 0),
            GO_TEST_VECTOR
        );

        // Combining in the other order is a different value — the arguments
        // are not interchangeable.
        assert_ne!(
            crc64_combine(GO_TEST_VECTOR_2, GO_TEST_VECTOR, 9),
            COMBINED_VECTOR
        );
    }
}
