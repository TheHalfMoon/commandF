use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompatibilityError {
    #[error("unsupported CF-03 structural diff schema {schema}")]
    UnsupportedDiffSchema { schema: u32 },
    #[error("invalid compatibility evidence for field {field}: {message}")]
    InvalidChangeValue { field: String, message: String },
    #[error("CF-04 has no rule for structural field {field}")]
    UnsupportedStructuralField { field: String },
}
