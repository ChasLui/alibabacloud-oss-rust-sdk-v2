//! Client-side encryption primitives.
//!
//! Mirrors Go's `oss/crypto` package. The scheme is envelope encryption: the
//! object's bytes are encrypted with a fresh AES key, and that key (plus its
//! IV) is encrypted with a long-lived master key. Only the wrapped key and IV
//! are stored on the object, so the master key never leaves the client.
//!
//! Two properties of the construction drive most of the code here:
//!
//! - AES-CTR turns a block cipher into a stream cipher by encrypting a counter,
//!   so the keystream for a byte offset depends only on the IV and the offset —
//!   not on any preceding bytes. That is what makes a ranged read of an
//!   encrypted object possible at all: the counter is advanced to the offset
//!   instead of the data being re-read. It is also why the IV's low 8 bytes are
//!   treated as a big-endian counter that can be seeked.
//! - The IV must stay in step between the writer and the reader. A multipart
//!   upload encrypts each part from a counter derived from its part number, so
//!   a part can be re-uploaded or downloaded independently and still decrypt to
//!   the same bytes.
//!
//! AES-CTR provides no integrity on its own: a modified ciphertext decrypts to
//! modified plaintext without any error. The envelope records the plaintext
//! MD5 and length so a caller can detect tampering, but the SDK cannot check
//! them for a ranged read, which sees only a slice.

use std::io::Read;

use aes::cipher::{KeyIvInit, StreamCipher};
use base64::engine::general_purpose;
use base64::Engine;
use rand::{Rng, RngCore};

/// The wrap algorithm used when the master key is an RSA key pair. Mirrors
/// Go's `RsaCryptoWrap`.
pub const RSA_CRYPTO_WRAP: &str = "RSA/NONE/PKCS1Padding";

/// The content algorithm: AES in counter mode, no padding. Mirrors Go's
/// `AesCtrAlgorithm`.
pub const AES_CTR_ALGORITHM: &str = "AES/CTR/NoPadding";

type Aes128Ctr = ctr::Ctr64BE<aes::Aes128>;

/// The metadata stored on an encrypted object.
///
/// Mirrors Go's `Envelope`. Every field travels base64-encoded in the object's
/// `x-oss-meta-client-side-encryption-*` headers (the wrapped key and IV) or
/// as plain metadata (the algorithms).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Envelope {
    /// The IV, wrapped with the master key. Base64 on the wire.
    pub iv: String,
    /// The content key, wrapped with the master key. Base64 on the wire.
    pub cipher_key: String,
    /// The master key description, chosen by the caller to identify the key.
    pub mat_desc: String,
    /// The algorithm that wrapped the key and IV.
    pub wrap_alg: String,
    /// The algorithm that encrypts the content.
    pub cek_alg: String,
    /// The MD5 of the plaintext, when the caller supplied one.
    pub unencrypted_md5: String,
    /// The plaintext length, when the caller supplied one.
    pub unencrypted_content_len: String,
}

impl Envelope {
    /// Whether the envelope carries everything needed to decrypt.
    ///
    /// A missing field means the object cannot be read: without the key there
    /// is nothing to unwrap, and without the algorithms there is no way to
    /// know how to use it.
    pub fn is_valid(&self) -> bool {
        !self.iv.is_empty()
            && !self.cipher_key.is_empty()
            && !self.wrap_alg.is_empty()
            && !self.cek_alg.is_empty()
    }
}

/// The secret material for one object: the content key and IV, plus their
/// wrapped forms.
#[derive(Debug, Clone, Default)]
pub struct CipherData {
    /// The plaintext IV. Its last 8 bytes are the counter.
    pub iv: Vec<u8>,
    /// The plaintext content key.
    pub key: Vec<u8>,
    /// The master key description.
    pub mat_desc: String,
    /// The wrapping algorithm.
    pub wrap_algorithm: String,
    /// The content algorithm.
    pub cek_algorithm: String,
    /// The IV, wrapped with the master key.
    pub encrypted_iv: Vec<u8>,
    /// The content key, wrapped with the master key.
    pub encrypted_key: Vec<u8>,
}

impl CipherData {
    /// Generates a random key and IV.
    ///
    /// The IV's last 8 bytes are randomised too, but only 4 random bytes are
    /// used for the counter before it is written big-endian: the counter is
    /// advanced by byte offsets, and a full 64-bit random counter could
    /// overflow when added to a large offset. Mirrors Go's `RandomKeyIv`.
    pub fn random_key_iv(key_len: usize, iv_len: usize) -> Result<Self, String> {
        if iv_len < 8 {
            return Err(format!("ivLen:{iv_len} less than 8"));
        }
        let mut key = vec![0u8; key_len];
        rand::thread_rng().fill_bytes(&mut key);

        let mut iv = vec![0u8; iv_len];
        rand::thread_rng().fill_bytes(&mut iv[..iv_len - 8]);
        let mut data = CipherData {
            iv,
            key,
            ..Default::default()
        };
        // Only 4 bytes of randomness, so seeking by a large offset cannot
        // wrap the counter.
        let rand_number = rand::thread_rng().gen::<u32>();
        data.set_iv(rand_number as u64);
        Ok(data)
    }

    /// Writes the counter into the IV's last 8 bytes, big-endian.
    pub fn set_iv(&mut self, iv: u64) {
        let len = self.iv.len();
        self.iv[len - 8..].copy_from_slice(&iv.to_be_bytes());
    }

    /// Reads the counter from the IV's last 8 bytes.
    pub fn get_iv(&self) -> u64 {
        let len = self.iv.len();
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.iv[len - 8..]);
        u64::from_be_bytes(bytes)
    }

    /// Advances the counter by the number of blocks `start_pos` covers.
    ///
    /// AES-CTR's keystream advances one block per `iv.len()` bytes, which is
    /// what makes a byte offset translatable into a counter value.
    pub fn seek_iv(&mut self, start_pos: u64) {
        let block_len = self.iv.len() as u64;
        self.set_iv(self.get_iv() + start_pos / block_len);
    }
}

/// A master key that wraps and unwraps content keys.
pub trait MasterCipher {
    /// Wraps `plain_data`.
    fn encrypt(&self, plain_data: &[u8]) -> Result<Vec<u8>, String>;
    /// Unwraps `crypto_data`.
    fn decrypt(&self, crypto_data: &[u8]) -> Result<Vec<u8>, String>;
    /// The wrapping algorithm this cipher implements.
    fn wrap_algorithm(&self) -> String;
    /// The description identifying this key.
    fn mat_desc(&self) -> String;
}

/// An RSA master key, given as PEM text.
///
/// Mirrors Go's `MasterRsaCipher`. Both PKCS#8 (`PUBLIC KEY` / `PRIVATE KEY`)
/// and PKCS#1 (`RSA PUBLIC KEY` / `RSA PRIVATE KEY`) PEM blocks are accepted,
/// because keys in the wild use both.
pub struct MasterRsaCipher {
    mat_desc: String,
    public_key: String,
    private_key: String,
}

impl MasterRsaCipher {
    /// Builds an RSA master key from PEM text.
    ///
    /// `mat_desc` is rendered as JSON so a key can carry more than one
    /// attribute; an empty map produces an empty description.
    pub fn new(
        mat_desc: &std::collections::HashMap<String, String>,
        public_key: &str,
        private_key: &str,
    ) -> Self {
        let mat_desc = if mat_desc.is_empty() {
            String::new()
        } else {
            serde_json::to_string(mat_desc).unwrap_or_default()
        };
        MasterRsaCipher {
            mat_desc,
            public_key: public_key.to_string(),
            private_key: private_key.to_string(),
        }
    }

    /// Parses the PEM public key.
    fn public_key(&self) -> Result<rsa::RsaPublicKey, String> {
        let (label, der) = pem::parse(&self.public_key)
            .map(|block| (block.tag().to_string(), block.contents().to_vec()))
            .map_err(|err| format!("cannot decode public key PEM: {err}"))?;
        use rsa::pkcs1::DecodeRsaPublicKey;
        use rsa::pkcs8::DecodePublicKey;
        match label.as_str() {
            "PUBLIC KEY" => {
                rsa::RsaPublicKey::from_public_key_der(&der).map_err(|err| err.to_string())
            }
            "RSA PUBLIC KEY" => {
                rsa::RsaPublicKey::from_pkcs1_der(&der).map_err(|err| err.to_string())
            }
            other => Err(format!("not supported public key,type:{other}")),
        }
    }

    /// Parses the PEM private key.
    fn private_key(&self) -> Result<rsa::RsaPrivateKey, String> {
        let (label, der) = pem::parse(&self.private_key)
            .map(|block| (block.tag().to_string(), block.contents().to_vec()))
            .map_err(|err| format!("cannot decode private key PEM: {err}"))?;
        use rsa::pkcs1::DecodeRsaPrivateKey;
        use rsa::pkcs8::DecodePrivateKey;
        match label.as_str() {
            "PRIVATE KEY" => {
                rsa::RsaPrivateKey::from_pkcs8_der(&der).map_err(|err| err.to_string())
            }
            "RSA PRIVATE KEY" => {
                rsa::RsaPrivateKey::from_pkcs1_der(&der).map_err(|err| err.to_string())
            }
            other => Err(format!("not supported private key,type:{other}")),
        }
    }
}

impl MasterCipher for MasterRsaCipher {
    fn encrypt(&self, plain_data: &[u8]) -> Result<Vec<u8>, String> {
        let mut rng = rand::thread_rng();
        self.public_key()?
            .encrypt(&mut rng, rsa::Pkcs1v15Encrypt, plain_data)
            .map_err(|err| err.to_string())
    }

    fn decrypt(&self, crypto_data: &[u8]) -> Result<Vec<u8>, String> {
        let _rng = rand::thread_rng();
        self.private_key()?
            .decrypt(rsa::Pkcs1v15Encrypt, crypto_data)
            .map_err(|err| err.to_string())
    }

    fn wrap_algorithm(&self) -> String {
        RSA_CRYPTO_WRAP.to_string()
    }

    fn mat_desc(&self) -> String {
        self.mat_desc.clone()
    }
}

/// The content cipher: AES-CTR over the object's bytes.
#[derive(Debug, Clone)]
pub struct AesCtrCipher {
    cipher_data: CipherData,
}

impl AesCtrCipher {
    /// Builds a cipher from resolved key material.
    pub fn new(cipher_data: CipherData) -> Result<Self, String> {
        if cipher_data.key.len() != 16 {
            return Err(format!(
                "invalid AES-128 key length {}, expected 16",
                cipher_data.key.len()
            ));
        }
        if cipher_data.iv.len() != 16 {
            return Err(format!(
                "invalid IV length {}, expected 16",
                cipher_data.iv.len()
            ));
        }
        Ok(AesCtrCipher { cipher_data })
    }

    /// The key material, including its wrapped form.
    pub fn cipher_data(&self) -> &CipherData {
        &self.cipher_data
    }

    /// The block size the counter advances by, which is also the granularity a
    /// ranged read has to be aligned to.
    pub fn align_len(&self) -> usize {
        self.cipher_data.iv.len()
    }

    /// CTR is a stream cipher: the ciphertext is the same length as the
    /// plaintext.
    pub fn encrypted_len(&self, plain_text_len: i64) -> i64 {
        plain_text_len
    }

    /// Returns a copy positioned at `plaintext_offset`.
    ///
    /// A ranged read of an encrypted object can only start at a block
    /// boundary, so the caller asks for an aligned range and discards the
    /// leading bytes it did not want.
    pub fn clone_at_offset(&self, plaintext_offset: u64) -> Result<Self, String> {
        let mut cipher_data = self.cipher_data.clone();
        cipher_data.seek_iv(plaintext_offset);
        AesCtrCipher::new(cipher_data)
    }

    /// Encrypts `data`, advancing the keystream by its length.
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let mut cipher = Aes128Ctr::new(
            (&self.cipher_data.key[..]).into(),
            (&self.cipher_data.iv[..]).into(),
        );
        let mut output = data.to_vec();
        cipher.apply_keystream(&mut output);
        Ok(output)
    }

    /// Decrypts `data`, which is the same operation as encrypting it.
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        self.encrypt(data)
    }

    /// Encrypts a plaintext stream.
    pub fn encrypt_reader<R: Read>(&self, mut reader: R) -> Result<Vec<u8>, String> {
        let mut plain = Vec::new();
        reader
            .read_to_end(&mut plain)
            .map_err(|err| err.to_string())?;
        self.encrypt(&plain)
    }
}

/// A cipher builder backed by a master key.
///
/// Mirrors Go's `ContentCipherBuilder`: it either mints a fresh content key for
/// an upload, or reconstructs one from the envelope stored on an object.
pub struct ContentCipherBuilder {
    master_cipher: Box<dyn MasterCipher>,
}

impl ContentCipherBuilder {
    /// Builds a builder around `master_cipher`.
    pub fn new(master_cipher: Box<dyn MasterCipher>) -> Self {
        ContentCipherBuilder { master_cipher }
    }

    /// The master key's description, used to pick the right key when reading.
    pub fn mat_desc(&self) -> String {
        self.master_cipher.mat_desc()
    }

    /// Mints a fresh content key and IV for an upload.
    ///
    /// The key and IV are wrapped immediately, so the envelope can be written
    /// before any content is encrypted.
    pub fn content_cipher(&self) -> Result<AesCtrCipher, String> {
        let mut cipher_data = CipherData::random_key_iv(16, 16)?;
        cipher_data.cek_algorithm = AES_CTR_ALGORITHM.to_string();
        cipher_data.mat_desc = self.master_cipher.mat_desc();
        cipher_data.wrap_algorithm = self.master_cipher.wrap_algorithm();
        cipher_data.encrypted_key = self.master_cipher.encrypt(&cipher_data.key)?;
        cipher_data.encrypted_iv = self.master_cipher.encrypt(&cipher_data.iv)?;
        AesCtrCipher::new(cipher_data)
    }

    /// Rebuilds the content cipher from an object's envelope.
    ///
    /// The stored key and IV are base64 of the *wrapped* bytes, so they are
    /// unwrapped before they can be used; an unwrap failure means the object
    /// was not written by this master key.
    pub fn content_cipher_from_envelope(
        &self,
        envelope: &Envelope,
    ) -> Result<AesCtrCipher, String> {
        let encrypted_key = general_purpose::STANDARD
            .decode(&envelope.cipher_key)
            .map_err(|err| format!("cannot decode CipherKey: {err}"))?;
        let key = self.master_cipher.decrypt(&encrypted_key)?;

        let encrypted_iv = general_purpose::STANDARD
            .decode(&envelope.iv)
            .map_err(|err| format!("cannot decode IV: {err}"))?;
        let iv = self.master_cipher.decrypt(&encrypted_iv)?;

        let cipher_data = CipherData {
            iv,
            key,
            mat_desc: envelope.mat_desc.clone(),
            wrap_algorithm: envelope.wrap_alg.clone(),
            cek_algorithm: envelope.cek_alg.clone(),
            encrypted_iv,
            encrypted_key,
        };
        AesCtrCipher::new(cipher_data)
    }
}

/// The client-side-encryption metadata keys.
///
/// Two spellings are needed because the crate's request models treat user
/// metadata as a map whose keys the `usermeta` macro prefixes with
/// `x-oss-meta-` itself. Writing into `PutObjectRequest::metadata` therefore
/// takes the bare key, while reading a response's header map — where the
/// service has already applied the prefix — takes the full name.
pub mod headers {
    /// The bare metadata keys, for request models that add the prefix.
    pub mod meta {
        /// The wrapped content key, base64.
        pub const KEY: &str = "client-side-encryption-key";
        /// The wrapped IV, base64.
        pub const START: &str = "client-side-encryption-start";
        /// The content algorithm.
        pub const CEK_ALG: &str = "client-side-encryption-cek-alg";
        /// The wrapping algorithm.
        pub const WRAP_ALG: &str = "client-side-encryption-wrap-alg";
        /// The master key description.
        pub const MAT_DESC: &str = "client-side-encryption-matdesc";
        /// The plaintext length.
        pub const UNENCRYPTED_CONTENT_LENGTH: &str =
            "client-side-encryption-unencrypted-content-length";
        /// The plaintext MD5.
        pub const UNENCRYPTED_CONTENT_MD5: &str = "client-side-encryption-unencrypted-content-md5";
        /// The plaintext length of a multipart upload.
        pub const DATA_SIZE: &str = "client-side-encryption-data-size";
        /// The part size of a multipart upload.
        pub const PART_SIZE: &str = "client-side-encryption-part-size";
    }

    /// The bare metadata keys, re-exported under the names callers use when
    /// filling a request model.
    pub use meta::{
        CEK_ALG, DATA_SIZE, KEY, MAT_DESC, PART_SIZE, START, UNENCRYPTED_CONTENT_LENGTH,
        UNENCRYPTED_CONTENT_MD5, WRAP_ALG,
    };

    /// The full header names, as they appear on the wire.
    pub mod wire {
        /// The wrapped content key, base64.
        pub const KEY: &str = "x-oss-meta-client-side-encryption-key";
        /// The wrapped IV, base64.
        pub const START: &str = "x-oss-meta-client-side-encryption-start";
        /// The content algorithm.
        pub const CEK_ALG: &str = "x-oss-meta-client-side-encryption-cek-alg";
        /// The wrapping algorithm.
        pub const WRAP_ALG: &str = "x-oss-meta-client-side-encryption-wrap-alg";
        /// The master key description.
        pub const MAT_DESC: &str = "x-oss-meta-client-side-encryption-matdesc";
        /// The plaintext length.
        pub const UNENCRYPTED_CONTENT_LENGTH: &str =
            "x-oss-meta-client-side-encryption-unencrypted-content-length";
        /// The plaintext MD5.
        pub const UNENCRYPTED_CONTENT_MD5: &str =
            "x-oss-meta-client-side-encryption-unencrypted-content-md5";
        /// The plaintext length of a multipart upload.
        pub const DATA_SIZE: &str = "x-oss-meta-client-side-encryption-data-size";
        /// The part size of a multipart upload.
        pub const PART_SIZE: &str = "x-oss-meta-client-side-encryption-part-size";
    }
}

/// Reads an envelope out of an object's headers.
///
/// The wrapped key and IV arrive base64-encoded; the rest is plain text. Every
/// field is optional on the wire, and a missing one is reported by
/// [`Envelope::is_valid`] rather than here.
pub fn envelope_from_headers(
    headers: &std::collections::HashMap<String, String>,
) -> Result<Envelope, String> {
    let get = |name: &str| -> String {
        headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.clone())
            .unwrap_or_default()
    };
    Ok(Envelope {
        iv: get(headers::wire::START),
        cipher_key: get(headers::wire::KEY),
        mat_desc: get(headers::wire::MAT_DESC),
        wrap_alg: get(headers::wire::WRAP_ALG),
        cek_alg: get(headers::wire::CEK_ALG),
        unencrypted_md5: get(headers::wire::UNENCRYPTED_CONTENT_MD5),
        unencrypted_content_len: get(headers::wire::UNENCRYPTED_CONTENT_LENGTH),
    })
}

/// Whether an object is encrypted, judged by whether it carries a wrapped key.
pub fn has_encrypted_header(headers: &std::collections::HashMap<String, String>) -> bool {
    headers
        .iter()
        .any(|(key, value)| key.eq_ignore_ascii_case(headers::wire::KEY) && !value.is_empty())
}

/// Whether the content algorithm is one this crate can decrypt.
pub fn is_valid_content_alg(alg_name: &str) -> bool {
    alg_name == AES_CTR_ALGORITHM
}

/// Rounds `start` down to the previous block boundary.
///
/// A ranged read of a CTR-encrypted object has to start on a boundary,
/// because the keystream is only defined per block; the leading bytes are
/// fetched and discarded.
pub fn adjust_range_start(start: i64, align: i64) -> i64 {
    (start / align) * align
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test fixtures, copied verbatim from the upstream Go SDK's own mock test
    // (`encryption_client_mock_test.go`). They are a throwaway key pair that
    // exists only in that public repository and guard nothing; they are here
    // so the two implementations are exercised against the same material.
    const PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCokfiAVXXf5ImFzKDw+XO/UByW
6mse2QsIgz3ZwBtMNu59fR5zttSx+8fB7vR4CN3bTztrP9A6bjoN0FFnhlQ3vNJC
5MFO1PByrE/MNd5AAfSVba93I6sx8NSk5MzUCA4NJzAUqYOEWGtGBcom6kEF6MmR
1EKib1Id8hpooY5xaQIDAQAB
-----END PUBLIC KEY-----";

    const PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIICdQIBADANBgkqhkiG9w0BAQEFAASCAl8wggJbAgEAAoGBAKiR+IBVdd/kiYXM
oPD5c79QHJbqax7ZCwiDPdnAG0w27n19HnO21LH7x8Hu9HgI3dtPO2s/0DpuOg3Q
UWeGVDe80kLkwU7U8HKsT8w13kAB9JVtr3cjqzHw1KTkzNQIDg0nMBSpg4RYa0YF
yibqQQXoyZHUQqJvUh3yGmihjnFpAgMBAAECgYA49RmCQ14QyKevDfVTdvYlLmx6
kbqgMbYIqk+7w611kxoCTMR9VMmJWgmk/Zic9mIAOEVbd7RkCdqT0E+xKzJJFpI2
ZHjrlwb21uqlcUqH1Gn+wI+jgmrafrnKih0kGucavr/GFi81rXixDrGON9KBE0FJ
cPVdc0XiQAvCBnIIAQJBANXu3htPH0VsSznfqcDE+w8zpoAJdo6S/p30tcjsDQnx
l/jYV4FXpErSrtAbmI013VYkdJcghNSLNUXppfk2e8UCQQDJt5c07BS9i2SDEXiz
byzqCfXVzkdnDj9ry9mba1dcr9B9NCslVelXDGZKvQUBqNYCVxg398aRfWlYDTjU
IoVVAkAbTyjPN6R4SkC4HJMg5oReBmvkwFCAFsemBk0GXwuzD0IlJAjXnAZ+/rIO
ItewfwXIL1Mqz53lO/gK+q6TR585AkB304KUIoWzjyF3JqLP3IQOxzns92u9EV6l
V2P+CkbMPXiZV6sls6I4XppJXX2i3bu7iidN3/dqJ9izQK94fMU9AkBZvgsIPCot
y1/POIbv9LtnviDKrmpkXgVQSU4BmTPvXwTJm8APC7P/horSh3SVf1zgmnsyjm9D
hO92gGc+4ajL
-----END PRIVATE KEY-----";

    fn master() -> MasterRsaCipher {
        MasterRsaCipher::new(&std::collections::HashMap::new(), PUBLIC_KEY, PRIVATE_KEY)
    }

    /// The keystream at an offset must not depend on the bytes before it: that
    /// property is what makes a ranged read decrypt correctly.
    #[test]
    fn test_ctr_offset_round_trip() {
        let builder = ContentCipherBuilder::new(Box::new(master()));
        let cipher = builder.content_cipher().expect("cipher");
        let plaintext: Vec<u8> = (0..256u32).map(|i| (i % 251) as u8).collect();

        let ciphertext = cipher.encrypt(&plaintext).expect("encrypt");
        assert_eq!(
            ciphertext.len(),
            plaintext.len(),
            "CTR must not change the length"
        );

        // Decrypting from an aligned offset must reproduce the same bytes as
        // decrypting the whole thing, because the counter is seeked to it.
        let offset = 64u64;
        let positioned = cipher.clone_at_offset(offset).expect("clone at offset");
        let tail = positioned
            .decrypt(&ciphertext[offset as usize..])
            .expect("decrypt tail");
        assert_eq!(tail, plaintext[offset as usize..]);

        // And the full decryption still matches.
        let full = cipher.decrypt(&ciphertext).expect("decrypt");
        assert_eq!(full, plaintext);
    }

    /// The same plaintext must encrypt differently every time, because a fresh
    /// key and IV are minted per object.
    #[test]
    fn test_fresh_key_and_iv_per_object() {
        let builder = ContentCipherBuilder::new(Box::new(master()));
        let first = builder.content_cipher().expect("first");
        let second = builder.content_cipher().expect("second");
        assert_ne!(
            first.cipher_data().key,
            second.cipher_data().key,
            "content keys must not repeat"
        );

        let data = b"same plaintext";
        assert_ne!(
            first.encrypt(data).unwrap(),
            second.encrypt(data).unwrap(),
            "identical plaintext must not produce identical ciphertext"
        );
    }

    /// An envelope written to headers must be readable back and must still
    /// decrypt the object.
    #[test]
    fn test_envelope_round_trip_through_headers() {
        let builder = ContentCipherBuilder::new(Box::new(master()));
        let cipher = builder.content_cipher().expect("cipher");
        let cipher_data = cipher.cipher_data();

        let mut headers = std::collections::HashMap::new();
        headers.insert(
            headers::wire::KEY.to_string(),
            general_purpose::STANDARD.encode(&cipher_data.encrypted_key),
        );
        headers.insert(
            headers::wire::START.to_string(),
            general_purpose::STANDARD.encode(&cipher_data.encrypted_iv),
        );
        headers.insert(
            headers::wire::CEK_ALG.to_string(),
            AES_CTR_ALGORITHM.to_string(),
        );
        headers.insert(
            headers::wire::WRAP_ALG.to_string(),
            RSA_CRYPTO_WRAP.to_string(),
        );
        headers.insert(headers::wire::MAT_DESC.to_string(), "{}".to_string());

        assert!(has_encrypted_header(&headers));
        let envelope = envelope_from_headers(&headers).expect("envelope");
        assert!(envelope.is_valid());
        assert!(is_valid_content_alg(&envelope.cek_alg));

        let rebuilt = builder
            .content_cipher_from_envelope(&envelope)
            .expect("rebuild");
        let plaintext = b"round trip";
        let ciphertext = cipher.encrypt(plaintext).unwrap();
        assert_eq!(rebuilt.decrypt(&ciphertext).unwrap(), plaintext);
    }

    /// Without the key a valid envelope cannot decrypt: the master key must be
    /// the one that wrapped it.
    #[test]
    fn test_wrong_master_key_cannot_unwrap() {
        let other_public = "-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQC6VIgfsPq979hYMNEoDG1pfG58
1FXN7d2GwPPR9d5a8O7+kVGy/PhxpbOWBqKg+JTmxv7AkMmlndAf18zoY5UnNW+d
58mYZrPODBiepxdjUD/tYI2NQcgzCs3slRRRb5faa5a+l6biUJWNBf1uKW4y7JPD
8eEIseQKCW0oRJVLtQIDAQAB
-----END PUBLIC KEY-----";
        let other_private = "-----BEGIN RSA PRIVATE KEY-----
MIICXgIBAAKBgQC6VIgfsPq979hYMNEoDG1pfG581FXN7d2GwPPR9d5a8O7+kVGy
/PhxpbOWBqKg+JTmxv7AkMmlndAf18zoY5UnNW+d58mYZrPODBiepxdjUD/tYI2N
QcgzCs3slRRRb5faa5a+l6biUJWNBf1uKW4y7JPD8eEIseQKCW0oRJVLtQIDAQAB
AoGBAJrzWRAhuSLipeMRFZ5cV1B1rdwZKBHMUYCSTTC5amPuIJGKf4p9XI4F4kZM
1klO72TK72dsAIS9rCoO59QJnCpG4CvLYlJ37wA2UbhQ1rBH5dpBD/tv3CUyfdtI
9CLUsZR3DGBWXYwGG0KGMYPExe5Hq3PUH9+QmuO+lXqJO4IBAkEA6iLee6oBzu6v
90zrr4YA9NNr+JvtplpISOiL/XzsU6WmdXjzsFLSsZCeaJKsfdzijYEceXY7zUNa
0/qQh2BKoQJBAMu61rQ5wKtql2oR4ePTSm00/iHoIfdFnBNU+b8uuPXlfwU80OwJ
Gbs0xBHe+dt4uT53QLci4KgnNkHS5lu4XJUCQQCisCvrvcuX4B6BNf+mbPSJKcci
biaJqr4DeyKatoz36mhpw+uAH2yrWRPZEeGtayg4rvf8Jf2TuTOJi9eVWYFBAkEA
uIPzyS81TQsxL6QajpjjI52HPXZcrPOis++Wco0Cf9LnA/tczSpA38iefAETEq94
bqkGRbY9W0f+Bn8sHVhcBQ==
-----END RSA PRIVATE KEY-----";

        let builder = ContentCipherBuilder::new(Box::new(master()));
        let cipher = builder.content_cipher().expect("cipher");
        let cipher_data = cipher.cipher_data();
        let envelope = Envelope {
            iv: general_purpose::STANDARD.encode(&cipher_data.encrypted_iv),
            cipher_key: general_purpose::STANDARD.encode(&cipher_data.encrypted_key),
            mat_desc: String::new(),
            wrap_alg: RSA_CRYPTO_WRAP.to_string(),
            cek_alg: AES_CTR_ALGORITHM.to_string(),
            unencrypted_md5: String::new(),
            unencrypted_content_len: String::new(),
        };

        let wrong = ContentCipherBuilder::new(Box::new(MasterRsaCipher::new(
            &std::collections::HashMap::new(),
            other_public,
            other_private,
        )));
        assert!(
            wrong.content_cipher_from_envelope(&envelope).is_err(),
            "a different master key must not unwrap the envelope"
        );
    }

    /// An incomplete envelope must be rejected rather than half-used.
    #[test]
    fn test_envelope_validity() {
        let complete = Envelope {
            iv: "a".to_string(),
            cipher_key: "b".to_string(),
            mat_desc: String::new(),
            wrap_alg: "c".to_string(),
            cek_alg: "d".to_string(),
            unencrypted_md5: String::new(),
            unencrypted_content_len: String::new(),
        };
        assert!(complete.is_valid());

        for missing in ["iv", "cipher_key", "wrap_alg", "cek_alg"] {
            let mut envelope = complete.clone();
            match missing {
                "iv" => envelope.iv.clear(),
                "cipher_key" => envelope.cipher_key.clear(),
                "wrap_alg" => envelope.wrap_alg.clear(),
                _ => envelope.cek_alg.clear(),
            }
            assert!(
                !envelope.is_valid(),
                "an envelope missing {missing} must be invalid"
            );
        }
    }

    /// The pivot of a ranged read must land on a block boundary, and the
    /// discarded prefix must be exactly the bytes before the requested offset.
    #[test]
    fn test_adjust_range_start() {
        assert_eq!(adjust_range_start(0, 16), 0);
        assert_eq!(adjust_range_start(15, 16), 0);
        assert_eq!(adjust_range_start(16, 16), 16);
        assert_eq!(adjust_range_start(17, 16), 16);
        assert_eq!(adjust_range_start(100, 16), 96);
    }

    /// The counter must round-trip through the IV's last 8 bytes, and seeking
    /// must advance it by whole blocks.
    #[test]
    fn test_iv_counter() {
        let mut data = CipherData {
            iv: vec![0u8; 16],
            key: vec![0u8; 16],
            ..Default::default()
        };
        data.set_iv(0x0102030405060708);
        assert_eq!(data.get_iv(), 0x0102030405060708);
        // The first 8 bytes stay untouched by the counter.
        assert_eq!(&data.iv[..8], &[0u8; 8]);

        data.set_iv(0);
        data.seek_iv(32);
        assert_eq!(data.get_iv(), 2, "32 bytes is two 16-byte blocks");
        data.seek_iv(31);
        assert_eq!(data.get_iv(), 3, "a partial block still advances one");
    }
}
