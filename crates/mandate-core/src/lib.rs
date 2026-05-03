//! `mandate-core` — types, canonical serialization, and verification for the
//! Mandate v0 wire format.
//!
//! See `docs/specs/2026-05-03-mandate-v0.md` for the canonical specification.

#![forbid(unsafe_code)]
#![allow(dead_code)]

pub mod types;
pub mod canonical;
pub mod sign;
pub mod verify;
pub mod attest;
pub mod eth;
pub mod error;

pub use error::Error;
pub use types::Mandate;
