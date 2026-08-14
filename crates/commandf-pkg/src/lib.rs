mod archive;
mod artifact_diff;
mod artifact_diff_change;
mod artifact_diff_error;
mod artifact_diff_model;
mod artifact_diff_normalize;
mod artifact_diff_structure;
mod artifact_error;
mod artifact_inspect;
mod artifact_model;
mod artifact_scan;
mod cache;
mod error;
mod lock;
mod model;
mod registry;
mod resolver;
mod source;

pub use artifact_diff::diff_package_archives;
pub use artifact_diff_error::StructuralDiffError;
pub use artifact_diff_model::{
    PackageEvidence, ResourceKey, ResourceKeyKind, StructuralChange, StructuralChangeKind,
    StructuralDiffReport,
};
pub use artifact_error::ArtifactError;
pub use artifact_inspect::inspect_package;
pub use artifact_model::{ElementAddress, ElementView, PackageInspection, ResourceArtifact};
pub use cache::PackageCache;
pub use error::PackageError;
pub use lock::{LockedPackage, Lockfile};
pub use model::{PackageName, PackageRequest, VersionConstraint};
pub use registry::FhirRegistrySource;
pub use resolver::Resolver;
pub use source::{LocalMirrorSource, PackageArchive, PackageSource};
