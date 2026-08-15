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
mod corpus;
mod corpus_error;
mod corpus_evaluate;
mod corpus_model;
mod error;
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
pub use corpus::{
    attest_corpus_package_state, canonical_corpus_manifest_bytes, parse_corpus_manifest,
    validate_corpus_manifest, MAX_CORPUS_ARCHIVE_BYTES, MAX_CORPUS_CASES,
    MAX_CORPUS_MANIFEST_BYTES,
};
pub use corpus_error::CorpusError;
pub use corpus_evaluate::{
    evaluate_corpus_case, evaluate_corpus_compatibility, evaluate_corpus_structural,
    evaluate_corpus_terminology, failed_corpus_case_summary,
    failed_corpus_case_summary_with_closure, summarize_corpus_case, CorpusCaseReports,
    CorpusPackageStateInput,
};
pub use corpus_model::{
    CorpusCaseStatus, CorpusCaseSummary, CorpusClosurePackage, CorpusCompatibilitySummary,
    CorpusOracleMode, CorpusOracleSummary, CorpusPackageAttestation, CorpusPackageSide,
    CorpusPackageState, CorpusRightsMode, CorpusRunSummary, CorpusSelectionPolicy,
    CorpusStructuralSummary, CorpusSummaryPackageState, CorpusTerminologySummary, RealIgCase,
    RealIgCorpus,
};
pub use error::PackageError;
pub use lock::{LockedPackage, Lockfile};
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
