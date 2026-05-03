use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("not implemented yet: {0}")]
    NotImplemented(&'static str),

    #[error("schema validation failed: {0}")]
    Schema(String),

    #[error("signature verification failed")]
    BadSignature,

    #[error("attestation verification failed: {0}")]
    BadAttestation(String),

    #[error("canonicalization error: {0}")]
    Canonical(String),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
