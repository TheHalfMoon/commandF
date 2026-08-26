use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub enum ImpactError {
    UnsupportedDiffSchema { found: u32 },
    UnsupportedContextSchema { side: &'static str, found: u32 },
    UnsupportedLockSchema { side: &'static str, found: u32 },
    SubjectPackageMissing { side: &'static str, identity: String },
    SubjectPackageAmbiguous { side: &'static str, identity: String },
    ArtifactMissing { side: &'static str, file: String },
    ArtifactAmbiguous { side: &'static str, file: String },
    ConflictingResourceFilename { resource: String, side: &'static str },
    InconsistentResolvedReference { side: &'static str, canonical: String, candidates: usize },
}

impl fmt::Display for ImpactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedDiffSchema { found } => {
                write!(f, "impact requires structural diff schema 1, found {found}")
            }
            Self::UnsupportedContextSchema { side, found } => {
                write!(f, "impact requires context graph schema 1 on {side}, found {found}")
            }
            Self::UnsupportedLockSchema { side, found } => {
                write!(f, "impact requires lock schema 2 context evidence on {side}, found {found}")
            }
            Self::SubjectPackageMissing { side, identity } => {
                write!(f, "impact subject package is missing from {side} context graph: {identity}")
            }
            Self::SubjectPackageAmbiguous { side, identity } => {
                write!(f, "impact subject package is duplicated in {side} context graph: {identity}")
            }
            Self::ArtifactMissing { side, file } => {
                write!(f, "impact diff artifact is missing from {side} context graph: {file}")
            }
            Self::ArtifactAmbiguous { side, file } => {
                write!(f, "impact diff artifact is duplicated in {side} context graph: {file}")
            }
            Self::ConflictingResourceFilename { resource, side } => write!(
                f,
                "impact diff contains conflicting {side} filenames for resource {resource}"
            ),
            Self::InconsistentResolvedReference {
                side,
                canonical,
                candidates,
            } => write!(
                f,
                "resolved canonical reference on {side} must have exactly one candidate: {canonical} has {candidates}"
            ),
        }
    }
}

impl std::error::Error for ImpactError {}
