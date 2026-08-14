use thiserror::Error;

use crate::{ArtifactError, PackageError, StructuralDiffError};

#[derive(Debug, Error)]
pub enum TerminologyError {
    #[error("unsupported structural diff schema {schema}")]
    UnsupportedDiffSchema { schema: u32 },

    #[error("unsupported compatibility report schema/ruleset: schema={schema}, ruleset={ruleset}")]
    UnsupportedCompatibility { schema: u32, ruleset: String },

    #[error("invalid terminology field {field} in {resource}: {message}")]
    InvalidField {
        resource: String,
        field: String,
        message: String,
    },

    #[error("duplicate complete CodeSystem concept code {code} in {resource}")]
    DuplicateCode { resource: String, code: String },

    #[error("duplicate exact terminology canonical {canonical}: {first} and {second}")]
    DuplicateCanonical {
        canonical: String,
        first: String,
        second: String,
    },

    #[error("ambiguous terminology canonical {canonical}: {matches} matches")]
    AmbiguousCanonical { canonical: String, matches: usize },

    #[error("malformed terminology canonical reference {reference}")]
    MalformedCanonical { reference: String },

    #[error("terminology proof exceeds {field} limit: {actual} > {limit}")]
    Limit {
        field: &'static str,
        actual: usize,
        limit: usize,
    },

    #[error("terminology resource JSON is invalid in {file}: {source}")]
    Json {
        file: String,
        #[source]
        source: serde_json::Error,
    },

    #[error(transparent)]
    Structural(#[from] StructuralDiffError),

    #[error(transparent)]
    Artifact(#[from] ArtifactError),

    #[error(transparent)]
    Package(#[from] PackageError),
}
