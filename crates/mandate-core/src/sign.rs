//! Signing helpers for buyer wallet signatures and approval signatures.
//!
//! TODO(v0.0.1): EIP-191 (EVM) and Ed25519 (Solana) signing paths.

use crate::error::Error;
use crate::types::{Mandate, Signature};

pub fn sign_buyer_body(_m: &Mandate, _wallet_secret: &[u8]) -> Result<Signature, Error> {
    Err(Error::NotImplemented("sign_buyer_body"))
}

pub fn sign_approval(
    _mandate_id: &str,
    _quote_id: &str,
    _quote_total_canonical: &[u8],
    _wallet_secret: &[u8],
) -> Result<Signature, Error> {
    Err(Error::NotImplemented("sign_approval"))
}
