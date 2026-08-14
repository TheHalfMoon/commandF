use thiserror::Error;

use crate::ArtifactError;

#[derive(Debug, Error)]
pub enum StructuralDiffError {
    #[error(transparent)]
    Artifact(#[from] ArtifactError),
    #[error("ambiguous resource match key {key}: {first} and {second}")]
    AmbiguousResourceKey {
        key: String,
        first: String,
        second: String,
    },
    #[error("duplicate package resource filename: {file}")]
    DuplicateResourceFilename { file: String },
    #[error("inspected resource is missing from the scanned archive inventory: {file}")]
    MissingScannedResource { file: String },
    #[error("invalid structural field {field} in {file}: {message}")]
    InvalidStructuralField {
        file: String,
        field: String,
        message: String,
    },
    #[error("malformed FHIR resource JSON in {file}: {source}")]
    Json {
        file: String,
        #[source]
        source: serde_json::Error,
    },
}
