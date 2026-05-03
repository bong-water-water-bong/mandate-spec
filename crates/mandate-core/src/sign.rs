//! Buyer-side signing of mandates.
//!
//! v0 supports `eip191` (EVM wallets, SIWE-style). `ed25519-solana` is
//! reserved as a future variant.

use crate::canonical::canonical_body;
use crate::error::Error;
use crate::eth;
use crate::types::{Mandate, Signature};
use k256::ecdsa::SigningKey;

/// Sign a mandate's canonical body and populate `buyer.sig` in place
/// using EIP-191 `personal_sign` over the canonical body bytes.
///
/// `m.buyer.sig.alg` must be `"eip191"` (or empty). The id field, the
/// six paths nulled by `canonical_body`, and the signature itself are
/// excluded from hashing — see the canonical-body definition in §4.5
/// of the spec.
pub fn sign_buyer_eip191(m: &mut Mandate, secret: &SigningKey) -> Result<(), Error> {
    if !m.buyer.sig.alg.is_empty() && m.buyer.sig.alg != "eip191" {
        return Err(Error::Schema(format!(
            "sign_buyer_eip191 requires buyer.sig.alg = \"eip191\" or empty, got: {}",
            m.buyer.sig.alg
        )));
    }
    let body = canonical_body(m)?;
    let value = eth::sign_personal(&body, secret)?;
    m.buyer.sig = Signature {
        alg: "eip191".to_string(),
        value,
    };
    Ok(())
}

/// Sign an approval over `blake3(mandate.id || quote.id || quote.total_canonical_bytes)`
/// with EIP-191 `personal_sign`. Returns the populated `Signature` for the
/// caller to drop into `mandate.approval.approval_sig`.
pub fn sign_approval_eip191(
    mandate_id: &str,
    quote_id: &str,
    quote_total_canonical: &[u8],
    secret: &SigningKey,
) -> Result<Signature, Error> {
    let mut buf =
        Vec::with_capacity(mandate_id.len() + quote_id.len() + quote_total_canonical.len());
    buf.extend_from_slice(mandate_id.as_bytes());
    buf.extend_from_slice(quote_id.as_bytes());
    buf.extend_from_slice(quote_total_canonical);
    let h = blake3::hash(&buf);
    let value = eth::sign_personal(h.as_bytes(), secret)?;
    Ok(Signature {
        alg: "eip191".to_string(),
        value,
    })
}
