//! Verification helpers — the merchant side of the wire.
//!
//! `verify_mandate` performs the two checks every merchant must run before
//! accepting a mandate: (1) recompute the content-addressed `id` from the
//! canonical body and ensure it matches what's stored, and (2) verify the
//! buyer signature recovers to the wallet declared in `buyer.wallet`.
//!
//! Attestation verification (SEV-SNP report + VCEK chain) is in `attest`.

use crate::canonical::{canonical_body, mandate_id_for};
use crate::error::Error;
use crate::eth;
use crate::types::Mandate;

/// Verify a mandate end-to-end (excluding attestation).
pub fn verify_mandate(m: &Mandate) -> Result<(), Error> {
    let computed_id = mandate_id_for(m)?;
    if computed_id != m.id {
        return Err(Error::Schema(format!(
            "mandate id mismatch: stored={}, computed={}",
            m.id, computed_id
        )));
    }
    let body = canonical_body(m)?;
    match m.buyer.sig.alg.as_str() {
        "eip191" => eth::verify_personal(&body, &m.buyer.sig.value, &m.buyer.wallet),
        "ed25519-solana" => Err(Error::NotImplemented("ed25519-solana verify")),
        other => Err(Error::Schema(format!("unknown buyer.sig.alg: {other}"))),
    }
}
