//! End-to-end sign + verify against the canonical worked example.

use k256::ecdsa::SigningKey;
use mandate_core::canonical::mandate_id_for;
use mandate_core::error::Error;
use mandate_core::eth::{address_from_secret, caip10_eip155};
use mandate_core::sign::sign_buyer_eip191;
use mandate_core::types::Mandate;
use mandate_core::verify::verify_mandate;

const EXAMPLE_COFFEE: &str =
    include_str!("../../../examples/coffee.mandate.json");

fn example() -> Mandate {
    serde_json::from_str(EXAMPLE_COFFEE).unwrap()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_slice(&[seed; 32]).unwrap()
}

/// Reset a mandate to "ready to sign" with a fresh wallet we control.
fn rebind_for_signing(m: &mut Mandate, sk: &SigningKey) {
    m.buyer.wallet = caip10_eip155(8453, &address_from_secret(sk));
    m.buyer.sig.alg = "eip191".to_string();
    m.buyer.sig.value = String::new();
    m.id = mandate_id_for(m).unwrap();
}

#[test]
fn sign_and_verify_a_real_mandate_end_to_end() {
    let mut m = example();
    let sk = key(7);
    rebind_for_signing(&mut m, &sk);
    sign_buyer_eip191(&mut m, &sk).unwrap();
    verify_mandate(&m).unwrap();
}

#[test]
fn verify_rejects_tampered_intent() {
    let mut m = example();
    let sk = key(7);
    rebind_for_signing(&mut m, &sk);
    sign_buyer_eip191(&mut m, &sk).unwrap();

    m.intent.brief = "buy me a yacht".to_string();

    let err = verify_mandate(&m).unwrap_err();
    assert!(format!("{err}").contains("id mismatch"));
}

#[test]
fn verify_rejects_wrong_signer() {
    let mut m = example();
    let sk = key(7);
    let other = key(8);

    // Wallet says "other" but the signature is by sk.
    m.buyer.wallet = caip10_eip155(8453, &address_from_secret(&other));
    m.buyer.sig.alg = "eip191".to_string();
    m.buyer.sig.value = String::new();
    m.id = mandate_id_for(&m).unwrap();
    sign_buyer_eip191(&mut m, &sk).unwrap();

    assert!(matches!(verify_mandate(&m).unwrap_err(), Error::BadSignature));
}

#[test]
fn signing_does_not_perturb_id() {
    let mut m = example();
    let sk = key(7);
    rebind_for_signing(&mut m, &sk);
    let id_before_sign = m.id.clone();
    sign_buyer_eip191(&mut m, &sk).unwrap();
    let id_after_sign = mandate_id_for(&m).unwrap();
    assert_eq!(id_before_sign, id_after_sign);
}

#[test]
fn example_with_placeholder_signature_does_not_verify() {
    // The shipped example has placeholder signature data — verify must
    // refuse it. (Useful as a regression guard against accidentally
    // shipping a verifying example with throwaway keys.)
    let m = example();
    assert!(verify_mandate(&m).is_err());
}
