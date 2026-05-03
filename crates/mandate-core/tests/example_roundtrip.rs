//! Integration tests against the canonical worked example.
//!
//! These tests pin the mandate-id derivation rules: id is recomputable from
//! the canonical body, and mutable / signature fields do not perturb it.

use mandate_core::canonical::{canonical_body, mandate_id_for};
use mandate_core::types::{Mandate, Signature};

const EXAMPLE_COFFEE: &str =
    include_str!("../../../examples/coffee.mandate.json");

fn parse_example() -> Mandate {
    serde_json::from_str(EXAMPLE_COFFEE).expect("example must parse")
}

#[test]
fn example_coffee_id_matches_canonical_body() {
    let m = parse_example();
    let computed = mandate_id_for(&m).expect("compute id");
    assert_eq!(
        m.id, computed,
        "\n  stored id:   {}\n  computed id: {}\n  → update examples/coffee.mandate.json",
        m.id, computed
    );
}

#[test]
fn canonical_body_is_deterministic() {
    let m = parse_example();
    let a = canonical_body(&m).unwrap();
    let b = canonical_body(&m).unwrap();
    assert_eq!(a, b);
}

#[test]
fn buyer_sig_value_does_not_affect_id() {
    let mut m = parse_example();
    let before = mandate_id_for(&m).unwrap();
    m.buyer.sig = Signature {
        alg: "eip191".into(),
        value: "0xANY_OTHER_SIGNATURE".into(),
    };
    let after = mandate_id_for(&m).unwrap();
    assert_eq!(before, after);
}

#[test]
fn bound_to_merchant_does_not_affect_id() {
    let mut m = parse_example();
    let before = mandate_id_for(&m).unwrap();
    m.lifecycle.bound_to_merchant = Some("did:web:roaster.example.com".into());
    let after = mandate_id_for(&m).unwrap();
    assert_eq!(before, after);
}

#[test]
fn attestation_report_does_not_affect_id() {
    let mut m = parse_example();
    let before = mandate_id_for(&m).unwrap();
    m.agent.attestation.report = "BASE64_REAL_SNP_REPORT_OF_ANY_LENGTH".into();
    m.agent.attestation.vcek_chain = Some("BASE64_REAL_VCEK_CHAIN".into());
    let after = mandate_id_for(&m).unwrap();
    assert_eq!(before, after);
}
