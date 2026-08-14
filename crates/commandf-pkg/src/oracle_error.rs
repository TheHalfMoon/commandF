use std::io;

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

    #[error("oracle adapter path is not a regular file: {path}")]
    AdapterPath { path: String },

    #[error("oracle Java executable path is not a regular file: {path}")]
    JavaPath { path: String },

    #[error("--oracle-java is required when --oracle-adapter points to a JAR")]
    JavaRequiredForJar,

    #[error("oracle adapter I/O failure while {operation}: {source}")]
    AdapterIo {
        operation: &'static str,
        #[source]
        source: io::Error,
    },

    #[error("oracle adapter timed out after {millis} ms")]
    AdapterTimeout { millis: u128 },

    #[error("oracle adapter {stream} exceeded limit: {actual} > {limit} bytes")]
    AdapterOutputLimit {
        stream: &'static str,
        actual: usize,
        limit: usize,
    },

    #[error("oracle adapter {stream} capture thread failed")]
    AdapterCaptureThread { stream: &'static str },

    #[error("oracle adapter exited unsuccessfully with code {code:?}: {stderr}")]
    AdapterExit { code: Option<i32>, stderr: String },

    #[error("oracle report JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
}
