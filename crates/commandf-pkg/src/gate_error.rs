use thiserror::Error;

use crate::CheckError;

#[derive(Debug, Error)]
pub enum QualityGateError {
    #[error("unsupported CF-13 quality-gate schema {found}; expected {expected}")]
    UnsupportedGateSchema { found: u32, expected: u32 },
    #[error("unsupported CF-13 suppression schema {found}; expected {expected}")]
    UnsupportedSuppressionSchema { found: u32, expected: u32 },
    #[error("unsupported CF-13 fingerprint schema {found}; expected {expected}")]
    UnsupportedFingerprintSchema { found: u32, expected: u32 },
    #[error("malformed CF-13 SHA-256 identity {value:?}")]
    MalformedSha256Identity { value: String },
    #[error("baseline package {baseline:?} does not match current package {current:?}")]
    BaselinePackageMismatch { current: String, baseline: String },
    #[error("baseline ruleset {baseline:?} does not match current ruleset {current:?}")]
    BaselineRulesetMismatch { current: String, baseline: String },
    #[error("duplicate current finding fingerprint {fingerprint}")]
    DuplicateCurrentFingerprint { fingerprint: String },
    #[error("duplicate baseline finding fingerprint {fingerprint}")]
    DuplicateBaselineFingerprint { fingerprint: String },
    #[error("duplicate suppression fingerprint {fingerprint}")]
    DuplicateSuppressionFingerprint { fingerprint: String },
    #[error("suppression count {found} exceeds maximum {maximum}")]
    TooManySuppressions { found: usize, maximum: usize },
    #[error("suppression {field} length {found} exceeds maximum {maximum}")]
    SuppressionStringTooLong {
        field: &'static str,
        found: usize,
        maximum: usize,
    },
    #[error("suppression rationale must not be empty after trimming")]
    EmptySuppressionRationale,
    #[error("persisted CF-13 report is inconsistent: {reason}")]
    InconsistentReport { reason: &'static str },
    #[error("CF-05 validation failed: {0}")]
    Check(#[from] CheckError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
