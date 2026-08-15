use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum CorpusError {
    #[error("corpus manifest is {actual} bytes; maximum is {maximum}")]
    ManifestTooLarge { actual: usize, maximum: usize },
    #[error("corpus manifest JSON is invalid: {0}")]
    InvalidJson(String),
    #[error("unsupported corpus schema {0}; expected schema 1")]
    UnsupportedSchema(u64),
    #[error("corpus must contain at least one case")]
    EmptyCorpus,
    #[error("corpus contains {actual} cases; maximum is {maximum}")]
    TooManyCases { actual: usize, maximum: usize },
    #[error("duplicate corpus case id {0}")]
    DuplicateCaseId(String),
    #[error("corpus cases are not in canonical lexicographic order: {previous} before {current}")]
    NonCanonicalCaseOrder { previous: String, current: String },
    #[error("invalid corpus case id {0}")]
    InvalidCaseId(String),
    #[error("case {case_id} has invalid package name {package}")]
    InvalidPackageName { case_id: String, package: String },
    #[error("case {case_id} has invalid {side} version {version}")]
    InvalidVersion {
        case_id: String,
        side: &'static str,
        version: String,
    },
    #[error("case {0} uses the same before and after version")]
    SameVersion(String),
    #[error("case {case_id} uses unsupported FHIR version {version}; CF-10 v1 requires 4.0.1")]
    UnsupportedFhirVersion { case_id: String, version: String },
    #[error("case {case_id} has invalid {side} SHA-256 {sha256}")]
    InvalidArchiveSha256 {
        case_id: String,
        side: &'static str,
        sha256: String,
    },
    #[error("case {case_id} has invalid {side} archive size {bytes}; maximum is {maximum}")]
    InvalidArchiveSize {
        case_id: String,
        side: &'static str,
        bytes: u64,
        maximum: u64,
    },
    #[error("case {case_id} has invalid or missing {field}")]
    InvalidEvidence { case_id: String, field: &'static str },
    #[error("canonical corpus serialization failed: {0}")]
    Serialization(String),
}
