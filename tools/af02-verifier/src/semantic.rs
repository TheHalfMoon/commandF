use std::collections::{BTreeMap, BTreeSet};

use commandf_af02_verifier::canonical::{
    canonical_sha256, git_blob_sha1_hex, parse_json_no_duplicates, sha256_hex, CanonicalError,
};
use serde_json::Value;
use thiserror::Error;

const SEMANTIC_SCHEMA_ID: &str = "commandf.af02-semantic-contract/v1";
const SEMANTIC_SCHEMA_URL: &str =
    "https://commandf.dev/schemas/af02-semantic-contract-v1.schema.json";
const CORE_SCHEMA_URL: &str = "https://commandf.dev/schemas/af02-adversarial-proof-v1.schema.json";
const REQUIRED_SURFACE_CATEGORIES: [&str; 6] = [
    "ARCHIVE_OR_COMPRESSION",
    "CACHE_OR_PERSISTENCE",
    "FILESYSTEM",
    "NETWORK_OR_ACQUISITION",
    "SERDE_OR_TEXT_PARSE",
    "SUBPROCESS",
];
const ALLOWED_TERMINATIONS: [&str; 5] = [
    "SUCCESS",
    "CLASSIFIED_REJECT",
    "WALL_TIMEOUT_KILL",
    "MEMORY_LIMIT_KILL",
    "PID_LIMIT_KILL",
];

pub const ALGORITHM_IDS: [&str; 25] = [
    "CORE_SCHEMA_VALIDATION",
    "PROOF_ENVELOPE_CLOSURE",
    "SEMANTIC_CONTRACT_SELF_CHECK",
    "POLICY_PREDECESSOR_COMPARISON",
    "CANONICAL_SORT_UNIQUE",
    "SURFACE_CATEGORY_WITNESS_CLOSURE",
    "ASSERTION_REPLAY_BIJECTION",
    "SOURCE_BLOB_RECONSTRUCTION",
    "COVERAGE_ACCOUNTING",
    "MUTATION_MEMBERSHIP",
    "WAIVER_CANONICAL_ANCESTRY",
    "REQUIRED_CHECK_GITHUB_PROVENANCE",
    "REQUIRED_CHECK_CROSS_BINDING",
    "RETAINED_LOCATOR_RECONSTRUCTION",
    "RETAINED_PR_RUN_BINDING",
    "CONTRACT_DIGEST_RECONSTRUCTION",
    "EXTENSION_AUTHORITY_DIGEST_BINDING",
    "PATH_NOFOLLOW_CONTAINMENT",
    "COUNTER_EQUALITIES",
    "POLICY_INVENTORY_BINDING",
    "INVENTORY_KEY_MEMBERSHIP",
    "CANDIDATE_INPUT_LIMITS",
    "INPUT_PROCESS_ENFORCEMENT",
    "ENFORCEMENT_INVENTORY_CLOSURE",
    "FINAL_ENVELOPE_HASH",
];

pub const NEGATIVE_FIXTURE_IDS: [&str; 72] = [
    "CORE_SCHEMA_ALIAS_COLLISION",
    "CORE_EMPTY_DETERMINISTIC",
    "EXTENSION_DUPLICATE_ROLE",
    "EXTENSION_DUPLICATE_PATH",
    "EXTENSION_AUTHORITY_DIGEST_MISMATCH",
    "POLICY_BOOTSTRAP_WHEN_PREDECESSOR_EXISTS",
    "POLICY_REBASE_MISSING_PREDECESSOR",
    "POLICY_REBASE_WRONG_PREDECESSOR_BLOB",
    "POLICY_REBASE_WRONG_PREDECESSOR_DIGEST",
    "POLICY_CHANGE_WITH_DEPENDENT_EVIDENCE",
    "EMPTY_SURFACE_MATCHERS",
    "SURFACE_MISSING_CATEGORY",
    "SURFACE_DUPLICATE_MATCHER_ID",
    "SURFACE_DUPLICATE_SURFACE_ID",
    "SURFACE_DUPLICATE_WITNESS_ID",
    "SURFACE_STALE_WITNESS",
    "SURFACE_UNMATCHED_KNOWN_BOUNDARY",
    "EMPTY_TOOL_LOCK",
    "OMITTED_REQUIRED_TOOL",
    "UNEXPECTED_TOOL",
    "SUBSTITUTED_TOOL_ID",
    "WRONG_EXECUTABLE_DIGEST",
    "FABRICATED_WAIVER_ID",
    "WAIVER_WRONG_MUTANT",
    "WAIVER_NOT_PRECANONICAL",
    "CHECK_WRONG_APP",
    "CHECK_WRONG_REPOSITORY",
    "CHECK_WRONG_WORKFLOW_ID",
    "CHECK_WRONG_WORKFLOW_BLOB",
    "CHECK_WRONG_JOB",
    "CHECK_WRONG_HEAD",
    "CHECK_DUPLICATE_CHECK_RUN_ID",
    "CHECK_DUPLICATE_WORKFLOW_RUN_ID",
    "CHECK_DUPLICATE_JOB_ID",
    "RETAINED_WRONG_EVENT",
    "RETAINED_WRONG_WORKFLOW_ID",
    "RETAINED_WRONG_RUN_ATTEMPT",
    "RETAINED_WRONG_CHECK_SUITE",
    "RETAINED_WRONG_PR_ASSOCIATION",
    "RETAINED_WRONG_RUN_HEAD",
    "RETAINED_WRONG_ARTIFACT",
    "UNSORTED_SOURCE_UNIVERSE",
    "DUPLICATE_SOURCE_PATH",
    "ASSERTION_REPLAY_MISSING",
    "ASSERTION_REPLAY_DUPLICATE",
    "CORPUS_DUPLICATE_FIXTURE_PATH",
    "COVERAGE_MISSING_PATH",
    "COVERAGE_DUPLICATE_PATH",
    "COVERAGE_SURFACE_OVERLAP",
    "MUTATION_RESULT_MISSING",
    "MUTATION_EXTRA_RESULT",
    "POLICY_INVENTORY_DIGEST_MISMATCH",
    "INPUT_OVERSIZE",
    "INPUT_TOO_DEEP",
    "INPUT_TOO_MANY_RECORDS",
    "YAML_ALIAS",
    "YAML_CUSTOM_TAG",
    "INPUT_PARSER_WRONG_BINARY",
    "INPUT_PARSER_WRONG_LOCK_BLOB",
    "INPUT_PARSER_NO_CGROUP",
    "INPUT_PARSER_WALL_TIMEOUT_NOT_ENFORCED",
    "INPUT_PARSER_MEMORY_LIMIT_NOT_ENFORCED",
    "INPUT_PARSER_OUTPUT_EVIDENCE_MISSING",
    "INPUT_PARSER_STDOUT_LIMIT_EXCEEDED",
    "INPUT_PARSER_STDERR_LIMIT_EXCEEDED",
    "INPUT_PARSER_OUTPUT_FLAG_MISMATCH",
    "INPUT_PARSER_UNCLASSIFIED_TERMINATION",
    "PATH_SYMLINK_ESCAPE",
    "ENFORCEMENT_MISSING_ROLE",
    "ENFORCEMENT_DUPLICATE_ROLE",
    "ENFORCEMENT_UNCLASSIFIED_ROLE",
    "ENFORCEMENT_ACTIVE_PATH_MISSING",
];

const EXTENSION_ROLES_AND_PATHS: [(&str, &str); 17] = [
    (
        "proof_core_schema",
        "specs/016-af-02-adversarial-test-strength/schemas/af02-adversarial-proof-core-v1.schema.json",
    ),
    (
        "retained_authority_sources_schema",
        "specs/016-af-02-adversarial-test-strength/schemas/af02-retained-authority-sources-v1.schema.json",
    ),
    (
        "waiver_policy",
        "specs/016-af-02-adversarial-test-strength/waiver-policy.json",
    ),
    (
        "waiver_policy_schema",
        "specs/016-af-02-adversarial-test-strength/schemas/af02-waiver-policy-v1.schema.json",
    ),
    (
        "required_check_policy",
        "specs/016-af-02-adversarial-test-strength/required-check-policy.json",
    ),
    (
        "required_check_policy_schema",
        "specs/016-af-02-adversarial-test-strength/schemas/af02-required-check-policy-v1.schema.json",
    ),
    (
        "required_check_provenance_schema",
        "specs/016-af-02-adversarial-test-strength/schemas/af02-required-check-provenance-v1.schema.json",
    ),
    (
        "semantic_contract",
        "specs/016-af-02-adversarial-test-strength/semantic-contract.json",
    ),
    (
        "semantic_contract_schema",
        "specs/016-af-02-adversarial-test-strength/schemas/af02-semantic-contract-v1.schema.json",
    ),
    (
        "verifier_input_policy",
        "specs/016-af-02-adversarial-test-strength/verifier-input-policy.json",
    ),
    (
        "verifier_input_policy_schema",
        "specs/016-af-02-adversarial-test-strength/schemas/af02-verifier-input-policy-v1.schema.json",
    ),
    (
        "surface_policy_schema",
        "specs/016-af-02-adversarial-test-strength/schemas/af02-surface-policy-v1.schema.json",
    ),
    (
        "resource_policy_schema",
        "specs/016-af-02-adversarial-test-strength/schemas/af02-resource-policy-v1.schema.json",
    ),
    (
        "corpus_manifest_schema",
        "specs/016-af-02-adversarial-test-strength/schemas/af02-corpus-v1.schema.json",
    ),
    (
        "coverage_policy_schema",
        "specs/016-af-02-adversarial-test-strength/schemas/af02-coverage-policy-v1.schema.json",
    ),
    (
        "mutation_policy_schema",
        "specs/016-af-02-adversarial-test-strength/schemas/af02-mutation-policy-v1.schema.json",
    ),
    (
        "enforcement_inventory_schema",
        "specs/016-af-02-adversarial-test-strength/schemas/af02-enforcement-inventory-v1.schema.json",
    ),
];

#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("semantic JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("semantic canonicalization error: {0}")]
    Canonical(#[from] CanonicalError),
    #[error("semantic contract violation: {0}")]
    Contract(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticCoverage {
    pub algorithm_count: usize,
    pub negative_fixture_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractDescriptor {
    pub role: String,
    pub path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyMode {
    Bootstrap,
    Rebase,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyPredecessorEvidence {
    pub mode: PolicyMode,
    pub policy_path: String,
    pub canonical_base_blob: Option<String>,
    pub canonical_base_sha256: Option<String>,
    pub declared_predecessor_blob: Option<String>,
    pub declared_predecessor_sha256: Option<String>,
    pub changed_paths: Vec<String>,
    pub dependent_evidence_paths: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SurfaceCatalogEntry {
    pub category: String,
    pub matcher_id: String,
    pub surface_id: String,
    pub witness_id: String,
    pub witness_blob_sha: String,
    pub live_blob_sha: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BijectionKey {
    pub scenario_id: String,
    pub assertion_id: String,
    pub fixture_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceBlobEntry {
    pub path: String,
    pub git_blob_sha: String,
    pub regular_file: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageEntry {
    pub path: String,
    pub surface_id: Option<String>,
    pub covered: u64,
    pub total: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MutationEntry {
    pub mutant_id: String,
    pub in_target: bool,
    pub precanonical_excluded: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractBytes<'a> {
    pub role: &'a str,
    pub path: &'a str,
    pub blob_sha: &'a str,
    pub sha256: &'a str,
    pub bytes: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathObservation {
    pub repo_relative_path: String,
    pub has_symlink_component: bool,
    pub owner_uid: u32,
    pub expected_owner_uid: u32,
    pub link_count: u64,
    pub contained: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CounterEquality {
    pub label: String,
    pub observed: i128,
    pub expected: i128,
    pub denominator: Option<i128>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DigestBinding {
    pub label: String,
    pub observed: String,
    pub expected: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryKeySet {
    pub label: String,
    pub keys: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateInputLimits {
    pub max_files: u64,
    pub max_aggregate_bytes: u64,
    pub max_depth: u64,
    pub max_records: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateInputStats {
    pub files: u64,
    pub aggregate_bytes: u64,
    pub depth: u64,
    pub records: u64,
    pub yaml_alias_present: bool,
    pub yaml_merge_key_present: bool,
    pub yaml_custom_tag_present: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessEvidence {
    pub binary_sha256: String,
    pub expected_binary_sha256: String,
    pub cargo_lock_blob: String,
    pub expected_cargo_lock_blob: String,
    pub unprivileged: bool,
    pub cgroup_v2: bool,
    pub wall_timeout_enforced: bool,
    pub memory_limit_enforced: bool,
    pub pid_limit_enforced: bool,
    pub network_none: bool,
    pub root_read_only: bool,
    pub stdout_observed: u64,
    pub stdout_limit: u64,
    pub stdout_exceeded: bool,
    pub stderr_observed: u64,
    pub stderr_limit: u64,
    pub stderr_exceeded: bool,
    pub termination: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnforcementRole {
    pub role: String,
    pub planned_path: String,
    pub entrypoint: String,
    pub required_from_rank: u8,
    pub resolved_on_base: bool,
}

pub fn algorithm_implementation(id: &str) -> Option<&'static str> {
    match id {
        "CORE_SCHEMA_VALIDATION" => Some("semantic::validate_core_schema"),
        "PROOF_ENVELOPE_CLOSURE" => Some("semantic::validate_proof_envelope_closure"),
        "SEMANTIC_CONTRACT_SELF_CHECK" => Some("semantic::validate_semantic_contract"),
        "POLICY_PREDECESSOR_COMPARISON" => Some("semantic::validate_policy_predecessor_comparison"),
        "CANONICAL_SORT_UNIQUE" => Some("semantic::validate_sorted_unique"),
        "SURFACE_CATEGORY_WITNESS_CLOSURE" => Some("semantic::validate_surface_category_witness_closure"),
        "ASSERTION_REPLAY_BIJECTION" => Some("semantic::validate_assertion_replay_bijection"),
        "SOURCE_BLOB_RECONSTRUCTION" => Some("semantic::validate_source_blob_reconstruction"),
        "COVERAGE_ACCOUNTING" => Some("semantic::validate_coverage_accounting"),
        "MUTATION_MEMBERSHIP" => Some("semantic::validate_mutation_membership"),
        "WAIVER_CANONICAL_ANCESTRY" => Some("waiver::verify_waiver_canonical_ancestry"),
        "REQUIRED_CHECK_GITHUB_PROVENANCE" => Some("commandf_af02_verifier::verify_required_checks"),
        "REQUIRED_CHECK_CROSS_BINDING" => Some("semantic::validate_required_check_cross_binding"),
        "RETAINED_LOCATOR_RECONSTRUCTION" => Some("retained::locator_plan"),
        "RETAINED_PR_RUN_BINDING" => Some("retained::verify_workflow_run + retained::verify_artifacts"),
        "CONTRACT_DIGEST_RECONSTRUCTION" => Some("semantic::validate_contract_digest_reconstruction"),
        "EXTENSION_AUTHORITY_DIGEST_BINDING" => Some("semantic::validate_extension_authority_digest_binding"),
        "PATH_NOFOLLOW_CONTAINMENT" => Some("semantic::validate_path_nofollow_containment"),
        "COUNTER_EQUALITIES" => Some("semantic::validate_counter_equalities"),
        "POLICY_INVENTORY_BINDING" => Some("semantic::validate_policy_inventory_binding"),
        "INVENTORY_KEY_MEMBERSHIP" => Some("semantic::validate_inventory_key_membership"),
        "CANDIDATE_INPUT_LIMITS" => Some("semantic::validate_candidate_input_limits"),
        "INPUT_PROCESS_ENFORCEMENT" => Some("semantic::validate_input_process_enforcement"),
        "ENFORCEMENT_INVENTORY_CLOSURE" => Some("semantic::validate_enforcement_inventory_closure"),
        "FINAL_ENVELOPE_HASH" => Some("semantic::validate_final_envelope_hash"),
        _ => None,
    }
}

pub fn negative_fixture_algorithm(id: &str) -> Option<&'static str> {
    match id {
        "CORE_SCHEMA_ALIAS_COLLISION" | "CORE_EMPTY_DETERMINISTIC" => {
            Some("CORE_SCHEMA_VALIDATION")
        }
        "EXTENSION_DUPLICATE_ROLE" | "EXTENSION_DUPLICATE_PATH" => {
            Some("PROOF_ENVELOPE_CLOSURE")
        }
        "EXTENSION_AUTHORITY_DIGEST_MISMATCH" => Some("EXTENSION_AUTHORITY_DIGEST_BINDING"),
        "POLICY_BOOTSTRAP_WHEN_PREDECESSOR_EXISTS"
        | "POLICY_REBASE_MISSING_PREDECESSOR"
        | "POLICY_REBASE_WRONG_PREDECESSOR_BLOB"
        | "POLICY_REBASE_WRONG_PREDECESSOR_DIGEST"
        | "POLICY_CHANGE_WITH_DEPENDENT_EVIDENCE" => Some("POLICY_PREDECESSOR_COMPARISON"),
        "EMPTY_SURFACE_MATCHERS"
        | "SURFACE_MISSING_CATEGORY"
        | "SURFACE_DUPLICATE_MATCHER_ID"
        | "SURFACE_DUPLICATE_SURFACE_ID"
        | "SURFACE_DUPLICATE_WITNESS_ID"
        | "SURFACE_STALE_WITNESS"
        | "SURFACE_UNMATCHED_KNOWN_BOUNDARY" => Some("SURFACE_CATEGORY_WITNESS_CLOSURE"),
        "EMPTY_TOOL_LOCK"
        | "OMITTED_REQUIRED_TOOL"
        | "UNEXPECTED_TOOL"
        | "SUBSTITUTED_TOOL_ID"
        | "WRONG_EXECUTABLE_DIGEST"
        | "POLICY_INVENTORY_DIGEST_MISMATCH" => Some("POLICY_INVENTORY_BINDING"),
        "FABRICATED_WAIVER_ID" | "WAIVER_WRONG_MUTANT" | "WAIVER_NOT_PRECANONICAL" => {
            Some("WAIVER_CANONICAL_ANCESTRY")
        }
        "CHECK_WRONG_APP"
        | "CHECK_WRONG_REPOSITORY"
        | "CHECK_WRONG_WORKFLOW_ID"
        | "CHECK_WRONG_WORKFLOW_BLOB"
        | "CHECK_WRONG_JOB"
        | "CHECK_WRONG_HEAD" => Some("REQUIRED_CHECK_GITHUB_PROVENANCE"),
        "CHECK_DUPLICATE_CHECK_RUN_ID"
        | "CHECK_DUPLICATE_WORKFLOW_RUN_ID"
        | "CHECK_DUPLICATE_JOB_ID" => Some("REQUIRED_CHECK_CROSS_BINDING"),
        "RETAINED_WRONG_EVENT"
        | "RETAINED_WRONG_WORKFLOW_ID"
        | "RETAINED_WRONG_RUN_ATTEMPT"
        | "RETAINED_WRONG_CHECK_SUITE"
        | "RETAINED_WRONG_PR_ASSOCIATION"
        | "RETAINED_WRONG_RUN_HEAD"
        | "RETAINED_WRONG_ARTIFACT" => Some("RETAINED_PR_RUN_BINDING"),
        "UNSORTED_SOURCE_UNIVERSE" | "DUPLICATE_SOURCE_PATH" => {
            Some("SOURCE_BLOB_RECONSTRUCTION")
        }
        "ASSERTION_REPLAY_MISSING"
        | "ASSERTION_REPLAY_DUPLICATE"
        | "CORPUS_DUPLICATE_FIXTURE_PATH" => Some("ASSERTION_REPLAY_BIJECTION"),
        "COVERAGE_MISSING_PATH" | "COVERAGE_DUPLICATE_PATH" | "COVERAGE_SURFACE_OVERLAP" => {
            Some("COVERAGE_ACCOUNTING")
        }
        "MUTATION_RESULT_MISSING" | "MUTATION_EXTRA_RESULT" => Some("MUTATION_MEMBERSHIP"),
        "INPUT_OVERSIZE"
        | "INPUT_TOO_DEEP"
        | "INPUT_TOO_MANY_RECORDS"
        | "YAML_ALIAS"
        | "YAML_CUSTOM_TAG" => Some("CANDIDATE_INPUT_LIMITS"),
        "INPUT_PARSER_WRONG_BINARY"
        | "INPUT_PARSER_WRONG_LOCK_BLOB"
        | "INPUT_PARSER_NO_CGROUP"
        | "INPUT_PARSER_WALL_TIMEOUT_NOT_ENFORCED"
        | "INPUT_PARSER_MEMORY_LIMIT_NOT_ENFORCED"
        | "INPUT_PARSER_OUTPUT_EVIDENCE_MISSING"
        | "INPUT_PARSER_STDOUT_LIMIT_EXCEEDED"
        | "INPUT_PARSER_STDERR_LIMIT_EXCEEDED"
        | "INPUT_PARSER_OUTPUT_FLAG_MISMATCH"
        | "INPUT_PARSER_UNCLASSIFIED_TERMINATION" => Some("INPUT_PROCESS_ENFORCEMENT"),
        "PATH_SYMLINK_ESCAPE" => Some("PATH_NOFOLLOW_CONTAINMENT"),
        "ENFORCEMENT_MISSING_ROLE"
        | "ENFORCEMENT_DUPLICATE_ROLE"
        | "ENFORCEMENT_UNCLASSIFIED_ROLE"
        | "ENFORCEMENT_ACTIVE_PATH_MISSING" => Some("ENFORCEMENT_INVENTORY_CLOSURE"),
        _ => None,
    }
}

pub fn validate_semantic_contract(
    contract_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<SemanticCoverage, SemanticError> {
    let contract = parse_json_no_duplicates(contract_bytes)?;
    let schema = parse_json_no_duplicates(schema_bytes)?;
    if schema.get("$id").and_then(Value::as_str) != Some(SEMANTIC_SCHEMA_URL) {
        return contract_error("unexpected semantic-contract schema id");
    }
    let object = contract
        .as_object()
        .ok_or_else(|| SemanticError::Contract("semantic contract must be an object".to_owned()))?;
    let observed_keys: BTreeSet<&str> = object.keys().map(String::as_str).collect();
    let expected_keys = BTreeSet::from([
        "algorithms",
        "negative_fixture_ids",
        "schema",
        "verifier",
    ]);
    if observed_keys != expected_keys {
        return contract_error("semantic contract has missing or unknown top-level fields");
    }
    if contract.get("schema").and_then(Value::as_str) != Some(SEMANTIC_SCHEMA_ID) {
        return contract_error("unexpected semantic contract schema value");
    }
    for field in ["verifier", "algorithms", "negative_fixture_ids"] {
        let pointer = format!("/properties/{field}/const");
        let expected = schema.pointer(&pointer).ok_or_else(|| {
            SemanticError::Contract(format!("semantic schema has no {field} const"))
        })?;
        if contract.get(field) != Some(expected) {
            return contract_error(format!(
                "semantic contract {field} differs from the schema-frozen value"
            ));
        }
    }

    let algorithms = contract["algorithms"]
        .as_array()
        .ok_or_else(|| SemanticError::Contract("algorithms must be an array".to_owned()))?;
    if algorithms.len() != ALGORITHM_IDS.len() {
        return contract_error("semantic algorithm count mismatch");
    }
    for (entry, expected_id) in algorithms.iter().zip(ALGORITHM_IDS) {
        let observed_id = entry.get("id").and_then(Value::as_str).ok_or_else(|| {
            SemanticError::Contract("semantic algorithm entry has no id".to_owned())
        })?;
        if observed_id != expected_id || algorithm_implementation(observed_id).is_none() {
            return contract_error(format!("semantic algorithm {observed_id} is not mapped exactly"));
        }
    }

    let fixtures = contract["negative_fixture_ids"]
        .as_array()
        .ok_or_else(|| SemanticError::Contract("negative_fixture_ids must be an array".to_owned()))?;
    if fixtures.len() != NEGATIVE_FIXTURE_IDS.len() {
        return contract_error("negative fixture count mismatch");
    }
    for (entry, expected_id) in fixtures.iter().zip(NEGATIVE_FIXTURE_IDS) {
        let observed_id = entry.as_str().ok_or_else(|| {
            SemanticError::Contract("negative fixture id is not a string".to_owned())
        })?;
        let algorithm = negative_fixture_algorithm(observed_id).ok_or_else(|| {
            SemanticError::Contract(format!("negative fixture {observed_id} has no verifier mapping"))
        })?;
        if observed_id != expected_id || algorithm_implementation(algorithm).is_none() {
            return contract_error(format!("negative fixture {observed_id} is not mapped exactly"));
        }
    }

    Ok(SemanticCoverage {
        algorithm_count: algorithms.len(),
        negative_fixture_count: fixtures.len(),
    })
}

pub fn validate_core_schema(core_bytes: &[u8], schema_bytes: &[u8]) -> Result<(), SemanticError> {
    let core = parse_json_no_duplicates(core_bytes)?;
    let schema = parse_json_no_duplicates(schema_bytes)?;
    if schema.get("$id").and_then(Value::as_str) != Some(CORE_SCHEMA_URL) {
        return contract_error("unexpected proof-core schema id");
    }
    validate_schema_node(&core, &schema, &schema, "$")?;
    let deterministic = core
        .get("deterministic")
        .and_then(Value::as_object)
        .ok_or_else(|| SemanticError::Contract("core deterministic object is missing".to_owned()))?;
    if deterministic.is_empty() {
        return contract_error("core deterministic object must not be empty");
    }
    Ok(())
}

pub fn validate_proof_envelope_closure(
    core_contract_paths: &[String],
    extension_files: &[ContractDescriptor],
) -> Result<(), SemanticError> {
    if extension_files.len() != EXTENSION_ROLES_AND_PATHS.len() {
        return contract_error("extension contract files must contain exactly 17 entries");
    }
    let mut roles = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for (observed, expected) in extension_files.iter().zip(EXTENSION_ROLES_AND_PATHS) {
        if observed.role != expected.0 || observed.path != expected.1 {
            return contract_error(format!(
                "extension role/path order mismatch at {}",
                observed.role
            ));
        }
        if !roles.insert(observed.role.as_str()) {
            return contract_error(format!("duplicate extension role {}", observed.role));
        }
        if !paths.insert(observed.path.as_str()) {
            return contract_error(format!("duplicate extension path {}", observed.path));
        }
    }
    let core_paths: BTreeSet<&str> = core_contract_paths.iter().map(String::as_str).collect();
    if core_paths.len() != core_contract_paths.len() {
        return contract_error("duplicate core contract path");
    }
    if let Some(path) = paths.intersection(&core_paths).next() {
        return contract_error(format!("extension path duplicates core contract path {path}"));
    }
    Ok(())
}

pub fn validate_policy_predecessor_comparison(
    evidence: &PolicyPredecessorEvidence,
) -> Result<(), SemanticError> {
    match evidence.mode {
        PolicyMode::Bootstrap => {
            if evidence.canonical_base_blob.is_some()
                || evidence.canonical_base_sha256.is_some()
                || evidence.declared_predecessor_blob.is_some()
                || evidence.declared_predecessor_sha256.is_some()
            {
                return contract_error("BOOTSTRAP requires the policy to be absent at canonical base");
            }
        }
        PolicyMode::Rebase => {
            let base_blob = evidence
                .canonical_base_blob
                .as_deref()
                .ok_or_else(|| SemanticError::Contract("REBASE missing canonical-base blob".to_owned()))?;
            let base_digest = evidence
                .canonical_base_sha256
                .as_deref()
                .ok_or_else(|| SemanticError::Contract("REBASE missing canonical-base digest".to_owned()))?;
            if evidence.declared_predecessor_blob.as_deref() != Some(base_blob) {
                return contract_error("REBASE predecessor blob differs from canonical base");
            }
            if evidence.declared_predecessor_sha256.as_deref() != Some(base_digest) {
                return contract_error("REBASE predecessor digest differs from canonical base");
            }
        }
    }
    if evidence.changed_paths != [evidence.policy_path.clone()] {
        return contract_error("policy change must be policy-only");
    }
    if !evidence.dependent_evidence_paths.is_empty() {
        return contract_error("policy change cannot carry dependent evidence in the same candidate");
    }
    Ok(())
}

pub fn validate_sorted_unique(values: &[String], label: &str) -> Result<(), SemanticError> {
    let mut previous: Option<&str> = None;
    for value in values {
        if previous.is_some_and(|item| item.as_bytes() >= value.as_bytes()) {
            return contract_error(format!("{label} must be strictly UTF-8-byte sorted and unique"));
        }
        previous = Some(value);
    }
    Ok(())
}

pub fn validate_surface_category_witness_closure(
    entries: &[SurfaceCatalogEntry],
) -> Result<(), SemanticError> {
    if entries.is_empty() {
        return contract_error("surface matcher catalog must not be empty");
    }
    let mut categories = BTreeSet::new();
    let mut matchers = BTreeSet::new();
    let mut surfaces = BTreeSet::new();
    let mut witnesses = BTreeSet::new();
    for entry in entries {
        if !REQUIRED_SURFACE_CATEGORIES.contains(&entry.category.as_str()) {
            return contract_error(format!("unknown surface category {}", entry.category));
        }
        categories.insert(entry.category.as_str());
        if !matchers.insert(entry.matcher_id.as_str()) {
            return contract_error(format!("duplicate matcher id {}", entry.matcher_id));
        }
        if !surfaces.insert(entry.surface_id.as_str()) {
            return contract_error(format!("duplicate surface id {}", entry.surface_id));
        }
        if !witnesses.insert(entry.witness_id.as_str()) {
            return contract_error(format!("duplicate witness id {}", entry.witness_id));
        }
        if entry.witness_blob_sha != entry.live_blob_sha {
            return contract_error(format!("stale witness {}", entry.witness_id));
        }
    }
    let required: BTreeSet<&str> = REQUIRED_SURFACE_CATEGORIES.into_iter().collect();
    if categories != required {
        return contract_error("surface catalog does not cover all six boundary categories");
    }
    Ok(())
}

pub fn validate_assertion_replay_bijection(
    manifest: &[BijectionKey],
    assertions: &[BijectionKey],
    replay: &[BijectionKey],
) -> Result<(), SemanticError> {
    let manifest_set = unique_bijection_set(manifest, "manifest")?;
    let assertion_set = unique_bijection_set(assertions, "assertion registry")?;
    let replay_set = unique_bijection_set(replay, "replay results")?;
    if manifest_set != assertion_set || manifest_set != replay_set {
        return contract_error("scenario/assertion/fixture membership is not bijective");
    }
    Ok(())
}

pub fn validate_source_blob_reconstruction(
    tracked_paths: &[String],
    entries: &[SourceBlobEntry],
) -> Result<(), SemanticError> {
    validate_sorted_unique(tracked_paths, "source universe")?;
    let expected: BTreeSet<&str> = tracked_paths.iter().map(String::as_str).collect();
    let mut observed = BTreeSet::new();
    for entry in entries {
        if !entry.regular_file {
            return contract_error(format!("source path {} is not a regular file", entry.path));
        }
        validate_lower_hex(&entry.git_blob_sha, 40, "source git blob")?;
        if !observed.insert(entry.path.as_str()) {
            return contract_error(format!("duplicate source path {}", entry.path));
        }
    }
    if observed != expected {
        return contract_error("source universe does not equal reconstructed Git source membership");
    }
    Ok(())
}

pub fn validate_coverage_accounting(
    expected_paths: &[String],
    expected_surface_ids: &[String],
    entries: &[CoverageEntry],
) -> Result<(), SemanticError> {
    let expected_paths_set: BTreeSet<&str> = expected_paths.iter().map(String::as_str).collect();
    if expected_paths_set.len() != expected_paths.len() {
        return contract_error("coverage expected path universe is not unique");
    }
    let expected_surfaces: BTreeSet<&str> = expected_surface_ids.iter().map(String::as_str).collect();
    if expected_surfaces.len() != expected_surface_ids.len() {
        return contract_error("coverage expected surface universe is not unique");
    }
    let mut observed_paths = BTreeSet::new();
    let mut observed_surfaces = BTreeSet::new();
    for entry in entries {
        if !observed_paths.insert(entry.path.as_str()) {
            return contract_error(format!("duplicate coverage path {}", entry.path));
        }
        if entry.covered > entry.total {
            return contract_error(format!("coverage arithmetic invalid for {}", entry.path));
        }
        if let Some(surface_id) = entry.surface_id.as_deref() {
            if !observed_surfaces.insert(surface_id) {
                return contract_error(format!("coverage surface overlap {surface_id}"));
            }
        }
    }
    if observed_paths != expected_paths_set {
        return contract_error("coverage path membership differs from source universe");
    }
    if observed_surfaces != expected_surfaces {
        return contract_error("coverage critical-surface membership differs from surface universe");
    }
    Ok(())
}

pub fn validate_mutation_membership(
    inventory: &[MutationEntry],
    result_ids: &[String],
) -> Result<(), SemanticError> {
    let mut mutant_ids = BTreeSet::new();
    let mut required = BTreeSet::new();
    for entry in inventory {
        if !mutant_ids.insert(entry.mutant_id.as_str()) {
            return contract_error(format!("duplicate mutant id {}", entry.mutant_id));
        }
        if entry.in_target && !entry.precanonical_excluded {
            required.insert(entry.mutant_id.as_str());
        }
    }
    let results: BTreeSet<&str> = result_ids.iter().map(String::as_str).collect();
    if results.len() != result_ids.len() {
        return contract_error("duplicate mutation result id");
    }
    if results != required {
        return contract_error("mutation result membership differs from required mutant membership");
    }
    Ok(())
}

pub fn validate_required_check_cross_binding(provenance: &Value) -> Result<(), SemanticError> {
    let head = required_string(provenance, "head_sha")?;
    let base = required_string(provenance, "base_sha")?;
    validate_lower_hex(head, 40, "required-check head_sha")?;
    validate_lower_hex(base, 40, "required-check base_sha")?;
    let checks = provenance
        .get("checks")
        .and_then(Value::as_object)
        .ok_or_else(|| SemanticError::Contract("required-check checks object is missing".to_owned()))?;
    let expected_contexts = ["assurance-proof", "rust", "scorecard"];
    if checks.len() != expected_contexts.len() {
        return contract_error("required-check context membership mismatch");
    }
    let mut domain_ids = [BTreeSet::new(), BTreeSet::new(), BTreeSet::new(), BTreeSet::new()];
    for context in expected_contexts {
        let entry = checks.get(context).ok_or_else(|| {
            SemanticError::Contract(format!("required-check context {context} is missing"))
        })?;
        for (index, field) in [
            "check_run_ref",
            "check_suite_ref",
            "workflow_run_ref",
            "job_ref",
        ]
        .iter()
        .enumerate()
        {
            let value = required_string(entry, field)?;
            let id = parse_context_ref(context, value, field)?;
            if !domain_ids[index].insert(id) {
                return contract_error(format!("duplicate {field} numeric id {id}"));
            }
        }
    }
    Ok(())
}

pub fn validate_contract_digest_reconstruction(
    expected_roles: &[String],
    files: &[ContractBytes<'_>],
) -> Result<(), SemanticError> {
    let expected: BTreeSet<&str> = expected_roles.iter().map(String::as_str).collect();
    if expected.len() != expected_roles.len() {
        return contract_error("expected contract roles contain duplicates");
    }
    let mut roles = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for file in files {
        if !roles.insert(file.role) {
            return contract_error(format!("duplicate contract role {}", file.role));
        }
        if !paths.insert(file.path) {
            return contract_error(format!("duplicate contract path {}", file.path));
        }
        if git_blob_sha1_hex(file.bytes) != file.blob_sha {
            return contract_error(format!("contract {} Git blob mismatch", file.role));
        }
        if sha256_hex(file.bytes) != file.sha256 {
            return contract_error(format!("contract {} SHA-256 mismatch", file.role));
        }
    }
    if roles != expected {
        return contract_error("contract role membership mismatch");
    }
    Ok(())
}

pub fn validate_extension_authority_digest_binding(
    authority: &BTreeMap<String, String>,
    contract_role_sha256: &BTreeMap<String, String>,
) -> Result<(), SemanticError> {
    let mappings = [
        ("retained_authority_sources_schema_sha256", "retained_authority_sources_schema"),
        ("waiver_policy_sha256", "waiver_policy"),
        ("waiver_policy_schema_sha256", "waiver_policy_schema"),
        ("required_check_policy_sha256", "required_check_policy"),
        ("required_check_policy_schema_sha256", "required_check_policy_schema"),
        ("required_check_provenance_schema_sha256", "required_check_provenance_schema"),
        ("semantic_contract_sha256", "semantic_contract"),
        ("semantic_contract_schema_sha256", "semantic_contract_schema"),
        ("verifier_input_policy_sha256", "verifier_input_policy"),
        ("verifier_input_policy_schema_sha256", "verifier_input_policy_schema"),
        ("surface_policy_schema_sha256", "surface_policy_schema"),
        ("resource_policy_schema_sha256", "resource_policy_schema"),
        ("corpus_schema_sha256", "corpus_manifest_schema"),
        ("coverage_policy_schema_sha256", "coverage_policy_schema"),
        ("mutation_policy_schema_sha256", "mutation_policy_schema"),
        ("enforcement_inventory_sha256", "enforcement_inventory"),
        ("enforcement_inventory_schema_sha256", "enforcement_inventory_schema"),
    ];
    if authority.len() != mappings.len() {
        return contract_error("extension authority field membership mismatch");
    }
    for (field, role) in mappings {
        let observed = authority.get(field).ok_or_else(|| {
            SemanticError::Contract(format!("extension authority field {field} is missing"))
        })?;
        let expected = contract_role_sha256.get(role).ok_or_else(|| {
            SemanticError::Contract(format!("contract digest role {role} is missing"))
        })?;
        if observed != expected {
            return contract_error(format!("extension authority field {field} does not bind role {role}"));
        }
    }
    Ok(())
}

pub fn validate_path_nofollow_containment(observation: &PathObservation) -> Result<(), SemanticError> {
    validate_repo_path(&observation.repo_relative_path)?;
    if observation.has_symlink_component {
        return contract_error("path contains a symlink component");
    }
    if observation.owner_uid != observation.expected_owner_uid {
        return contract_error("path owner does not match expected unprivileged owner");
    }
    if observation.link_count != 1 {
        return contract_error("path link count must equal one");
    }
    if !observation.contained {
        return contract_error("path escapes the required containment root");
    }
    Ok(())
}

pub fn validate_counter_equalities(counters: &[CounterEquality]) -> Result<(), SemanticError> {
    let mut labels = BTreeSet::new();
    for counter in counters {
        if !labels.insert(counter.label.as_str()) {
            return contract_error(format!("duplicate counter label {}", counter.label));
        }
        if counter.observed < 0 || counter.expected < 0 {
            return contract_error(format!("counter {} is negative", counter.label));
        }
        if counter.denominator == Some(0) {
            return contract_error(format!("counter {} has a prohibited zero denominator", counter.label));
        }
        if counter.observed != counter.expected {
            return contract_error(format!("counter {} equality mismatch", counter.label));
        }
    }
    Ok(())
}

pub fn validate_policy_inventory_binding(bindings: &[DigestBinding]) -> Result<(), SemanticError> {
    let mut labels = BTreeSet::new();
    for binding in bindings {
        if !labels.insert(binding.label.as_str()) {
            return contract_error(format!("duplicate policy/inventory binding {}", binding.label));
        }
        validate_lower_hex(&binding.observed, 64, &binding.label)?;
        validate_lower_hex(&binding.expected, 64, &binding.label)?;
        if binding.observed != binding.expected {
            return contract_error(format!("policy/inventory binding {} mismatch", binding.label));
        }
    }
    Ok(())
}

pub fn validate_inventory_key_membership(inventories: &[InventoryKeySet]) -> Result<(), SemanticError> {
    let mut labels = BTreeSet::new();
    for inventory in inventories {
        if !labels.insert(inventory.label.as_str()) {
            return contract_error(format!("duplicate inventory label {}", inventory.label));
        }
        validate_sorted_unique(&inventory.keys, &inventory.label)?;
    }
    Ok(())
}

pub fn validate_candidate_input_limits(
    limits: &CandidateInputLimits,
    stats: &CandidateInputStats,
) -> Result<(), SemanticError> {
    if stats.files > limits.max_files {
        return contract_error("candidate input file count exceeds policy");
    }
    if stats.aggregate_bytes > limits.max_aggregate_bytes {
        return contract_error("candidate input aggregate bytes exceed policy");
    }
    if stats.depth > limits.max_depth {
        return contract_error("candidate input nesting depth exceeds policy");
    }
    if stats.records > limits.max_records {
        return contract_error("candidate input record count exceeds policy");
    }
    if stats.yaml_alias_present || stats.yaml_merge_key_present || stats.yaml_custom_tag_present {
        return contract_error("candidate YAML contains prohibited alias, merge key, or custom tag");
    }
    Ok(())
}

pub fn validate_input_process_enforcement(evidence: &ProcessEvidence) -> Result<(), SemanticError> {
    if evidence.binary_sha256 != evidence.expected_binary_sha256 {
        return contract_error("verifier subprocess binary digest mismatch");
    }
    if evidence.cargo_lock_blob != evidence.expected_cargo_lock_blob {
        return contract_error("verifier subprocess Cargo.lock blob mismatch");
    }
    if !evidence.unprivileged
        || !evidence.cgroup_v2
        || !evidence.wall_timeout_enforced
        || !evidence.memory_limit_enforced
        || !evidence.pid_limit_enforced
        || !evidence.network_none
        || !evidence.root_read_only
    {
        return contract_error("verifier subprocess enforcement envelope is incomplete");
    }
    validate_stream_limit(
        "stdout",
        evidence.stdout_observed,
        evidence.stdout_limit,
        evidence.stdout_exceeded,
    )?;
    validate_stream_limit(
        "stderr",
        evidence.stderr_observed,
        evidence.stderr_limit,
        evidence.stderr_exceeded,
    )?;
    if !ALLOWED_TERMINATIONS.contains(&evidence.termination.as_str()) {
        return contract_error(format!("unclassified verifier termination {}", evidence.termination));
    }
    Ok(())
}

pub fn validate_enforcement_inventory_closure(
    expected_roles: &[String],
    inventory: &[EnforcementRole],
    current_stack_rank: u8,
) -> Result<(), SemanticError> {
    let expected: BTreeSet<&str> = expected_roles.iter().map(String::as_str).collect();
    if expected.len() != expected_roles.len() {
        return contract_error("schema-frozen enforcement roles contain duplicates");
    }
    let mut observed = BTreeSet::new();
    for entry in inventory {
        if !expected.contains(entry.role.as_str()) {
            return contract_error(format!("unclassified enforcement role {}", entry.role));
        }
        if !observed.insert(entry.role.as_str()) {
            return contract_error(format!("duplicate enforcement role {}", entry.role));
        }
        if entry.required_from_rank <= current_stack_rank
            && (!entry.resolved_on_base || entry.planned_path.is_empty() || entry.entrypoint.is_empty())
        {
            return contract_error(format!("active enforcement role {} is unresolved", entry.role));
        }
    }
    if observed != expected {
        return contract_error("enforcement inventory is missing a schema-frozen role");
    }
    Ok(())
}

pub fn validate_final_envelope_hash(
    core_deterministic: &Value,
    extension_contract_files: &Value,
    extension_authority: &Value,
    required_check_provenance: &Value,
    expected_sha256: &str,
) -> Result<(), SemanticError> {
    validate_lower_hex(expected_sha256, 64, "af02_adversarial_sha256")?;
    let envelope = serde_json::json!({
        "core_deterministic": core_deterministic,
        "extension_contract_files": extension_contract_files,
        "extension_authority": extension_authority,
        "required_check_provenance": required_check_provenance,
    });
    let observed = canonical_sha256(&envelope)?;
    if observed != expected_sha256 {
        return contract_error("final deterministic envelope hash mismatch");
    }
    Ok(())
}

fn unique_bijection_set(
    entries: &[BijectionKey],
    label: &str,
) -> Result<BTreeSet<BijectionKey>, SemanticError> {
    let mut scenarios = BTreeSet::new();
    let mut assertions = BTreeSet::new();
    let mut fixtures = BTreeSet::new();
    let mut complete = BTreeSet::new();
    for entry in entries {
        if !scenarios.insert(entry.scenario_id.as_str()) {
            return contract_error(format!("duplicate {label} scenario_id {}", entry.scenario_id));
        }
        if !assertions.insert(entry.assertion_id.as_str()) {
            return contract_error(format!("duplicate {label} assertion_id {}", entry.assertion_id));
        }
        if !fixtures.insert(entry.fixture_path.as_str()) {
            return contract_error(format!("duplicate {label} fixture path {}", entry.fixture_path));
        }
        complete.insert(entry.clone());
    }
    Ok(complete)
}

fn validate_stream_limit(
    label: &str,
    observed: u64,
    limit: u64,
    exceeded: bool,
) -> Result<(), SemanticError> {
    let expected_flag = observed > limit;
    if exceeded != expected_flag {
        return contract_error(format!("{label} exceeded flag does not match observed byte count"));
    }
    if exceeded {
        return contract_error(format!("{label} byte limit exceeded"));
    }
    Ok(())
}

fn parse_context_ref(context: &str, value: &str, field: &str) -> Result<u64, SemanticError> {
    let prefix = format!("{context}:");
    let numeric = value.strip_prefix(&prefix).ok_or_else(|| {
        SemanticError::Contract(format!("{context} {field} lacks the context prefix"))
    })?;
    if numeric.is_empty() || numeric.starts_with('0') || !numeric.bytes().all(|byte| byte.is_ascii_digit()) {
        return contract_error(format!("{context} {field} has invalid numeric identity"));
    }
    numeric
        .parse::<u64>()
        .map_err(|_| SemanticError::Contract(format!("{context} {field} overflows u64")))
}

fn required_string<'a>(value: &'a Value, field: &str) -> Result<&'a str, SemanticError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| SemanticError::Contract(format!("{field} is missing or not a string")))
}

fn validate_lower_hex(value: &str, len: usize, label: &str) -> Result<(), SemanticError> {
    if value.len() != len
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return contract_error(format!("{label} must be {len} lowercase hexadecimal characters"));
    }
    Ok(())
}

fn validate_repo_path(value: &str) -> Result<(), SemanticError> {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value.contains("//")
        || value.contains('\0')
        || value.split('/').any(|part| matches!(part, "." | ".."))
    {
        return contract_error(format!("invalid portable repository path {value}"));
    }
    Ok(())
}

fn validate_schema_node(
    instance: &Value,
    schema: &Value,
    root: &Value,
    path: &str,
) -> Result<(), SemanticError> {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        let pointer = reference.strip_prefix('#').ok_or_else(|| {
            SemanticError::Contract(format!("{path}: external schema ref is unsupported in proof core"))
        })?;
        let target = root.pointer(pointer).ok_or_else(|| {
            SemanticError::Contract(format!("{path}: unresolved local schema ref {reference}"))
        })?;
        return validate_schema_node(instance, target, root, path);
    }
    if let Some(expected) = schema.get("const") {
        if instance != expected {
            return contract_error(format!("{path}: value differs from schema const"));
        }
    }
    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        if !values.contains(instance) {
            return contract_error(format!("{path}: value is outside schema enum"));
        }
    }
    if let Some(all_of) = schema.get("allOf").and_then(Value::as_array) {
        for child in all_of {
            validate_schema_node(instance, child, root, path)?;
        }
    }
    if let Some(one_of) = schema.get("oneOf").and_then(Value::as_array) {
        let matches = one_of
            .iter()
            .filter(|child| validate_schema_node(instance, child, root, path).is_ok())
            .count();
        if matches != 1 {
            return contract_error(format!("{path}: oneOf matched {matches} branches"));
        }
    }
    if let Some(not_schema) = schema.get("not") {
        if validate_schema_node(instance, not_schema, root, path).is_ok() {
            return contract_error(format!("{path}: value matches prohibited schema"));
        }
    }
    if let Some(condition) = schema.get("if") {
        let condition_matches = validate_schema_node(instance, condition, root, path).is_ok();
        let branch = if condition_matches { "then" } else { "else" };
        if let Some(branch_schema) = schema.get(branch) {
            validate_schema_node(instance, branch_schema, root, path)?;
        }
    }
    if let Some(kind) = schema.get("type").and_then(Value::as_str) {
        let matches = match kind {
            "object" => instance.is_object(),
            "array" => instance.is_array(),
            "string" => instance.is_string(),
            "integer" => instance.as_i64().is_some() || instance.as_u64().is_some(),
            "boolean" => instance.is_boolean(),
            "null" => instance.is_null(),
            other => return contract_error(format!("{path}: unsupported schema type {other}")),
        };
        if !matches {
            return contract_error(format!("{path}: expected schema type {kind}"));
        }
    }
    if let Some(object) = instance.as_object() {
        let properties = schema.get("properties").and_then(Value::as_object);
        if schema.get("additionalProperties") == Some(&Value::Bool(false)) {
            let allowed: BTreeSet<&str> = properties
                .into_iter()
                .flat_map(|value| value.keys().map(String::as_str))
                .collect();
            for key in object.keys() {
                if !allowed.contains(key.as_str()) {
                    return contract_error(format!("{path}.{key}: unknown field"));
                }
            }
        }
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for key in required {
                let key = key.as_str().ok_or_else(|| {
                    SemanticError::Contract(format!("{path}: schema required key is not a string"))
                })?;
                if !object.contains_key(key) {
                    return contract_error(format!("{path}.{key}: required field is missing"));
                }
            }
        }
        if let Some(properties) = properties {
            for (key, child) in object {
                if let Some(child_schema) = properties.get(key) {
                    validate_schema_node(child, child_schema, root, &format!("{path}.{key}"))?;
                }
            }
        }
    }
    if let Some(array) = instance.as_array() {
        if let Some(min_items) = schema.get("minItems").and_then(Value::as_u64) {
            if array.len() < min_items as usize {
                return contract_error(format!("{path}: array is shorter than minItems"));
            }
        }
        if let Some(max_items) = schema.get("maxItems").and_then(Value::as_u64) {
            if array.len() > max_items as usize {
                return contract_error(format!("{path}: array is longer than maxItems"));
            }
        }
        if schema.get("uniqueItems") == Some(&Value::Bool(true)) {
            for index in 0..array.len() {
                if array[..index].contains(&array[index]) {
                    return contract_error(format!("{path}: array contains duplicate items"));
                }
            }
        }
        if let Some(prefix) = schema.get("prefixItems").and_then(Value::as_array) {
            for (index, child_schema) in prefix.iter().enumerate() {
                if let Some(child) = array.get(index) {
                    validate_schema_node(child, child_schema, root, &format!("{path}[{index}]"))?;
                }
            }
            if schema.get("items") == Some(&Value::Bool(false)) && array.len() > prefix.len() {
                return contract_error(format!("{path}: array contains items beyond prefixItems"));
            }
        } else if let Some(items) = schema.get("items") {
            if items != &Value::Bool(false) {
                for (index, child) in array.iter().enumerate() {
                    validate_schema_node(child, items, root, &format!("{path}[{index}]"))?;
                }
            } else if !array.is_empty() {
                return contract_error(format!("{path}: schema prohibits array items"));
            }
        }
    }
    if let Some(text) = instance.as_str() {
        let chars = text.chars().count() as u64;
        if let Some(minimum) = schema.get("minLength").and_then(Value::as_u64) {
            if chars < minimum {
                return contract_error(format!("{path}: string is shorter than minLength"));
            }
        }
        if let Some(maximum) = schema.get("maxLength").and_then(Value::as_u64) {
            if chars > maximum {
                return contract_error(format!("{path}: string is longer than maxLength"));
            }
        }
        if let Some(pattern) = schema.get("pattern").and_then(Value::as_str) {
            if !known_pattern_matches(pattern, text)? {
                return contract_error(format!("{path}: string does not match schema pattern"));
            }
        }
    }
    if instance.as_i64().is_some() || instance.as_u64().is_some() {
        if let Some(minimum) = schema.get("minimum").and_then(Value::as_i64) {
            if instance.as_i64().is_some_and(|value| value < minimum) {
                return contract_error(format!("{path}: integer is below minimum"));
            }
        }
        if let Some(maximum) = schema.get("maximum").and_then(Value::as_u64) {
            if instance.as_u64().is_some_and(|value| value > maximum) {
                return contract_error(format!("{path}: integer exceeds maximum"));
            }
        }
    }
    Ok(())
}

fn known_pattern_matches(pattern: &str, value: &str) -> Result<bool, SemanticError> {
    match pattern {
        "^[0-9a-f]{40}$" => Ok(value.len() == 40 && value.bytes().all(is_lower_hex)),
        "^[0-9a-f]{64}$" => Ok(value.len() == 64 && value.bytes().all(is_lower_hex)),
        "^/af02-output(?:/[^/]+)*$" => Ok(value == "/af02-output"
            || value
                .strip_prefix("/af02-output/")
                .is_some_and(|tail| !tail.is_empty() && !tail.split('/').any(str::is_empty))),
        "^AF02-W[0-9]{4}$" => Ok(value.len() == 10
            && value.starts_with("AF02-W")
            && value.as_bytes()[6..].iter().all(u8::is_ascii_digit)),
        "^MUT-X[0-9]{4}$" => Ok(value.len() == 9
            && value.starts_with("MUT-X")
            && value.as_bytes()[5..].iter().all(u8::is_ascii_digit)),
        "^(?!/)(?!.*\\\\)(?!.*//)(?!.*(?:^|/)\\.(?:/|$))(?!.*(?:^|/)\\.\\.(?:/|$))[^\\u0000]+$" => {
            Ok(validate_repo_path(value).is_ok())
        }
        other => contract_error(format!("unsupported trusted schema pattern {other}")),
    }
}

fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

fn contract_error<T>(message: impl Into<String>) -> Result<T, SemanticError> {
    Err(SemanticError::Contract(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTRACT_BYTES: &[u8] = include_bytes!(
        "../../../specs/016-af-02-adversarial-test-strength/semantic-contract.json"
    );
    const CONTRACT_SCHEMA_BYTES: &[u8] = include_bytes!(
        "../../../specs/016-af-02-adversarial-test-strength/schemas/af02-semantic-contract-v1.schema.json"
    );

    #[test]
    fn semantic_contract_maps_every_algorithm_and_negative_fixture() {
        let coverage = validate_semantic_contract(CONTRACT_BYTES, CONTRACT_SCHEMA_BYTES).unwrap();
        assert_eq!(coverage.algorithm_count, 25);
        assert_eq!(coverage.negative_fixture_count, 72);
        assert!(ALGORITHM_IDS
            .iter()
            .all(|id| algorithm_implementation(id).is_some()));
        assert!(NEGATIVE_FIXTURE_IDS
            .iter()
            .all(|id| negative_fixture_algorithm(id).is_some()));
    }

    #[test]
    fn semantic_contract_rejects_schema_frozen_algorithm_drift() {
        let mut value = parse_json_no_duplicates(CONTRACT_BYTES).unwrap();
        value["algorithms"][0]["id"] = Value::String("CORE_SCHEMA_ALIAS".to_owned());
        let bytes = serde_json::to_vec(&value).unwrap();
        let error = validate_semantic_contract(&bytes, CONTRACT_SCHEMA_BYTES).unwrap_err();
        assert!(error.to_string().contains("schema-frozen"));
    }

    #[test]
    fn core_schema_rejects_duplicate_json_keys_before_semantic_validation() {
        let schema = br#"{"$id":"https://commandf.dev/schemas/af02-adversarial-proof-v1.schema.json","type":"object","required":["deterministic"],"properties":{"deterministic":{"type":"object"}}}"#;
        let error = validate_core_schema(br#"{"deterministic":{"x":1,"x":2}}"#, schema).unwrap_err();
        assert!(error.to_string().contains("duplicate JSON object key"));
    }

    #[test]
    fn core_schema_rejects_empty_deterministic_object() {
        let schema = br#"{"$id":"https://commandf.dev/schemas/af02-adversarial-proof-v1.schema.json","type":"object","additionalProperties":false,"required":["deterministic"],"properties":{"deterministic":{"type":"object","additionalProperties":true}}}"#;
        let error = validate_core_schema(br#"{"deterministic":{}}"#, schema).unwrap_err();
        assert!(error.to_string().contains("must not be empty"));
    }

    #[test]
    fn envelope_rejects_duplicate_extension_role_and_core_path_overlap() {
        let mut files = EXTENSION_ROLES_AND_PATHS
            .iter()
            .map(|(role, path)| ContractDescriptor {
                role: (*role).to_owned(),
                path: (*path).to_owned(),
            })
            .collect::<Vec<_>>();
        files[1].role = files[0].role.clone();
        assert!(validate_proof_envelope_closure(&[], &files).is_err());
        let files = EXTENSION_ROLES_AND_PATHS
            .iter()
            .map(|(role, path)| ContractDescriptor {
                role: (*role).to_owned(),
                path: (*path).to_owned(),
            })
            .collect::<Vec<_>>();
        let overlap = vec![files[0].path.clone()];
        assert!(validate_proof_envelope_closure(&overlap, &files).is_err());
    }

    #[test]
    fn policy_predecessor_rejects_bootstrap_and_rebase_counterexamples() {
        let bootstrap = PolicyPredecessorEvidence {
            mode: PolicyMode::Bootstrap,
            policy_path: "policy.json".to_owned(),
            canonical_base_blob: Some("a".repeat(40)),
            canonical_base_sha256: None,
            declared_predecessor_blob: None,
            declared_predecessor_sha256: None,
            changed_paths: vec!["policy.json".to_owned()],
            dependent_evidence_paths: Vec::new(),
        };
        assert!(validate_policy_predecessor_comparison(&bootstrap).is_err());
        let rebase = PolicyPredecessorEvidence {
            mode: PolicyMode::Rebase,
            policy_path: "policy.json".to_owned(),
            canonical_base_blob: Some("a".repeat(40)),
            canonical_base_sha256: Some("b".repeat(64)),
            declared_predecessor_blob: Some("c".repeat(40)),
            declared_predecessor_sha256: Some("b".repeat(64)),
            changed_paths: vec!["policy.json".to_owned()],
            dependent_evidence_paths: Vec::new(),
        };
        assert!(validate_policy_predecessor_comparison(&rebase).is_err());
    }

    #[test]
    fn source_coverage_mutation_and_bijection_reject_membership_drift() {
        assert!(validate_source_blob_reconstruction(
            &["b.rs".to_owned(), "a.rs".to_owned()],
            &[]
        )
        .is_err());
        assert!(validate_coverage_accounting(
            &["a.rs".to_owned()],
            &[],
            &[]
        )
        .is_err());
        assert!(validate_mutation_membership(
            &[MutationEntry {
                mutant_id: "m1".to_owned(),
                in_target: true,
                precanonical_excluded: false,
            }],
            &[]
        )
        .is_err());
        let key = BijectionKey {
            scenario_id: "s1".to_owned(),
            assertion_id: "a1".to_owned(),
            fixture_path: "f1".to_owned(),
        };
        assert!(validate_assertion_replay_bijection(
            std::slice::from_ref(&key),
            std::slice::from_ref(&key),
            &[]
        )
        .is_err());
    }

    #[test]
    fn input_process_path_and_enforcement_counterexamples_fail_closed() {
        let limits = CandidateInputLimits {
            max_files: 1,
            max_aggregate_bytes: 10,
            max_depth: 2,
            max_records: 2,
        };
        let stats = CandidateInputStats {
            files: 1,
            aggregate_bytes: 11,
            depth: 1,
            records: 1,
            yaml_alias_present: false,
            yaml_merge_key_present: false,
            yaml_custom_tag_present: false,
        };
        assert!(validate_candidate_input_limits(&limits, &stats).is_err());
        let path = PathObservation {
            repo_relative_path: "safe/file".to_owned(),
            has_symlink_component: true,
            owner_uid: 1000,
            expected_owner_uid: 1000,
            link_count: 1,
            contained: true,
        };
        assert!(validate_path_nofollow_containment(&path).is_err());
        let process = ProcessEvidence {
            binary_sha256: "a".repeat(64),
            expected_binary_sha256: "a".repeat(64),
            cargo_lock_blob: "b".repeat(40),
            expected_cargo_lock_blob: "b".repeat(40),
            unprivileged: true,
            cgroup_v2: false,
            wall_timeout_enforced: true,
            memory_limit_enforced: true,
            pid_limit_enforced: true,
            network_none: true,
            root_read_only: true,
            stdout_observed: 0,
            stdout_limit: 1,
            stdout_exceeded: false,
            stderr_observed: 0,
            stderr_limit: 1,
            stderr_exceeded: false,
            termination: "SUCCESS".to_owned(),
        };
        assert!(validate_input_process_enforcement(&process).is_err());
        let roles = vec!["parser".to_owned()];
        assert!(validate_enforcement_inventory_closure(&roles, &[], 1).is_err());
    }

    #[test]
    fn contract_digest_authority_counter_and_final_hash_checks_are_exact() {
        let bytes = b"contract";
        let role = "semantic_contract".to_owned();
        let roles = vec![role.clone()];
        let file = ContractBytes {
            role: &role,
            path: "contract.json",
            blob_sha: &git_blob_sha1_hex(bytes),
            sha256: &sha256_hex(bytes),
            bytes,
        };
        validate_contract_digest_reconstruction(&roles, &[file]).unwrap();
        assert!(validate_counter_equalities(&[CounterEquality {
            label: "count".to_owned(),
            observed: 1,
            expected: 2,
            denominator: None,
        }])
        .is_err());
        let core = serde_json::json!({"value": 1});
        let extension = serde_json::json!([]);
        let authority = serde_json::json!({});
        let checks = serde_json::json!({});
        let envelope = serde_json::json!({
            "core_deterministic": core,
            "extension_contract_files": extension,
            "extension_authority": authority,
            "required_check_provenance": checks,
        });
        let digest = canonical_sha256(&envelope).unwrap();
        validate_final_envelope_hash(
            &envelope["core_deterministic"],
            &envelope["extension_contract_files"],
            &envelope["extension_authority"],
            &envelope["required_check_provenance"],
            &digest,
        )
        .unwrap();
    }
}