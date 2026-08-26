use thiserror::Error;

use crate::{ArtifactError, PackageError};

#[derive(Debug, Error)]
pub enum ContextGraphError {
    #[error("commandf context requires commandf.lock schema 2; found schema {found}")]
    RequiresLockV2 { found: u32 },
    #[error("context graph package cache error: {0}")]
    Package(#[from] PackageError),
    #[error("context graph artifact inspection error: {0}")]
    Artifact(#[from] ArtifactError),
    #[error("context graph resource {file} field {path} must be {expected}")]
    InvalidResourceField {
        file: String,
        path: String,
        expected: &'static str,
    },
    #[error("context graph artifact inventory mismatch for {file}")]
    ArtifactInventoryMismatch { file: String },
    #[error("context graph canonical reference in {file} at {path} has an empty target URL")]
    EmptyCanonicalTarget { file: String, path: String },
    #[error("context graph canonical reference in {file} at {path} has an empty explicit version")]
    EmptyCanonicalVersion { file: String, path: String },
}
