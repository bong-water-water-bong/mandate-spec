//! Canonical serialization (JCS, RFC 8785) and content-address derivation.
//!
//! All hashes and signatures over a mandate use the same *canonical body*:
//! the mandate, JCS-canonicalized, with the following paths set to `null`:
//!
//! - `id`
//! - `buyer.sig.value`
//! - `approval.approval_sig`
//! - `agent.attestation.report`
//! - `agent.attestation.vcek_chain`
//! - `lifecycle.bound_to_merchant`
//!
//! This makes the canonical body well-defined at every lifecycle state.
//! The mandate id is `mandate://b3:` + base32-lowercase(blake3(canonical_body)).
//! Buyer signatures and the SEV-SNP `REPORT_DATA[0..64]` cover the same bytes.

use crate::error::Error;
use crate::types::Mandate;
use serde_json::Value;
use std::cmp::Ordering;

/// Paths that are nulled to form the canonical body (see module docs).
const CANONICAL_NULLED_PATHS: &[&[&str]] = &[
    &["id"],
    &["buyer", "sig", "value"],
    &["approval", "approval_sig"],
    &["agent", "attestation", "report"],
    &["agent", "attestation", "vcek_chain"],
    &["lifecycle", "bound_to_merchant"],
];

/// Serialize a mandate to JCS-canonical bytes with hashing/signing paths nulled.
pub fn canonical_body(m: &Mandate) -> Result<Vec<u8>, Error> {
    let mut v = serde_json::to_value(m)?;
    for path in CANONICAL_NULLED_PATHS {
        null_path(&mut v, path);
    }
    canonical_value(&v)
}

/// JCS-canonical serialization of an arbitrary JSON value (RFC 8785).
///
/// Restrictions for mandate v0: numbers must be integers (no floats / NaN /
/// Infinity). The mandate body uses string-encoded amounts and integer-only
/// counters so this is sufficient.
pub fn canonical_value(v: &Value) -> Result<Vec<u8>, Error> {
    let mut buf = Vec::new();
    write_canonical(&mut buf, v)?;
    Ok(buf)
}

/// Compute a mandate's id from its current value.
pub fn mandate_id_for(m: &Mandate) -> Result<String, Error> {
    Ok(mandate_id(&canonical_body(m)?))
}

/// Compute a mandate id from already-canonical body bytes.
pub fn mandate_id(canonical: &[u8]) -> String {
    let h = blake3::hash(canonical);
    format!("mandate://b3:{}", base32_lower_no_pad(h.as_bytes()))
}

fn null_path(v: &mut Value, path: &[&str]) {
    let Value::Object(map) = v else { return };
    let Some((head, rest)) = path.split_first() else { return };
    if rest.is_empty() {
        if let Some(slot) = map.get_mut(*head) {
            *slot = Value::Null;
        }
    } else if let Some(child) = map.get_mut(*head) {
        null_path(child, rest);
    }
}

fn write_canonical(out: &mut Vec<u8>, v: &Value) -> Result<(), Error> {
    match v {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(true) => out.extend_from_slice(b"true"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Number(n) => write_number(out, n)?,
        Value::String(s) => write_string(out, s),
        Value::Array(a) => {
            out.push(b'[');
            for (i, item) in a.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_canonical(out, item)?;
            }
            out.push(b']');
        }
        Value::Object(map) => {
            let mut entries: Vec<(&String, &Value)> = map.iter().collect();
            entries.sort_by(|a, b| utf16_cmp(a.0, b.0));
            out.push(b'{');
            for (i, (k, val)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_string(out, k);
                out.push(b':');
                write_canonical(out, val)?;
            }
            out.push(b'}');
        }
    }
    Ok(())
}

fn write_number(out: &mut Vec<u8>, n: &serde_json::Number) -> Result<(), Error> {
    if let Some(i) = n.as_i64() {
        out.extend_from_slice(i.to_string().as_bytes());
        Ok(())
    } else if let Some(u) = n.as_u64() {
        out.extend_from_slice(u.to_string().as_bytes());
        Ok(())
    } else {
        Err(Error::Canonical(format!(
            "non-integer numbers are not allowed in mandate v0: {n}"
        )))
    }
}

fn write_string(out: &mut Vec<u8>, s: &str) {
    out.push(b'"');
    for c in s.chars() {
        match c {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\u{08}' => out.extend_from_slice(b"\\b"),
            '\u{0c}' => out.extend_from_slice(b"\\f"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\t' => out.extend_from_slice(b"\\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write;
                let mut s = String::new();
                let _ = write!(s, "\\u{:04x}", c as u32);
                out.extend_from_slice(s.as_bytes());
            }
            c => {
                let mut buf = [0u8; 4];
                out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            }
        }
    }
    out.push(b'"');
}

fn utf16_cmp(a: &str, b: &str) -> Ordering {
    let mut ai = a.encode_utf16();
    let mut bi = b.encode_utf16();
    loop {
        match (ai.next(), bi.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => match x.cmp(&y) {
                Ordering::Equal => continue,
                o => return o,
            },
        }
    }
}

fn base32_lower_no_pad(bytes: &[u8]) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn objects_canonicalize_independent_of_key_order() {
        let a = json!({"a": 1, "b": 2, "c": 3});
        let b = json!({"c": 3, "b": 2, "a": 1});
        assert_eq!(canonical_value(&a).unwrap(), canonical_value(&b).unwrap());
    }

    #[test]
    fn keys_sorted_lexicographically() {
        let v = json!({"b": 1, "a": 2});
        assert_eq!(canonical_value(&v).unwrap(), br#"{"a":2,"b":1}"#.to_vec());
    }

    #[test]
    fn no_whitespace_anywhere() {
        let v = json!([{"k": "v"}, 1, "x", null, true]);
        assert_eq!(
            canonical_value(&v).unwrap(),
            br#"[{"k":"v"},1,"x",null,true]"#.to_vec()
        );
    }

    #[test]
    fn strings_get_minimal_jcs_escaping() {
        let v = json!("a\"b\\c\nd\te");
        assert_eq!(
            canonical_value(&v).unwrap(),
            b"\"a\\\"b\\\\c\\nd\\te\"".to_vec()
        );
    }

    #[test]
    fn control_chars_are_uescaped() {
        let v = json!("\x01");
        assert_eq!(canonical_value(&v).unwrap(), b"\"\\u0001\"".to_vec());
    }

    #[test]
    fn unicode_strings_pass_through_as_utf8() {
        let v = json!("café \u{1f600}");
        let expected = "\"café 😀\"".as_bytes().to_vec();
        assert_eq!(canonical_value(&v).unwrap(), expected);
    }

    #[test]
    fn integers_serialize_as_decimal() {
        assert_eq!(canonical_value(&json!(0)).unwrap(), b"0".to_vec());
        assert_eq!(canonical_value(&json!(-5)).unwrap(), b"-5".to_vec());
        assert_eq!(
            canonical_value(&json!(1234567890_i64)).unwrap(),
            b"1234567890".to_vec()
        );
    }

    #[test]
    fn floats_are_rejected_in_v0() {
        let v = json!(1.5);
        assert!(canonical_value(&v).is_err());
    }

    #[test]
    fn nested_objects_are_recursively_sorted() {
        let a = json!({"outer": {"b": 1, "a": 2}, "first": true});
        let b = json!({"first": true, "outer": {"a": 2, "b": 1}});
        assert_eq!(canonical_value(&a).unwrap(), canonical_value(&b).unwrap());
    }

    #[test]
    fn array_order_is_preserved() {
        let v = json!([3, 1, 2]);
        assert_eq!(canonical_value(&v).unwrap(), b"[3,1,2]".to_vec());
    }

    #[test]
    fn null_path_zeros_the_target() {
        let mut v = json!({"a": {"b": "secret"}, "c": 1});
        null_path(&mut v, &["a", "b"]);
        assert_eq!(v, json!({"a": {"b": null}, "c": 1}));
    }

    #[test]
    fn null_path_is_a_noop_for_missing_paths() {
        let mut v = json!({"a": 1});
        null_path(&mut v, &["x", "y"]);
        assert_eq!(v, json!({"a": 1}));
    }

    #[test]
    fn mandate_id_format_matches_schema_regex() {
        let id = mandate_id(b"hello world");
        assert!(id.starts_with("mandate://b3:"));
        let suffix = &id["mandate://b3:".len()..];
        assert_eq!(suffix.len(), 52);
        assert!(suffix.chars().all(|c| matches!(c, 'a'..='z' | '2'..='7')));
    }

    #[test]
    fn mandate_id_is_deterministic() {
        let id1 = mandate_id(b"hello");
        let id2 = mandate_id(b"hello");
        assert_eq!(id1, id2);
        assert_ne!(mandate_id(b"hello"), mandate_id(b"world"));
    }

    #[test]
    fn utf16_cmp_handles_surrogate_pairs_correctly() {
        // U+1F600 is a surrogate pair in UTF-16; "z" < surrogate-leading-byte
        // but the actual ordering should be by encoded code units.
        // "a" < "z" < "\u{1f600}" in code-point order; same in UTF-16 order
        // because the high surrogate (0xD83D) > 'z' (0x007A).
        assert_eq!(utf16_cmp("a", "z"), Ordering::Less);
        assert_eq!(utf16_cmp("z", "\u{1f600}"), Ordering::Less);
    }
}
