//! AMD SEV-SNP attestation parsing and binding-check.
//!
//! The mandate v0 spec requires that `agent.attestation.report`'s
//! `REPORT_DATA` field's first 64 bytes equal `blake3(canonical_body)[0..64]`.
//!
//! TODO(v0.0.1): parse SNP report, validate VCEK chain against AMD root,
//! check `report_data` against the recomputed canonical-body hash.

use crate::error::Error;
use crate::types::Mandate;

pub fn verify_attestation(_m: &Mandate) -> Result<(), Error> {
    Err(Error::NotImplemented("verify_attestation"))
}
