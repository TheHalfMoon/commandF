use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("invalid package name: {0}")]
    InvalidPackageName(String),
    #[error("invalid package request: {0}")]
    InvalidRequest(String),
    #[error("unsupported version constraint for {name}: {constraint}")]
    UnsupportedConstraint { name: String, constraint: String },
    #[error("no version of {name} satisfies {constraint}")]
    NoMatchingVersion { name: String, constraint: String },
    #[error("package version conflict for {name}: selected {selected}, requested {requested}")]
    VersionConflict {
        name: String,
        selected: String,
        requested: String,
    },
    #[error("package not found in source: {name}@{version}")]
    PackageNotFound { name: String, version: String },
    #[error("package archive is missing package/package.json")]
    MissingManifest,
    #[error("package manifest exceeds the maximum supported size")]
    ManifestTooLarge,
    #[error("package identity mismatch: expected {expected}, found {found}")]
    IdentityMismatch { expected: String, found: String },
    #[error("cache object missing: {0}")]
    CacheMissing(String),
    #[error("cache object digest mismatch for {path:?}: expected {expected}, found {found}")]
    CacheDigestMismatch {
        path: PathBuf,
        expected: String,
        found: String,
    },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("semantic version error: {0}")]
    Semver(#[from] semver::Error),
}
