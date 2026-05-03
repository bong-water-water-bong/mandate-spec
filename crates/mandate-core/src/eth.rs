//! EVM (`eip155` / EIP-191) crypto helpers — sign, recover, verify,
//! address derivation, and CAIP-10 parsing.
//!
//! Mandate v0 uses EIP-191 `personal_sign` over the canonical body for
//! buyer wallets on EVM chains. The signature is encoded as the standard
//! 65-byte `r || s || v` form, hex-prefixed with `0x`.

use crate::error::Error;
use k256::ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey};
use sha3::{Digest, Keccak256};

pub type Address = [u8; 20];

/// Compute the EIP-191 `personal_sign` prehash:
/// `keccak256("\x19Ethereum Signed Message:\n" || len(message) || message)`.
pub fn personal_sign_hash(message: &[u8]) -> [u8; 32] {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut hasher = Keccak256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(message);
    hasher.finalize().into()
}

/// Sign a message under EIP-191 `personal_sign`. Returns a `0x`-prefixed
/// 65-byte hex signature (`r || s || v`, where `v` is 27 or 28).
pub fn sign_personal(message: &[u8], secret: &SigningKey) -> Result<String, Error> {
    let prehash = personal_sign_hash(message);
    let (sig, recovery_id) = secret
        .sign_prehash_recoverable(&prehash)
        .map_err(|e| Error::Crypto(format!("ecdsa sign: {e}")))?;
    let mut bytes = [0u8; 65];
    bytes[..64].copy_from_slice(&sig.to_bytes());
    bytes[64] = recovery_id.to_byte() + 27;
    Ok(format!("0x{}", hex::encode(bytes)))
}

/// Recover the signer's Ethereum address from an EIP-191 signature.
pub fn recover_personal(message: &[u8], sig_hex: &str) -> Result<Address, Error> {
    let bytes = decode_hex_prefixed(sig_hex)?;
    if bytes.len() != 65 {
        return Err(Error::Crypto(format!(
            "expected 65-byte signature, got {}",
            bytes.len()
        )));
    }
    let v = bytes[64];
    let recovery_id_byte = v.wrapping_sub(27);
    if recovery_id_byte > 1 {
        return Err(Error::Crypto(format!("unsupported v: {v}")));
    }
    let recovery_id = RecoveryId::try_from(recovery_id_byte)
        .map_err(|e| Error::Crypto(format!("recovery id: {e}")))?;
    let sig = Signature::from_slice(&bytes[..64])
        .map_err(|e| Error::Crypto(format!("ecdsa sig: {e}")))?;
    let prehash = personal_sign_hash(message);
    let pk = VerifyingKey::recover_from_prehash(&prehash, &sig, recovery_id)
        .map_err(|e| Error::Crypto(format!("recover: {e}")))?;
    Ok(address_from_pubkey(&pk))
}

/// Verify that an EIP-191 signature was produced by the wallet whose
/// address matches `expected_caip10` (a CAIP-10 `eip155:<chain>:0x...` id).
pub fn verify_personal(
    message: &[u8],
    sig_hex: &str,
    expected_caip10: &str,
) -> Result<(), Error> {
    let recovered = recover_personal(message, sig_hex)?;
    let (_chain, expected) = parse_caip10_eip155(expected_caip10)?;
    if recovered != expected {
        return Err(Error::BadSignature);
    }
    Ok(())
}

/// Derive an Ethereum address from a public key
/// (last 20 bytes of `keccak256(uncompressed_pubkey[1..])`).
pub fn address_from_pubkey(pk: &VerifyingKey) -> Address {
    let encoded = pk.to_encoded_point(false);
    let hash = Keccak256::digest(&encoded.as_bytes()[1..]);
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&hash[12..]);
    addr
}

/// Convenience: derive an address from a secret key.
pub fn address_from_secret(secret: &SigningKey) -> Address {
    address_from_pubkey(secret.verifying_key())
}

/// Render an address as a CAIP-10 `eip155` identifier.
pub fn caip10_eip155(chain_id: u64, address: &Address) -> String {
    format!("eip155:{}:0x{}", chain_id, hex::encode(address))
}

/// Parse a CAIP-10 `eip155` identifier into `(chain_id, address)`.
pub fn parse_caip10_eip155(s: &str) -> Result<(u64, Address), Error> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 || parts[0] != "eip155" {
        return Err(Error::Schema(format!(
            "expected eip155 CAIP-10 identifier, got: {s}"
        )));
    }
    let chain: u64 = parts[1]
        .parse()
        .map_err(|e| Error::Schema(format!("bad chain id: {e}")))?;
    let addr_str = parts[2]
        .strip_prefix("0x")
        .ok_or_else(|| Error::Schema(format!("address must start with 0x: {}", parts[2])))?;
    if addr_str.len() != 40 {
        return Err(Error::Schema(format!(
            "expected 40 hex chars in address, got {}",
            addr_str.len()
        )));
    }
    let bytes = hex::decode(addr_str)?;
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&bytes);
    Ok((chain, addr))
}

fn decode_hex_prefixed(s: &str) -> Result<Vec<u8>, Error> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    Ok(hex::decode(s)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> SigningKey {
        SigningKey::from_slice(&[1u8; 32]).unwrap()
    }

    #[test]
    fn sign_then_verify_roundtrip() {
        let sk = key();
        let caip = caip10_eip155(8453, &address_from_secret(&sk));
        let sig = sign_personal(b"hello mandate", &sk).unwrap();
        verify_personal(b"hello mandate", &sig, &caip).unwrap();
    }

    #[test]
    fn verify_fails_on_tampered_message() {
        let sk = key();
        let caip = caip10_eip155(8453, &address_from_secret(&sk));
        let sig = sign_personal(b"hello mandate", &sk).unwrap();
        assert!(verify_personal(b"hello mAndate", &sig, &caip).is_err());
    }

    #[test]
    fn verify_fails_on_wrong_wallet() {
        let sk = key();
        let other = SigningKey::from_slice(&[2u8; 32]).unwrap();
        let other_caip = caip10_eip155(8453, &address_from_secret(&other));
        let sig = sign_personal(b"hello", &sk).unwrap();
        assert!(matches!(
            verify_personal(b"hello", &sig, &other_caip).unwrap_err(),
            Error::BadSignature
        ));
    }

    #[test]
    fn caip10_roundtrip() {
        let addr = [0xabu8; 20];
        let s = caip10_eip155(1, &addr);
        let (chain, parsed) = parse_caip10_eip155(&s).unwrap();
        assert_eq!(chain, 1);
        assert_eq!(parsed, addr);
    }

    #[test]
    fn caip10_rejects_wrong_namespace() {
        assert!(parse_caip10_eip155("solana:1:0xabcd").is_err());
    }

    #[test]
    fn caip10_rejects_missing_0x() {
        assert!(parse_caip10_eip155("eip155:1:abcdef0123456789012345678901234567890123").is_err());
    }

    #[test]
    fn caip10_rejects_wrong_address_length() {
        assert!(parse_caip10_eip155("eip155:1:0xabcd").is_err());
    }

    #[test]
    fn personal_sign_hash_is_deterministic_and_input_dependent() {
        assert_eq!(personal_sign_hash(b"x"), personal_sign_hash(b"x"));
        assert_ne!(personal_sign_hash(b"x"), personal_sign_hash(b"y"));
    }

    #[test]
    fn signature_is_65_bytes_hex_prefixed() {
        let sig = sign_personal(b"x", &key()).unwrap();
        assert!(sig.starts_with("0x"));
        assert_eq!(sig.len(), 2 + 65 * 2);
    }
}
