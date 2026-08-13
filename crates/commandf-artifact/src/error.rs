use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("archive digest mismatch: expected {expected}, found {found}")]
    ArchiveDigestMismatch { expected: String, found: String },
    #[error("archive exceeds the maximum decompressed size")]
    ArchiveTooLarge,
    #[error("archive exceeds the maximum entry count")]
    TooManyEntries,
    #[error("resource exceeds the maximum supported size: {0}")]
    ResourceTooLarge(String),
    #[error("malformed FHIR resource JSON in {file}: {source}")]
    Json {
        file: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("missing or invalid resourceType in {0}")]
    InvalidResourceType(String),
    #[error("field {field} in {file} must be a string when present")]
    InvalidStringField { file: String, field: String },
    #[error("duplicate canonical identity {identity}: {first} and {second}")]
    DuplicateCanonical {
        identity: String,
        first: String,
        second: String,
    },
    #[error("missing element id in {file} {view} element {index}")]
    MissingElementId {
        file: String,
        view: String,
        index: usize,
    },
    #[error("duplicate element id {id} in {file} {view}")]
    DuplicateElementId {
        file: String,
        view: String,
        id: String,
    },
    #[error("invalid StructureDefinition element array in {file} {view}")]
    InvalidElementArray { file: String, view: String },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
