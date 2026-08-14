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
mod check;
mod check_error;
mod check_github;
mod check_model;
mod check_sarif;
mod compatibility;
mod compatibility_error;
mod compatibility_model;
mod compatibility_validate;
mod error;
mod lock;
mod model;
mod registry;
mod resolver;
mod source;
mod source_map;
mod source_map_error;
mod source_map_model;

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
pub use check::{evaluate_compatibility_policy, validate_check_report};
pub use check_error::CheckError;
pub use check_github::{
    check_report_to_github_annotations_bytes,
    source_mapped_check_report_to_github_annotations_bytes,
};
pub use check_model::{CheckDecision, CheckDirection, CheckFailOn, CheckPolicy, CheckReport};
pub use check_sarif::check_report_to_sarif_bytes;
pub use compatibility_error::CompatibilityError;
pub use compatibility_model::{
    CompatibilityDirection, CompatibilityFinding, CompatibilityReport, CompatibilitySeverity,
};
pub use compatibility_validate::classify_structural_diff;
pub use error::PackageError;
pub use lock::{LockedPackage, Lockfile};
pub use model::{PackageName, PackageRequest, VersionConstraint};
pub use registry::FhirRegistrySource;
pub use resolver::Resolver;
pub use source::{LocalMirrorSource, PackageArchive, PackageSource};
pub use source_map::{
    build_source_mapped_check_report, validate_source_mapped_check_report,
    MAX_SUSHI_INDEX_ENTRIES,
};
pub use source_map_error::SourceMapError;
pub use source_map_model::{
    SourceIndexEvidence, SourceLocation, SourceMappedCheckReport, SourceMappingEntry,
    SourceMappingStatus,
};
