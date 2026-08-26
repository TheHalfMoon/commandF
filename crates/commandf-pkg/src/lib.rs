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
mod context;
mod context_error;
mod context_model;
mod error;
mod gate;
mod gate_error;
mod gate_model;
mod impact;
mod impact_error;
mod impact_model;
mod lock;
mod model;
mod oracle_error;
mod oracle_model;
mod oracle_process;
mod oracle_reconcile;
mod registry;
mod resolver;
mod source;
mod source_map;
mod source_map_error;
mod source_map_model;
mod terminology;
mod terminology_error;
mod terminology_index;
mod terminology_model;
mod terminology_set;

pub use artifact_diff::{
    diff_package_archives, matched_structure_definition_pairs, MatchedStructureDefinitionPair,
};
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
pub use context::build_context_graph;
pub use context_error::ContextGraphError;
pub use context_model::{
    CanonicalReferenceRelation, CanonicalResolutionStatus, ContextArtifactIdentity,
    ContextArtifactNode, ContextCanonicalReferenceEdge, ContextCoverage, ContextGraphReport,
    ContextPackageDependencyEdge, ContextPackageIdentity, ContextPackageNode,
};
pub use error::PackageError;
pub use gate::{
    evaluate_quality_gate, finding_fingerprint_v1, validate_quality_gate_report,
    MAX_GATE_SUPPRESSIONS, MAX_GATE_SUPPRESSION_RATIONALE_CHARS,
    MAX_GATE_SUPPRESSION_REFERENCE_CHARS,
};
pub use gate_error::QualityGateError;
pub use gate_model::{
    FindingFingerprint, GateSuppression, GateSuppressions, QualityGateBaselineEvidence,
    QualityGateDecision, QualityGateDisposition, QualityGateFinding, QualityGateReport,
    QualityGateSuppressionEvidence,
};
pub use impact::build_impact_report;
pub use impact_error::ImpactError;
pub use impact_model::{
    ImpactArtifactPathStep, ImpactArtifactRelation, ImpactCoverage, ImpactGraphEvidence,
    ImpactPackagePathStep, ImpactPackageRelation, ImpactReport, ImpactSeed, ImpactSeedKind,
    ImpactSide, ImpactSubject, ImpactUnresolvedBoundary,
};
pub use lock::{LockedPackage, Lockfile, ResolvedDependency};
pub use model::{PackageName, PackageRequest, VersionConstraint};
pub use oracle_error::OracleError;
pub use oracle_model::{
    Hl7OracleReport, OracleChangeState, OracleDivergenceReport, OracleIdentity, OracleMessage,
    OracleMessageLevel, OracleResourceIdentity, OracleResourceResult, OracleResourceStatus,
    OracleStates, HL7_ORACLE_PROJECT, HL7_ORACLE_RELEASE, HL7_ORACLE_SOURCE_COMMIT,
    HL7_VALIDATOR_JAR_SHA256,
};
pub use oracle_process::{
    run_hl7_oracle_adapter, validate_hl7_oracle_adapter, Hl7OracleInvocation,
    DEFAULT_ORACLE_TIMEOUT_SECS, MAX_ORACLE_STDERR_BYTES, MAX_ORACLE_STDOUT_BYTES,
};
pub use oracle_reconcile::{
    parse_hl7_oracle_report, reconcile_hl7_oracle, validate_hl7_oracle_report,
};
pub use registry::FhirRegistrySource;
pub use resolver::Resolver;
pub use source::{LocalMirrorSource, PackageArchive, PackageSource};
pub use source_map::{
    build_source_mapped_check_report, validate_source_mapped_check_report,
    MAX_SOURCE_MAPPED_REPORT_BYTES, MAX_SUSHI_INDEX_ENTRIES, MAX_SUSHI_INDEX_INPUT_BYTES,
};
pub use source_map_error::SourceMapError;
pub use source_map_model::{
    SourceIndexEvidence, SourceLocation, SourceMappedCheckReport, SourceMappingEntry,
    SourceMappingStatus,
};
pub use terminology::{build_terminology_diff_report, TerminologyPackageState};
pub use terminology_error::TerminologyError;
pub use terminology_model::{
    BindingRefinement, TerminologyDiffReport, TerminologyIndeterminateReason, TerminologyMember,
    TerminologyProofMode, TerminologyRelation, TerminologySetDelta,
};
pub use terminology_set::{compare_complete_code_systems, compare_value_set_expansions};
