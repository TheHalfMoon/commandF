use thiserror::Error;

use crate::CheckError;

#[derive(Debug, Error)]
pub enum SourceMapError {
    #[error(transparent)]
    Check(#[from] CheckError),

    #[error("unsupported source-map schema {found}; expected {expected}")]
    UnsupportedSchema { found: u32, expected: u32 },

    #[error("unsupported source-index format {found}; expected {expected}")]
    UnsupportedSourceIndexFormat { found: String, expected: String },

    #[error("SUSHI source index has {found} bytes; maximum is {maximum}")]
    IndexTooLarge { found: usize, maximum: usize },

    #[error("SUSHI source index has {found} entries; maximum is {maximum}")]
    TooManyEntries { found: usize, maximum: usize },

    #[error("source-mapped report has {found} bytes; maximum is {maximum}")]
    ReportTooLarge { found: usize, maximum: usize },

    #[error("invalid SUSHI source index: {0}")]
    InvalidIndex(String),

    #[error("duplicate SUSHI outputFile mapping: {0}")]
    DuplicateOutputFile(String),

    #[error("invalid source path: {0}")]
    InvalidPath(String),

    #[error("mapped FSH source does not exist as a regular file: {0}")]
    MissingSource(String),

    #[error("mapped FSH source escapes the configured source root: {0}")]
    SourceEscape(String),

    #[error(
        "source-map finding count {found} does not match compatibility finding count {expected}"
    )]
    FindingCountMismatch { found: usize, expected: usize },

    #[error("source-map finding index {found} is invalid; expected {expected}")]
    FindingIndexMismatch { found: usize, expected: usize },

    #[error("source-map entry {index} has an invalid status/location combination")]
    InvalidMappingEntry { index: usize },

    #[error("source-map CheckReport does not match the CheckReport being rendered")]
    CheckReportMismatch,

    #[error(
        "persisted source-map evidence does not match the current SUSHI index and source tree"
    )]
    SourceEvidenceMismatch,

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
