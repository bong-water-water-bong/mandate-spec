//! Verification helpers — the merchant side of the wire.
//!
//! Verifies (1) the buyer wallet signature over the canonical body,
//! (2) optional approval signature, and (3) the agent attestation
//! report against the canonical body hash.
//!
//! TODO(v0.0.1): wire to `sign` + `attest` + `canonical`.

use crate::error::Error;
use crate::types::Mandate;

pub fn verify_mandate(_m: &Mandate) -> Result<(), Error> {
    Err(Error::NotImplemented("verify_mandate"))
}
