use thiserror::Error;

#[derive(Debug, Error)]
pub enum CheckError {
    #[error("unsupported CF-05 check schema {found}; expected {expected}")]
    UnsupportedCheckSchema { found: u32, expected: u32 },
    #[error("unsupported CF-04 compatibility schema {found}; expected {expected}")]
    UnsupportedCompatibilitySchema { found: u32, expected: u32 },
    #[error("unsupported CF-04 ruleset {found:?}; expected {expected:?}")]
    UnsupportedCompatibilityRuleset { found: String, expected: String },
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
