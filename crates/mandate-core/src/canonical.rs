//! Canonical serialization (JCS, RFC 8785) and content-address derivation.
//!
//! TODO(v0.0.1): Implement JCS canonicalization.
//! For now this module exposes the placeholder API the spec describes.

use crate::error::Error;
use crate::types::Mandate;

/// Serialize a mandate body to JCS-canonical bytes with all signature
/// fields and `lifecycle.bound_to_merchant` zeroed for hashing.
pub fn canonical_body(_m: &Mandate) -> Result<Vec<u8>, Error> {
    Err(Error::NotImplemented("canonical_body"))
}

/// Compute the content-addressed mandate id (`mandate://b3:<base32-blake3>`).
pub fn mandate_id(body: &[u8]) -> String {
    let hash = blake3::hash(body);
    format!("mandate://b3:{}", base32_lower(hash.as_bytes()))
}

fn base32_lower(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";
    let mut out = String::with_capacity((bytes.len() * 8 + 4) / 5);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &b in bytes {
        buf = (buf << 8) | b as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let idx = ((buf >> bits) & 0x1F) as usize;
            out.push(ALPHABET[idx] as char);
        }
    }
    if bits > 0 {
        let idx = ((buf << (5 - bits)) & 0x1F) as usize;
        out.push(ALPHABET[idx] as char);
    }
    out
}
