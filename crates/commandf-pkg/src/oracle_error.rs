use thiserror::Error;

#[derive(Debug, Error)]
pub enum OracleError {
    #[error("unsupported oracle report schema {actual}; expected {expected}")]
    UnsupportedSchema { actual: u32, expected: u32 },

    #[error("oracle identity mismatch: {field} expected {expected}, got {actual}")]
    IdentityMismatch {
        field: &'static str,
        expected: &'static str,
        actual: String,
    },

    #[error("oracle {field} must not be empty")]
    EmptyField { field: &'static str },

    #[error("oracle evidence exceeds limit for {field}: {actual} > {limit}")]
    EvidenceLimit {
        field: &'static str,
        actual: usize,
        limit: usize,
    },

    #[error("duplicate oracle observation for resource {resource}")]
    DuplicateObservation { resource: String },

    #[error(
        "oracle observation identity mismatch for resource {resource}: left={left}, right={right}"
    )]
    ObservationIdentityMismatch {
        resource: String,
        left: String,
        right: String,
    },

    #[error("oracle report JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
}
