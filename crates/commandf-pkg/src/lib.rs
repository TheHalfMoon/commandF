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
mod durable_retained;
mod ecosystem_cache_identity;
mod ecosystem_closure;
mod ecosystem_comparison;
mod ecosystem_history;
mod ecosystem_lifecycle;
mod ecosystem_snapshot;
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
pub use durable_retained::{
    classify_regenerated_archive, evaluate_durable_packet, DurableProjection, DurableRetainedError,
    HistoricalByteState, LiveArtifactObservation, RegenerationClass, TrustedRetainedBinding,
};
pub use ecosystem_cache_identity::{
    project_cache_identity, require_cache_reuse, CacheIdentity, CacheIdentityError,
    ECOSYSTEM_CACHE_IDENTITY_SCHEMA, MAX_ENGINE_SCHEMA_CHARS,
};
pub use ecosystem_closure::{
    project_closures, query_closures, CanonicalReferenceEdge, ClosureError, ClosurePackage,
    ClosureQuery, EcosystemClosures, PackageDependencyEdge, CANONICAL_AMBIGUOUS,
    CANONICAL_RESOLVED, CANONICAL_UNRESOLVED, ECOSYSTEM_CLOSURE_SCHEMA, ECOSYSTEM_QUERY_SCHEMA,
    MAX_CANONICAL_CHARS, MAX_CLOSURE_EDGES,
};
pub use ecosystem_comparison::{
    project_snapshot_comparison, require_comparison_replay, verify_comparison_identity,
    ComparisonError, ComparisonWitnesses, LifecycleStateChange, PackageChange, PackageMembership,
    ResolutionChange, SnapshotComparison, ECOSYSTEM_COMPARISON_SCHEMA, EVIDENCE_ABSENT,
    EVIDENCE_PRESENT, MAX_COMPARISON_ENGINE_CHARS, MAX_COMPARISON_OUTPUT_BYTES,
    MAX_COMPARISON_RECORDS, STATUS_ABSENT,
};
pub use ecosystem_history::{
    decode_history_machine, encode_history_machine, project_snapshot_history, HistoryError,
    HistoryMachineBytes, SnapshotHistory, ECOSYSTEM_HISTORY_BYTES_SCHEMA, ECOSYSTEM_HISTORY_SCHEMA,
    MAX_HISTORY_MACHINE_BYTES, MAX_HISTORY_STEPS,
};
pub use ecosystem_lifecycle::{
    decode_lifecycle_machine, encode_lifecycle_machine, project_source_lifecycle,
    require_current_sources, verify_lifecycle_record, LifecycleError, LifecycleMachineBytes,
    LifecycleRecord, SourceLifecycle, ECOSYSTEM_LIFECYCLE_BYTES_SCHEMA, ECOSYSTEM_LIFECYCLE_SCHEMA,
    LIFECYCLE_CURRENT, LIFECYCLE_STALE, LIFECYCLE_WITHDRAWN, MAX_LIFECYCLE_MACHINE_BYTES,
    MAX_LIFECYCLE_SOURCES, MAX_LIFECYCLE_STRING_BYTES,
};
pub use ecosystem_snapshot::{
    decode_snapshot_machine, encode_snapshot_machine, project_snapshot,
    require_published_authority, verify_snapshot_identity, EcosystemSnapshot, SnapshotError,
    SnapshotMachineBytes, SnapshotPackage, ECOSYSTEM_SNAPSHOT_BYTES_SCHEMA,
    ECOSYSTEM_SNAPSHOT_SCHEMA, IMMUTABLE_RELEASE, MAX_SNAPSHOT_MACHINE_BYTES, MAX_SNAPSHOT_RECORDS,
    MAX_SNAPSHOT_STRING_BYTES, MUTABLE_CI,
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
